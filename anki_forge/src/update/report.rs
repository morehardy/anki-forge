use super::{json, RiskCode, RiskLevel};
use crate::{build::BuildCounts, diagnostics::Diagnostic};
use serde::Serialize;

/// Concrete before/after facts associated with one comparison finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ComparisonEvidence {
    /// Stable-key selector identifying the changed entity or property.
    pub selector: String,
    /// Baseline fact; absent for a newly introduced entity.
    pub before: Option<serde_json::Value>,
    /// Candidate fact; absent for an omitted entity.
    pub after: Option<serde_json::Value>,
}

/// A completed observation, whose severity is never reduced by an allowance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RiskFinding {
    pub(super) code: RiskCode,
    pub(super) level: RiskLevel,
    pub(super) message: String,
    pub(super) evidence: Vec<ComparisonEvidence>,
}
impl RiskFinding {
    /// Returns the registered category of this finding.
    pub fn code(&self) -> RiskCode {
        self.code
    }
    /// Returns the original severity before policy evaluation.
    pub fn level(&self) -> RiskLevel {
        self.level
    }
    /// Returns the human-readable explanation.
    pub fn message(&self) -> &str {
        &self.message
    }
    /// Returns the actual baseline and candidate facts.
    pub fn evidence(&self) -> &[ComparisonEvidence] {
        &self.evidence
    }
}

/// The independent policy decision over all completed findings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyEvaluation {
    pub(super) threshold: RiskLevel,
    pub(super) allowed_codes: Vec<RiskCode>,
    pub(super) unmatched_allowances: Vec<RiskCode>,
    pub(super) blocking_findings: Vec<RiskFinding>,
}
impl PolicyEvaluation {
    /// Whether every finding satisfies the policy. This is not a build outcome.
    pub fn allows_publication(&self) -> bool {
        self.blocking_findings.is_empty()
    }
    /// The configured blocking severity.
    pub fn threshold(&self) -> RiskLevel {
        self.threshold
    }
    /// Explicitly accepted categories that matched at least one finding.
    pub fn allowed_codes(&self) -> &[RiskCode] {
        &self.allowed_codes
    }
    /// Requested allowances that matched no finding and produce warnings.
    pub fn unmatched_allowances(&self) -> &[RiskCode] {
        &self.unmatched_allowances
    }
    /// Original findings still blocking publication after allowances.
    pub fn blocking_findings(&self) -> &[RiskFinding] {
        &self.blocking_findings
    }
    /// Copies the policy decision into a serializable value.
    pub fn snapshot(&self) -> json::PolicySnapshot {
        json::PolicySnapshot {
            allows_publication: self.allows_publication(),
            threshold: self.threshold,
            allowed_codes: self.allowed_codes.clone(),
            unmatched_allowances: self.unmatched_allowances.clone(),
            blocking_findings: self.blocking_findings.clone(),
        }
    }
}

/// A completed comparison, including risks that may block a later build.
/// It owns no candidate or baseline file.
#[derive(Debug, Clone)]
pub struct ComparisonReport {
    pub(super) findings: Vec<RiskFinding>,
    pub(super) policy: PolicyEvaluation,
    pub(super) diagnostics: Vec<Diagnostic>,
    pub(super) baseline_counts: BuildCounts,
    pub(super) candidate_counts: BuildCounts,
}
impl ComparisonReport {
    /// Returns every completed risk finding, including accepted categories.
    pub fn findings(&self) -> &[RiskFinding] {
        &self.findings
    }
    /// Returns the maximum original severity, including accepted findings.
    pub fn highest_risk(&self) -> Option<RiskLevel> {
        self.findings.iter().map(RiskFinding::level).max()
    }
    /// Returns the decision under the requested policy.
    pub fn policy(&self) -> &PolicyEvaluation {
        &self.policy
    }
    /// Returns comparison observations, including unmatched-allowance warnings.
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }
    /// Copies the complete analysis without any artifact ownership or build outcome.
    pub fn snapshot(&self) -> json::ComparisonSnapshot {
        json::ComparisonSnapshot {
            schema_version: "ankiforge-comparison-v1".into(),
            findings: self.findings.clone(),
            highest_risk: self.highest_risk(),
            policy: self.policy.snapshot(),
            diagnostics: self.diagnostics.clone(),
            baseline_counts: self.baseline_counts,
            candidate_counts: self.candidate_counts,
        }
    }
}
