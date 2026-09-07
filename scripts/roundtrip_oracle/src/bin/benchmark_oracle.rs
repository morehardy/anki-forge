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
    #[serde(default)]
    media: Vec<Media>,
}
#[derive(Deserialize)]
struct Record {
    id: String,
    category: String,
    front: String,
    back: String,
    #[serde(default)]
    front_media: Vec<String>,
    #[serde(default)]
    back_media: Vec<String>,
}
#[derive(Deserialize)]
struct Media {
    id: String,
    kind: String,
    filename: String,
    path: String,
}

fn suffix(refs: &[String], media: &BTreeMap<&str, &Media>) -> anyhow::Result<String> {
    let mut result = String::new();
    for id in refs {
        let item = media.get(id.as_str()).context("unknown media reference")?;
        // Frozen benchmark filenames are simple ASCII names.
        ensure!(
            item.filename
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b)),
            "unsafe fixture filename"
        );
        match item.kind.as_str() {
            "image" => result.push_str(&format!("\n<img src=\"{}\">", item.filename)),
            "audio" => result.push_str(&format!("\n[sound:{}]", item.filename)),
            _ => anyhow::bail!("unknown media kind"),
        }
    }
    Ok(result)
}
fn field(raw: &str, suffix: &str) -> anyhow::Result<String> {
    literal(
        raw.strip_suffix(suffix)
            .context("media reference markup mismatch")?,
    )
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
    let media_by_id: BTreeMap<_, _> = workload
        .media
        .iter()
        .map(|item| (item.id.as_str(), item))
        .collect();
    ensure!(
        media_by_id.len() == workload.media.len(),
        "duplicate fixture media id"
    );
    let mut selected = selected;
    for kind in ["image", "audio"] {
        if let Some(note) = workload.notes.iter().find(|note| {
            note.front_media.iter().chain(&note.back_media).any(|id| {
                media_by_id
                    .get(id.as_str())
                    .is_some_and(|media| media.kind == kind)
            })
        }) {
            selected.insert(note.id.clone());
        }
    }
    let expected_fields = workload
        .notes
        .iter()
        .map(|note| {
            Ok((
                note.front.as_str(),
                (
                    suffix(&note.front_media, &media_by_id)?,
                    suffix(&note.back_media, &media_by_id)?,
                ),
            ))
        })
        .collect::<anyhow::Result<BTreeMap<_, _>>>()?;
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
        let raw_front = &note.fields()[0];
        let text_end = raw_front
            .find("\n<img src=")
            .or_else(|| raw_front.find("\n[sound:"))
            .unwrap_or(raw_front.len());
        let front = literal(&raw_front[..text_end])?;
        let (front_suffix, back_suffix) = expected_fields
            .get(front.as_str())
            .context("unexpected front")?;
        ensure!(
            field(raw_front, front_suffix)? == front,
            "front references mismatch"
        );
        let back = field(&note.fields()[1], back_suffix)?;
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
            let question = field(&rendered.question(), front_suffix)?;
            let answer_raw = rendered.answer().into_owned();
            let (answer_front, answer_back) = answer_raw
                .split_once("\n\n<hr id=answer>\n\n")
                .context("rendered answer separator")?;
            ensure!(
                question == source.front
                    && field(answer_front, front_suffix)? == source.front
                    && field(answer_back, back_suffix)? == source.back,
                "rendered literal content mismatch"
            );
            representatives.insert(source.category.clone());
            renders.push(json!({"id": source.id, "category": source.category,
                               "question": question, "answer_front": field(answer_front, front_suffix)?,
                               "answer_back": field(answer_back, back_suffix)?}));
        }
    }
    ensure!(
        models.len() == 1 && decks.len() == 1 && representatives.len() >= 3,
        "used models/decks/renders"
    );
    let media = collection_path.with_extension("media");
    let mut imported_names = BTreeSet::new();
    if media.exists() {
        for entry in fs::read_dir(&media)? {
            imported_names.insert(
                entry?
                    .file_name()
                    .into_string()
                    .map_err(|_| anyhow::anyhow!("non-UTF8 media name"))?,
            );
        }
    }
    ensure!(
        imported_names
            == workload
                .media
                .iter()
                .map(|item| item.filename.clone())
                .collect(),
        "imported media filenames mismatch"
    );
    let input_path = PathBuf::from(&args[0]);
    let input_parent = input_path.parent().context("input parent")?;
    for item in &workload.media {
        ensure!(
            fs::read(media.join(&item.filename))? == fs::read(input_parent.join(&item.path))?,
            "imported media bytes mismatch: {}",
            item.filename
        );
    }
    renders.sort_by_key(|r| r["id"].as_str().unwrap().to_owned());
    fs::write(
        PathBuf::from(&args[2]),
        serde_json::to_vec_pretty(&json!({
            "status": "passed", "oracle": "upstream-anki-media-artifact-v2",
            "notes": note_ids.len(), "cards": observed.len(), "new_notes": imported.output.new.len(),
            "conflicting": 0, "used_models": models.len(), "populated_decks": decks.len(),
            "all_fields_checked": true, "media_files": workload.media.len(), "renders": renders,
        }))?,
    )?;
    Ok(())
}
