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
        .build(BuildOptions::new().output(&second_apkg).identity_lockfile(&lockfile))
        .expect("use lockfile");

    assert_eq!(report.update_safety.unwrap().notes_preserved, 1);
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
