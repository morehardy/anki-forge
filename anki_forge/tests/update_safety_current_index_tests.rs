use anki_forge::build::{BuildOptions, UpdateSafetyMode};
use anki_forge::prelude::*;

#[test]
fn strict_update_safety_blocks_note_without_resolved_stable_id() {
    let root = tempfile::tempdir().expect("tempdir");
    let output = root.path().join("missing-stable-id.apkg");
    let mut project = Project::new("Strict Missing Identity").stable_id("strict-missing");

    project
        .add_note(Note::basic("hola", "hello"))
        .expect("add note without stable id");

    let err = project
        .build(
            BuildOptions::new()
                .output(&output)
                .update_safety(UpdateSafetyMode::Strict),
        )
        .expect_err("strict update safety should require stable ids");

    assert!(err.report.diagnostic_codes().contains(&"UPDATE.STABLE_ID_MISSING_IN_STRICT_MODE".into()));
    assert!(!output.exists());
}

#[test]
fn strict_update_safety_allows_explicit_stable_id() {
    let root = tempfile::tempdir().expect("tempdir");
    let output = root.path().join("stable.apkg");
    let mut project = Project::new("Strict Stable").stable_id("strict-stable");

    project
        .add_note(Note::basic("hola", "hello").stable_id("es:hola"))
        .expect("add note");

    let report = project
        .build(
            BuildOptions::new()
                .output(&output)
                .update_safety(UpdateSafetyMode::Strict),
        )
        .expect("strict build with stable id");

    assert!(report.ensure_success().is_ok());
}

#[test]
fn strict_update_safety_blocks_invalid_anki_guid_candidate() {
    let root = tempfile::tempdir().expect("tempdir");
    let output = root.path().join("invalid-guid.apkg");
    let mut project = Project::new("Strict Invalid Guid").stable_id("strict-invalid-guid");

    project
        .add_note(Note::basic("hola", "hello").stable_id("bad\nid"))
        .expect("add note");

    let err = project
        .build(
            BuildOptions::new()
                .output(&output)
                .update_safety(UpdateSafetyMode::Strict),
        )
        .expect_err("invalid GUID candidate should block writer execution");

    assert!(err.report.diagnostic_codes().contains(&"UPDATE.ANKI_GUID_INVALID".into()));
    assert!(!output.exists());
}
