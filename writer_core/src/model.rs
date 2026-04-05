use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriterPolicy {
    pub id: String,
    pub version: String,
    pub compatibility_target: String,
    pub stock_notetype_mode: String,
    pub media_entry_mode: String,
    pub apkg_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationPolicy {
    pub id: String,
    pub version: String,
    pub writer_fast_gate: VerificationGateRule,
    pub system_gate: VerificationGateRule,
    pub compat_gate: VerificationGateRule,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationGateRule {
    pub minimum_comparison_status: String,
    pub allowed_observation_statuses: Vec<String>,
    pub blocking_severities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildContext {
    pub id: String,
    pub version: String,
    pub emit_apkg: bool,
    pub materialize_staging: bool,
    pub media_resolution_mode: String,
    pub unresolved_asset_behavior: String,
    pub fingerprint_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildDiagnosticItem {
    pub level: String,
    pub code: String,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildDiagnostics {
    pub kind: String,
    pub items: Vec<BuildDiagnosticItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageBuildResult {
    pub kind: String,
    pub result_status: String,
    pub tool_contract_version: String,
    pub writer_policy_ref: String,
    pub build_context_ref: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub staging_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact_fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apkg_ref: Option<String>,
    pub diagnostics: BuildDiagnostics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectObservations {
    pub notetypes: Vec<Value>,
    pub templates: Vec<Value>,
    pub fields: Vec<Value>,
    pub media: Vec<Value>,
    pub metadata: Vec<Value>,
    pub references: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectReport {
    pub kind: String,
    pub observation_model_version: String,
    pub source_kind: String,
    pub source_ref: String,
    pub artifact_fingerprint: String,
    pub observation_status: String,
    pub missing_domains: Vec<String>,
    pub degradation_reasons: Vec<String>,
    pub observations: InspectObservations,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffChange {
    pub category: String,
    pub domain: String,
    pub severity: String,
    pub selector: String,
    pub message: String,
    pub compatibility_hint: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffReport {
    pub kind: String,
    pub comparison_status: String,
    pub left_fingerprint: String,
    pub right_fingerprint: String,
    pub left_observation_model_version: String,
    pub right_observation_model_version: String,
    pub summary: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub uncompared_domains: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub comparison_limitations: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub changes: Vec<DiffChange>,
}
