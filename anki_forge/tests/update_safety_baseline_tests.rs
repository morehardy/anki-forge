use anki_forge::build::BuildOptions;
use anki_forge::prelude::*;
use anki_forge::update_safety::baseline::load_previous_apkg_identity_index;

#[test]
fn previous_apkg_identity_index_recovers_embedded_metadata() {
    let root = tempfile::tempdir().expect("tempdir");
    let previous = root.path().join("previous.apkg");
    let mut project = Project::new("Baseline").stable_id("baseline-project");
    project
        .add_note(Note::basic("hola", "hello").stable_id("es:hola"))
        .expect("add note");
    project
        .build(BuildOptions::new().output(&previous))
        .expect("build previous");

    let index = load_previous_apkg_identity_index(&previous, None, None)
        .expect("load previous apkg identity");

    assert_eq!(index.source_kind, "previous_apkg");
    assert_eq!(index.source_ref, "baseline.previous_apkg.primary");
    assert!(index.notes.iter().any(|note| note.stable_id == "es:hola"));
}
