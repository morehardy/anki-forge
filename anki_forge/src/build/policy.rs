use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildPolicyStatus {
    Passed,
    Blocked,
    NotEvaluated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildPolicyResult {
    pub status: BuildPolicyStatus,
    pub threshold: Option<RiskLevel>,
    pub highest_risk: Option<RiskLevel>,
    pub blocking_findings: Vec<String>,
}

impl Default for BuildPolicyResult {
    fn default() -> Self {
        Self {
            status: BuildPolicyStatus::NotEvaluated,
            threshold: None,
            highest_risk: None,
            blocking_findings: Vec::new(),
        }
    }
}

impl BuildPolicyResult {
    pub fn evaluate(
        threshold: Option<RiskLevel>,
        highest_risk: Option<RiskLevel>,
        candidate_findings: Vec<String>,
    ) -> Self {
        let Some(threshold) = threshold else {
            return Self {
                status: BuildPolicyStatus::NotEvaluated,
                threshold: None,
                highest_risk,
                blocking_findings: Vec::new(),
            };
        };

        let blocked = highest_risk
            .map(|level| level >= threshold)
            .unwrap_or(false);

        Self {
            status: if blocked {
                BuildPolicyStatus::Blocked
            } else {
                BuildPolicyStatus::Passed
            },
            threshold: Some(threshold),
            highest_risk,
            blocking_findings: if blocked {
                candidate_findings
            } else {
                Vec::new()
            },
        }
    }
}
