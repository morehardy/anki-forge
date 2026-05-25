pub mod summary;

use serde::{Deserialize, Serialize};

pub use summary::summarize_writer_diff;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildDiffSummary {
    pub artifact_diff: Option<ArtifactDiffSummary>,
    pub semantic_changes: Vec<SemanticDiffChange>,
    pub summary_counts: DiffSummaryCounts,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffSummaryCounts {
    pub added: usize,
    pub removed: usize,
    pub modified: usize,
    pub reordered: usize,
    pub uncompared_domains: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactDiffSummary {
    pub changes: Vec<ArtifactDiffChange>,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactDiffChange {
    pub category: String,
    pub domain: String,
    pub severity: String,
    pub selector: String,
    pub message: String,
    pub evidence_refs: Vec<EvidenceRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticDiffCategory {
    Notetype,
    Field,
    Template,
    NoteIdentity,
    CardCount,
    Media,
    Baseline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticDiffChangeKind {
    Added,
    Removed,
    Modified,
    Reordered,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticDiffChange {
    pub category: SemanticDiffCategory,
    pub selector: String,
    pub change_kind: SemanticDiffChangeKind,
    pub risk_codes: Vec<String>,
    pub message: String,
    pub source: Option<crate::diagnostics::SourcePath>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceRefKind {
    Diagnostic,
    DiffChange,
    InspectObservation,
    UpdateSafety,
    Oracle,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceRef {
    pub kind: EvidenceRefKind,
    pub ref_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectDiffReport {
    pub status: crate::build::BuildStatus,
    pub comparison: crate::build::ComparisonStatus,
    pub diagnostics: Vec<crate::diagnostics::Diagnostic>,
    pub current_inspect: Option<crate::build::InspectSummary>,
    pub previous_inspect: Option<crate::build::InspectSummary>,
    pub update_safety: Option<crate::build::UpdateSafetySummary>,
    pub diff: Option<BuildDiffSummary>,
    pub risk: Option<crate::risk::ImportRiskReport>,
    pub metrics: ComparisonMetrics,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComparisonMetrics {
    pub duration_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectDiffError {
    pub report: Box<ProjectDiffReport>,
    pub cause: crate::build::BuildFailureCause,
}

impl ProjectDiffError {
    pub fn new(report: ProjectDiffReport, cause: crate::build::BuildFailureCause) -> Self {
        Self {
            report: Box::new(report),
            cause,
        }
    }
}
