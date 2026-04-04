use authoring_core::{normalize, AuthoringDocument, ComparisonContext, NormalizationRequest};
use serde_json::{json, Value};

fn assert_json_object_has_keys(value: &Value, keys: &[&str]) {
    let object = value.as_object().expect("expected JSON object");
    for key in keys {
        assert!(object.contains_key(*key), "missing key {key}");
    }
}

fn request_from_json(value: Value) -> NormalizationRequest {
    serde_json::from_value(value).expect("deserialize normalization request")
}

#[test]
fn missing_document_id_returns_invalid_result_with_diagnostics() {
    let input = AuthoringDocument {
        kind: "authoring-document".into(),
        schema_version: "1.0".into(),
        metadata_document_id: "   ".into(),
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
            "metadata_document_id": "doc-random"
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
            "metadata_document_id": "doc-external"
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
            "metadata_document_id": "doc-random-missing-reason"
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
            "metadata_document_id": "doc-external"
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
            "metadata_document_id": "doc-selector"
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
