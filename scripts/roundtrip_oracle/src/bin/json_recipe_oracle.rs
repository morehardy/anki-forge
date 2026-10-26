//! Verify the recipe in an isolated real Anki collection, without touching a user profile.
use anki::{
    collection::CollectionBuilder,
    import_export::package::{ImportAnkiPackageOptions, UpdateCondition},
    search::SortMode,
    services::CardsService,
};
use anyhow::{ensure, Context};
use serde_json::{json, Value};
use std::{collections::BTreeMap, fs, path::PathBuf};

fn observe(col: &mut anki::prelude::Collection) -> anyhow::Result<Value> {
    let mut notes = BTreeMap::new();
    for id in col.search_notes_unordered("")? {
        let note = col.storage.get_note(id)?.context("note")?;
        notes.insert(
            note.guid.clone(),
            json!({"id":id.0, "fields":note.fields(), "mtime":note.mtime.0}),
        );
    }
    let mut cards = BTreeMap::new();
    for id in col.search_cards("", SortMode::NoOrder)? {
        let card = col.get_card(anki_proto::cards::CardId { cid: id.0 })?;
        cards.insert(
            id.0.to_string(),
            json!({
                "nid":card.note_id, "ordinal":card.template_idx, "type":card.ctype,
                "queue":card.queue, "due":card.due, "interval":card.interval,
                "factor":card.ease_factor, "reps":card.reps, "lapses":card.lapses,
            }),
        );
    }
    Ok(json!({"notes":notes,"cards":cards}))
}

fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let packages = PathBuf::from(args.get(1).context("public consumer evidence directory")?);
    let fixtures = PathBuf::from(args.get(2).context("recipe fixtures")?);
    let mut observations = vec![];
    for language in ["rust", "python", "node"] {
        for condition in [UpdateCondition::IfNewer, UpdateCondition::Always] {
            let temporary = tempfile::tempdir()?;
            let collection = temporary.path().join("recipe.anki2");
            let mut builder = CollectionBuilder::new(&collection);
            builder.with_desktop_media_paths();
            let mut col = builder.build()?;
            let mut before: Option<Value> = None;
            for version in [1, 2] {
                let apkg = packages.join(language).join(format!("v{version}.apkg"));
                let imported = col.import_apkg(
                    &apkg,
                    ImportAnkiPackageOptions {
                        merge_notetypes: false,
                        update_notes: condition as i32,
                        update_notetypes: condition as i32,
                        ..Default::default()
                    },
                )?;
                ensure!(imported.output.conflicting.is_empty(), "conflicting notes");
                if version == 1 {
                    ensure!(
                        imported.output.new.len() == 3,
                        "expected three initial notes"
                    );
                    let ids = col.search_cards("", SortMode::NoOrder)?;
                    col.set_due_date(&ids, "23", None)?;
                    let mut cards = vec![];
                    for id in ids {
                        let mut card = col.get_card(anki_proto::cards::CardId { cid: id.0 })?;
                        card.reps = 17;
                        card.lapses = 3;
                        cards.push(card);
                    }
                    let _ = col.update_cards(anki_proto::cards::UpdateCardsRequest {
                        cards,
                        skip_undo_entry: false,
                    })?;
                } else {
                    ensure!(imported.output.new.is_empty(), "update duplicated notes");
                }
                let observed = observe(&mut col)?;
                ensure!(
                    observed["notes"].as_object().context("notes")?.len() == 3,
                    "note count"
                );
                ensure!(
                    observed["cards"].as_object().context("cards")?.len() == 3,
                    "card count"
                );
                let data: Value =
                    serde_json::from_slice(&fs::read(fixtures.join(format!("v{version}.json")))?)?;
                for expected in data["notes"].as_array().context("fixture notes")? {
                    let row = observed["notes"]
                        .as_object()
                        .unwrap()
                        .values()
                        .find(|note| {
                            note["fields"][0]
                                .as_str()
                                .unwrap_or("")
                                .starts_with(expected["front"].as_str().unwrap())
                        })
                        .context("imported question")?;
                    ensure!(
                        row["fields"][1] == expected["back"],
                        "answer was not imported"
                    );
                }
                let media: Vec<_> = fs::read_dir(collection.with_extension("media"))?
                    .map(|entry| fs::read(entry?.path()))
                    .collect::<Result<_, std::io::Error>>()?;
                ensure!(media.len() == 2, "imported media count");
                for name in ["cell.png", "tone.wav"] {
                    let expected = fs::read(fixtures.join(name))?;
                    ensure!(media.contains(&expected), "imported media changed");
                }
                if let Some(previous) = &before {
                    ensure!(
                        previous["cards"] == observed["cards"],
                        "card identity or scheduling changed"
                    );
                    for (guid, note) in previous["notes"].as_object().unwrap() {
                        ensure!(
                            note["id"] == observed["notes"][guid]["id"],
                            "note identity changed"
                        );
                    }
                }
                observations.push(json!({"language":language,"version":version,"condition":format!("{condition:?}"),"merge_notetypes":false,"observed":observed}));
                before = Some(observed);
            }
        }
    }
    fs::write(
        args.get(3).context("report destination")?,
        serde_json::to_vec_pretty(&json!({
            "anki_version":anki::version::version(),"os":std::env::consts::OS,
            "scope":"isolated Anki backend import, identity, scheduling and exact media; no GUI or audible playback",
            "observations":observations
        }))?,
    )?;
    println!("Verified all three SDK release chains with real Anki imports (IfNewer and Always, no model merge).");
    Ok(())
}
