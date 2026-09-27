//! Imports successive original distributions into the same real Anki collection.
//! No package or collection mtimes are changed to force a successful update.
use anki::services::CardsService;
use anki::{
    collection::CollectionBuilder,
    import_export::package::{ImportAnkiPackageOptions, UpdateCondition},
    search::SortMode,
};
use anyhow::{ensure, Context};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Deserialize)]
struct Input {
    format_version: String,
    scenarios: Vec<Scenario>,
}
#[derive(Deserialize)]
struct Scenario {
    name: String,
    stages: Vec<Stage>,
}
#[derive(Deserialize)]
struct Stage {
    apkg: PathBuf,
    expected_fields: BTreeMap<String, String>,
    structural: bool,
    source_mtime: i64,
    guid: String,
    candidate_cards: BTreeMap<String, u16>,
    comparison: Value,
}
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
struct Schedule {
    kind: i64,
    queue: i64,
    due: i64,
    interval: i64,
    factor: i64,
    reps: i64,
    lapses: i64,
    left: i64,
    odue: i64,
    odid: i64,
}
#[derive(Clone, Debug, Serialize)]
struct Card {
    id: i64,
    note_id: i64,
    ordinal: u16,
    schedule: Schedule,
}
#[derive(Clone, Debug, Serialize)]
struct Observation {
    notetype_count: usize,
    note_id: i64,
    guid: String,
    model_id: i64,
    mtime: i64,
    model_name: String,
    fields: BTreeMap<String, String>,
    field_order: Vec<String>,
    template_order: Vec<String>,
    cards: Vec<Card>,
}

