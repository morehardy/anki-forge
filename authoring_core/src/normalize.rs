use crate::identity::{resolve_identity, DefaultNonceSource};
use crate::model::{
    DiagnosticItem, NormalizationDiagnostics, NormalizationRequest, NormalizationResult,
    NormalizedIr, PolicyRefs,
};
use crate::risk::{assess_risk, unavailable_report};
use crate::selector::{
    parse_selector, resolve_selector, SelectorError, SelectorResolveError, SelectorTarget,
};

pub fn selector_resolve_error_code(error: &SelectorResolveError) -> &'static str {
    match error {
        SelectorResolveError::Unmatched => "PHASE2.SELECTOR_UNMATCHED",
        SelectorResolveError::Ambiguous => "PHASE2.SELECTOR_AMBIGUOUS",
    }
}

pub fn normalize(request: NormalizationRequest) -> NormalizationResult {
    let risk_policy_ref = request
        .comparison_context
        .as_ref()
        .map(|context| context.risk_policy_ref.clone());

    let policy_refs = PolicyRefs {
        identity_policy_ref: "identity-policy.default@1.0.0".into(),
        risk_policy_ref,
    };

    let metadata_document_id = request.input.metadata_document_id.trim().to_string();

    if metadata_document_id.is_empty() {
        return invalid_result(
            policy_refs,
            request.comparison_context,
            vec![DiagnosticItem {
                level: "error".into(),
                code: "PHASE2.MISSING_DOCUMENT_ID".into(),
                summary: "metadata_document_id cannot be blank".into(),
            }],
            "det:unavailable".into(),
            "missing document id".into(),
        );
    }

    if let Some(raw_selector) = request.target_selector.as_deref() {
        let selector = match parse_selector(raw_selector) {
            Ok(selector) => selector,
            Err(error) => {
                return invalid_result(
                    policy_refs,
                    request.comparison_context,
                    vec![DiagnosticItem {
                        level: "error".into(),
                        code: "PHASE2.SELECTOR_INVALID".into(),
                        summary: selector_invalid_summary(&error).into(),
                    }],
                    format!("det:{metadata_document_id}"),
                    "target selector did not match any resolvable object".into(),
                );
            }
        };

        let targets = vec![SelectorTarget::new(
            request.input.kind.clone(),
            [("id", metadata_document_id.clone())],
        )];

        if let Err(error) = resolve_selector(&selector, &targets) {
            return invalid_result(
                policy_refs,
                request.comparison_context,
                vec![DiagnosticItem {
                    level: "error".into(),
                    code: selector_resolve_error_code(&error).into(),
                    summary: "target_selector resolution failed".into(),
                }],
                format!("det:{metadata_document_id}"),
                "target selector did not match any resolvable object".into(),
            );
        }
    }

    let mut diagnostics = Vec::new();
    let mut nonce_source = DefaultNonceSource;
    let resolved_identity = match resolve_identity(&request, &mut diagnostics, &mut nonce_source) {
        Ok(identity) => identity,
        Err(error) => {
            let (code, summary) = identity_error_diagnostic(&error);
            diagnostics.push(DiagnosticItem {
                level: "error".into(),
                code: code.into(),
                summary,
            });

            return invalid_result(
                policy_refs,
                request.comparison_context,
                diagnostics,
                format!("det:{metadata_document_id}"),
                "identity resolution failed".into(),
            );
        }
    };

    let normalized_ir = NormalizedIr {
        kind: "normalized-ir".into(),
        schema_version: request.input.schema_version,
        document_id: metadata_document_id,
        resolved_identity: resolved_identity.clone(),
    };

    let merge_risk_report = assess_risk(&normalized_ir, request.comparison_context.as_ref());

    NormalizationResult {
        kind: "normalization-result".into(),
        result_status: "success".into(),
        tool_contract_version: crate::tool_contract_version().into(),
        policy_refs,
        comparison_context: request.comparison_context.clone(),
        diagnostics: NormalizationDiagnostics {
            kind: "normalization-diagnostics".into(),
            status: "valid".into(),
            items: diagnostics,
        },
        normalized_ir: Some(normalized_ir),
        merge_risk_report,
    }
}

fn selector_invalid_summary(error: &SelectorError) -> &'static str {
    match error {
        SelectorError::Empty => "target_selector cannot be blank",
        SelectorError::ArrayIndexNotAllowed => "target_selector cannot use array index segments",
        SelectorError::InvalidPredicate => "target_selector does not match selector grammar",
    }
}

fn identity_error_diagnostic(error: &anyhow::Error) -> (&'static str, String) {
    let message = error.to_string();
    if message == "reason_code required" || message == "external_id required" {
        (
            "PHASE2.EXTERNAL_ID_REQUIRED",
            "override modes require reason_code, and external overrides require external_id".into(),
        )
    } else {
        ("PHASE2.IDENTITY_OVERRIDE_UNSUPPORTED", message)
    }
}

fn invalid_result(
    policy_refs: PolicyRefs,
    comparison_context: Option<crate::model::ComparisonContext>,
    diagnostics: Vec<DiagnosticItem>,
    current_artifact_fingerprint: String,
    comparison_reason: String,
) -> NormalizationResult {
    let merge_risk_report = unavailable_report(
        comparison_context.as_ref(),
        current_artifact_fingerprint,
        comparison_reason,
    );

    NormalizationResult {
        kind: "normalization-result".into(),
        result_status: "invalid".into(),
        tool_contract_version: crate::tool_contract_version().into(),
        policy_refs,
        comparison_context,
        diagnostics: NormalizationDiagnostics {
            kind: "normalization-diagnostics".into(),
            status: "invalid".into(),
            items: diagnostics,
        },
        normalized_ir: None,
        merge_risk_report,
    }
}
