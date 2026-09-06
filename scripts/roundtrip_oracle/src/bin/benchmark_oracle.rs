//! Independent, single-artifact Basic import/render oracle. Never used in timing.
use anki::{
    collection::CollectionBuilder,
    import_export::package::ImportAnkiPackageOptions,
    search::SortMode,
    text::{decode_entities, strip_html_preserving_entities},
};
use anyhow::{ensure, Context};
use serde::Deserialize;
use serde_json::json;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
};

#[derive(Deserialize)]
struct Workload {
    deck_name: String,
    note_count: usize,
    notes: Vec<Record>,
}
#[derive(Deserialize)]
struct Record {
    id: String,
    category: String,
    front: String,
    back: String,
}

fn literal(raw: &str) -> anyhow::Result<String> {
    ensure!(
        strip_html_preserving_entities(raw).as_ref() == raw,
        "unexpected active markup"
    );
    Ok(decode_entities(raw).into_owned())
}

fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    ensure!(
        args.len() == 3,
        "usage: benchmark_oracle INPUT APKG EVIDENCE"
    );
    let workload: Workload = serde_json::from_slice(&fs::read(&args[0])?)?;
    let root = tempfile::tempdir()?;
    let collection_path = root.path().join("benchmark.anki2");
    let mut builder = CollectionBuilder::new(&collection_path);
    builder.with_desktop_media_paths();
    let mut col = builder.build()?;
    let imported = col.import_apkg(&args[1], ImportAnkiPackageOptions::default())?;
    ensure!(
        imported.output.conflicting.is_empty(),
        "conflicting imports"
    );
    ensure!(
        imported.output.new.len() == workload.note_count,
        "not all notes imported as new"
    );
    let note_ids = col.search_notes_unordered("")?;
    let card_ids = col.search_cards("", SortMode::NoOrder)?;
    ensure!(note_ids.len() == workload.note_count, "imported note count");
    ensure!(card_ids.len() == workload.note_count, "imported card count");
    let expected: BTreeMap<_, _> = workload
        .notes
        .iter()
        .map(|n| (n.front.as_str(), n))
        .collect();
    let mut observed = BTreeSet::new();
    let mut models = BTreeSet::new();
    let mut decks = BTreeSet::new();
    let mut representatives = BTreeSet::new();
    let selected: BTreeSet<_> = ["english", "mixed", "escaping"]
        .into_iter()
        .map(|category| {
            workload
                .notes
                .iter()
                .find(|n| n.category == category)
                .unwrap()
                .id
                .clone()
        })
        .collect();
    let mut renders = vec![];
    for cid in card_ids {
        let card = col
            .storage
            .get_card(cid)?
            .context("missing imported card")?;
        ensure!(card.template_idx() == 0, "card ordinal");
        ensure!(observed.insert(card.note_id()), "multiple cards for a note");
        let note = col
            .storage
            .get_note(card.note_id())?
            .context("missing imported note")?;
        ensure!(
            note.fields().len() == 2 && note.tags.is_empty(),
            "fields/tags"
        );
        let front = literal(&note.fields()[0])?;
        let back = literal(&note.fields()[1])?;
        let source = expected
            .get(front.as_str())
            .context("unexpected imported front")?;
        ensure!(back == source.back, "imported back mismatch");
        let nt = col
            .get_notetype(note.notetype_id)?
            .context("missing note type")?;
        ensure!(
            nt.fields
                .iter()
                .map(|f| f.name.as_str())
                .collect::<Vec<_>>()
                == ["Front", "Back"],
            "field names"
        );
        ensure!(
            nt.templates.len() == 1 && nt.config.kind == 0,
            "Basic note type"
        );
        ensure!(
            nt.templates[0].config.q_format == "{{Front}}",
            "question template"
        );
        ensure!(
            nt.templates[0].config.a_format == "{{FrontSide}}\n\n<hr id=answer>\n\n{{Back}}",
            "answer template"
        );
        models.insert(note.notetype_id);
        let deck = col.get_deck(card.deck_id())?.context("missing deck")?;
        ensure!(deck.human_name() == workload.deck_name, "deck assignment");
        decks.insert(card.deck_id());
        if selected.contains(&source.id) {
            let rendered = col.render_existing_card(cid, false, false)?;
            ensure!(!rendered.is_empty, "empty render");
            let question = literal(&rendered.question())?;
            let answer_raw = rendered.answer().into_owned();
            let (answer_front, answer_back) = answer_raw
                .split_once("\n\n<hr id=answer>\n\n")
                .context("rendered answer separator")?;
            ensure!(
                question == source.front
                    && literal(answer_front)? == source.front
                    && literal(answer_back)? == source.back,
                "rendered literal content mismatch"
            );
            representatives.insert(source.category.clone());
            renders.push(json!({"id": source.id, "category": source.category,
                               "question": question, "answer_front": literal(answer_front)?,
                               "answer_back": literal(answer_back)?}));
        }
    }
    ensure!(
        models.len() == 1 && decks.len() == 1 && representatives.len() == 3,
        "used models/decks/renders"
    );
    let media = collection_path.with_extension("media");
    ensure!(
        !media.exists() || fs::read_dir(media)?.next().is_none(),
        "unexpected imported media"
    );
    renders.sort_by_key(|r| r["id"].as_str().unwrap().to_owned());
    fs::write(
        PathBuf::from(&args[2]),
        serde_json::to_vec_pretty(&json!({
            "status": "passed", "oracle": "upstream-anki-single-artifact-v1",
            "notes": note_ids.len(), "cards": observed.len(), "new_notes": imported.output.new.len(),
            "conflicting": 0, "used_models": models.len(), "populated_decks": decks.len(),
            "all_fields_checked": true, "media_files": 0, "renders": renders,
        }))?,
    )?;
    Ok(())
}
