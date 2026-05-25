use serde::{Deserialize, Serialize};

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
