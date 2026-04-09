use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthoringDocument {
    pub kind: String,
    pub schema_version: String,
    pub metadata_document_id: String,
    #[serde(default)]
    pub notetypes: Vec<AuthoringNotetype>,
    #[serde(default)]
    pub notes: Vec<AuthoringNote>,
    #[serde(default)]
    pub media: Vec<AuthoringMedia>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthoringNotetype {
    pub id: String,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_stock_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<AuthoringField>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub templates: Option<Vec<AuthoringTemplate>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub css: Option<String>,
    #[serde(default)]
    pub field_metadata: Vec<AuthoringFieldMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthoringField {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ord: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<u32>,
    #[serde(default)]
    pub prevent_deletion: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthoringTemplate {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ord: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config_id: Option<i64>,
    pub question_format: String,
    pub answer_format: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub browser_question_format: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub browser_answer_format: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_deck_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub browser_font_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub browser_font_size: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthoringFieldMetadata {
    pub field_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthoringNote {
    pub id: String,
    pub notetype_id: String,
    pub deck_name: String,
    pub fields: BTreeMap<String, String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthoringMedia {
    pub filename: String,
    pub mime: String,
    pub data_base64: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonContext {
    pub kind: String,
    pub baseline_kind: String,
    pub baseline_artifact_fingerprint: String,
    pub risk_policy_ref: String,
    pub comparison_mode: String,
}

impl ComparisonContext {
    pub fn normalized(fingerprint: impl Into<String>, policy_ref: impl Into<String>) -> Self {
        Self {
            kind: "comparison-context".into(),
            baseline_kind: "normalized_ir".into(),
            baseline_artifact_fingerprint: fingerprint.into(),
            risk_policy_ref: policy_ref.into(),
            comparison_mode: "strict".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizationRequest {
    pub input: AuthoringDocument,
    pub comparison_context: Option<ComparisonContext>,
    pub identity_override_mode: Option<String>,
    pub target_selector: Option<String>,
    pub external_id: Option<String>,
    pub reason_code: Option<String>,
    pub reason: Option<String>,
}

impl NormalizationRequest {
    pub fn new(input: AuthoringDocument) -> Self {
        Self {
            input,
            comparison_context: None,
            identity_override_mode: None,
            target_selector: None,
            external_id: None,
            reason_code: None,
            reason: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedIr {
    pub kind: String,
    pub schema_version: String,
    pub document_id: String,
    pub resolved_identity: String,
    pub notetypes: Vec<NormalizedNotetype>,
    pub notes: Vec<NormalizedNote>,
    pub media: Vec<NormalizedMedia>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedNotetype {
    pub id: String,
    pub kind: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_stock_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_id: Option<i64>,
    pub fields: Vec<NormalizedField>,
    pub templates: Vec<NormalizedTemplate>,
    pub css: String,
    #[serde(default)]
    pub field_metadata: Vec<NormalizedFieldMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedField {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ord: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config_id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<u32>,
    #[serde(default)]
    pub prevent_deletion: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedTemplate {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ord: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config_id: Option<i64>,
    pub question_format: String,
    pub answer_format: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub browser_question_format: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub browser_answer_format: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_deck_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub browser_font_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub browser_font_size: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedFieldMetadata {
    pub field_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedNote {
    pub id: String,
    pub notetype_id: String,
    pub deck_name: String,
    pub fields: BTreeMap<String, String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedMedia {
    pub filename: String,
    pub mime: String,
    pub data_base64: String,
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
