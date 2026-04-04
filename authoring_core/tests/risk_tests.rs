use authoring_core::{normalize, AuthoringDocument, ComparisonContext, NormalizationRequest};

fn request_with_comparison_context(
    comparison_mode: &str,
    baseline_kind: &str,
    baseline_artifact_fingerprint: &str,
) -> NormalizationRequest {
    let input = AuthoringDocument {
        kind: "authoring-document".into(),
        schema_version: "1.0".into(),
        metadata_document_id: "doc-risk".into(),
    };
    let mut request = NormalizationRequest::new(input);
    request.comparison_context = Some(ComparisonContext {
        kind: "comparison-context".into(),
        baseline_kind: baseline_kind.into(),
        baseline_artifact_fingerprint: baseline_artifact_fingerprint.into(),
        risk_policy_ref: "risk-policy.default@1.0.0".into(),
        comparison_mode: comparison_mode.into(),
    });
    request
}

#[test]
fn no_comparison_context_yields_no_merge_risk_report() {
    let input = AuthoringDocument {
        kind: "authoring-document".into(),
        schema_version: "1.0".into(),
        metadata_document_id: "doc-risk-none".into(),
    };

    let result = normalize(NormalizationRequest::new(input));

    assert!(result.merge_risk_report.is_none());
}

#[test]
fn best_effort_identity_index_is_partial_with_identity_only_reason() {
    let result = normalize(request_with_comparison_context(
        "best_effort",
        "identity_index",
        "baseline-identity-only",
    ));
    let report = result.merge_risk_report.expect("merge risk report");

    assert_eq!(report.comparison_status, "partial");
    assert_eq!(report.overall_level, "medium");
    assert_eq!(
        report.comparison_reasons,
        vec!["BASELINE_IDENTITY_INDEX_ONLY".to_string()]
    );
}

#[test]
fn strict_with_empty_baseline_fingerprint_is_unavailable() {
    let result = normalize(request_with_comparison_context(
        "strict",
        "normalized_ir",
        "",
    ));
    let report = result.merge_risk_report.expect("merge risk report");

    assert_eq!(report.comparison_status, "unavailable");
    assert_eq!(report.overall_level, "high");
    assert_eq!(
        report.comparison_reasons,
        vec!["BASELINE_UNAVAILABLE".to_string()]
    );
}

#[test]
fn strict_with_normalized_baseline_is_complete() {
    let result = normalize(request_with_comparison_context(
        "strict",
        "normalized_ir",
        "baseline-normalized",
    ));
    let report = result.merge_risk_report.expect("merge risk report");

    assert_eq!(report.comparison_status, "complete");
    assert_eq!(report.overall_level, "low");
}

#[test]
fn invalid_result_with_comparison_context_keeps_merge_risk_report_non_null() {
    let mut request =
        request_with_comparison_context("strict", "normalized_ir", "baseline-normalized");
    request.target_selector = Some("note[id='missing']".into());

    let result = normalize(request);

    assert_eq!(result.result_status, "invalid");
    assert!(result.merge_risk_report.is_some());
}
