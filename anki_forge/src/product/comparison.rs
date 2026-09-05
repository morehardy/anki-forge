use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::build::{BuildStatus, ComparisonStatus, InspectSummary, UpdateSafetySummary};
use crate::diagnostics::{Diagnostic, DiagnosticCode, Severity, SourcePath};
use crate::diff::{summarize_writer_diff, BuildDiffSummary};
use crate::risk::rules::{classify_import_risk, RiskInput};
use crate::risk::ImportRiskReport;
use crate::update_safety::model::IdentityIndex;
use crate::writer_core::{InspectError, InspectLimits};

#[derive(Debug, Clone)]
pub struct ComparisonInput<'a> {
    pub current_artifact: &'a Path,
    pub previous_artifact: Option<&'a Path>,
    pub diagnostics: &'a [Diagnostic],
    pub update_safety: Option<&'a UpdateSafetySummary>,
    pub started: Instant,
}

#[derive(Debug, Clone)]
pub struct ComparisonOutput {
    pub comparison: ComparisonStatus,
    pub current_inspect: Option<InspectSummary>,
    pub previous_inspect: Option<InspectSummary>,
    pub diff: Option<BuildDiffSummary>,
    pub risk: Option<ImportRiskReport>,
    pub diagnostics: Vec<Diagnostic>,
    pub status: BuildStatus,
    pub duration: Duration,
}

pub fn assemble_comparison(input: ComparisonInput<'_>) -> ComparisonOutput {
    let baseline = input.previous_artifact.map(BaselineSnapshot::capture);
    assemble_comparison_with_baseline(input, baseline.as_ref())
}

/// Inspection facts are captured before any build writes, then shared by GUID
/// reconciliation and diff. No later stage reopens the caller's baseline path.
pub(crate) struct BaselineSnapshot {
    path: PathBuf,
    inspected: Result<InspectedArtifact, InspectError>,
}

impl BaselineSnapshot {
    pub(crate) fn capture(path: &Path) -> Self {
        Self::capture_with_limits(path, &InspectLimits::default())
    }

    pub(crate) fn capture_with_limits(path: &Path, limits: &InspectLimits) -> Self {
        Self {
            path: path.to_owned(),
            inspected: inspect_artifact(path, limits),
        }
    }

    pub(crate) fn identity_index(
        &self,
        current: Option<&IdentityIndex>,
        lockfile: Option<&IdentityIndex>,
    ) -> Result<IdentityIndex, InspectError> {
        self.inspected
            .as_ref()
            .map_err(Clone::clone)
            .and_then(|artifact| {
                crate::update_safety::baseline::identity_index_from_inspect(
                    &self.path,
                    &artifact.report,
                    current,
                    lockfile,
                )
                .map_err(InspectError::from_anyhow)
            })
    }
}

pub(crate) fn assemble_comparison_with_baseline(
    input: ComparisonInput<'_>,
    baseline: Option<&BaselineSnapshot>,
) -> ComparisonOutput {
    assemble_comparison_with_limits(input, baseline, &InspectLimits::default())
}

