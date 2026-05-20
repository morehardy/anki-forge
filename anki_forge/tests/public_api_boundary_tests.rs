use anki_forge::prelude::*;

#[test]
fn product_source_map_is_not_a_public_mutation_surface() {
    let product_mod =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/product/mod.rs"))
            .expect("read product mod");
    let lowering = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/product/lowering.rs"
    ))
    .expect("read product lowering");

    assert!(
        !product_mod.contains("ProductSourceMap"),
        "ProductSourceMap should not be re-exported from anki_forge::product"
    );
    assert!(
        !lowering.contains("pub fn insert(&mut self"),
        "ProductSourceMap::insert should not be public API"
    );
}

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
