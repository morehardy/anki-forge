use anki_forge::product::model::{CustomField, CustomNote, CustomNoteType, CustomTemplate};
use anki_forge::product::ProductDocument;
use std::collections::BTreeMap;

#[test]
fn basic_product_document_lowers_to_authoring_ir_with_mapping_evidence() {
    let plan = ProductDocument::new("demo-doc")
        .with_basic("basic-main")
        .add_basic_note(
            "basic-main",
            "note-1",
            "Default",
            "front".to_string(),
            "back".to_string(),
            ["demo"],
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
    assert_eq!(notetype.original_stock_kind.as_deref(), Some("basic"));

    let note = plan
        .authoring_document
        .notes
        .first()
        .expect("lower should produce one note");
    assert_eq!(note.fields.get("Front").map(String::as_str), Some("front"));
    assert_eq!(note.tags, vec!["demo"]);

    assert_eq!(plan.mappings.len(), 2);
    assert!(plan.product_diagnostics.is_empty());
    assert!(plan.lowering_diagnostics.is_empty());
}

#[test]
fn cloze_and_image_occlusion_lanes_lower_to_stock_compatible_authoring_shapes() {
    let cloze_text = "A {{c1::cloze}} card";
    let plan = ProductDocument::new("cloze-doc")
        .with_cloze("cloze-main")
        .add_cloze_note(
            "cloze-main",
            "note-1",
            "Default",
            cloze_text,
            "extra",
            ["tagged"],
        )
        .lower()
        .expect("lower should succeed");

    let notetype = plan
        .authoring_document
        .notetypes
        .first()
        .expect("lower should produce one notetype");
    assert_eq!(notetype.kind, "cloze");
    assert_eq!(notetype.original_stock_kind.as_deref(), Some("cloze"));

    let note = plan
        .authoring_document
        .notes
        .first()
        .expect("lower should produce one note");
    assert_eq!(
        note.fields.get("Text").map(String::as_str),
        Some(cloze_text)
    );
    assert_eq!(note.tags, vec!["tagged"]);

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
            ["image-tag"],
        )
        .lower()
        .expect("lower should succeed");

    let notetype = plan
        .authoring_document
        .notetypes
        .first()
        .expect("lower should produce one notetype");
    assert_eq!(notetype.kind, "cloze");
    assert_eq!(
        notetype.original_stock_kind.as_deref(),
        Some("image_occlusion")
    );

    let note = plan
        .authoring_document
        .notes
        .first()
        .expect("lower should produce one note");
    assert_eq!(
        note.fields.get("Header").map(String::as_str),
        Some("Header")
    );
    assert_eq!(note.tags, vec!["image-tag"]);
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
            std::iter::empty::<&str>(),
        )
        .lower()
        .expect_err("lower should fail");

    assert!(err
        .product_diagnostics
        .iter()
        .any(|d| d.code == "PHASE5A.IO_IMAGE_REQUIRED"));
}

#[test]
fn custom_escape_hatch_lowers_to_explicit_authoring_normal_notetype_shape() {
    let plan = ProductDocument::new("custom-doc")
        .with_custom_notetype(CustomNoteType {
            id: "custom-main".into(),
            name: Some("Custom Normal".into()),
            fields: vec![
                CustomField {
                    name: "Front".into(),
                },
                CustomField {
                    name: "Back".into(),
                },
            ],
            templates: vec![CustomTemplate {
                name: "Card 1".into(),
                question_format: "{{Front}}".into(),
                answer_format: "{{FrontSide}}\n\n<hr id=answer>\n\n{{Back}}".into(),
            }],
            css: Some(".card { color: red; }".into()),
        })
        .add_custom_note(CustomNote {
            id: "note-1".into(),
            note_type_id: "custom-main".into(),
            deck_name: "Default".into(),
            fields: BTreeMap::from([
                ("Front".into(), "front".into()),
                ("Back".into(), "back".into()),
            ]),
            tags: vec![],
        })
        .lower()
        .expect("lower should succeed");

    let notetype = plan
        .authoring_document
        .notetypes
        .first()
        .expect("lower should produce one notetype");
    assert_eq!(notetype.kind, "normal");
    assert_eq!(notetype.css.as_deref(), Some(".card { color: red; }"));

    let fields = notetype.fields.as_ref().expect("explicit custom fields");
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].name, "Front");
    assert_eq!(fields[0].ord, Some(0));
    assert_eq!(fields[1].name, "Back");
    assert_eq!(fields[1].ord, Some(1));

    let templates = notetype
        .templates
        .as_ref()
        .expect("explicit custom templates");
    assert_eq!(templates.len(), 1);
    assert_eq!(templates[0].name, "Card 1");
    assert_eq!(templates[0].ord, Some(0));
    assert_eq!(templates[0].question_format, "{{Front}}");
    assert_eq!(
        templates[0].answer_format,
        "{{FrontSide}}\n\n<hr id=answer>\n\n{{Back}}"
    );
}
