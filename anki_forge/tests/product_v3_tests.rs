use anki_forge::build::BuildOptions;
use anki_forge::product::{ProductDocument, Project};
use anki_forge::writer::inspect_apkg;

#[test]
fn product_v3_custom_cloze_builds_through_product_build_pipeline() {
    let document: ProductDocument = serde_json::from_value(serde_json::json!({
        "product_document_version": "product-v3",
        "document_id": "product-v3-cloze",
        "default_deck_name": "Product v3",
        "note_types": [{
            "kind": "custom",
            "note_type_kind": "cloze",
            "cloze_field": "text",
            "id": "language-cloze",
            "name": "Language Cloze",
            "fields": [
                {"name": "Sentence", "key": "text", "identity": true, "sort": true, "required": true},
                {"name": "Extra", "key": "extra"}
            ],
            "templates": [{
                "name": "Cloze",
                "key": "cloze",
                "front": "{{cloze:Sentence}}",
                "back": "{{cloze:Sentence}}<br>{{Extra}}"
            }],
            "identity": {"kind": "fields", "fields": ["text"]},
            "css": ".cloze { color: #c00; }"
        }],
        "notes": [{
            "kind": "custom",
            "note_type_id": "language-cloze",
            "stable_id": "v3:1",
            "deck_name": "Product v3",
            "fields": {
                "text": {"kind": "text", "value": "{{c1::Madrid}} is in {{c2::Spain}}"},
                "extra": {"kind": "text", "value": "geography"}
            }
        }]
    }))
    .expect("product-v3 document");
    let output = tempfile::tempdir().expect("output");
    let apkg = output.path().join("product-v3.apkg");

    let report = Project::from_product_document(document)
        .build(BuildOptions::new().output(&apkg))
        .expect("product-v3 build");

    assert_eq!(report.counts.cards, 2);
    let inspected = inspect_apkg(&apkg).expect("inspect");
    assert!(inspected
        .observations
        .notetypes
        .iter()
        .any(|value| { value["id"] == "language-cloze" && value["kind"] == "cloze" }));
}

#[test]
fn product_v3_rejects_unknown_template_field_before_writing_apkg() {
    let document: ProductDocument = serde_json::from_value(serde_json::json!({
        "product_document_version": "product-v3",
        "document_id": "product-v3-invalid-template",
        "note_types": [{
            "kind": "custom",
            "note_type_kind": "normal",
            "id": "custom",
            "fields": [{"name": "Front", "key": "front", "identity": true}],
            "templates": [{
                "name": "Card",
                "key": "card",
                "front": "{{TypoField}}",
                "back": "{{Front}}",
                "source_path": "project.note_types[\"custom\"].templates[\"card\"]"
            }]
        }],
        "notes": []
    }))
    .expect("product-v3 document");
    let output = tempfile::tempdir().expect("output");
    let apkg = output.path().join("invalid.apkg");

    let error = Project::from_product_document(document)
        .build(BuildOptions::new().output(&apkg))
        .expect_err("invalid template should fail");

    assert!(error
        .report
        .diagnostic_codes()
        .contains(&"TEMPLATE.RENDER_FIELD_UNKNOWN".to_string()));
    let diagnostic = error
        .report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code.as_str() == "TEMPLATE.RENDER_FIELD_UNKNOWN")
        .expect("template diagnostic");
    assert_eq!(
        diagnostic.source_span().map(|span| span.byte_start),
        Some(0)
    );
    assert!(!apkg.exists());
}

#[test]
fn product_v3_rejects_duplicate_generation_rule_fields() {
    let document: ProductDocument = serde_json::from_value(serde_json::json!({
        "product_document_version": "product-v3",
        "document_id": "duplicate-generation-fields",
        "note_types": [{
            "kind": "custom",
            "note_type_kind": "normal",
            "id": "custom",
            "fields": [{"name": "Front", "key": "front", "identity": true}],
            "templates": [{
                "name": "Card",
                "key": "card",
                "front": "{{Front}}",
                "back": "{{Front}}",
                "generation_rule": {"kind": "all", "fields": ["front", "front"]}
            }]
        }],
        "notes": []
    }))
    .expect("product-v3 document");
    let output = tempfile::tempdir().expect("output");
    let apkg = output.path().join("invalid.apkg");

    let error = Project::from_product_document(document)
        .build(BuildOptions::new().output(&apkg))
        .expect_err("duplicate generation fields must fail");

    assert!(error
        .report
        .diagnostic_codes()
        .contains(&"PRODUCT.GENERATION_RULE_INVALID".to_string()));
    assert!(!apkg.exists());
}

