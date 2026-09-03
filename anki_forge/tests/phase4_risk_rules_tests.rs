#![cfg(feature = "internal-tools")]

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
        domain: None,
        stage: None,
        message: format!("{code} message"),
        source: None,
        help: None,
    }
}

fn diagnostic_with_source(code: &str, severity: Severity, source: &str) -> Diagnostic {
    Diagnostic {
        code: DiagnosticCode::new(code),
        severity,
        domain: None,
        stage: None,
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
fn template_ord_changed_preserves_source_and_only_skips_own_config_id_drift() {
    let diagnostics = vec![
        diagnostic_with_source(
            "UPDATE.TEMPLATE_ORD_CHANGED",
            Severity::Warning,
            "project.notetypes[0].templates[production].ord",
        ),
        diagnostic("UPDATE.FIELD_MERGE_ID_CHANGED", Severity::Warning),
    ];
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
        .find(|finding| finding.code == "RISK.TEMPLATE_REORDER")
        .expect("template reorder finding");
    assert_eq!(finding.level, RiskLevel::High);
    assert_eq!(
        finding.source.as_ref().map(SourcePath::as_str),
        Some("project.notetypes[0].templates[production].ord")
    );
    assert_eq!(
        report
            .findings
            .iter()
            .filter(|finding| finding.code == "RISK.NOTETYPE_CONFIG_ID_DRIFT")
            .count(),
        1
    );
    assert!(report
        .findings
        .iter()
        .any(|finding| finding.code == "RISK.NOTETYPE_CONFIG_ID_DRIFT"));
}

#[test]
fn template_ord_changed_merges_semantic_evidence_without_duplicate_reorder_findings() {
    let diagnostics = vec![diagnostic_with_source(
        "UPDATE.TEMPLATE_ORD_CHANGED",
        Severity::Warning,
        "selector:1",
    )];
    let diff = diff_with_semantic_change("RISK.TEMPLATE_REORDER", SemanticDiffCategory::Template);

    let report = classify_import_risk(RiskInput {
        diagnostics: &diagnostics,
        comparison: ComparisonStatus::Complete,
        diff: Some(&diff),
        current_inspect: None,
        previous_inspect: None,
        update_safety: None,
    });

    let reorder_findings = report
        .findings
        .iter()
        .filter(|finding| finding.code == "RISK.TEMPLATE_REORDER")
        .collect::<Vec<_>>();
    assert_eq!(reorder_findings.len(), 1);
    let finding = reorder_findings[0];
    assert!(finding
        .evidence_refs
        .iter()
        .any(|evidence| evidence.kind == EvidenceRefKind::Diagnostic));
    assert!(finding
        .evidence_refs
        .iter()
        .any(|evidence| evidence.ref_id.starts_with("semantic:")));
    assert_eq!(
        finding.source.as_ref().map(SourcePath::as_str),
        Some("selector:1")
    );
}

#[test]
fn template_ord_changed_keeps_unmatched_semantic_reorder_separate() {
    let diagnostics = vec![diagnostic_with_source(
        "UPDATE.TEMPLATE_ORD_CHANGED",
        Severity::Warning,
        "notetype[id='jp-vocab']::template[Production]",
    )];
    let diff = diff_with_semantic_change("RISK.TEMPLATE_REORDER", SemanticDiffCategory::Template);

    let report = classify_import_risk(RiskInput {
        diagnostics: &diagnostics,
        comparison: ComparisonStatus::Complete,
        diff: Some(&diff),
        current_inspect: None,
        previous_inspect: None,
        update_safety: None,
    });

    assert_eq!(
        report
            .findings
            .iter()
            .filter(|finding| finding.code == "RISK.TEMPLATE_REORDER")
            .count(),
        2
    );
}

#[test]
fn template_ord_changed_does_not_attach_unrelated_template_artifact_diff() {
    let diagnostics = vec![diagnostic_with_source(
        "UPDATE.TEMPLATE_ORD_CHANGED",
        Severity::Warning,
        "notetype[id='jp-vocab']::template[Production]",
    )];
    let diff = BuildDiffSummary {
        artifact_diff: Some(ArtifactDiffSummary {
            changes: vec![ArtifactDiffChange {
                category: "modified".to_string(),
                domain: "templates".to_string(),
                severity: "medium".to_string(),
                selector: "notetype[id='jp-vocab']::template[Recognition]".to_string(),
                message: "template front changed".to_string(),
                evidence_refs: Vec::new(),
            }],
            limitations: Vec::new(),
        }),
        semantic_changes: Vec::new(),
        summary_counts: DiffSummaryCounts {
            added: 0,
            removed: 0,
            modified: 1,
            reordered: 0,
            uncompared_domains: 0,
        },
        limitations: Vec::new(),
    };

    let report = classify_import_risk(RiskInput {
        diagnostics: &diagnostics,
        comparison: ComparisonStatus::Complete,
        diff: Some(&diff),
        current_inspect: None,
        previous_inspect: None,
        update_safety: None,
    });

    let finding = report
        .findings
        .iter()
        .find(|finding| finding.code == "RISK.TEMPLATE_REORDER")
        .expect("template reorder finding");
    assert!(!finding
        .evidence_refs
        .iter()
        .any(|evidence| evidence.ref_id.starts_with("diff:templates")));
}

#[test]
fn template_ord_changed_attaches_matching_template_artifact_diff() {
    let diagnostics = vec![diagnostic_with_source(
        "UPDATE.TEMPLATE_ORD_CHANGED",
        Severity::Warning,
        "notetype[id='jp-vocab']::template[Production]",
    )];
    let diff = BuildDiffSummary {
        artifact_diff: Some(ArtifactDiffSummary {
            changes: vec![ArtifactDiffChange {
                category: "modified".to_string(),
                domain: "templates".to_string(),
                severity: "medium".to_string(),
                selector: "notetype[id='jp-vocab']::template[Production]".to_string(),
                message: "template changed".to_string(),
                evidence_refs: Vec::new(),
            }],
            limitations: Vec::new(),
        }),
        semantic_changes: Vec::new(),
        summary_counts: DiffSummaryCounts {
            added: 0,
            removed: 0,
            modified: 1,
            reordered: 0,
            uncompared_domains: 0,
        },
        limitations: Vec::new(),
    };

    let report = classify_import_risk(RiskInput {
        diagnostics: &diagnostics,
        comparison: ComparisonStatus::Complete,
        diff: Some(&diff),
        current_inspect: None,
        previous_inspect: None,
        update_safety: None,
    });

    let finding = report
        .findings
        .iter()
        .find(|finding| finding.code == "RISK.TEMPLATE_REORDER")
        .expect("template reorder finding");
    assert!(finding
        .evidence_refs
        .iter()
        .any(|evidence| evidence.ref_id.starts_with("diff:templates")));
}
