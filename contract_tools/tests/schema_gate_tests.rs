use contract_tools::{
    contract_manifest_path,
    manifest::{load_manifest, resolve_asset_path},
    schema::{load_schema, run_schema_gates, validate_value},
};
use serde_json::json;
use serde_json::Value;
use std::fs;

#[test]
fn authoring_ir_schema_accepts_the_minimal_valid_shape() {
    let manifest = load_manifest(contract_manifest_path()).unwrap();
    let schema =
        load_schema(resolve_asset_path(&manifest, "authoring_ir_schema").unwrap()).unwrap();
    let value = json!({
        "kind": "authoring-ir",
        "schema_version": "0.1.0",
        "metadata": { "document_id": "demo-doc" },
        "notetypes": [],
        "notes": []
    });

    assert!(validate_value(&schema, &value).is_ok());
}

#[test]
fn authoring_ir_schema_accepts_stock_notetype_note_and_media_entries() {
    let manifest = load_manifest(contract_manifest_path()).unwrap();
    let schema =
        load_schema(resolve_asset_path(&manifest, "authoring_ir_schema").unwrap()).unwrap();
    let value = json!({
        "kind": "authoring-ir",
        "schema_version": "0.1.0",
        "metadata": { "document_id": "demo-doc" },
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
                    "Back": "back <img src=\"sample.jpg\"> [sound:sample.mp3]"
                },
                "tags": ["demo"]
            }
        ],
        "media": [
            {
                "filename": "sample.jpg",
                "mime": "image/jpeg",
                "data_base64": "MQ=="
            },
            {
                "filename": "sample.mp3",
                "mime": "audio/mpeg",
                "data_base64": "Mg=="
            }
        ]
    });

    assert!(validate_value(&schema, &value).is_ok());
}

#[test]
fn validation_report_schema_requires_a_diagnostics_array() {
    let manifest = load_manifest(contract_manifest_path()).unwrap();
    let schema =
        load_schema(resolve_asset_path(&manifest, "validation_report_schema").unwrap()).unwrap();
    let value = json!({
        "kind": "validation-report",
        "status": "invalid"
    });

    assert!(validate_value(&schema, &value).is_err());
}

#[test]
fn schema_gates_run_against_the_bundled_contract_manifest() {
    run_schema_gates(contract_manifest_path().to_str().unwrap()).unwrap();
}

#[test]
fn manifest_registers_phase2_schema_assets() {
    let manifest = load_manifest(contract_manifest_path()).unwrap();

    for asset_key in [
        "normalized_ir_schema",
        "normalization_diagnostics_schema",
        "comparison_context_schema",
        "merge_risk_report_schema",
        "normalization_result_schema",
        "normalization_semantics",
        "merge_risk_semantics",
    ] {
        assert!(
            resolve_asset_path(&manifest, asset_key).is_ok(),
            "manifest is missing asset key {asset_key}"
        );
    }
}

#[test]
fn normalization_result_schema_allows_null_comparison_context_without_merge_risk_report() {
    let manifest = load_manifest(contract_manifest_path()).unwrap();
    let schema =
        load_schema(resolve_asset_path(&manifest, "normalization_result_schema").unwrap()).unwrap();
    let value = json!({
        "kind": "normalization-result",
        "result_status": "success",
        "tool_contract_version": "phase2-v1",
        "policy_refs": {
            "identity_policy_ref": "identity-policy.default@1.0.0",
            "risk_policy_ref": null
        },
        "comparison_context": null,
        "diagnostics": {
            "kind": "normalization-diagnostics",
            "status": "valid",
            "items": []
        },
        "normalized_ir": writer_ready_normalized_ir_value()
    });

    assert!(validate_value(&schema, &value).is_ok());
}

