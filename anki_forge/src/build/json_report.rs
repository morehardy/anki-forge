use anyhow::Context;
use serde::Serialize;
use std::io::Write;
use std::path::Path;

use crate::build::{
    ApkgArtifact, BuildCounts, BuildMetrics, BuildPolicyResult, BuildReport, BuildStatus,
    ComparisonStatus, InspectSummary, MediaEntrySummary, MediaSourceMode, MediaSummary,
    UpdateSafetySummary,
};

#[derive(Debug, Clone, Serialize)]
pub struct BuildReportJson {
    pub kind: &'static str,
    pub schema_version: &'static str,
    pub tool_version: String,
    pub artifact: Option<ApkgArtifactJson>,
    pub status: BuildStatus,
    pub comparison: ComparisonStatus,
    pub counts: BuildCountsJson,
    pub media: MediaSummaryJson,
    pub diagnostics: Vec<crate::diagnostics::Diagnostic>,
    pub metrics: BuildMetricsJson,
    pub inspect: Option<InspectSummaryJson>,
    pub previous_inspect: Option<InspectSummaryJson>,
    pub update_safety: Option<UpdateSafetySummary>,
    pub diff: Option<crate::diff::BuildDiffSummary>,
    pub risk: Option<crate::risk::ImportRiskReport>,
    pub policy: BuildPolicyResult,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApkgArtifactJson {
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BuildCountsJson {
    pub notes: usize,
    pub cards: usize,
    pub media: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct MediaSummaryJson {
    pub objects: usize,
    pub bindings: usize,
    pub references: usize,
    pub missing_references: usize,
    pub unsafe_references: usize,
    pub unused_bindings: usize,
    pub unique_bytes: u64,
    pub entries: Vec<MediaEntrySummaryJson>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MediaEntrySummaryJson {
    pub id: String,
    pub filename: String,
    pub source_mode: MediaSourceMode,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct BuildMetricsJson {
    pub duration_ms: u128,
}

#[derive(Debug, Clone, Serialize)]
pub struct InspectSummaryJson {
    pub source_kind: String,
    pub observation_status: String,
    pub notes: usize,
    pub cards: usize,
    pub notetypes: usize,
    pub templates: usize,
    pub fields: usize,
    pub media: usize,
}

pub trait SerializableBuildReport {
    fn to_report_json(&self) -> BuildReportJson;
}

impl SerializableBuildReport for BuildReport {
    fn to_report_json(&self) -> BuildReportJson {
        BuildReportJson::from_report(self)
    }
}

impl BuildReportJson {
    pub fn from_report(report: &BuildReport) -> Self {
        Self {
            kind: "anki-forge-build-report",
            schema_version: "phase4-build-report-v1",
            tool_version: crate::facade_api_version().to_string(),
            artifact: report.artifact.as_ref().map(ApkgArtifactJson::from),
            status: report.status,
            comparison: report.comparison,
            counts: BuildCountsJson::from(report.counts),
            media: MediaSummaryJson::from(&report.media),
            diagnostics: report.diagnostics.clone(),
            metrics: BuildMetricsJson::from(report.metrics),
            inspect: report.inspect.as_ref().map(InspectSummaryJson::from),
            previous_inspect: report
                .previous_inspect
                .as_ref()
                .map(InspectSummaryJson::from),
            update_safety: report.update_safety.clone(),
            diff: report.diff.clone(),
            risk: report.risk.clone(),
            policy: report.policy.clone(),
        }
    }
}

impl From<&ApkgArtifact> for ApkgArtifactJson {
    fn from(value: &ApkgArtifact) -> Self {
        Self {
            path: value.path.display().to_string(),
        }
    }
}

impl From<BuildCounts> for BuildCountsJson {
    fn from(value: BuildCounts) -> Self {
        Self {
            notes: value.notes,
            cards: value.cards,
            media: value.media,
        }
    }
}

impl From<&MediaSummary> for MediaSummaryJson {
    fn from(value: &MediaSummary) -> Self {
        Self {
            objects: value.objects,
            bindings: value.bindings,
            references: value.references,
            missing_references: value.missing_references,
            unsafe_references: value.unsafe_references,
            unused_bindings: value.unused_bindings,
            unique_bytes: value.unique_bytes,
            entries: value
                .entries
                .iter()
                .map(MediaEntrySummaryJson::from)
                .collect(),
        }
    }
}

impl From<&MediaEntrySummary> for MediaEntrySummaryJson {
    fn from(value: &MediaEntrySummary) -> Self {
        Self {
            id: value.id.clone(),
            filename: value.filename.clone(),
            source_mode: value.source_mode,
            size_bytes: value.size_bytes,
        }
    }
}

impl From<BuildMetrics> for BuildMetricsJson {
    fn from(value: BuildMetrics) -> Self {
        Self {
            duration_ms: value.duration.as_millis(),
        }
    }
}

impl From<&InspectSummary> for InspectSummaryJson {
    fn from(value: &InspectSummary) -> Self {
        Self {
            source_kind: value.source_kind.clone(),
            observation_status: value.observation_status.clone(),
            notes: value.notes,
            cards: value.cards,
            notetypes: value.notetypes,
            templates: value.templates,
            fields: value.fields,
            media: value.media,
        }
    }
}

pub fn write_report_json_atomic(path: &Path, report: &BuildReport) -> anyhow::Result<()> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(parent)
        .with_context(|| format!("create build report directory {}", parent.display()))?;

    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("build-report.json");
    let mut temp = tempfile::Builder::new()
        .prefix(&format!(".{file_name}."))
        .suffix(".tmp")
        .tempfile_in(parent)
        .with_context(|| format!("create temporary build report in {}", parent.display()))?;

    let bytes = serde_json::to_vec_pretty(&BuildReportJson::from_report(report))
        .context("serialize build report json")?;
    temp.write_all(&bytes)
        .with_context(|| format!("write temporary build report {}", temp.path().display()))?;
    temp.flush()
        .with_context(|| format!("flush temporary build report {}", temp.path().display()))?;
    temp.persist(path)
        .map_err(|err| err.error)
        .with_context(|| format!("replace build report {}", path.display()))?;
    Ok(())
}
