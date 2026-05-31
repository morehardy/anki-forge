use anki_forge::build::{BuildOptions, UpdateSafetyMode};
use anki_forge::prelude::*;

#[test]
fn strict_update_safety_blocks_note_without_resolved_stable_id() {
    let root = tempfile::tempdir().expect("tempdir");
    let output = root.path().join("missing-stable-id.apkg");
    let mut project = Project::new("Strict Missing Identity").stable_id("strict-missing");

    project
        .add_note(Note::basic("hola", "hello").stable_id("generated:Strict Missing Identity:1"))
        .expect("add note with generated stable id");

    let err = project
        .build(
            BuildOptions::new()
                .output(&output)
                .update_safety(UpdateSafetyMode::Strict),
        )
        .expect_err("strict update safety should require stable ids");

    assert!(err
        .report
        .diagnostic_codes()
        .contains(&"UPDATE.STABLE_ID_MISSING_IN_STRICT_MODE".into()));
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
fn strict_update_safety_requires_project_stable_id_even_without_baseline() {
    let root = tempfile::tempdir().expect("tempdir");
    let output = root.path().join("missing-project-stable-id.apkg");
    let mut project = Project::new("Missing Project Stable Id");

    project
        .add_note(Note::basic("hola", "hello").stable_id("es:hola"))
        .expect("add note");

    let err = project
        .build(
            BuildOptions::new()
                .output(&output)
                .update_safety(UpdateSafetyMode::Strict),
        )
        .expect_err("strict update-safe builds require Project::stable_id");

    assert!(err
        .report
        .diagnostic_codes()
        .contains(&"UPDATE.PROJECT_STABLE_ID_MISSING".into()));
    assert!(!output.exists());
}

#[test]
fn report_only_update_safety_missing_project_stable_id_is_warning() {
    let root = tempfile::tempdir().expect("tempdir");
    let output = root
        .path()
        .join("missing-project-stable-id-report-only.apkg");
    let mut project = Project::new("Missing Project Stable Id");

    project
        .add_note(Note::basic("hola", "hello").stable_id("es:hola"))
        .expect("add note");

    let report = project
        .build(
            BuildOptions::new()
                .output(&output)
                .update_safety(UpdateSafetyMode::ReportOnly),
        )
        .expect("report-only update safety should warn without blocking");

    let diagnostic = report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code.as_str() == "UPDATE.PROJECT_STABLE_ID_MISSING")
        .expect("project stable id diagnostic");
    assert_eq!(diagnostic.severity, Severity::Warning);
    assert!(output.exists());
}

#[test]
fn report_only_update_safety_write_identity_lockfile_requires_project_stable_id() {
    let root = tempfile::tempdir().expect("tempdir");
    let output = root
        .path()
        .join("missing-project-stable-id-report-only-write.apkg");
    let lockfile = root.path().join("anki-forge.lock.json");
    let mut project = Project::new("Missing Project Stable Id");

    project
        .add_note(Note::basic("hola", "hello").stable_id("es:hola"))
        .expect("add note");

    let err = project
        .build(
            BuildOptions::new()
                .output(&output)
                .identity_lockfile(&lockfile)
                .write_identity_lockfile(true)
                .update_safety(UpdateSafetyMode::ReportOnly),
        )
        .expect_err("writing a lockfile requires Project::stable_id before writer runs");

    assert!(err
        .report
        .diagnostic_codes()
        .contains(&"UPDATE.PROJECT_STABLE_ID_MISSING".into()));
    assert!(!output.exists());
    assert!(!lockfile.exists());
}