pub(crate) fn assemble_comparison_with_limits(
    input: ComparisonInput<'_>,
    baseline: Option<&BaselineSnapshot>,
    limits: &InspectLimits,
) -> ComparisonOutput {
    let mut diagnostics = input.diagnostics.to_vec();
    let current = match inspect_artifact(input.current_artifact, limits) {
        Ok(artifact) => Some(artifact),
        Err(error) => {
            push_resource_diagnostic(
                &mut diagnostics,
                &error,
                input.current_artifact,
                Severity::Error,
            );
            diagnostics.push(Diagnostic {
                code: DiagnosticCode::new("COMPARE.CURRENT_UNAVAILABLE"),
                severity: Severity::Error,
                domain: None,
                stage: None,
                message: error.to_string(),
                source: Some(SourcePath::new(
                    input.current_artifact.display().to_string(),
                )),
                help: Some("verify the current APKG path and package contents".to_string()),
            });
            None
        }
    };
    let Some(baseline) = baseline else {
        let risk = classify_import_risk(RiskInput {
            diagnostics: &diagnostics,
            comparison: ComparisonStatus::NotRequested,
            diff: None,
            current_inspect: current.as_ref().map(|artifact| &artifact.summary),
            previous_inspect: None,
            update_safety: input.update_safety,
        });
        return ComparisonOutput {
            comparison: ComparisonStatus::NotRequested,
            current_inspect: current.map(|artifact| artifact.summary),
            previous_inspect: None,
            diff: None,
            risk: Some(risk),
            diagnostics,
            status: BuildStatus::Success,
            duration: input.started.elapsed(),
        };
    };

    let mut baseline_unavailable = false;
    let previous = match &baseline.inspected {
        Ok(artifact) => Some(artifact),
        Err(error) => {
            baseline_unavailable = true;
            push_resource_diagnostic(&mut diagnostics, error, &baseline.path, Severity::Warning);
            diagnostics.push(Diagnostic {
                code: DiagnosticCode::new("COMPARE.BASELINE_UNAVAILABLE"),
                severity: Severity::Error,
                domain: None,
                stage: None,
                message: error.to_string(),
                source: Some(SourcePath::new(baseline.path.display().to_string())),
                help: Some("verify the previous APKG path and package contents".to_string()),
            });
            None
        }
    };

    let mut comparison = if current.is_some() && previous.is_some() {
        ComparisonStatus::Complete
    } else {
        ComparisonStatus::Unavailable
    };
    let diff = if let (Some(current), Some(previous)) = (current.as_ref(), previous.as_ref()) {
        match writer_diff_from_reports(&current.report, &previous.report) {
            Ok((summary, writer_status)) => {
                comparison = writer_status;
                Some(summary)
            }
            Err(message) => {
                diagnostics.push(Diagnostic {
                    code: DiagnosticCode::new("COMPARE.DIFF_FAILED"),
                    severity: Severity::Error,
                    domain: None,
                    stage: None,
                    message,
                    source: Some(SourcePath::new("compare.diff")),
                    help: Some("inspect both APKG files before comparing".to_string()),
                });
                comparison = ComparisonStatus::Unavailable;
                None
            }
        }
    } else {
        None
    };

    let risk_comparison = if comparison == ComparisonStatus::Unavailable && !baseline_unavailable {
        ComparisonStatus::Partial
    } else {
        comparison
    };
    let risk = classify_import_risk(RiskInput {
        diagnostics: &diagnostics,
        comparison: risk_comparison,
        diff: diff.as_ref(),
        current_inspect: current.as_ref().map(|artifact| &artifact.summary),
        previous_inspect: previous.as_ref().map(|artifact| &artifact.summary),
        update_safety: input.update_safety,
    });
    let status = if comparison == ComparisonStatus::Unavailable {
        BuildStatus::Invalid
    } else {
        BuildStatus::Success
    };

    ComparisonOutput {
        comparison,
        current_inspect: current.map(|artifact| artifact.summary),
        previous_inspect: previous.map(|artifact| artifact.summary.clone()),
        diff,
        risk: Some(risk),
        diagnostics,
        status,
        duration: input.started.elapsed(),
    }
}

#[derive(Debug, Clone)]
struct InspectedArtifact {
    report: crate::writer_core::InspectReport,
    summary: InspectSummary,
}

fn inspect_artifact(
    path: &Path,
    limits: &InspectLimits,
) -> Result<InspectedArtifact, InspectError> {
    crate::writer_core::inspect_apkg_with_limits(path, limits).map(|report| {
        let summary = inspect_summary_from_report(&report);
        InspectedArtifact { report, summary }
    })
}

