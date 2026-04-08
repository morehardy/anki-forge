use anki_forge::product::model::{CustomField, CustomNoteType, CustomTemplate};
use anki_forge::product::ProductDocument;
use std::collections::BTreeMap;

#[test]
fn inline_font_asset_lowers_to_media_and_font_face_css() {
    let plan = ProductDocument::new("demo-doc")
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
        .bundle_inline_template_asset(
            "fonts",
            "demo.woff2",
            "font/woff2",
            "aGVsbG8=",
        )
        .bind_font("custom-main", "Demo Font", "demo.woff2")
        .add_custom_note(anki_forge::product::model::CustomNote {
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

    assert_eq!(plan.authoring_document.media.len(), 1);
    let media = &plan.authoring_document.media[0];
    assert!(media.filename.starts_with("_fonts_"));
    assert_eq!(media.mime, "font/woff2");
    assert_eq!(media.data_base64, "aGVsbG8=");

    let notetype = plan
        .authoring_document
        .notetypes
        .iter()
        .find(|notetype| notetype.id == "custom-main")
        .expect("custom notetype should lower");
    let css = notetype.css.as_deref().expect("lowered css");
    assert!(css.contains("@font-face"));
    assert!(css.contains("Demo Font"));
    assert!(
        css.contains(&media.filename),
        "font-face css should reference lowered asset filename"
    );

    assert!(
        plan.mappings
            .iter()
            .any(|mapping| mapping.source_kind == "asset"),
        "asset lowering should emit a bundled asset mapping"
    );
}