#[test]
fn disabled_update_safety_allows_identity_lockfile_without_project_stable_id() {
    let root = tempfile::tempdir().expect("tempdir");
    let output = root.path().join("disabled-no-project-stable-id.apkg");
    let lockfile = root.path().join("ignored.lock.json");
    let mut project = Project::new("Missing Project Stable Id");

    project
        .add_note(Note::basic("hola", "hello").stable_id("es:hola"))
        .expect("add note");

    let report = project
        .build(
            BuildOptions::new()
                .output(&output)
                .identity_lockfile(&lockfile)
                .update_safety(UpdateSafetyMode::Disabled),
        )
        .expect("disabled update safety should ignore lockfile baselines");

    assert!(report.ensure_success().is_ok());
    assert!(output.exists());
}

#[test]
fn disabled_update_safety_write_identity_lockfile_requires_project_stable_id() {
    let root = tempfile::tempdir().expect("tempdir");
    let output = root
        .path()
        .join("missing-project-stable-id-disabled-write.apkg");
    let lockfile = root.path().join("anki-forge.lock.json");
    let mut project = Project::new("Missing Project Stable Id");

    project
        .add_note(Note::basic("hola", "hello").stable_id("es:hola"))
        .expect("add note");

    let err = project
        .build(
            BuildOptions::new()
                .output(&output)
                .identity_lockfile(&lockfile)
                .write_identity_lockfile(true)
                .update_safety(UpdateSafetyMode::Disabled),
        )
        .expect_err("writing a lockfile requires Project::stable_id before writer runs");

    assert!(err
        .report
        .diagnostic_codes()
        .contains(&"UPDATE.PROJECT_STABLE_ID_MISSING".into()));
    assert!(!output.exists());
    assert!(!lockfile.exists());
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

    assert!(err
        .report
        .diagnostic_codes()
        .contains(&"UPDATE.ANKI_GUID_INVALID".into()));
    assert!(!output.exists());
}

#[test]
fn missing_identity_lockfile_path_uses_message_without_redundant_help() {
    let root = tempfile::tempdir().expect("tempdir");
    let output = root.path().join("missing-lockfile-path.apkg");
    let mut project = Project::new("Spanish").stable_id("spanish");
    project
        .add_note(Note::basic("hola", "hello").stable_id("es:hola"))
        .expect("add note");

    let err = project
        .build(
            BuildOptions::new()
                .output(&output)
                .write_identity_lockfile(true),
        )
        .expect_err("write_identity_lockfile requires identity_lockfile path");

    let diagnostic = err
        .report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code.as_str() == "UPDATE.LOCKFILE_PATH_REQUIRED")
        .expect("lockfile path diagnostic");
    assert_eq!(
        diagnostic.message,
        "write_identity_lockfile(true) requires identity_lockfile(path)"
    );
    assert_eq!(diagnostic.help, None);
    assert!(!output.exists());
}

#[test]
fn identity_lockfile_path_must_not_equal_apkg_output_path() {
    let root = tempfile::tempdir().expect("tempdir");
    let output_and_lockfile = root.path().join("deck.apkg");
    let report_json = root.path().join("report.json");
    let mut project = Project::new("Spanish").stable_id("spanish");

    project
        .add_note(Note::basic("hola", "hello").stable_id("es:hola"))
        .expect("add note");

    let err = project
        .build(
            BuildOptions::new()
                .output(&output_and_lockfile)
                .identity_lockfile(&output_and_lockfile)
                .report_json(&report_json)
                .write_identity_lockfile(true),
        )
        .expect_err("APKG output and lockfile must be distinct paths");

    assert!(err
        .report
        .diagnostic_codes()
        .contains(&"PROJECT.PATH_COLLISION".into()));
    assert!(!output_and_lockfile.exists());
    assert!(report_json.exists());
    let report_json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&report_json).expect("read report_json"))
            .expect("parse report_json");
    assert!(report_json["diagnostics"]
        .as_array()
        .expect("diagnostics array")
        .iter()
        .any(
            |diagnostic| diagnostic.get("code").and_then(|code| code.as_str())
                == Some("PROJECT.PATH_COLLISION")
        ));
}

