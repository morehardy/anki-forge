use anki_forge::product::model::{CustomField, CustomNoteType, CustomTemplate};
use anki_forge::product::ProductDocument;
use anki_forge::AuthoringMediaSource;
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
        .bundle_inline_template_asset("custom-main", "demo.woff2", "font/woff2", "aGVsbG8=")
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
    assert!(media.desired_filename.starts_with("_custom-main_"));
    assert_eq!(media.declared_mime.as_deref(), Some("font/woff2"));
    assert!(matches!(
        &media.source,
        AuthoringMediaSource::InlineBytes { data_base64 } if data_base64 == "aGVsbG8="
    ));

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
        css.contains(&media.desired_filename),
        "font-face css should reference lowered asset filename"
    );

    assert!(
        plan.mappings
            .iter()
            .any(|mapping| mapping.source_kind == "asset"),
        "asset lowering should emit a bundled asset mapping"
    );
}

#[test]
fn inline_font_asset_lowering_escapes_css_literals() {
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
            "custom-main",
            "font'slash\\name.woff2",
            "font/woff2",
            "aGVsbG8=",
        )
        .bind_font("custom-main", "O'Brien \\ Mono", "font'slash\\name.woff2")
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

    let notetype = plan
        .authoring_document
        .notetypes
        .iter()
        .find(|notetype| notetype.id == "custom-main")
        .expect("custom notetype should lower");
    let css = notetype.css.as_deref().expect("lowered css");

    assert!(css.contains("font-family: 'O\\'Brien \\\\ Mono'"));
    assert!(css.contains("src: url('"));
    assert!(css.contains(".woff2"));
}

#[test]
fn font_binding_resolves_namespaced_asset_and_reports_missing_bindings() {
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
            css: None,
        })
        .bundle_inline_template_asset("custom-main", "shared.woff2", "font/woff2", "aGVsbG8=")
        .bundle_inline_template_asset("other-main", "shared.woff2", "font/woff2", "d29ybGQ=")
        .bind_font("custom-main", "Demo Font", "shared.woff2")
        .bind_font("missing-main", "Missing Font", "missing.woff2")
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

    assert_eq!(plan.authoring_document.media.len(), 2);

    let notetype = plan
        .authoring_document
        .notetypes
        .iter()
        .find(|notetype| notetype.id == "custom-main")
        .expect("custom notetype should lower");
    let css = notetype.css.as_deref().expect("lowered css");
    let chosen_media = plan
        .authoring_document
        .media
        .iter()
        .find(|media| {
            matches!(
                &media.source,
                AuthoringMediaSource::InlineBytes { data_base64 } if data_base64 == "aGVsbG8="
            )
        })
        .expect("custom-main asset should lower");
    let other_media = plan
        .authoring_document
        .media
        .iter()
        .find(|media| {
            matches!(
                &media.source,
                AuthoringMediaSource::InlineBytes { data_base64 } if data_base64 == "d29ybGQ="
            )
        })
        .expect("other-main asset should lower");

    assert!(css.contains(&chosen_media.desired_filename));
    assert!(!css.contains(&other_media.desired_filename));
    assert!(
        plan.lowering_diagnostics
            .iter()
            .any(|diag| diag.code == "PHASE5A.FONT_BINDING_UNKNOWN_NOTETYPE"),
        "missing note type binding should emit a lowering diagnostic"
    );
}
