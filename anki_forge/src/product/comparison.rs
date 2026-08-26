use std::path::Path;
use std::time::{Duration, Instant};

use crate::build::{BuildStatus, ComparisonStatus, InspectSummary, UpdateSafetySummary};
use crate::diagnostics::{Diagnostic, DiagnosticCode, Severity, SourcePath};
use crate::diff::{summarize_writer_diff, BuildDiffSummary};
use crate::risk::rules::{classify_import_risk, RiskInput};
use crate::risk::ImportRiskReport;

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
    let mut diagnostics = input.diagnostics.to_vec();
    let current = match inspect_artifact(input.current_artifact) {
        Ok(artifact) => Some(artifact),
        Err(message) => {
            diagnostics.push(Diagnostic {
                code: DiagnosticCode::new("COMPARE.CURRENT_UNAVAILABLE"),
                severity: Severity::Error,
                domain: None,
                stage: None,
                message,
                source: Some(SourcePath::new(
                    input.current_artifact.display().to_string(),
                )),
                help: Some("verify the current APKG path and package contents".to_string()),
            });
            None
        }
    };
    let Some(previous_artifact) = input.previous_artifact else {
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
    let previous = match inspect_artifact(previous_artifact) {
        Ok(artifact) => Some(artifact),
        Err(message) => {
            baseline_unavailable = true;
            diagnostics.push(Diagnostic {
                code: DiagnosticCode::new("COMPARE.BASELINE_UNAVAILABLE"),
                severity: Severity::Error,
                domain: None,
                stage: None,
                message,
                source: Some(SourcePath::new(previous_artifact.display().to_string())),
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
        previous_inspect: previous.map(|artifact| artifact.summary),
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

fn inspect_artifact(path: &Path) -> Result<InspectedArtifact, String> {
    crate::writer_core::inspect_apkg(path)
        .map_err(|err| format!("APKG could not be inspected: {}: {err}", path.display()))
        .map(|report| {
            let summary = inspect_summary_from_report(&report);
            InspectedArtifact { report, summary }
        })
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
