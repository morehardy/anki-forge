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
