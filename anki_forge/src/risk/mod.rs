use serde::{Deserialize, Serialize};

use crate::build::RiskLevel;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportRiskFinding {
    pub code: String,
    pub level: RiskLevel,
    pub category: String,
    pub message: String,
    pub source: Option<crate::diagnostics::SourcePath>,
    pub evidence_refs: Vec<crate::diff::EvidenceRef>,
    pub suggested_action: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportRiskReport {
    pub highest_level: Option<RiskLevel>,
    pub findings: Vec<ImportRiskFinding>,
    pub limitations: Vec<String>,
}
