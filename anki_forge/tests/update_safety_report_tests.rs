#![cfg(feature = "internal-tools")]

use anki_forge::build::{BuildOptions, UpdateSafetyMode};
use anki_forge::prelude::*;

#[test]
fn disabled_mode_ignores_baseline_but_records_summary() {
    let root = tempfile::tempdir().expect("tempdir");
    let previous = root.path().join("missing.apkg");
    let output = root.path().join("out.apkg");
    let mut project = Project::new("Disabled").stable_id("disabled");
    project
        .add_note(Note::basic("hola", "hello").stable_id("es:hola"))
        .expect("add note");

    let report = project
        .build(
            BuildOptions::new()
                .output(&output)
                .compare_to(&previous)
                .update_safety(UpdateSafetyMode::Disabled),
        )
        .expect("disabled ignores missing baseline");

    assert!(report
        .diagnostic_codes()
        .contains(&"UPDATE.BASELINE_IGNORED_DISABLED".into()));
    let summary = report.update_safety.as_ref().expect("summary");
    assert_eq!(summary.mode, "disabled");
    assert_eq!(summary.baseline_sources[0].status, "ignored_disabled");
}

#[test]
fn report_only_update_safety_records_unreadable_baseline_without_blocking_build() {
    let root = tempfile::tempdir().expect("tempdir");
    let previous = root.path().join("missing.apkg");
    let output = root.path().join("out.apkg");
    let mut project = Project::new("Report Only").stable_id("report-only");
    project
        .add_note(Note::basic("hola", "hello").stable_id("es:hola"))
        .expect("add note");

    let report = project
        .build(
            BuildOptions::new()
                .output(&output)
                .compare_to(&previous)
                .update_safety(UpdateSafetyMode::ReportOnly),
        )
        .expect("report-only should not block artifact creation");

    assert!(output.exists());
    assert!(report
        .diagnostic_codes()
        .contains(&"UPDATE.BASELINE_APKG_UNREADABLE".into()));
    assert!(report
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code.as_str().starts_with("UPDATE."))
        .all(|diagnostic| diagnostic.severity != Severity::Error));
    let summary = report.update_safety.as_ref().expect("summary");
    assert_eq!(summary.mode, "report_only");
    assert_eq!(summary.baseline_sources[0].source_kind, "previous_apkg");
    assert_eq!(summary.baseline_sources[0].status, "unreadable");
    assert!(!summary.baseline_sources[0].used_for_reconcile);
}
