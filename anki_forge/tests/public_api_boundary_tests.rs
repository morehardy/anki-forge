use anki_forge::prelude::*;

#[test]
fn prelude_exports_product_happy_path_types() {
    let mut project = Project::new("Spanish")
        .stable_id("spanish")
        .default_deck("Spanish");
    project
        .add_note(Note::basic("hola", "hello").stable_id("es:hola"))
        .expect("add note");

    let _options = BuildOptions::new().inspect(true);
}

#[test]
fn advanced_authoring_reexports_are_namespaced() {
    let document = anki_forge::authoring::AuthoringDocument {
        kind: "authoring-ir".into(),
        schema_version: "0.1.0".into(),
        metadata_document_id: "doc".into(),
        notetypes: vec![],
        notes: vec![],
        media: vec![],
    };

    assert_eq!(document.kind, "authoring-ir");
}

#[test]
fn advanced_writer_reexports_are_namespaced() {
    let _build = anki_forge::writer::build;
    let _policy: Option<anki_forge::writer::WriterPolicy> = None;
    let _target: Option<anki_forge::writer::BuildArtifactTarget> = None;
}
