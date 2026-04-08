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

#[test]
fn cloze_and_image_occlusion_lanes_lower_to_stock_compatible_authoring_shapes() {
    let cloze_text = "A {{c1::cloze}} card";
    let plan = ProductDocument::new("cloze-doc")
        .with_cloze("cloze-main")
        .add_cloze_note("cloze-main", "note-1", "Default", cloze_text, "extra")
        .lower()
        .expect("lower should succeed");

    let notetype = plan
        .authoring_document
        .notetypes
        .first()
        .expect("lower should produce one notetype");
    assert_eq!(notetype.kind, "cloze");

    let note = plan
        .authoring_document
        .notes
        .first()
        .expect("lower should produce one note");
    assert_eq!(
        note.fields.get("Text").map(String::as_str),
        Some(cloze_text)
    );

    let plan = ProductDocument::new("io-doc")
        .with_image_occlusion("io-main")
        .add_image_occlusion_note(
            "io-main",
            "note-1",
            "Default",
            "occlusion",
            "image.png",
            "Header",
            "back_extra",
            "comments",
        )
        .lower()
        .expect("lower should succeed");

    let notetype = plan
        .authoring_document
        .notetypes
        .first()
        .expect("lower should produce one notetype");
    assert_eq!(notetype.kind, "cloze");

    let note = plan
        .authoring_document
        .notes
        .first()
        .expect("lower should produce one note");
    assert_eq!(
        note.fields.get("Header").map(String::as_str),
        Some("Header")
    );
}

#[test]
fn image_occlusion_missing_image_emits_product_diagnostic() {
    let err = ProductDocument::new("io-doc")
        .with_image_occlusion("io-main")
        .add_image_occlusion_note(
            "io-main",
            "note-1",
            "Default",
            "occlusion",
            "",
            "Header",
            "back_extra",
            "comments",
        )
        .lower()
        .expect_err("lower should fail");

    assert!(err
        .product_diagnostics
        .iter()
        .any(|d| d.code == "PHASE5A.IO_IMAGE_REQUIRED"));
}
