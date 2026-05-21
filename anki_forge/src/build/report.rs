use std::collections::BTreeSet;
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MediaSummary {
    pub objects: usize,
    pub bindings: usize,
    pub references: usize,
    pub missing_references: usize,
    pub unsafe_references: usize,
    pub unused_bindings: usize,
    pub unique_bytes: u64,
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
    pub media: MediaSummary,
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
    pub fn pretty_report(&self) -> String {
        let mut lines = vec![
            "Media:".to_string(),
            format!("  objects: {}", self.media.objects),
            format!("  bindings: {}", self.media.bindings),
            format!("  references: {}", self.media.references),
            format!("  missing_references: {}", self.media.missing_references),
            format!("  unsafe_references: {}", self.media.unsafe_references),
            format!("  unused_bindings: {}", self.media.unused_bindings),
            format!("  unique_bytes: {}", self.media.unique_bytes),
        ];

        lines.extend(
            sorted_diagnostics(&self.diagnostics)
                .into_iter()
                .map(pretty_diagnostic),
        );

        lines.join("\n")
    }

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

impl MediaSummary {
    pub(crate) fn from_normalized_ir(
        normalized_ir: &authoring_core::NormalizedIr,
        diagnostics: &[Diagnostic],
    ) -> Self {
        let mut referenced_media_ids = BTreeSet::new();
        let mut missing_references = 0;
        for reference in &normalized_ir.media_references {
            match &reference.resolution {
                authoring_core::MediaReferenceResolution::Resolved { media_id } => {
                    referenced_media_ids.insert(media_id.as_str());
                }
                authoring_core::MediaReferenceResolution::Missing => {
                    missing_references += 1;
                }
                authoring_core::MediaReferenceResolution::Skipped { .. } => {}
            }
        }

        let unused_bindings = normalized_ir
            .media_bindings
            .iter()
            .filter(|binding| !referenced_media_ids.contains(binding.id.as_str()))
            .count();

        let unsafe_references = diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code.as_str() == "MEDIA.UNSAFE_REFERENCE")
            .count();

        let mut unique_object_refs = BTreeSet::new();
        let unique_bytes = normalized_ir
            .media_objects
            .iter()
            .filter(|object| unique_object_refs.insert(object.object_ref.as_str()))
            .map(|object| object.size_bytes)
            .sum();

        Self {
            objects: normalized_ir.media_objects.len(),
            bindings: normalized_ir.media_bindings.len(),
            references: normalized_ir.media_references.len(),
            missing_references,
            unsafe_references,
            unused_bindings,
            unique_bytes,
        }
    }
}

fn sorted_diagnostics(diagnostics: &[Diagnostic]) -> Vec<&Diagnostic> {
    let mut sorted = diagnostics.iter().collect::<Vec<_>>();
    sorted.sort_by(|left, right| {
        severity_rank(left.severity)
            .cmp(&severity_rank(right.severity))
            .then_with(|| diagnostic_source(left).cmp(diagnostic_source(right)))
            .then_with(|| left.code.as_str().cmp(right.code.as_str()))
            .then_with(|| left.message.as_bytes().cmp(right.message.as_bytes()))
    });
    sorted
}

fn pretty_diagnostic(diagnostic: &Diagnostic) -> String {
    let mut line = format!(
        "[{} {}] ",
        severity_label(diagnostic.severity),
        diagnostic.code.as_str()
    );
    if let Some(source) = &diagnostic.source {
        line.push_str(source.as_str());
        line.push_str(": ");
    }
    line.push_str(&diagnostic.message);
    if let Some(help) = &diagnostic.help {
        if !help.is_empty() {
            if !line.ends_with(' ') {
                line.push(' ');
            }
            line.push_str(help);
        }
    }
    line
}

fn severity_rank(severity: Severity) -> u8 {
    match severity {
        Severity::Error => 0,
        Severity::Warning => 1,
        Severity::Info => 2,
    }
}

fn severity_label(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Info => "info",
    }
}

fn diagnostic_source(diagnostic: &Diagnostic) -> &str {
    diagnostic
        .source
        .as_ref()
        .map(|source| source.as_str())
        .unwrap_or_default()
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "build failed: {:?}", self.cause)
    }
}

impl std::error::Error for BuildError {}
