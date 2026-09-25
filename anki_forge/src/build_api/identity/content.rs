//! Revision fingerprints cover content observable in the collection. In
//! particular, card destinations use deck names, not allocation-local deck IDs.
//! Card flags participate because templates can render `{{CardFlag}}`.
//! A default deck overridden by every template (or producing no cards) has no
//! stored effect and does not by itself create a new note revision.
use std::collections::BTreeMap;

use anyhow::{ensure, Context};
use prost::Message;
use serde::Serialize;

use crate::{
    authoring_core::{NormalizedNote, NormalizedNotetype},
    writer_core::{anki_proto, deck_name::DeckRegistry},
};

type ConfigRow = (u32, String, Vec<u8>);
type TemplateRow = (u32, String, Vec<u8>, Option<String>);

#[derive(Serialize)]
struct ModelContent {
    name: String,
    config: Vec<u8>,
    fields: Vec<ConfigRow>,
    templates: Vec<TemplateRow>,
}

#[derive(Serialize)]
struct NoteContent<'a> {
    model: i64,
    fields: &'a str,
    tags: &'a str,
    cards: &'a [(u32, String, i64)],
}

fn hash(value: &impl Serialize) -> anyhow::Result<String> {
    Ok(blake3::hash(&serde_json::to_vec(value)?)
        .to_hex()
        .to_string())
}

fn template_row(
    ordinal: u32,
    name: String,
    bytes: &[u8],
    deck_name: impl FnOnce(i64) -> anyhow::Result<String>,
) -> anyhow::Result<TemplateRow> {
    let mut config = anki_proto::decode_template_config(bytes)?;
    let destination = if config.target_deck_id == 0 {
        None
    } else {
        Some(deck_name(config.target_deck_id)?)
    };
    config.target_deck_id = 0;
    Ok((ordinal, name, config.encode_to_vec(), destination))
}

pub(super) fn model_from_plan(
    model: &NormalizedNotetype,
    decks: &DeckRegistry,
) -> anyhow::Result<String> {
    let templates = model
        .templates
        .iter()
        .enumerate()
        .map(|(ordinal, template)| {
            let deck = template
                .target_deck_name
                .as_deref()
                .map(|name| {
                    decks
                        .deck_for_human_name(name)
                        .context("missing target deck")
                })
                .transpose()?;
            template_row(
                template.ord.unwrap_or(ordinal as u32),
                template.name.clone(),
                &anki_proto::encode_template_config(template, deck.map_or(0, |deck| deck.id)),
                |_| {
                    Ok(deck
                        .expect("nonzero deck ID has a resolved deck")
                        .native_name
                        .clone())
                },
            )
        })
        .collect::<anyhow::Result<_>>()?;
    hash(&ModelContent {
        name: model.name.clone(),
        config: anki_proto::encode_notetype_config(model)?,
        fields: model
            .fields
            .iter()
            .enumerate()
            .map(|(ordinal, field)| {
                (
                    field.ord.unwrap_or(ordinal as u32),
                    field.name.clone(),
                    anki_proto::encode_field_config(field),
                )
            })
            .collect(),
        templates,
    })
}

fn config_rows(
    db: &rusqlite::Connection,
    model: i64,
    table: &str,
) -> anyhow::Result<Vec<ConfigRow>> {
    // The only callers supply the literal Anki model table names below.
    Ok(db
        .prepare(&format!(
            "SELECT ord, name, config FROM {table} WHERE ntid = ?1 ORDER BY ord"
        ))?
        .query_map([model], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
        .collect::<Result<_, _>>()?)
}

pub(super) fn model_from_collection(
    db: &rusqlite::Connection,
    model: i64,
) -> anyhow::Result<String> {
    let (name, config) = db.query_row(
        "SELECT name, config FROM notetypes WHERE id = ?1",
        [model],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    hash(&ModelContent {
        name,
        config,
        fields: config_rows(db, model, "fields")?,
        templates: config_rows(db, model, "templates")?
            .into_iter()
            .map(|(ordinal, name, config)| {
                template_row(ordinal, name, &config, |deck| {
                    Ok(
                        db.query_row("SELECT name FROM decks WHERE id = ?1", [deck], |row| {
                            row.get(0)
                        })?,
                    )
                })
            })
            .collect::<anyhow::Result<_>>()?,
    })
}

pub(super) fn note_from_plan(
    note: &NormalizedNote,
    model: &NormalizedNotetype,
    model_id: i64,
    decks: &DeckRegistry,
) -> anyhow::Result<String> {
    let fields = model
        .fields
        .iter()
        .map(|field| note.fields.get(&field.name).map_or("", String::as_str))
        .collect::<Vec<_>>()
        .join("\u{1f}");
    let tags = note.tags.join(" ");
    let mut cards = crate::writer_core::card_plan::plan_cards(note, model)
        .into_iter()
        .map(|card| {
            let template = &model.templates[card.template_index];
            let name = template
                .target_deck_name
                .as_deref()
                .unwrap_or(&note.deck_name);
            let deck = decks
                .deck_for_human_name(name)
                .context("missing card deck")?;
            Ok((card.card_ord, deck.native_name.clone(), 0))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    cards.sort_unstable_by_key(|(ordinal, _, _)| *ordinal);
    hash(&NoteContent {
        model: model_id,
        fields: &fields,
        tags: &tags,
        cards: &cards,
    })
}

pub(super) fn note_from_collection(
    db: &rusqlite::Connection,
    note: i64,
    model: i64,
    cloze: bool,
) -> anyhow::Result<String> {
    let (fields, tags): (String, String) = db.query_row(
        "SELECT flds, tags FROM notes WHERE id = ?1",
        [note],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    let template_decks = config_rows(db, model, "templates")?
        .into_iter()
        .map(|(ordinal, _, bytes)| {
            Ok((
                ordinal,
                anki_proto::decode_template_config(&bytes)?.target_deck_id,
            ))
        })
        .collect::<anyhow::Result<BTreeMap<_, _>>>()?;
    let rows = db.prepare(
        "SELECT cards.ord, cards.did, decks.name, cards.flags FROM cards LEFT JOIN decks ON decks.id = cards.did WHERE cards.nid = ?1 ORDER BY cards.ord",
    )?.query_map([note], |row| Ok((row.get::<_, u32>(0)?, row.get::<_, i64>(1)?, row.get::<_, Option<String>>(2)?, row.get::<_, i64>(3)?)))?
        .collect::<Result<Vec<_>, _>>()?;
    let cards = rows
        .into_iter()
        .map(|(ordinal, deck, name, flags)| {
            let target = template_decks
                .get(&if cloze { 0 } else { ordinal })
                .context("card refers to a missing template")?;
            ensure!(
                *target == 0 || *target == deck,
                "card destination disagrees with template target deck"
            );
            Ok((
                ordinal,
                name.context("card refers to a missing deck")?,
                flags,
            ))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    hash(&NoteContent {
        model,
        fields: &fields,
        tags: &tags,
        cards: &cards,
    })
}
