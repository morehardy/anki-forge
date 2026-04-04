use crate::model::{
    DiagnosticItem, MergeRiskReport, NormalizationDiagnostics, NormalizationRequest,
    NormalizationResult, NormalizedIr, PolicyRefs,
};

pub fn normalize(request: NormalizationRequest) -> NormalizationResult {
    let risk_policy_ref = request
        .comparison_context
        .as_ref()
        .map(|context| context.risk_policy_ref.clone());

    let policy_refs = PolicyRefs {
        identity_policy_ref: "identity-policy.default@1.0.0".into(),
        risk_policy_ref,
    };

    let comparison_context = request.comparison_context;
    let metadata_document_id = request.input.metadata_document_id.trim().to_string();

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
            merge_risk_report: None,
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
            schema_version: request.input.schema_version,
            document_id: metadata_document_id,
        }),
        merge_risk_report: None::<MergeRiskReport>,
    }
}