#[test]
fn apkg_output_path_must_not_equal_report_json_path() {
    let root = tempfile::tempdir().expect("tempdir");
    let output_and_report = root.path().join("deck.apkg");
    let mut project = Project::new("Spanish").stable_id("spanish");

    project
        .add_note(Note::basic("hola", "hello").stable_id("es:hola"))
        .expect("add note");

    let err = project
        .build(
            BuildOptions::new()
                .output(&output_and_report)
                .report_json(&output_and_report),
        )
        .expect_err("APKG output and report_json must be distinct paths");

    let diagnostic = err
        .report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code.as_str() == "PROJECT.PATH_COLLISION")
        .expect("path collision diagnostic");
    assert_eq!(diagnostic.severity, Severity::Error);
    assert!(!output_and_report.exists());
}

#[test]
fn implicit_apkg_output_path_must_not_equal_report_json_path() {
    let root = tempfile::tempdir().expect("tempdir");
    let artifacts_dir = root.path().join("artifacts");
    let package = artifacts_dir.join("package.apkg");
    let mut project = Project::new("Spanish").stable_id("spanish");

    project
        .add_note(Note::basic("hola", "hello").stable_id("es:hola"))
        .expect("add note");

    let err = project
        .build(
            BuildOptions::new()
                .artifacts_dir(&artifacts_dir)
                .report_json(&package),
        )
        .expect_err("implicit APKG output and report_json must be distinct paths");

    assert!(err
        .report
        .diagnostic_codes()
        .contains(&"PROJECT.PATH_COLLISION".into()));
    assert!(!package.exists());
}

#[test]
fn implicit_apkg_output_path_must_not_equal_identity_lockfile_path() {
    let root = tempfile::tempdir().expect("tempdir");
    let artifacts_dir = root.path().join("artifacts");
    let package = artifacts_dir.join("package.apkg");
    let mut project = Project::new("Spanish").stable_id("spanish");

    project
        .add_note(Note::basic("hola", "hello").stable_id("es:hola"))
        .expect("add note");

    let err = project
        .build(
            BuildOptions::new()
                .artifacts_dir(&artifacts_dir)
                .identity_lockfile(&package)
                .write_identity_lockfile(true),
        )
        .expect_err("implicit APKG output and lockfile must be distinct paths");

    assert!(err
        .report
        .diagnostic_codes()
        .contains(&"PROJECT.PATH_COLLISION".into()));
    assert!(!package.exists());
}

#[test]
fn identity_lockfile_path_must_not_equal_report_json_path() {
    let root = tempfile::tempdir().expect("tempdir");
    let output = root.path().join("deck.apkg");
    let lockfile_and_report = root.path().join("anki-forge.lock.json");
    let sentinel = "sentinel lockfile contents";
    std::fs::write(&lockfile_and_report, sentinel).expect("write sentinel lockfile");
    let mut project = Project::new("Spanish").stable_id("spanish");

    project
        .add_note(Note::basic("hola", "hello").stable_id("es:hola"))
        .expect("add note");

    let err = project
        .build(
            BuildOptions::new()
                .output(&output)
                .identity_lockfile(&lockfile_and_report)
                .report_json(&lockfile_and_report),
        )
        .expect_err("report_json and lockfile must be distinct paths");

    assert!(err
        .report
        .diagnostic_codes()
        .contains(&"PROJECT.PATH_COLLISION".into()));
    assert_eq!(
        std::fs::read_to_string(&lockfile_and_report).expect("read sentinel lockfile"),
        sentinel
    );
    assert!(!output.exists());
}

