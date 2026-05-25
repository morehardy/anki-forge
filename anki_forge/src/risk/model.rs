use serde::{Deserialize, Serialize};

use crate::build::RiskLevel;
use crate::diff::EvidenceRef;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportRiskFinding {
    pub code: String,
    pub level: RiskLevel,
    pub category: String,
    pub message: String,
    pub source: Option<crate::diagnostics::SourcePath>,
    pub evidence_refs: Vec<EvidenceRef>,
    pub suggested_action: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportRiskReport {
    pub highest_level: Option<RiskLevel>,
    pub findings: Vec<ImportRiskFinding>,
    pub limitations: Vec<String>,
}

impl ImportRiskReport {
    pub fn from_findings(findings: Vec<ImportRiskFinding>) -> Self {
        let highest_level = findings.iter().map(|finding| finding.level).max();
        Self {
            highest_level,
            findings,
            limitations: Vec::new(),
        }
    }

    pub fn blocking_codes_at_or_above(&self, threshold: RiskLevel) -> Vec<String> {
        self.findings
            .iter()
            .filter(|finding| finding.level >= threshold)
            .map(|finding| finding.code.clone())
            .collect()
    }
}