#[test]
fn product_v3_validates_browser_template_fields() {
    let document: ProductDocument = serde_json::from_value(serde_json::json!({
        "product_document_version": "product-v3",
        "document_id": "product-v3-invalid-browser-template",
        "note_types": [{
            "kind": "custom",
            "note_type_kind": "normal",
            "id": "custom",
            "fields": [{"name": "Back Extra", "key": "back_extra", "identity": true}],
            "templates": [{
                "name": "Card",
                "key": "card",
                "front": "{{Back Extra}}",
                "back": "{{FrontSide}}",
                "browser_front": "{{Back Extr}}"
            }]
        }],
        "notes": []
    }))
    .expect("product-v3 document");
    let output = tempfile::tempdir().expect("output");
    let apkg = output.path().join("invalid-browser.apkg");

    let error = Project::from_product_document(document)
        .build(BuildOptions::new().output(&apkg))
        .expect_err("invalid browser template should fail");

    assert!(error
        .report
        .diagnostic_codes()
        .contains(&"TEMPLATE.RENDER_FIELD_UNKNOWN".to_string()));
    assert!(!apkg.exists());
}

#[test]
fn product_v2_custom_cloze_generation_rule_requires_product_v3() {
    let document: ProductDocument = serde_json::from_value(serde_json::json!({
        "product_document_version": "product-v2",
        "document_id": "product-v2-explicit-cloze-rule",
        "note_types": [{
            "kind": "custom",
            "id": "custom",
            "fields": [{"name": "Text", "key": "text", "identity": true}],
            "templates": [{
                "name": "Card",
                "key": "card",
                "front": "{{cloze:Text}}",
                "back": "{{cloze:Text}}",
                "generation_rule": {"kind": "cloze", "field": "text"}
            }]
        }],
        "notes": []
    }))
    .expect("product-v2 document");
    let output = tempfile::tempdir().expect("output");
    let apkg = output.path().join("invalid-v2-cloze.apkg");

    let error = Project::from_product_document(document)
        .build(BuildOptions::new().output(&apkg))
        .expect_err("product-v2 custom Cloze rule should fail");

    assert!(error
        .report
        .diagnostic_codes()
        .contains(&"TEMPLATE.CLOZE_RULE_REQUIRES_CLOZE_NOTETYPE".to_string()));
    assert!(!apkg.exists());
}

#[test]
fn product_v3_rejects_incomplete_or_unknown_custom_notetype_kinds() {
    for (note_type_kind, cloze_field, expected_code) in [
        ("cloze", None, "PRODUCT.CLOZE_FIELD_REQUIRED"),
        ("cloze", Some("missing"), "PRODUCT.CLOZE_FIELD_UNKNOWN"),
        ("mystery", None, "PRODUCT.NOTE_TYPE_KIND_UNSUPPORTED"),
    ] {
        let document: ProductDocument = serde_json::from_value(serde_json::json!({
            "product_document_version": "product-v3",
            "document_id": format!("invalid-{note_type_kind}"),
            "note_types": [{
                "kind": "custom",
                "note_type_kind": note_type_kind,
                "cloze_field": cloze_field,
                "id": "custom",
                "fields": [{"name": "Text", "key": "text", "identity": true}],
                "templates": [{
                    "name": "Card",
                    "key": "card",
                    "front": "{{cloze:Text}}",
                    "back": "{{cloze:Text}}"
                }]
            }],
            "notes": []
        }))
        .expect("product-v3 document");
        let output = tempfile::tempdir().expect("output");
        let apkg = output.path().join("invalid.apkg");

        let error = Project::from_product_document(document)
            .build(BuildOptions::new().output(&apkg))
            .expect_err("invalid custom note type kind should fail");

        assert!(
            error
                .report
                .diagnostic_codes()
                .contains(&expected_code.to_string()),
            "missing {expected_code}: {:?}",
            error.report.diagnostic_codes()
        );
        assert!(!apkg.exists());
    }
}

#[test]
fn product_v3_rejects_cloze_rule_on_normal_custom_notetype() {
    let document: ProductDocument = serde_json::from_value(serde_json::json!({
        "product_document_version": "product-v3",
        "document_id": "normal-with-cloze-rule",
        "note_types": [{
            "kind": "custom",
            "note_type_kind": "normal",
            "id": "custom",
            "fields": [{"name": "Text", "key": "text", "identity": true}],
            "templates": [{
                "name": "Card",
                "key": "card",
                "front": "{{cloze:Text}}",
                "back": "{{cloze:Text}}",
                "generation_rule": {"kind": "cloze", "field": "text"}
            }]
        }],
        "notes": []
    }))
    .expect("product-v3 document");
    let output = tempfile::tempdir().expect("output");

    let error = Project::from_product_document(document)
        .build(BuildOptions::new().output(output.path().join("invalid.apkg")))
        .expect_err("normal note types must reject Cloze generation rules");

    assert!(error
        .report
        .diagnostic_codes()
        .contains(&"TEMPLATE.CLOZE_RULE_REQUIRES_CLOZE_NOTETYPE".to_string()));
}

