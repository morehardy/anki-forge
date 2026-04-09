use authoring_core::{
    normalize, AuthoringDocument, AuthoringField, AuthoringFieldMetadata, AuthoringNote,
    AuthoringNotetype, AuthoringTemplate, ComparisonContext, NormalizationRequest,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;

fn assert_json_object_has_keys(value: &Value, keys: &[&str]) {
    let object = value.as_object().expect("expected JSON object");
    for key in keys {
        assert!(object.contains_key(*key), "missing key {key}");
    }
}

fn request_from_json(value: Value) -> NormalizationRequest {
    serde_json::from_value(value).expect("deserialize normalization request")
}

fn string_map(value: Value) -> std::collections::BTreeMap<String, String> {
    serde_json::from_value(value).expect("deserialize string map")
}

fn normalized_field_names(fields: &[authoring_core::NormalizedField]) -> Vec<&str> {
    fields.iter().map(|field| field.name.as_str()).collect()
}

#[test]
fn missing_document_id_returns_invalid_result_with_diagnostics() {
    let input = AuthoringDocument {
        kind: "authoring-document".into(),
        schema_version: "1.0".into(),
        metadata_document_id: "   ".into(),
        notetypes: vec![],
        notes: vec![],
        media: vec![],
    };

    let result = normalize(NormalizationRequest::new(input));
    let json = serde_json::to_value(result).expect("serialize normalization result");

    assert_eq!(json["result_status"], "invalid");
    assert_eq!(json["diagnostics"]["status"], "invalid");
    assert!(
        !json["diagnostics"]["items"]
            .as_array()
            .expect("diagnostics items array")
            .is_empty(),
        "expected at least one diagnostic"
    );
    assert!(json["normalized_ir"].is_null());
    assert!(json["merge_risk_report"].is_null());
}

#[test]
fn success_returns_normalized_ir_and_contract_envelope_keys() {
    let input = AuthoringDocument {
        kind: "authoring-document".into(),
        schema_version: "1.0".into(),
        metadata_document_id: "doc-123".into(),
        notetypes: vec![],
        notes: vec![],
        media: vec![],
    };

    let result = normalize(NormalizationRequest::new(input));
    let json = serde_json::to_value(result).expect("serialize normalization result");

    assert_eq!(json["kind"], "normalization-result");
    assert_eq!(json["result_status"], "success");
    assert_eq!(json["tool_contract_version"], "phase2-v1");
    assert_json_object_has_keys(
        &json,
        &[
            "kind",
            "result_status",
            "tool_contract_version",
            "policy_refs",
            "comparison_context",
            "diagnostics",
        ],
    );
    assert!(json["normalized_ir"].is_object());
    assert!(!json["normalized_ir"]["resolved_identity"]
        .as_str()
        .expect("resolved_identity string")
        .trim()
        .is_empty());
}

#[test]
fn success_without_comparison_context_serializes_null_comparison_and_merge_report() {
    let input = AuthoringDocument {
        kind: "authoring-document".into(),
        schema_version: "1.0".into(),
        metadata_document_id: "doc-456".into(),
        notetypes: vec![],
        notes: vec![],
        media: vec![],
    };

    let result = normalize(NormalizationRequest::new(input));
    let json = serde_json::to_value(result).expect("serialize normalization result");

    assert!(json["comparison_context"].is_null());
    assert!(json["merge_risk_report"].is_null());
}

#[test]
fn success_with_comparison_context_preserves_risk_policy_ref() {
    let input = AuthoringDocument {
        kind: "authoring-document".into(),
        schema_version: "1.0".into(),
        metadata_document_id: "doc-789".into(),
        notetypes: vec![],
        notes: vec![],
        media: vec![],
    };
    let mut request = NormalizationRequest::new(input);
    request.comparison_context = Some(ComparisonContext {
        kind: "comparison-context".into(),
        baseline_kind: "normalized_ir".into(),
        baseline_artifact_fingerprint: "baseline-fingerprint".into(),
        risk_policy_ref: "risk-policy.default@1.0.0".into(),
        comparison_mode: "strict".into(),
    });

    let result = normalize(request);
    let json = serde_json::to_value(result).expect("serialize normalization result");

    assert_eq!(
        json["policy_refs"]["risk_policy_ref"],
        "risk-policy.default@1.0.0"
    );
    assert!(json["comparison_context"].is_object());
    assert!(json["merge_risk_report"].is_object());
    assert_eq!(json["merge_risk_report"]["comparison_status"], "complete");
    assert_eq!(json["merge_risk_report"]["overall_level"], "low");
    assert_eq!(
        json["merge_risk_report"]["policy_version"],
        "risk-policy.default@1.0.0"
    );
    assert_eq!(
        json["merge_risk_report"]["baseline_artifact_fingerprint"],
        "baseline-fingerprint"
    );
    assert!(!json["merge_risk_report"]["current_artifact_fingerprint"]
        .as_str()
        .expect("current_artifact_fingerprint string")
        .trim()
        .is_empty());
}

#[test]
fn random_override_emits_warning_success_and_random_identity_prefix() {
    let request = request_from_json(json!({
        "input": {
            "kind": "authoring-document",
            "schema_version": "1.0",
            "metadata_document_id": "doc-random",
            "notetypes": [],
            "notes": [],
            "media": []
        },
        "identity_override_mode": "random",
        "reason_code": "manual_randomization"
    }));

    let result = normalize(request);

    assert_eq!(result.result_status, "success");
    assert!(result
        .diagnostics
        .items
        .iter()
        .any(|item| item.code == "PHASE2.IDENTITY_RANDOM_OVERRIDE"));
    assert!(result
        .normalized_ir
        .as_ref()
        .expect("normalized ir")
        .resolved_identity
        .starts_with("rnd:"));
}

#[test]
fn missing_reason_code_for_override_is_invalid() {
    let request = request_from_json(json!({
        "input": {
            "kind": "authoring-document",
            "schema_version": "1.0",
            "metadata_document_id": "doc-external",
            "notetypes": [],
            "notes": [],
            "media": []
        },
        "identity_override_mode": "external"
    }));

    let result = normalize(request);

    assert_eq!(result.result_status, "invalid");
    assert!(result
        .diagnostics
        .items
        .iter()
        .any(|item| item.code == "PHASE2.REASON_CODE_REQUIRED"));
}

#[test]
fn random_override_missing_reason_code_is_invalid_with_reason_code_diagnostic() {
    let request = request_from_json(json!({
        "input": {
            "kind": "authoring-document",
            "schema_version": "1.0",
            "metadata_document_id": "doc-random-missing-reason",
            "notetypes": [],
            "notes": [],
            "media": []
        },
        "identity_override_mode": "random"
    }));

    let result = normalize(request);

    assert_eq!(result.result_status, "invalid");
    assert!(result
        .diagnostics
        .items
        .iter()
        .any(|item| item.code == "PHASE2.REASON_CODE_REQUIRED"));
}

#[test]
fn external_override_missing_external_id_is_invalid() {
    let request = request_from_json(json!({
        "input": {
            "kind": "authoring-document",
            "schema_version": "1.0",
            "metadata_document_id": "doc-external",
            "notetypes": [],
            "notes": [],
            "media": []
        },
        "identity_override_mode": "external",
        "reason_code": "preserve_external_identity"
    }));

    let result = normalize(request);

    assert_eq!(result.result_status, "invalid");
    assert!(result
        .diagnostics
        .items
        .iter()
        .any(|item| item.code == "PHASE2.EXTERNAL_ID_REQUIRED"));
}

#[test]
fn unmatched_target_selector_is_invalid() {
    let request = request_from_json(json!({
        "input": {
            "kind": "authoring-document",
            "schema_version": "1.0",
            "metadata_document_id": "doc-selector",
            "notetypes": [],
            "notes": [],
            "media": []
        },
        "target_selector": "note[id='missing']"
    }));

    let result = normalize(request);

    assert_eq!(result.result_status, "invalid");
    assert!(result
        .diagnostics
        .items
        .iter()
        .any(|item| item.code == "PHASE2.SELECTOR_UNMATCHED"));
}

#[test]
fn basic_authoring_input_expands_to_resolved_basic_notetype() {
    let request = request_from_json(json!({
        "input": {
            "kind": "authoring-ir",
            "schema_version": "0.1.0",
            "metadata_document_id": "demo-doc",
            "notetypes": [
                {
                    "id": "basic-main",
                    "kind": "normal",
                    "original_stock_kind": "basic",
                    "name": "Basic"
                }
            ],
            "notes": [
                {
                    "id": "note-1",
                    "notetype_id": "basic-main",
                    "deck_name": "Default",
                    "fields": {
                        "Front": "front",
                        "Back": "back"
                    },
                    "tags": ["demo"]
                }
            ],
            "media": []
        },
    }));

    let result = normalize(request);
    let normalized = result.normalized_ir.expect("normalized_ir");

    assert_eq!(normalized.notetypes.len(), 1);
    assert_eq!(normalized.notes.len(), 1);
    assert!(normalized.media.is_empty());

    let notetype = &normalized.notetypes[0];
    assert_eq!(notetype.id, "basic-main");
    assert_eq!(notetype.kind, "normal");
    assert_eq!(notetype.original_stock_kind.as_deref(), Some("basic"));
    assert_eq!(notetype.name, "Basic");
    assert_eq!(
        normalized_field_names(&notetype.fields),
        vec!["Front", "Back"]
    );
    assert_eq!(notetype.templates.len(), 1);
    assert_eq!(notetype.templates[0].name, "Card 1");
    assert_eq!(notetype.templates[0].question_format, "{{Front}}");
    assert_eq!(
        notetype.templates[0].answer_format,
        "{{FrontSide}}\n\n<hr id=answer>\n\n{{Back}}"
    );
    assert_eq!(notetype.css, "");

    let note = &normalized.notes[0];
    assert_eq!(note.id, "note-1");
    assert_eq!(note.notetype_id, "basic-main");
    assert_eq!(note.deck_name, "Default");
    assert_eq!(
        note.fields,
        string_map(json!({
            "Front": "front",
            "Back": "back"
        }))
    );
    assert_eq!(note.tags, vec!["demo"]);
}

#[test]
fn cloze_authoring_input_expands_to_source_grounded_cloze_template() {
    let request = request_from_json(json!({
        "input": {
            "kind": "authoring-ir",
            "schema_version": "0.1.0",
            "metadata_document_id": "demo-doc",
            "notetypes": [
                {
                    "id": "cloze-main",
                    "kind": "cloze",
                    "name": "Cloze"
                }
            ],
            "notes": [
                {
                    "id": "note-1",
                    "notetype_id": "cloze-main",
                    "deck_name": "Default",
                    "fields": {
                        "Text": "A {{c1::cloze}} example",
                        "Back Extra": "extra"
                    },
                    "tags": []
                }
            ],
            "media": []
        },
    }));

    let result = normalize(request);
    let normalized = result.normalized_ir.expect("normalized_ir");

    assert_eq!(normalized.notetypes.len(), 1);
    let notetype = &normalized.notetypes[0];
    assert_eq!(notetype.id, "cloze-main");
    assert_eq!(notetype.kind, "cloze");
    assert_eq!(notetype.name, "Cloze");
    assert_eq!(
        normalized_field_names(&notetype.fields),
        vec!["Text", "Back Extra"]
    );
    assert_eq!(notetype.templates.len(), 1);
    assert_eq!(notetype.templates[0].name, "Cloze");
    assert_eq!(notetype.templates[0].question_format, "{{cloze:Text}}");
    assert_eq!(
        notetype.templates[0].answer_format,
        "{{cloze:Text}}<br>\n{{Back Extra}}"
    );
    assert_eq!(
        notetype.css,
        ".cloze {\n    font-weight: bold;\n    color: blue;\n}\n.nightMode .cloze {\n    color: lightblue;\n}\n"
    );
}

#[test]
fn image_occlusion_lane_uses_source_grounded_fields_and_css() {
    let request = request_from_json(json!({
        "input": {
            "kind": "authoring-ir",
            "schema_version": "0.1.0",
            "metadata_document_id": "demo-doc",
            "notetypes": [
                {
                    "id": "io-main",
                    "kind": "cloze",
                    "original_stock_kind": "image_occlusion",
                    "name": "Image Occlusion"
                }
            ],
            "notes": [
                {
                    "id": "note-1",
                    "notetype_id": "io-main",
                    "deck_name": "Default",
                    "fields": {
                        "Occlusion": "mask",
                        "Image": "<img src=\"mask.png\">",
                        "Header": "header",
                        "Back Extra": "extra",
                        "Comments": "comment"
                    },
                    "tags": ["demo"]
                }
            ],
            "media": [
                {
                    "filename": "mask.png",
                    "mime": "image/png",
                    "data_base64": "MQ=="
                }
            ]
        },
    }));

    let result = normalize(request);
    let normalized = result.normalized_ir.expect("normalized_ir");

    assert_eq!(normalized.notetypes.len(), 1);
    assert_eq!(normalized.notes.len(), 1);
    assert_eq!(normalized.media.len(), 1);

    let notetype = &normalized.notetypes[0];
    assert_eq!(notetype.id, "io-main");
    assert_eq!(notetype.kind, "cloze");
    assert_eq!(
        notetype.original_stock_kind.as_deref(),
        Some("image_occlusion")
    );
    assert_eq!(notetype.name, "Image Occlusion");
    assert_eq!(
        normalized_field_names(&notetype.fields),
        vec!["Occlusion", "Image", "Header", "Back Extra", "Comments"]
    );
    assert_eq!(notetype.templates.len(), 1);
    assert_eq!(notetype.templates[0].name, "Image Occlusion");
    assert!(notetype.templates[0]
        .question_format
        .contains("{{cloze:Occlusion}}"));
    assert!(notetype.templates[0]
        .question_format
        .contains("anki.imageOcclusion.setup();"));
    assert!(notetype.templates[0]
        .question_format
        .contains("id=\"image-occlusion-container\""));
    assert!(notetype.templates[0]
        .answer_format
        .contains("{{#Back Extra}}<div>{{Back Extra}}</div>{{/Back Extra}}"));
    assert_eq!(
        notetype.css,
        "#image-occlusion-canvas {\n    --inactive-shape-color: #ffeba2;\n    --active-shape-color: #ff8e8e;\n    --inactive-shape-border: 1px #212121;\n    --active-shape-border: 1px #212121;\n    --highlight-shape-color: #ff8e8e00;\n    --highlight-shape-border: 1px #ff8e8e;\n}\n\n.card {\n    font-family: arial;\n    font-size: 20px;\n    text-align: center;\n    color: black;\n    background-color: white;\n}\n"
    );

    let note = &normalized.notes[0];
    assert_eq!(note.id, "note-1");
    assert_eq!(note.notetype_id, "io-main");
    assert_eq!(note.tags, vec!["demo"]);
    assert_eq!(
        note.fields,
        string_map(json!({
            "Occlusion": "mask",
            "Image": "<img src=\"mask.png\">",
            "Header": "header",
            "Back Extra": "extra",
            "Comments": "comment"
        }))
    );

    let media = &normalized.media[0];
    assert_eq!(media.filename, "mask.png");
    assert_eq!(media.mime, "image/png");
    assert_eq!(media.data_base64, "MQ==");
}

#[test]
fn unknown_notetype_id_returns_invalid_result() {
    let request = request_from_json(json!({
        "input": {
            "kind": "authoring-ir",
            "schema_version": "0.1.0",
            "metadata_document_id": "demo-doc",
            "notetypes": [
                {
                    "id": "basic-main",
                    "kind": "basic",
                    "name": "Basic"
                }
            ],
            "notes": [
                {
                    "id": "note-1",
                    "notetype_id": "missing-main",
                    "deck_name": "Default",
                    "fields": {
                        "Front": "front",
                        "Back": "back"
                    },
                    "tags": []
                }
            ],
            "media": []
        }
    }));

    let result = normalize(request);

    assert_eq!(result.result_status, "invalid");
    assert!(result
        .diagnostics
        .items
        .iter()
        .any(|item| item.code == "PHASE3.UNKNOWN_NOTETYPE_ID"));
}

#[test]
fn unexpected_extra_field_on_stock_note_returns_invalid_result() {
    let request = request_from_json(json!({
        "input": {
            "kind": "authoring-ir",
            "schema_version": "0.1.0",
            "metadata_document_id": "demo-doc",
            "notetypes": [
                {
                    "id": "basic-main",
                    "kind": "basic",
                    "name": "Basic"
                }
            ],
            "notes": [
                {
                    "id": "note-1",
                    "notetype_id": "basic-main",
                    "deck_name": "Default",
                    "fields": {
                        "Front": "front",
                        "Back": "back",
                        "Hint": "unexpected"
                    },
                    "tags": []
                }
            ],
            "media": []
        }
    }));

    let result = normalize(request);

    assert_eq!(result.result_status, "invalid");
    assert!(result
        .diagnostics
        .items
        .iter()
        .any(|item| item.code == "PHASE3.NOTE_FIELD_MISMATCH"));
}

#[test]
fn duplicate_notetype_ids_return_invalid_result() {
    let request = request_from_json(json!({
        "input": {
            "kind": "authoring-ir",
            "schema_version": "0.1.0",
            "metadata_document_id": "demo-doc",
            "notetypes": [
                {
                    "id": "dup-main",
                    "kind": "basic",
                    "name": "Basic"
                },
                {
                    "id": "dup-main",
                    "kind": "cloze",
                    "name": "Cloze"
                }
            ],
            "notes": [],
            "media": []
        }
    }));

    let result = normalize(request);

    assert_eq!(result.result_status, "invalid");
    assert!(result
        .diagnostics
        .items
        .iter()
        .any(|item| item.code == "PHASE3.DUPLICATE_NOTETYPE_ID"));
}

#[test]
fn explicit_lowered_notetype_identities_and_io_config_survive_normalization() {
    let input = AuthoringDocument {
        kind: "authoring-ir".into(),
        schema_version: "0.1.0".into(),
        metadata_document_id: "explicit-io-doc".into(),
        notetypes: vec![AuthoringNotetype {
            id: "io-main".into(),
            kind: "cloze".into(),
            name: Some("Image Occlusion".into()),
            original_stock_kind: Some("image_occlusion".into()),
            original_id: Some(1729000000),
            fields: Some(vec![
                AuthoringField {
                    name: "Occlusion".into(),
                    ord: Some(0),
                    config_id: Some(1101),
                    tag: Some(1),
                    prevent_deletion: true,
                },
                AuthoringField {
                    name: "Image".into(),
                    ord: Some(1),
                    config_id: Some(1102),
                    tag: Some(2),
                    prevent_deletion: true,
                },
            ]),
            templates: Some(vec![AuthoringTemplate {
                name: "Image Occlusion".into(),
                ord: Some(0),
                config_id: Some(2101),
                question_format: "{{cloze:Occlusion}}".into(),
                answer_format: "{{cloze:Occlusion}}<br>{{Image}}".into(),
                browser_question_format: Some("{{Image}}".into()),
                browser_answer_format: Some("{{Image}}<hr>{{Header}}".into()),
                target_deck_name: Some("Target Deck".into()),
                browser_font_name: Some("Arial".into()),
                browser_font_size: Some(18),
            }]),
            css: Some(".card { color: black; }".into()),
            field_metadata: vec![AuthoringFieldMetadata {
                field_name: "Occlusion".into(),
                label: Some("Mask".into()),
                role_hint: Some("occlusion-mask".into()),
            }],
        }],
        notes: vec![AuthoringNote {
            id: "note-1".into(),
            notetype_id: "io-main".into(),
            deck_name: "Default".into(),
            fields: BTreeMap::from([
                ("Occlusion".into(), "mask".into()),
                ("Image".into(), "<img src=\"mask.png\">".into()),
            ]),
            tags: vec!["demo".into()],
        }],
        media: vec![],
    };

    let result = normalize(NormalizationRequest::new(input));
    let normalized = result.normalized_ir.expect("normalized_ir");
    let notetype = normalized.notetypes.first().expect("normalized notetype");

    assert_eq!(notetype.kind, "cloze");
    assert_eq!(
        notetype.original_stock_kind.as_deref(),
        Some("image_occlusion")
    );
    assert_eq!(notetype.original_id, Some(1729000000));

    assert_eq!(notetype.fields.len(), 2);
    assert_eq!(notetype.fields[0].name, "Occlusion");
    assert_eq!(notetype.fields[0].ord, Some(0));
    assert_eq!(notetype.fields[0].config_id, Some(1101));
    assert_eq!(notetype.fields[0].tag, Some(1));
    assert!(notetype.fields[0].prevent_deletion);

    assert_eq!(notetype.templates.len(), 1);
    assert_eq!(notetype.templates[0].ord, Some(0));
    assert_eq!(notetype.templates[0].config_id, Some(2101));
    assert_eq!(
        notetype.templates[0].target_deck_name.as_deref(),
        Some("Target Deck")
    );
    assert_eq!(
        notetype.templates[0].browser_font_name.as_deref(),
        Some("Arial")
    );
    assert_eq!(notetype.templates[0].browser_font_size, Some(18));

    assert_eq!(notetype.field_metadata.len(), 1);
    assert_eq!(notetype.field_metadata[0].field_name, "Occlusion");
    assert_eq!(
        notetype.field_metadata[0].role_hint.as_deref(),
        Some("occlusion-mask")
    );
}
