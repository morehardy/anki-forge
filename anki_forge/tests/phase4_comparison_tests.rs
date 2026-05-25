use std::time::Instant;

use anki_forge::build::{BuildOptions, BuildStatus, ComparisonStatus};
use anki_forge::prelude::*;
use tempfile::tempdir;

fn basic_project(front: &str) -> Project {
    let mut project = Project::new("Phase4 Basic")
        .stable_id("phase4-basic")
        .default_deck("Phase4");
    project
        .add_note(Note::basic(front, "back").stable_id("note-1"))
        .expect("add note");
    project
}

#[test]
fn comparison_assembler_compares_two_built_apkgs() {
    let temp = tempdir().expect("tempdir");
    let previous = temp.path().join("previous.apkg");
    let current = temp.path().join("current.apkg");

    basic_project("front")
        .build(BuildOptions::new().output(&previous))
        .expect("previous build");

    basic_project("changed front")
        .build(BuildOptions::new().output(&current))
        .expect("current build");

    let comparison = anki_forge::product::comparison::assemble_comparison(
        anki_forge::product::comparison::ComparisonInput {
            current_artifact: &current,
            previous_artifact: Some(&previous),
            diagnostics: &[],
            update_safety: None,
            started: Instant::now(),
        },
    );

    assert_eq!(comparison.comparison, ComparisonStatus::Complete);
    assert!(
        comparison.current_inspect.is_some(),
        "current inspect exists"
    );
    assert!(
        comparison.previous_inspect.is_some(),
        "previous inspect exists"
    );
    assert!(comparison.diff.is_some(), "diff summary exists");
    assert!(comparison.risk.is_some(), "risk report exists");
}

#[test]
fn comparison_assembler_reports_unavailable_baseline() {
    let temp = tempdir().expect("tempdir");
    let current = temp.path().join("current.apkg");
    let missing = temp.path().join("missing.apkg");

    basic_project("front")
        .build(BuildOptions::new().output(&current))
        .expect("current build");

    let comparison = anki_forge::product::comparison::assemble_comparison(
        anki_forge::product::comparison::ComparisonInput {
            current_artifact: &current,
            previous_artifact: Some(&missing),
            diagnostics: &[],
            update_safety: None,
            started: Instant::now(),
        },
    );

    assert_eq!(comparison.comparison, ComparisonStatus::Unavailable);
    assert!(comparison
        .risk
        .as_ref()
        .expect("risk")
        .findings
        .iter()
        .any(|finding| finding.code == "RISK.BASELINE_UNAVAILABLE"));
}

#[test]
fn comparison_assembler_reports_current_unavailable_without_baseline_risk() {
    let temp = tempdir().expect("tempdir");
    let previous = temp.path().join("previous.apkg");
    let missing_current = temp.path().join("missing-current.apkg");

    basic_project("front")
        .build(BuildOptions::new().output(&previous))
        .expect("previous build");

    let comparison = anki_forge::product::comparison::assemble_comparison(
        anki_forge::product::comparison::ComparisonInput {
            current_artifact: &missing_current,
            previous_artifact: Some(&previous),
            diagnostics: &[],
            update_safety: None,
            started: Instant::now(),
        },
    );

    assert_eq!(comparison.comparison, ComparisonStatus::Unavailable);
    assert_eq!(comparison.status, BuildStatus::Invalid);
    assert!(comparison
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code.as_str() == "COMPARE.CURRENT_UNAVAILABLE"));
    assert!(!comparison
        .risk
        .as_ref()
        .expect("risk")
        .findings
        .iter()
        .any(|finding| finding.code == "RISK.BASELINE_UNAVAILABLE"));
}