#[test]
fn product_v3_preserves_browser_templates_and_target_deck() {
    let document: ProductDocument = serde_json::from_value(serde_json::json!({
        "product_document_version": "product-v3",
        "document_id": "product-v3-template-metadata",
        "note_types": [{
            "kind": "custom",
            "note_type_kind": "normal",
            "id": "custom",
            "fields": [{"name": "Front", "key": "front", "identity": true}],
            "templates": [{
                "name": "Card",
                "key": "card",
                "front": "{{Front}}",
                "back": "{{FrontSide}}",
                "browser_front": "{{text:Front}}",
                "browser_back": "{{Front}}",
                "target_deck": "Languages::Spanish"
            }]
        }],
        "notes": [{
            "kind": "custom",
            "note_type_id": "custom",
            "stable_id": "custom:1",
            "deck_name": "Default",
            "fields": {"front": {"kind": "text", "value": "hola"}}
        }]
    }))
    .expect("product-v3 document");
    let output = tempfile::tempdir().expect("output");
    let apkg = output.path().join("metadata.apkg");

    Project::from_product_document(document)
        .build(BuildOptions::new().output(&apkg))
        .expect("product-v3 build");

    let inspected = inspect_apkg(&apkg).expect("inspect");
    assert!(inspected
        .observations
        .browser_templates
        .iter()
        .any(|value| {
            value["notetype_id"] == "custom"
                && value["browser_question_format"] == "{{text:Front}}"
                && value["browser_answer_format"] == "{{Front}}"
        }));
    assert!(inspected
        .observations
        .template_target_decks
        .iter()
        .any(|value| value["notetype_id"] == "custom"
            && value["target_deck_name"] == "Languages::Spanish"));
}

#[test]
fn product_v2_does_not_silently_enable_custom_cloze_semantics() {
    let document: ProductDocument = serde_json::from_value(serde_json::json!({
        "product_document_version": "product-v2",
        "document_id": "product-v2-remains-normal",
        "note_types": [{
            "kind": "custom",
            "note_type_kind": "cloze",
            "cloze_field": "text",
            "id": "legacy-custom",
            "fields": [{"name": "Text", "key": "text", "identity": true}],
            "templates": [{
                "name": "Card",
                "key": "card",
                "front": "{{cloze:Text}}",
                "back": "{{cloze:Text}}"
            }],
            "identity": {"kind": "fields", "fields": ["text"]}
        }],
        "notes": [{
            "kind": "custom",
            "note_type_id": "legacy-custom",
            "stable_id": "legacy:1",
            "deck_name": "Legacy",
            "fields": {
                "text": {"kind": "text", "value": "{{c1::one}} {{c2::two}}"}
            }
        }]
    }))
    .expect("product-v2 document");
    let output = tempfile::tempdir().expect("output");
    let apkg = output.path().join("legacy.apkg");

    let report = Project::from_product_document(document)
        .build(BuildOptions::new().output(&apkg))
        .expect("product-v2 build");

    assert_eq!(report.counts.cards, 1);
    let inspected = inspect_apkg(&apkg).expect("inspect");
    assert!(inspected
        .observations
        .notetypes
        .iter()
        .any(|value| value["id"] == "legacy-custom" && value["kind"] == "normal"));
}

#[test]
fn product_v3_unknown_filter_builds_with_a_structured_warning() {
    let document: ProductDocument = serde_json::from_value(serde_json::json!({
        "product_document_version": "product-v3",
        "document_id": "product-v3-filter-warning",
        "note_types": [{
            "kind": "custom",
            "id": "custom",
            "fields": [{"name": "Front", "key": "front", "identity": true}],
            "templates": [{
                "name": "Card",
                "key": "card",
                "front": "{{addon_filter:Front}}",
                "back": "{{Front}}",
                "source_path": "project.note_types[\"custom\"].templates[\"card\"]"
            }],
            "identity": {"kind": "fields", "fields": ["front"]}
        }],
        "notes": [{
            "kind": "custom",
            "note_type_id": "custom",
            "stable_id": "custom:1",
            "deck_name": "Warnings",
            "fields": {"front": {"kind": "text", "value": "portable"}}
        }]
    }))
    .expect("product-v3 document");
    let output = tempfile::tempdir().expect("output");
    let apkg = output.path().join("warning.apkg");

    let report = Project::from_product_document(document)
        .build(BuildOptions::new().output(&apkg))
        .expect("warning-only build succeeds");
    let diagnostic = report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code.as_str() == "TEMPLATE.FILTER_UNKNOWN")
        .expect("filter warning");

    assert_eq!(
        diagnostic.severity,
        anki_forge::diagnostics::Severity::Warning
    );
    assert_eq!(
        diagnostic.source.as_ref().map(|source| source.as_str()),
        Some("project.note_types[\"custom\"].templates[\"card\"].front")
    );
    assert!(apkg.exists());
}