#[test]
fn normalization_result_schema_allows_omitting_comparison_context_and_merge_risk_report() {
    let manifest = load_manifest(contract_manifest_path()).unwrap();
    let schema =
        load_schema(resolve_asset_path(&manifest, "normalization_result_schema").unwrap()).unwrap();
    let value = json!({
        "kind": "normalization-result",
        "result_status": "success",
        "tool_contract_version": "phase2-v1",
        "policy_refs": {
            "identity_policy_ref": "identity-policy.default@1.0.0",
            "risk_policy_ref": null
        },
        "diagnostics": {
            "kind": "normalization-diagnostics",
            "status": "valid",
            "items": []
        },
        "normalized_ir": writer_ready_normalized_ir_value()
    });

    assert!(validate_value(&schema, &value).is_ok());
}

#[test]
fn normalization_result_schema_requires_merge_risk_report_when_comparison_context_is_present() {
    let manifest = load_manifest(contract_manifest_path()).unwrap();
    let schema =
        load_schema(resolve_asset_path(&manifest, "normalization_result_schema").unwrap()).unwrap();
    let value = json!({
        "kind": "normalization-result",
        "result_status": "success",
        "tool_contract_version": "phase2-v1",
        "policy_refs": {
            "identity_policy_ref": "identity-policy.default@1.0.0",
            "risk_policy_ref": "risk-policy.default@1.0.0"
        },
        "comparison_context": {
            "kind": "comparison-context",
            "baseline_kind": "normalized_ir",
            "baseline_artifact_fingerprint": "baseline-1",
            "risk_policy_ref": "risk-policy.default@1.0.0",
            "comparison_mode": "strict"
        },
        "diagnostics": {
            "kind": "normalization-diagnostics",
            "status": "valid",
            "items": []
        }
    });

    assert!(validate_value(&schema, &value).is_err());
}

#[test]
fn normalization_result_schema_rejects_null_merge_risk_report_when_comparison_context_is_present() {
    let manifest = load_manifest(contract_manifest_path()).unwrap();
    let schema =
        load_schema(resolve_asset_path(&manifest, "normalization_result_schema").unwrap()).unwrap();
    let value = json!({
        "kind": "normalization-result",
        "result_status": "success",
        "tool_contract_version": "phase2-v1",
        "policy_refs": {
            "identity_policy_ref": "identity-policy.default@1.0.0",
            "risk_policy_ref": "risk-policy.default@1.0.0"
        },
        "comparison_context": {
            "kind": "comparison-context",
            "baseline_kind": "normalized_ir",
            "baseline_artifact_fingerprint": "baseline-1",
            "risk_policy_ref": "risk-policy.default@1.0.0",
            "comparison_mode": "strict"
        },
        "diagnostics": {
            "kind": "normalization-diagnostics",
            "status": "valid",
            "items": []
        },
        "merge_risk_report": null
    });

    assert!(validate_value(&schema, &value).is_err());
}

#[test]
fn normalization_result_schema_accepts_valid_merge_risk_report_when_comparison_context_is_present()
{
    let manifest = load_manifest(contract_manifest_path()).unwrap();
    let schema =
        load_schema(resolve_asset_path(&manifest, "normalization_result_schema").unwrap()).unwrap();
    let value = json!({
        "kind": "normalization-result",
        "result_status": "success",
        "tool_contract_version": "phase2-v1",
        "policy_refs": {
            "identity_policy_ref": "identity-policy.default@1.0.0",
            "risk_policy_ref": "risk-policy.default@1.0.0"
        },
        "comparison_context": {
            "kind": "comparison-context",
            "baseline_kind": "normalized_ir",
            "baseline_artifact_fingerprint": "baseline-1",
            "risk_policy_ref": "risk-policy.default@1.0.0",
            "comparison_mode": "strict"
        },
        "diagnostics": {
            "kind": "normalization-diagnostics",
            "status": "valid",
            "items": []
        },
        "normalized_ir": writer_ready_normalized_ir_value(),
        "merge_risk_report": {
            "kind": "merge-risk-report",
            "comparison_status": "complete",
            "overall_level": "low",
            "policy_version": "risk-policy.default@1.0.0",
            "baseline_artifact_fingerprint": "baseline-1",
            "current_artifact_fingerprint": "current-1",
            "comparison_reasons": []
        }
    });

    assert!(validate_value(&schema, &value).is_ok());
}

