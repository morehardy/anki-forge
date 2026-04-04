use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthoringDocument {
    pub kind: String,
    pub schema_version: String,
    pub metadata_document_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonContext {
    pub kind: String,
    pub baseline_kind: String,
    pub baseline_artifact_fingerprint: String,
    pub risk_policy_ref: String,
    pub comparison_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizationRequest {
    pub input: AuthoringDocument,
    pub comparison_context: Option<ComparisonContext>,
}

impl NormalizationRequest {
    pub fn new(input: AuthoringDocument) -> Self {
        Self {
            input,
            comparison_context: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedIr {
    pub kind: String,
    pub schema_version: String,
    pub document_id: String,
    pub resolved_identity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticItem {
    pub level: String,
    pub code: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizationDiagnostics {
    pub kind: String,
    pub status: String,
    pub items: Vec<DiagnosticItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRefs {
    pub identity_policy_ref: String,
    pub risk_policy_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeRiskReport {
    pub kind: String,
    pub comparison_status: String,
    pub overall_level: String,
    pub policy_version: String,
    pub baseline_artifact_fingerprint: String,
    pub current_artifact_fingerprint: String,
    pub comparison_reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizationResult {
    pub kind: String,
    pub result_status: String,
    pub tool_contract_version: String,
    pub policy_refs: PolicyRefs,
    pub comparison_context: Option<ComparisonContext>,
    pub diagnostics: NormalizationDiagnostics,
    pub normalized_ir: Option<NormalizedIr>,
    pub merge_risk_report: Option<MergeRiskReport>,
}