#[test]
fn identity_lockfile_report_json_collision_preserves_lockfile_on_early_empty_project_failure() {
    let root = tempfile::tempdir().expect("tempdir");
    let output = root.path().join("deck.apkg");
    let lockfile_and_report = root.path().join("anki-forge.lock.json");
    let sentinel = "sentinel lockfile contents";
    std::fs::write(&lockfile_and_report, sentinel).expect("write sentinel lockfile");
    let project = Project::new("Empty").stable_id("empty");

    let err = project
        .build(
            BuildOptions::new()
                .output(&output)
                .identity_lockfile(&lockfile_and_report)
                .report_json(&lockfile_and_report),
        )
        .expect_err("report_json and lockfile must be distinct paths before early failures");

    assert!(err
        .report
        .diagnostic_codes()
        .contains(&"PROJECT.PATH_COLLISION".into()));
    assert_eq!(
        std::fs::read_to_string(&lockfile_and_report).expect("read sentinel lockfile"),
        sentinel
    );
    assert!(!output.exists());
}

#[test]
fn custom_update_safety_identity_recipe_derives_stable_note_id_in_strict_mode() {
    let root = tempfile::tempdir().expect("tempdir");
    let output = root.path().join("custom-derived.apkg");
    let lockfile = root.path().join("anki-forge.lock.json");
    let mut project = Project::new("Japanese").stable_id("jp-core");
    project
        .add_notetype(custom_vocab_notetype().identity(IdentityRecipe::fields(["expr"])))
        .expect("add notetype");
    project
        .add_note(
            Note::new("jp-vocab")
                .text("expr", "食べる")
                .text("meaning", "to eat"),
        )
        .expect("add note");

    project
        .build(
            BuildOptions::new()
                .output(&output)
                .identity_lockfile(&lockfile)
                .write_identity_lockfile(true)
                .update_safety(UpdateSafetyMode::Strict),
        )
        .expect("strict build should accept recipe-derived identity");

    let loaded =
        anki_forge::update_safety::lockfile::read_lockfile(&lockfile).expect("read lockfile");
    let note = loaded.identity_index.notes.first().expect("note identity");
    assert!(note.stable_id.starts_with("afid:v1:"));
    assert_eq!(
        note.normalized_note_id.as_deref(),
        Some(note.stable_id.as_str())
    );
    assert_eq!(note.current_guid_candidate, note.stable_id);
    assert_eq!(note.recipe_id, "custom.notetype.fields.v1");
    assert!(note
        .canonical_payload_hash
        .as_deref()
        .is_some_and(|hash| hash.starts_with("blake3:")));
    assert_eq!(note.provenance, "InferredFromNotetypeFields");
    assert!(!note.used_override);
}

#[test]
fn note_level_update_safety_identity_override_marks_snapshot_and_derives_id() {
    let root = tempfile::tempdir().expect("tempdir");
    let output = root.path().join("custom-override.apkg");
    let lockfile = root.path().join("anki-forge.lock.json");
    let mut project = Project::new("Japanese").stable_id("jp-core");
    project
        .add_notetype(custom_vocab_notetype())
        .expect("add notetype");
    project
        .add_note(
            Note::new("jp-vocab")
                .text("expr", "食べる")
                .text("meaning", "to eat")
                .identity(["expr"]),
        )
        .expect("add note");

    project
        .build(
            BuildOptions::new()
                .output(&output)
                .identity_lockfile(&lockfile)
                .write_identity_lockfile(true)
                .update_safety(UpdateSafetyMode::Strict),
        )
        .expect("strict build should accept note-level identity override");

    let loaded =
        anki_forge::update_safety::lockfile::read_lockfile(&lockfile).expect("read lockfile");
    let note = loaded.identity_index.notes.first().expect("note identity");
    assert!(note.stable_id.starts_with("afid:v1:"));
    assert_eq!(note.recipe_id, "custom.note-override.fields.v1");
    assert_eq!(note.provenance, "InferredFromNoteFields");
    assert!(note.used_override);
}

fn custom_vocab_notetype() -> NoteType {
    NoteType::custom("jp-vocab")
        .field(Field::new("Expression").key("expr"))
        .field(Field::new("Meaning").key("meaning"))
        .template(
            Template::new("Recognition")
                .key("recognition")
                .front("{{Expression}}")
                .back("{{Meaning}}")
                .generate_when(GenerationRule::all(["expr"])),
        )
}