#[test]
fn normalized_ir_schema_accepts_resolved_writer_ready_shape() {
    let manifest = load_manifest(contract_manifest_path()).unwrap();
    let schema =
        load_schema(resolve_asset_path(&manifest, "normalized_ir_schema").unwrap()).unwrap();
    let value = json!({
        "kind": "normalized-ir",
        "schema_version": "0.1.0",
        "document_id": "demo-doc",
        "resolved_identity": "det:demo-doc",
        "notetypes": [
            {
                "id": "basic-main",
                "kind": "basic",
                "name": "Basic",
                "fields": ["Front", "Back"],
                "templates": [
                    {
                        "name": "Card 1",
                        "question_format": "{{Front}}",
                        "answer_format": "{{FrontSide}}\n\n<hr id=answer>\n\n{{Back}}"
                    }
                ],
                "css": ""
            }
        ],
        "notes": [
            {
                "id": "note-1",
                "notetype_id": "basic-main",
                "deck_name": "Default",
                "fields": {
                    "Front": "front",
                    "Back": "back <img src=\"sample.jpg\"> [sound:sample.mp3]"
                },
                "tags": ["demo"]
            }
        ],
        "media": [
            {
                "filename": "sample.jpg",
                "mime": "image/jpeg",
                "data_base64": "MQ=="
            },
            {
                "filename": "sample.mp3",
                "mime": "audio/mpeg",
                "data_base64": "Mg=="
            }
        ]
    });

    assert!(validate_value(&schema, &value).is_ok());
}

#[test]
fn diagnostic_item_schema_matches_the_validation_report_local_definition() {
    let manifest = load_manifest(contract_manifest_path()).unwrap();
    let diagnostic_item_path = resolve_asset_path(&manifest, "diagnostic_item_schema").unwrap();
    let validation_report_path = resolve_asset_path(&manifest, "validation_report_schema").unwrap();

    let standalone = normalized_schema_value(&diagnostic_item_path);
    let validation_report = schema_value(&validation_report_path);
    let local_definition = validation_report
        .get("$defs")
        .and_then(|defs| defs.get("diagnostic_item"))
        .cloned()
        .expect("validation report includes a local diagnostic_item definition");

    assert_eq!(standalone, local_definition);
}

fn schema_value(path: &std::path::Path) -> Value {
    let raw = fs::read_to_string(path).unwrap();
    serde_json::from_str(&raw).unwrap()
}

fn normalized_schema_value(path: &std::path::Path) -> Value {
    let mut value = schema_value(path);
    if let Value::Object(map) = &mut value {
        map.remove("$schema");
    }
    value
}

fn writer_ready_normalized_ir_value() -> Value {
    json!({
        "kind": "normalized-ir",
        "schema_version": "0.1.0",
        "document_id": "demo-doc",
        "resolved_identity": "det:demo-doc",
        "notetypes": [
            {
                "id": "basic-main",
                "kind": "basic",
                "name": "Basic",
                "fields": ["Front", "Back"],
                "templates": [
                    {
                        "name": "Card 1",
                        "question_format": "{{Front}}",
                        "answer_format": "{{FrontSide}}\n\n<hr id=answer>\n\n{{Back}}"
                    }
                ],
                "css": ""
            }
        ],
        "notes": [
            {
                "id": "note-1",
                "notetype_id": "basic-main",
                "deck_name": "Default",
                "fields": {
                    "Front": "front",
                    "Back": "back <img src=\"sample.jpg\"> [sound:sample.mp3]"
                },
                "tags": ["demo"]
            }
        ],
        "media": [
            {
                "filename": "sample.jpg",
                "mime": "image/jpeg",
                "data_base64": "MQ=="
            },
            {
                "filename": "sample.mp3",
                "mime": "audio/mpeg",
                "data_base64": "Mg=="
            }
        ]
    })
}
