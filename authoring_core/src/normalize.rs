use crate::model::{
    DiagnosticItem, MergeRiskReport, NormalizationDiagnostics, NormalizationRequest,
    NormalizationResult, NormalizedIr, PolicyRefs,
};

pub fn normalize(request: NormalizationRequest) -> NormalizationResult {
    let NormalizationRequest {
        input,
        comparison_context,
    } = request;
    let risk_policy_ref = comparison_context
        .as_ref()
        .map(|context| context.risk_policy_ref.clone());

    let policy_refs = PolicyRefs {
        identity_policy_ref: "identity-policy.default@1.0.0".into(),
        risk_policy_ref,
    };

    let metadata_document_id = input.metadata_document_id.trim().to_string();
    let current_artifact_fingerprint = format!("det:{metadata_document_id}");
    let merge_risk_report = comparison_context.as_ref().map(|context| {
        if metadata_document_id.is_empty() {
            MergeRiskReport {
                kind: "merge-risk-report".into(),
                comparison_status: "unavailable".into(),
                overall_level: "unknown".into(),
                policy_version: context.risk_policy_ref.clone(),
                baseline_artifact_fingerprint: context.baseline_artifact_fingerprint.clone(),
                current_artifact_fingerprint: "det:unavailable".into(),
                comparison_reasons: vec!["missing document id".into()],
            }
        } else {
            MergeRiskReport {
                kind: "merge-risk-report".into(),
                comparison_status: "complete".into(),
                overall_level: "low".into(),
                policy_version: context.risk_policy_ref.clone(),
                baseline_artifact_fingerprint: context.baseline_artifact_fingerprint.clone(),
                current_artifact_fingerprint: current_artifact_fingerprint.clone(),
                comparison_reasons: vec!["comparison completed".into()],
            }
        }
    });

    if metadata_document_id.is_empty() {
        return NormalizationResult {
            kind: "normalization-result".into(),
            result_status: "invalid".into(),
            tool_contract_version: crate::tool_contract_version().into(),
            policy_refs,
            comparison_context,
            diagnostics: NormalizationDiagnostics {
                kind: "normalization-diagnostics".into(),
                status: "invalid".into(),
                items: vec![DiagnosticItem {
                    level: "error".into(),
                    code: "PHASE2.MISSING_DOCUMENT_ID".into(),
                    summary: "metadata_document_id cannot be blank".into(),
                }],
            },
            normalized_ir: None,
            merge_risk_report,
        };
    }

    NormalizationResult {
        kind: "normalization-result".into(),
        result_status: "success".into(),
        tool_contract_version: crate::tool_contract_version().into(),
        policy_refs,
        comparison_context,
        diagnostics: NormalizationDiagnostics {
            kind: "normalization-diagnostics".into(),
            status: "valid".into(),
            items: Vec::new(),
        },
        normalized_ir: Some(NormalizedIr {
            kind: "normalized-ir".into(),
            schema_version: input.schema_version,
            document_id: metadata_document_id,
            resolved_identity: current_artifact_fingerprint,
        }),
        merge_risk_report,
    }
}
