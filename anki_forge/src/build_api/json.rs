//! Serializable observations and full outcome snapshots. Paths do not own files.

use super::{BuildCounts, BuildErrorKind};
use crate::diagnostics::Diagnostic;
use serde::Serialize;
use std::path::PathBuf;

/// Observation-only snapshot. It contains no success flag or artifact ownership.
#[derive(Debug, Clone, Serialize)]
pub struct ReportSnapshot {
    /// Version of this observation schema.
    pub schema_version: String,
    /// Counts collected during the operation.
    pub counts: BuildCounts,
    /// Counts from a completely verified baseline, retained after later failure.
    pub baseline_counts: Option<BuildCounts>,
    /// Diagnostics collected in deterministic stage order.
    pub diagnostics: Vec<Diagnostic>,
    /// Elapsed operation time in milliseconds.
    pub duration_ms: u64,
    /// Completed update analysis, including any policy blocking findings.
    pub comparison: Option<crate::update::json::ComparisonSnapshot>,
}

/// Actual file publication stage at the point an operation returned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PublicationStage {
    /// The operation has not replaced or created the destination.
    NotPublished,
    /// The destination was replaced or created.
    Published,
}

/// Whether the operation confirmed file and directory durability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Durability {
    /// File contents and the directory entry were synchronized successfully.
    Confirmed,
    /// Durability was not confirmed, including on platforms without that facility.
    Unconfirmed,
}

/// File publication facts, with no file ownership or lifetime extension.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PublicationSnapshot {
    /// Destination involved in the operation.
    pub path: PathBuf,
    /// The actual publication stage reached.
    pub stage: PublicationStage,
    /// Whether a runtime handle owns deletion of this path.
    pub temporary: bool,
    /// Whether durability was confirmed before returning.
    pub durability: Durability,
}

/// An outcome obtained from the owner of the real build result.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum BuildResultSnapshot {
    /// A completed build with a guaranteed published APKG.
    Success {
        /// Observable artifact path; saving it does not retain the file.
        artifact: PathBuf,
        /// Whether the artifact's runtime handle owns deletion.
        temporary: bool,
    },
    /// A failed request, potentially after a file was published.
    Failure {
        /// Structured operation classification.
        kind: BuildErrorKind,
        /// Primary machine code assigned at failure.
        code: String,
        /// Human-readable error explanation.
        message: String,
        /// Text snapshots of the actual source error chain.
        causes: Vec<String>,
        /// Actual publication facts, rather than a bare list of paths.
        publications: Vec<PublicationSnapshot>,
    },
}

/// A full build outcome plus observations. This DTO owns no files.
#[derive(Debug, Clone, Serialize)]
pub struct BuildSnapshot {
    /// Version of this outcome schema, independent of the crate version.
    pub schema_version: String,
    /// Version of the library that produced the snapshot.
    pub tool_version: String,
    /// Actual success or failure supplied by the runtime result owner.
    pub result: BuildResultSnapshot,
    /// Observations, which do not independently decide the outcome.
    pub report: ReportSnapshot,
}
