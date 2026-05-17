use std::path::PathBuf;
use std::time::Duration;

use crate::diagnostics::{Diagnostic, Severity};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApkgArtifact {
    pub path: PathBuf,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BuildCounts {
    pub notes: usize,
    pub cards: usize,
    pub media: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildMetrics {
    pub duration: Duration,
}

impl Default for BuildMetrics {
    fn default() -> Self {
        Self {
            duration: Duration::ZERO,
        }
    }
}

/// Summary of writer inspection data attached to a build report.
///
/// These fields are derived from the writer inspection layer and are intended
/// for reporting, not as a stable product-domain schema.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InspectSummary {
    pub source_kind: String,
    /// Writer-layer observation status passed through from the inspect report.
    ///
    /// Treat this as reporting metadata rather than a stable public enum.
    pub observation_status: String,
    pub notes: usize,
    pub cards: usize,
    pub notetypes: usize,
    pub templates: usize,
    pub fields: usize,
    pub media: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildReport {
    pub artifact: Option<ApkgArtifact>,
    pub counts: BuildCounts,
    pub diagnostics: Vec<Diagnostic>,
    pub metrics: BuildMetrics,
    pub inspect: Option<InspectSummary>,
    pub status: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildFailureCause {
    MissingArtifact,
    Diagnostics,
    BuildStatus,
    Io,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildError {
    pub report: Box<BuildReport>,
    pub cause: BuildFailureCause,
}

impl BuildError {
    pub fn new(report: BuildReport, cause: BuildFailureCause) -> Self {
        Self {
            report: Box::new(report),
            cause,
        }
    }
}

impl BuildReport {
    pub fn ensure_success(&self) -> Result<(), BuildError> {
        if self
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == Severity::Error)
        {
            return Err(BuildError::new(
                self.clone(),
                BuildFailureCause::Diagnostics,
            ));
        }

        if self.artifact.is_none() {
            return Err(BuildError::new(
                self.clone(),
                BuildFailureCause::MissingArtifact,
            ));
        }

        if self.status != "success" {
            return Err(BuildError::new(
                self.clone(),
                BuildFailureCause::BuildStatus,
            ));
        }

        Ok(())
    }

    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == Severity::Warning)
            .count()
    }

    pub fn diagnostic_codes(&self) -> Vec<String> {
        self.diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.as_str().to_string())
            .collect()
    }
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "build failed: {:?}", self.cause)
    }
}

impl std::error::Error for BuildError {}
