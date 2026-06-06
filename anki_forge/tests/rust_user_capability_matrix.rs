use std::path::{Path, PathBuf};

use anki_forge::build::{BuildError, BuildReport};
#[allow(unused_imports)]
use anki_forge::prelude::*;
use anki_forge::writer::{inspect_apkg, InspectReport};
use anki_forge::Deck;
use serde_json::Value;

fn scenario_dir() -> PathBuf {
    let value = std::env::var("ANKI_FORGE_CAPABILITY_ARTIFACT_DIR")
        .expect("set ANKI_FORGE_CAPABILITY_ARTIFACT_DIR by running scripts/run_rust_user_capabilities.sh");
    assert!(
        !value.trim().is_empty(),
        "ANKI_FORGE_CAPABILITY_ARTIFACT_DIR must not be empty; run scripts/run_rust_user_capabilities.sh"
    );
    let path = PathBuf::from(value);
    std::fs::create_dir_all(&path).expect("create scenario artifact dir");
    path
}

#[allow(dead_code)]
fn normalize_report(result: Result<BuildReport, BuildError>) -> BuildReport {
    match result {
        Ok(report) => report,
        Err(error) => *error.report,
    }
}

fn inspect_complete(path: &Path) -> InspectReport {
    let report = inspect_apkg(path).expect("inspect generated APKG");
    assert_eq!(report.source_kind, "apkg");
    assert_eq!(report.observation_status, "complete");
    assert!(
        report.missing_domains.is_empty(),
        "inspect missing domains: {:?}",
        report.missing_domains
    );
    assert!(
        report.degradation_reasons.is_empty(),
        "inspect degradation reasons: {:?}",
        report.degradation_reasons
    );
    report
}

fn counts(report: &InspectReport) -> &Value {
    report
        .observations
        .metadata
        .iter()
        .find(|value| value["selector"] == "counts")
        .expect("counts metadata observation")
}

fn has_observation(values: &[Value], key: &str, expected: &str) -> bool {
    values.iter().any(|value| value[key].as_str() == Some(expected))
}

#[ignore]
#[test]
fn deck_basic_apkg() {
    let root = scenario_dir();
    let apkg = root.join("package.apkg");
    let mut deck = Deck::builder("Spanish").stable_id("cap-deck-basic").build();

    deck.basic()
        .note("hola", "hello")
        .stable_id("es:hola")
        .add()
        .expect("add basic note");

    let report = deck.write_apkg(&apkg).expect("write apkg");
    report.ensure_success().expect("successful report");
    assert_eq!(report.counts.notes, 1);
    assert_eq!(report.counts.cards, 1);
    assert_eq!(report.counts.media, 0);
    assert!(apkg.is_file());

    let inspected = inspect_complete(&apkg);
    let counts = counts(&inspected);
    assert_eq!(counts["note_count"], 1);
    assert_eq!(counts["card_count"], 1);
    assert!(has_observation(
        &inspected.observations.notetypes,
        "name",
        "Basic"
    ));
}
