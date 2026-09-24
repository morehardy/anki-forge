use super::{
    json::{BuildResultSnapshot, BuildSnapshot, ReportSnapshot},
    ApkgArtifact,
};
use crate::diagnostics::Diagnostic;
use serde::Serialize;
use std::time::Duration;

/// Counts observed during generation and inspection.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct BuildCounts {
    /// Notes included in the candidate collection.
    pub notes: usize,
    /// Cards planned and verified in the candidate collection.
    pub cards: usize,
    /// Exported media names, including explicit assets.
    pub media: usize,
}

/// Observations from a build. This value neither owns files nor represents its outcome.
#[derive(Debug, Clone, Default)]
pub struct BuildReport {
    pub(crate) counts: BuildCounts,
    pub(crate) baseline_counts: Option<BuildCounts>,
    pub(crate) diagnostics: Vec<Diagnostic>,
    pub(crate) duration: Duration,
    pub(crate) comparison: Option<crate::update::ComparisonReport>,
}

impl BuildReport {
    /// Returns counts collected during generation and inspection.
    pub fn counts(&self) -> &BuildCounts {
        &self.counts
    }

    /// Returns counts from a completely verified baseline, even if generating
    /// the candidate later failed. None means no complete baseline observation.
    pub fn baseline_counts(&self) -> Option<&BuildCounts> {
        self.baseline_counts.as_ref()
    }

    /// Returns all observations in deterministic stage order.
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Returns the elapsed operation duration.
    pub fn duration(&self) -> Duration {
        self.duration
    }

    /// Returns a completed update analysis, if this request reached that stage.
    /// It may contain findings that blocked publication.
    pub fn comparison(&self) -> Option<&crate::update::ComparisonReport> {
        self.comparison.as_ref()
    }

    /// Copies observations into a serializable value, without outcome or files.
    pub fn snapshot(&self) -> ReportSnapshot {
        ReportSnapshot {
            schema_version: "ankiforge-report-v1".into(),
            counts: self.counts,
            baseline_counts: self.baseline_counts,
            diagnostics: self.diagnostics.clone(),
            duration_ms: self.duration.as_millis().min(u64::MAX as u128) as u64,
            comparison: self
                .comparison
                .as_ref()
                .map(crate::update::ComparisonReport::snapshot),
        }
    }
}

/// A successful build with a guaranteed artifact. Only the build pipeline creates it.
#[derive(Debug, Clone)]
pub struct BuildOutput {
    pub(crate) artifact: ApkgArtifact,
    pub(crate) report: BuildReport,
}

impl BuildOutput {
    /// Borrows the artifact handle, retaining temporary output while it is held.
    pub fn artifact(&self) -> &ApkgArtifact {
        &self.artifact
    }

    /// Borrows observations about the completed build.
    pub fn report(&self) -> &BuildReport {
        &self.report
    }

    /// Records the actual successful outcome. A warning cannot change it to failure.
    pub fn snapshot(&self) -> BuildSnapshot {
        BuildSnapshot {
            schema_version: "ankiforge-build-v1".into(),
            tool_version: env!("CARGO_PKG_VERSION").into(),
            result: BuildResultSnapshot::Success {
                artifact: self.artifact.path().to_owned(),
                temporary: self.artifact.is_temporary(),
            },
            report: self.report.snapshot(),
        }
    }
}