fn observe(col: &mut anki::prelude::Collection, _path: &Path) -> anyhow::Result<Observation> {
    let ids = col.search_notes_unordered("")?;
    ensure!(
        ids.len() == 1,
        "oracle expects one imported note, found {}",
        ids.len()
    );
    let note = col.storage.get_note(ids[0])?.context("imported note")?;
    let model = col
        .get_notetype(note.notetype_id)?
        .context("imported model")?;
    let mut cards = vec![];
    for cid in col.search_cards("", SortMode::NoOrder)? {
        let c = col.get_card(anki_proto::cards::CardId { cid: cid.0 })?;
        cards.push(Card {
            id: c.id,
            note_id: c.note_id,
            ordinal: c.template_idx as u16,
            schedule: Schedule {
                kind: c.ctype as i64,
                queue: c.queue as i64,
                due: c.due as i64,
                interval: c.interval as i64,
                factor: c.ease_factor as i64,
                reps: c.reps as i64,
                lapses: c.lapses as i64,
                left: c.remaining_steps as i64,
                odue: c.original_due as i64,
                odid: c.original_deck_id,
            },
        });
    }
    cards.sort_by_key(|c| c.ordinal);
    Ok(Observation {
        notetype_count: col.get_all_notetypes()?.len(),
        note_id: note.id.0,
        guid: note.guid.clone(),
        model_id: note.notetype_id.0,
        mtime: note.mtime.0,
        model_name: model.name.clone(),
        fields: model
            .fields
            .iter()
            .zip(note.fields())
            .map(|(f, v)| (f.name.clone(), v.clone()))
            .collect(),
        field_order: model.fields.iter().map(|f| f.name.clone()).collect(),
        template_order: model.templates.iter().map(|t| t.name.clone()).collect(),
        cards,
    })
}
fn run(scenario: &Scenario, merge: bool, condition: UpdateCondition) -> anyhow::Result<Value> {
    let root = tempfile::tempdir()?;
    let path = root.path().join("learner.anki2");
    let mut builder = CollectionBuilder::new(&path);
    builder.with_desktop_media_paths();
    let mut col = builder.build()?;
    let mut previous: Option<Observation> = None;
    let mut history = vec![];
    let mut limitations = vec![];
    for (index, stage) in scenario.stages.iter().enumerate() {
        let imported = col
            .import_apkg(
                &stage.apkg,
                ImportAnkiPackageOptions {
                    merge_notetypes: merge,
                    update_notes: condition as i32,
                    update_notetypes: condition as i32,
                    ..Default::default()
                },
            )
            .with_context(|| {
                format!(
                    "{} stage {index}, merge={merge}, condition={condition:?}",
                    scenario.name
                )
            })?;
        if index == 0 {
            let ids = col.search_cards("", SortMode::NoOrder)?;
            col.set_due_date(&ids, "23", None)?;
            let mut scheduled = vec![];
            for cid in ids {
                let mut card = col.get_card(anki_proto::cards::CardId { cid: cid.0 })?;
                card.reps = 17;
                card.lapses = 3;
                card.ease_factor = 2710;
                scheduled.push(card);
            }
            let _ = col.update_cards(anki_proto::cards::UpdateCardsRequest {
                cards: scheduled,
                skip_undo_entry: false,
            })?;
        }
        let observed = observe(&mut col, &path)?;
        ensure!(
            observed.guid == stage.guid,
            "{}: imported GUID differs from authored identity",
            scenario.name
        );
        let body_matches = stage
            .expected_fields
            .iter()
            .all(|(key, value)| observed.fields.get(key) == Some(value));
        if let Some(previous) = &previous {
            ensure!(
                observed.note_id == previous.note_id && observed.guid == previous.guid,
                "{}: existing note identity changed",
                scenario.name
            );
            for before in &previous.cards {
                let after = observed
                    .cards
                    .iter()
                    .find(|c| c.id == before.id)
                    .with_context(|| {
                        format!("{}: existing card {} disappeared", scenario.name, before.id)
                    })?;
                ensure!(
                    before.ordinal == after.ordinal,
                    "{}: existing card ordinal changed",
                    scenario.name
                );
                ensure!(
                    before.schedule == after.schedule,
                    "{}: existing scheduling changed for card {}",
                    scenario.name,
                    before.id
                );
            }
            if !body_matches {
                if stage.structural && !merge && !imported.output.conflicting.is_empty() {
                    limitations.push(json!({"stage":index,"kind":"schema_conflict_without_merge","detail":"Anki retains the old note; enabling model merge is required for this structural update."}));
                } else if timestamp_blocks(condition, previous.mtime, stage.source_mtime) {
                    limitations.push(json!({"stage":index,"kind":"existing_note_timestamp_prevented_update","condition":format!("{condition:?}"),"source_mtime":stage.source_mtime,"previous_mtime":previous.mtime,"observed_mtime":observed.mtime,"detail":"The previous imported learner note is not eligible for this timestamp-based update. This can persist after an earlier schema change advanced its timestamp."}));
                } else if stage.structural
                    && observed.mtime > previous.mtime
                    && timestamp_blocks(condition, observed.mtime, stage.source_mtime)
                {
                    limitations.push(json!({"stage":index,"kind":"schema_timestamp_prevented_content_update","condition":format!("{condition:?}"),"source_mtime":stage.source_mtime,"previous_mtime":previous.mtime,"observed_mtime":observed.mtime,"detail":"The schema or sort-field operation advanced the learner note timestamp before its body update decision. No timestamps were altered by this oracle."}));
                } else {
                    anyhow::bail!("{} stage {index} unexpectedly retained content (merge={merge}, {condition:?}): {:?}, expected {:?}",scenario.name,observed.fields,stage.expected_fields);
                }
            }
        } else {
            ensure!(body_matches, "{}: initial content mismatch", scenario.name);
        }
        if body_matches {
            for ordinal in stage.candidate_cards.values() {
                ensure!(
                    observed.cards.iter().any(|c| c.ordinal == *ordinal),
                    "{}: candidate active card {ordinal} absent after real import",
                    scenario.name
                );
            }
        }
        history.push(json!({"stage":index,"apkg":stage.apkg,"source_mtime":stage.source_mtime,
            "candidate_cards":stage.candidate_cards,"comparison":stage.comparison,
            "body_matches_candidate":body_matches,"conflicting_notes":imported.output.conflicting.len(),
            "new_notes":imported.output.new.len(),"updated_notes":imported.output.updated.len(),"observed":observed}));
        previous = Some(observed);
    }
    Ok(
        json!({"scenario":scenario.name,"merge_notetypes":merge,"update_condition":format!("{condition:?}"),
        "history":history,"limitations":limitations}),
    )
}
fn timestamp_blocks(condition: UpdateCondition, target: i64, incoming: i64) -> bool {
    match condition {
        UpdateCondition::IfNewer => target >= incoming,
        UpdateCondition::Always => target == incoming,
        UpdateCondition::Never => true,
    }
}
fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let input: Input = serde_json::from_slice(&fs::read(
        args.next().context("prepared-input.json required")?,
    )?)?;
    ensure!(
        input.format_version == "native-anki-oracle-v1",
        "unsupported prepared input"
    );
    let mut results = vec![];
    let mut failures = 0;
    for scenario in &input.scenarios {
        for merge in [false, true] {
            for condition in [UpdateCondition::IfNewer, UpdateCondition::Always] {
                match run(scenario, merge, condition) {
                    Ok(result) => results.push(result),
                    Err(error) => {
                        failures += 1;
                        results.push(json!({"scenario":scenario.name,"merge_notetypes":merge,
                            "update_condition":format!("{condition:?}"),"error":format!("{error:#}")}));
                    }
                }
            }
        }
    }
    let report = json!({"status":if failures==0{"verified"}else{"failed"},"failures":failures,"timestamp_intervention":false,"results":results});
    let encoded = serde_json::to_string_pretty(&report)?;
    if let Some(path) = args.next() {
        fs::write(path, &encoded)?;
    }
    println!("{encoded}");
    ensure!(
        failures == 0,
        "{failures} import-setting scenarios failed; see the recorded report"
    );
    Ok(())
}
