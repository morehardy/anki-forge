use anki_forge::product::ProductDocument;

#[test]
fn basic_product_document_lowers_to_authoring_ir_with_mapping_evidence() {
    let plan = ProductDocument::new("demo-doc")
        .with_basic("basic-main")
        .add_basic_note(
            "note-1",
            "basic-main",
            "Default",
            "front".to_string(),
            "back".to_string(),
        )
        .lower()
        .expect("lower should succeed");

    assert_eq!(plan.authoring_document.kind, "authoring-ir");

    let notetype = plan
        .authoring_document
        .notetypes
        .first()
        .expect("lower should produce one notetype");
    assert_eq!(notetype.kind, "normal");

    let note = plan
        .authoring_document
        .notes
        .first()
        .expect("lower should produce one note");
    assert_eq!(note.fields.get("Front").map(String::as_str), Some("front"));

    assert_eq!(plan.mappings.len(), 2);
    assert!(plan.product_diagnostics.is_empty());
    assert!(plan.lowering_diagnostics.is_empty());
}