pub(crate) fn push_resource_diagnostic(
    diagnostics: &mut Vec<Diagnostic>,
    error: &InspectError,
    path: &Path,
    severity: Severity,
) {
    if let Some(limit) = error.limit_exceeded() {
        let diagnostic = Diagnostic {
            code: DiagnosticCode::new("INSPECT.RESOURCE_LIMIT_EXCEEDED"),
            severity,
            domain: Some(crate::diagnostics::DiagnosticDomain::new("inspection")),
            stage: Some(crate::diagnostics::DiagnosticStage::new("inspect")),
            message: limit.to_string(),
            source: Some(SourcePath::new(path.display().to_string())),
            help: Some("use a smaller APKG or explicitly raise the relevant InspectLimits budget for a trusted input".into()),
        };
        if !diagnostics.contains(&diagnostic) {
            diagnostics.push(diagnostic);
        }
    }
}

#[cfg(test)]
mod snapshot_tests {
    use super::*;
    use crate::product::{Note, Project};

    fn build(path: &Path, stable_id: &str) {
        let mut project = Project::new("Snapshot").stable_id("snapshot");
        project
            .add_note(Note::basic(stable_id, "back").stable_id(stable_id))
            .unwrap();
        project.write_apkg(path).unwrap();
    }

    #[test]
    fn captured_baseline_survives_replacement_for_identity_and_diff() {
        let root = tempfile::tempdir().unwrap();
        let previous = root.path().join("previous.apkg");
        let current = root.path().join("current.apkg");
        build(&previous, "previous-note");
        build(&current, "current-note");
        let snapshot = BaselineSnapshot::capture(&previous);
        let captured_identity = snapshot.identity_index(None, None).unwrap();
        // Deterministically simulate another writer replacing the baseline
        // between preflight and the later comparison stage.
        std::fs::copy(&current, &previous).unwrap();
        let index = snapshot.identity_index(None, None).unwrap();
        assert_eq!(index.notes[0].stable_id, "previous-note");
        assert!(index.notes[0].revision.is_some());
        assert_eq!(index.notes[0].revision, captured_identity.notes[0].revision);
        let replaced_identity = BaselineSnapshot::capture(&previous)
            .identity_index(None, None)
            .unwrap();
        assert_ne!(index.notes[0].revision, replaced_identity.notes[0].revision);
        let input = || ComparisonInput {
            current_artifact: &current,
            previous_artifact: Some(&previous),
            diagnostics: &[],
            update_safety: None,
            started: Instant::now(),
        };
        let captured = assemble_comparison_with_baseline(input(), Some(&snapshot));
        assert_eq!(captured.comparison, ComparisonStatus::Complete);
        assert!(!captured
            .diff
            .unwrap()
            .artifact_diff
            .unwrap()
            .changes
            .is_empty());
        let reopened = assemble_comparison(input());
        assert!(reopened
            .diff
            .unwrap()
            .artifact_diff
            .unwrap()
            .changes
            .is_empty());
    }

    #[test]
    fn unreadable_baseline_stays_unavailable_after_a_file_appears() {
        let root = tempfile::tempdir().unwrap();
        let previous = root.path().join("previous.apkg");
        let snapshot = BaselineSnapshot::capture(&previous);
        build(&previous, "note");
        assert!(snapshot.identity_index(None, None).is_err());
        let report = assemble_comparison_with_baseline(
            ComparisonInput {
                current_artifact: &previous,
                previous_artifact: Some(&previous),
                diagnostics: &[],
                update_safety: None,
                started: Instant::now(),
            },
            Some(&snapshot),
        );
        assert_eq!(report.comparison, ComparisonStatus::Unavailable);
        let diagnostic = report
            .diagnostics
            .iter()
            .find(|item| item.code.as_str() == "COMPARE.BASELINE_UNAVAILABLE")
            .unwrap();
        assert_eq!(
            diagnostic.source.as_ref().unwrap().as_str(),
            previous.to_str().unwrap()
        );
    }
}

