use authoring_core::{normalize, AuthoringDocument, ComparisonContext, NormalizationRequest};
use serde_json::Value;

fn assert_json_object_has_keys(value: &Value, keys: &[&str]) {
    let object = value.as_object().expect("expected JSON object");
    for key in keys {
        assert!(object.contains_key(*key), "missing key {key}");
    }
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
        baseline_kind: "normalized-ir".into(),
        baseline_artifact_fingerprint: "baseline-fingerprint".into(),
        risk_policy_ref: "risk-policy.default@1.0.0".into(),
        comparison_mode: "strict".into(),
    });

    let result = normalize(request);
    let json = serde_json::to_value(result).expect("serialize normalization result");

    assert_eq!(json["policy_refs"]["risk_policy_ref"], "risk-policy.default@1.0.0");
    assert!(json["comparison_context"].is_object());
}
