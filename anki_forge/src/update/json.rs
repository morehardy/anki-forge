//! Serializable completed comparisons. These DTOs contain no artifact ownership.
use super::{RiskCode, RiskFinding, RiskLevel};
use crate::{build::BuildCounts, diagnostics::Diagnostic};
use serde::Serialize;

/// A policy decision over completed findings; it is not an operation outcome.
#[derive(Debug, Clone, Serialize)]
pub struct PolicySnapshot {
    /// Whether no unaccepted finding reaches the configured threshold.
    pub allows_publication: bool,
    /// Severity at which unaccepted findings block publication.
    pub threshold: RiskLevel,
    /// Explicitly allowed categories actually observed.
    pub allowed_codes: Vec<RiskCode>,
    /// Requested categories that were not observed.
    pub unmatched_allowances: Vec<RiskCode>,
    /// Original findings that still block publication.
    pub blocking_findings: Vec<RiskFinding>,
}

/// A completed analysis with no build outcome or artifact lifetime.
#[derive(Debug, Clone, Serialize)]
pub struct ComparisonSnapshot {
    /// Version of this comparison schema.
    pub schema_version: String,
    /// All original findings, including explicitly accepted categories.
    pub findings: Vec<RiskFinding>,
    /// Maximum severity before applying policy allowances.
    pub highest_risk: Option<RiskLevel>,
    /// Independent publication decision.
    pub policy: PolicySnapshot,
    /// Nonfatal comparison observations.
    pub diagnostics: Vec<Diagnostic>,
    /// Counts observed in the previous distribution.
    pub baseline_counts: BuildCounts,
    /// Counts observed in the candidate.
    pub candidate_counts: BuildCounts,
}