fn inspect_summary_from_report(report: &crate::writer_core::InspectReport) -> InspectSummary {
    InspectSummary {
        notes: inspect_metadata_count(report, "note_count"),
        cards: inspect_metadata_count(report, "card_count"),
        source_kind: report.source_kind.clone(),
        observation_status: report.observation_status.clone(),
        notetypes: report.observations.notetypes.len(),
        templates: report.observations.templates.len(),
        fields: report.observations.fields.len(),
        media: report.observations.media.len(),
    }
}

fn inspect_metadata_count(report: &crate::writer_core::InspectReport, key: &str) -> usize {
    report
        .observations
        .metadata
        .iter()
        .find_map(|value| value.get(key).and_then(serde_json::Value::as_u64))
        .unwrap_or_default() as usize
}

fn writer_diff_from_reports(
    current_report: &crate::writer_core::InspectReport,
    previous_report: &crate::writer_core::InspectReport,
) -> Result<(BuildDiffSummary, ComparisonStatus), String> {
    let report = crate::writer_core::diff_reports(previous_report, current_report)
        .map_err(|err| err.to_string())?;
    let status = writer_comparison_status(&report, previous_report, current_report);
    let mut summary = summarize_writer_diff(&report);
    match (
        card_evidence_status(previous_report),
        card_evidence_status(current_report),
    ) {
        (CardEvidenceStatus::Full, CardEvidenceStatus::Full) => {}
        (CardEvidenceStatus::Missing, _) | (_, CardEvidenceStatus::Missing) => {
            summary
                .limitations
                .push("card_evidence missing on at least one side".to_string());
        }
        (CardEvidenceStatus::Degraded, _) | (_, CardEvidenceStatus::Degraded) => {
            summary.limitations.push(
                "card_evidence degraded: card_count exists, but card/template ordinal references are incomplete"
                    .to_string(),
            );
        }
    }
    Ok((summary, status))
}

fn writer_comparison_status(
    report: &crate::writer_core::DiffReport,
    previous: &crate::writer_core::InspectReport,
    current: &crate::writer_core::InspectReport,
) -> ComparisonStatus {
    let core_missing = report.uncompared_domains.iter().any(|domain| {
        matches!(
            domain.as_str(),
            "notetypes" | "templates" | "fields" | "metadata" | "card_evidence"
        )
    });
    let card_previous = card_evidence_status(previous);
    let card_current = card_evidence_status(current);
    if report.comparison_status == "unavailable"
        || core_missing
        || matches!(card_previous, CardEvidenceStatus::Missing)
        || matches!(card_current, CardEvidenceStatus::Missing)
    {
        return ComparisonStatus::Unavailable;
    }
    if report.comparison_status == "partial"
        || !report.uncompared_domains.is_empty()
        || !report.comparison_limitations.is_empty()
        || matches!(card_previous, CardEvidenceStatus::Degraded)
        || matches!(card_current, CardEvidenceStatus::Degraded)
    {
        return ComparisonStatus::Partial;
    }
    ComparisonStatus::Complete
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CardEvidenceStatus {
    Full,
    Degraded,
    Missing,
}

fn card_evidence_status(report: &crate::writer_core::InspectReport) -> CardEvidenceStatus {
    let has_card_count = report.observations.metadata.iter().any(|value| {
        value
            .get("card_count")
            .and_then(serde_json::Value::as_u64)
            .is_some()
    });
    if !has_card_count {
        return CardEvidenceStatus::Missing;
    }

    let has_card_references = report.observations.references.iter().any(|value| {
        value
            .get("selector")
            .and_then(serde_json::Value::as_str)
            .map(|selector| selector.starts_with("card["))
            .unwrap_or(false)
    });
    let has_template_ordinals = report.observations.templates.iter().any(|value| {
        value
            .get("ord")
            .and_then(serde_json::Value::as_u64)
            .is_some()
    });

    if has_card_references && has_template_ordinals {
        CardEvidenceStatus::Full
    } else {
        CardEvidenceStatus::Degraded
    }
}
