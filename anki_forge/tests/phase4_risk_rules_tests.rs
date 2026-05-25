use anki_forge::build::{ComparisonStatus, RiskLevel};
use anki_forge::diagnostics::{Diagnostic, DiagnosticCode, Severity, SourcePath};
use anki_forge::diff::{
    ArtifactDiffChange, ArtifactDiffSummary, BuildDiffSummary, DiffSummaryCounts, EvidenceRef,
    EvidenceRefKind, SemanticDiffCategory, SemanticDiffChange, SemanticDiffChangeKind,
};
use anki_forge::risk::rules::{classify_import_risk, RiskInput};

fn diagnostic(code: &str, severity: Severity) -> Diagnostic {
    Diagnostic {
        code: DiagnosticCode::new(code),
        severity,
        message: format!("{code} message"),
        source: None,
        help: None,
    }
}

fn diagnostic_with_source(code: &str, severity: Severity, source: &str) -> Diagnostic {
    Diagnostic {
        code: DiagnosticCode::new(code),
        severity,
        message: format!("{code} message"),
        source: Some(SourcePath::new(source)),
        help: None,
    }
}

fn diff_with_semantic_change(code: &str, category: SemanticDiffCategory) -> BuildDiffSummary {
    BuildDiffSummary {
        artifact_diff: Some(ArtifactDiffSummary {
            changes: vec![ArtifactDiffChange {
                category: "removed".to_string(),
                domain: "templates".to_string(),
                severity: "high".to_string(),
                selector: "selector:1".to_string(),
                message: "artifact change".to_string(),
                evidence_refs: vec![EvidenceRef {
                    kind: EvidenceRefKind::DiffChange,
                    ref_id: "diff:templates:0".to_string(),
                }],
            }],
            limitations: Vec::new(),
        }),
        semantic_changes: vec![SemanticDiffChange {
            category,
            selector: "selector:1".to_string(),
            change_kind: SemanticDiffChangeKind::Removed,
            risk_codes: vec![code.to_string()],
            message: "semantic change".to_string(),
            source: None,
        }],
        summary_counts: DiffSummaryCounts {
            added: 0,
            removed: 1,
            modified: 0,
            reordered: 0,
            uncompared_domains: 0,
        },
        limitations: Vec::new(),
    }
}

#[test]
fn baseline_unavailable_emits_high_risk() {
    let report = classify_import_risk(RiskInput {
        diagnostics: &[],
        comparison: ComparisonStatus::Unavailable,
        diff: None,
        current_inspect: None,
        previous_inspect: None,
        update_safety: None,
    });

    let finding = report
        .findings
        .iter()
        .find(|finding| finding.code == "RISK.BASELINE_UNAVAILABLE")
        .expect("baseline finding");
    assert_eq!(finding.level, RiskLevel::High);
}

#[test]
fn broken_media_reference_emits_high_risk_without_baseline() {
    let diagnostics = vec![diagnostic("MEDIA.MISSING_REFERENCE", Severity::Error)];
    let report = classify_import_risk(RiskInput {
        diagnostics: &diagnostics,
        comparison: ComparisonStatus::NotRequested,
        diff: None,
        current_inspect: None,
        previous_inspect: None,
        update_safety: None,
    });

    let finding = report
        .findings
        .iter()
        .find(|finding| finding.code == "RISK.MEDIA_REFERENCE_BROKEN")
        .expect("media finding");
    assert_eq!(finding.level, RiskLevel::High);
}

#[test]
fn template_removed_emits_critical_and_promotes_card_count() {
    let mut diff =
        diff_with_semantic_change("RISK.TEMPLATE_REMOVED", SemanticDiffCategory::Template);
    diff.semantic_changes.push(SemanticDiffChange {
        category: SemanticDiffCategory::CardCount,
        selector: "cards".to_string(),
        change_kind: SemanticDiffChangeKind::Modified,
        risk_codes: vec!["RISK.CARD_COUNT_CHANGED".to_string()],
        message: "card count changed".to_string(),
        source: None,
    });

    let report = classify_import_risk(RiskInput {
        diagnostics: &[],
        comparison: ComparisonStatus::Complete,
        diff: Some(&diff),
        current_inspect: None,
        previous_inspect: None,
        update_safety: None,
    });

    let template = report
        .findings
        .iter()
        .find(|finding| finding.code == "RISK.TEMPLATE_REMOVED")
        .expect("template finding");
    let card = report
        .findings
        .iter()
        .find(|finding| finding.code == "RISK.CARD_COUNT_CHANGED")
        .expect("card finding");

    assert_eq!(template.level, RiskLevel::Critical);
    assert_eq!(card.level, RiskLevel::High);
    assert_eq!(report.highest_level, Some(RiskLevel::Critical));
}

#[test]
fn guid_drift_preserves_diagnostic_source() {
    let diagnostics = vec![diagnostic_with_source(
        "UPDATE.GUID_DERIVATION_DRIFT",
        Severity::Warning,
        "project.notes[0]",
    )];
    let report = classify_import_risk(RiskInput {
        diagnostics: &diagnostics,
        comparison: ComparisonStatus::Complete,
        diff: None,
        current_inspect: None,
        previous_inspect: None,
        update_safety: None,
    });

    let finding = report
        .findings
        .iter()
        .find(|finding| finding.code == "RISK.NOTE_GUID_DRIFT")
        .expect("guid drift finding");
    assert_eq!(
        finding.source.as_ref().map(SourcePath::as_str),
        Some("project.notes[0]")
    );
}

#[test]
fn notetype_config_id_drift_preserves_diagnostic_source() {
    let diagnostics = vec![diagnostic_with_source(
        "UPDATE.FIELD_MERGE_ID_CHANGED",
        Severity::Warning,
        "project.notetypes[0].fields[expr]",
    )];
    let report = classify_import_risk(RiskInput {
        diagnostics: &diagnostics,
        comparison: ComparisonStatus::Complete,
        diff: None,
        current_inspect: None,
        previous_inspect: None,
        update_safety: None,
    });

    let finding = report
        .findings
        .iter()
        .find(|finding| finding.code == "RISK.NOTETYPE_CONFIG_ID_DRIFT")
        .expect("config id drift finding");
    assert_eq!(
        finding.source.as_ref().map(SourcePath::as_str),
        Some("project.notetypes[0].fields[expr]")
    );
}

#[test]
fn template_ord_changed_stays_config_id_drift_for_task_5() {
    let diagnostics = vec![diagnostic("UPDATE.TEMPLATE_ORD_CHANGED", Severity::Warning)];
    let report = classify_import_risk(RiskInput {
        diagnostics: &diagnostics,
        comparison: ComparisonStatus::Complete,
        diff: None,
        current_inspect: None,
        previous_inspect: None,
        update_safety: None,
    });

    assert!(report
        .findings
        .iter()
        .any(|finding| finding.code == "RISK.NOTETYPE_CONFIG_ID_DRIFT"));
    assert!(!report
        .findings
        .iter()
        .any(|finding| finding.code == "RISK.TEMPLATE_REORDER"));
}
