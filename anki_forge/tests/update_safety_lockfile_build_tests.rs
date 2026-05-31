use anki_forge::build::BuildOptions;
use anki_forge::prelude::*;

#[test]
fn build_writes_lockfile_and_second_build_preserves_guid_from_it() {
    let root = tempfile::tempdir().expect("tempdir");
    let first_apkg = root.path().join("first.apkg");
    let second_apkg = root.path().join("second.apkg");
    let lockfile = root.path().join("anki-forge.lock.json");

    let mut first = Project::new("Spanish").stable_id("spanish");
    first
        .add_note(Note::basic("hola", "hello").stable_id("es:hola"))
        .expect("add first note");
    first
        .build(
            BuildOptions::new()
                .output(&first_apkg)
                .identity_lockfile(&lockfile)
                .write_identity_lockfile(true),
        )
        .expect("write initial lockfile");
    assert!(lockfile.exists());

    let mut second = Project::new("Spanish").stable_id("spanish");
    second
        .add_note(Note::basic("hola", "hello again").stable_id("es:hola"))
        .expect("add second note");
    let report = second
        .build(
            BuildOptions::new()
                .output(&second_apkg)
                .identity_lockfile(&lockfile),
        )
        .expect("use lockfile");

    assert_eq!(report.update_safety.unwrap().notes_preserved, 1);
}

#[test]
fn strict_lockfile_project_stable_id_mismatch_blocks_build() {
    let root = tempfile::tempdir().expect("tempdir");
    let first_apkg = root.path().join("first.apkg");
    let second_apkg = root.path().join("second.apkg");
    let lockfile = root.path().join("anki-forge.lock.json");

    let mut first = Project::new("Spanish").stable_id("spanish-v1");
    first
        .add_note(Note::basic("hola", "hello").stable_id("es:hola"))
        .expect("add first note");
    first
        .build(
            BuildOptions::new()
                .output(&first_apkg)
                .first_update_safe_build(&lockfile),
        )
        .expect("write initial lockfile");

    let mut second = Project::new("Spanish").stable_id("spanish-v2");
    second
        .add_note(Note::basic("hola", "hello").stable_id("es:hola"))
        .expect("add second note");

    let err = second
        .build(
            BuildOptions::new()
                .output(&second_apkg)
                .update_safe(&lockfile),
        )
        .expect_err("project stable id mismatch must block strict update-safe builds");

    assert!(err
        .report
        .diagnostic_codes()
        .contains(&"UPDATE.PROJECT_STABLE_ID_MISMATCH".into()));
    assert!(!second_apkg.exists());
}

#[test]
fn report_only_lockfile_project_stable_id_mismatch_is_warning() {
    let root = tempfile::tempdir().expect("tempdir");
    let first_apkg = root.path().join("first.apkg");
    let second_apkg = root.path().join("second.apkg");
    let lockfile = root.path().join("anki-forge.lock.json");

    let mut first = Project::new("Spanish").stable_id("spanish-v1");
    first
        .add_note(Note::basic("hola", "hello").stable_id("es:hola"))
        .expect("add first note");
    first
        .build(
            BuildOptions::new()
                .output(&first_apkg)
                .first_update_safe_build(&lockfile),
        )
        .expect("write initial lockfile");

    let mut second = Project::new("Spanish").stable_id("spanish-v2");
    second
        .add_note(Note::basic("hola", "hello").stable_id("es:hola"))
        .expect("add second note");

    let report = second
        .build(
            BuildOptions::new()
                .output(&second_apkg)
                .identity_lockfile(&lockfile)
                .update_safety(anki_forge::build::UpdateSafetyMode::ReportOnly),
        )
        .expect("report-only mismatch should build with warning diagnostics");

    let diagnostic = report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code.as_str() == "UPDATE.PROJECT_STABLE_ID_MISMATCH")
        .expect("mismatch diagnostic");
    assert_eq!(diagnostic.severity, Severity::Warning);
    assert!(second_apkg.exists());
}

#[test]
fn lockfile_carries_forward_absent_entries() {
    let root = tempfile::tempdir().expect("tempdir");
    let first_apkg = root.path().join("first.apkg");
    let second_apkg = root.path().join("second.apkg");
    let lockfile = root.path().join("anki-forge.lock.json");

    let mut first = Project::new("Spanish").stable_id("spanish");
    first
        .add_note(Note::basic("hola", "hello").stable_id("es:hola"))
        .expect("add hola");
    first
        .add_note(Note::basic("adios", "goodbye").stable_id("es:adios"))
        .expect("add adios");
    first
        .build(
            BuildOptions::new()
                .output(&first_apkg)
                .identity_lockfile(&lockfile)
                .write_identity_lockfile(true),
        )
        .expect("write initial lockfile");

    let mut second = Project::new("Spanish").stable_id("spanish");
    second
        .add_note(Note::basic("hola", "hello again").stable_id("es:hola"))
        .expect("add hola only");
    second
        .build(
            BuildOptions::new()
                .output(&second_apkg)
                .identity_lockfile(&lockfile)
                .write_identity_lockfile(true),
        )
        .expect("rewrite lockfile");

    let loaded = anki_forge::update_safety::lockfile::read_lockfile(&lockfile)
        .expect("read rewritten lockfile");
    assert!(loaded.identity_index.notes.iter().any(|note| {
        note.stable_id == "es:adios" && note.entry_lifecycle == "absent_from_current"
    }));
}

#[test]
fn update_safety_build_does_not_write_lockfile_when_writer_fails() {
    let root = tempfile::tempdir().expect("tempdir");
    let output = root.path().join("out.apkg");
    let lockfile = root.path().join("anki-forge.lock.json");
    let artifact_file = root.path().join("artifact-root-is-file");
    std::fs::write(&artifact_file, b"not a directory").expect("seed artifact file");

    let mut project = Project::new("Spanish").stable_id("spanish");
    project
        .add_note(Note::basic("hola", "hello").stable_id("es:hola"))
        .expect("add note");

    let err = project
        .build(
            BuildOptions::new()
                .output(&output)
                .artifacts_dir(&artifact_file)
                .identity_lockfile(&lockfile)
                .write_identity_lockfile(true),
        )
        .expect_err("writer failure should fail build");

    assert!(err.report.diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == Severity::Error && diagnostic.code.as_str().starts_with("PHASE3.")
    }));
    assert!(!lockfile.exists());
}
