#![cfg(feature = "internal-tools")]

use anki_forge::build::{BuildStatus, ComparisonStatus, RiskLevel};
use anki_forge::diff::{
    ArtifactDiffChange, ArtifactDiffSummary, BuildDiffSummary, DiffSummaryCounts, EvidenceRef,
    EvidenceRefKind, ProjectDiffReport, SemanticDiffCategory, SemanticDiffChange,
    SemanticDiffChangeKind,
};
use anki_forge::risk::{ImportRiskFinding, ImportRiskReport};

#[test]
fn diff_summary_counts_artifact_and_semantic_changes() {
    let summary = BuildDiffSummary {
        artifact_diff: Some(ArtifactDiffSummary {
            changes: vec![ArtifactDiffChange {
                category: "removed".to_string(),
                domain: "templates".to_string(),
                severity: "high".to_string(),
                selector: "notetype:jp/template:Recognition".to_string(),
                message: "template removed".to_string(),
                evidence_refs: vec![EvidenceRef {
                    kind: EvidenceRefKind::DiffChange,
                    ref_id: "diff:templates:0".to_string(),
                }],
            }],
            limitations: Vec::new(),
        }),
        semantic_changes: vec![SemanticDiffChange {
            category: SemanticDiffCategory::Template,
            selector: "notetype:jp/template:Recognition".to_string(),
            change_kind: SemanticDiffChangeKind::Removed,
            risk_codes: vec!["RISK.TEMPLATE_REMOVED".to_string()],
            message: "template Recognition was removed".to_string(),
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
    };

    assert_eq!(summary.summary_counts.removed, 1);
    assert_eq!(
        summary.semantic_changes[0].risk_codes[0],
        "RISK.TEMPLATE_REMOVED"
    );
}

#[test]
fn import_risk_report_computes_highest_level() {
    let report = ImportRiskReport::from_findings(vec![
        ImportRiskFinding {
            code: "RISK.FIELD_REMOVED_OR_RENAMED".to_string(),
            level: RiskLevel::Medium,
            category: "field".to_string(),
            message: "field removed".to_string(),
            source: None,
            evidence_refs: Vec::new(),
            suggested_action: Some(
                "restore the stable field key or confirm the rename".to_string(),
            ),
        },
        ImportRiskFinding {
            code: "RISK.TEMPLATE_REMOVED".to_string(),
            level: RiskLevel::Critical,
            category: "template".to_string(),
            message: "template removed".to_string(),
            source: None,
            evidence_refs: Vec::new(),
            suggested_action: Some("restore the template or migrate existing cards".to_string()),
        },
    ]);

    assert_eq!(report.highest_level, Some(RiskLevel::Critical));
}

#[test]
fn writer_diff_summary_maps_template_removal_to_semantic_risk() {
    let writer = anki_forge::writer::DiffReport {
        kind: "inspect-diff".to_string(),
        comparison_status: "complete".to_string(),
        left_fingerprint: "left".to_string(),
        right_fingerprint: "right".to_string(),
        left_observation_model_version: "v1".to_string(),
        right_observation_model_version: "v1".to_string(),
        summary: "1 change".to_string(),
        uncompared_domains: Vec::new(),
        comparison_limitations: Vec::new(),
        changes: vec![anki_forge::writer::DiffChange {
            category: "removed".to_string(),
            domain: "templates".to_string(),
            severity: "medium".to_string(),
            selector: "notetype[jp] template[Recognition]".to_string(),
            message: "template removed".to_string(),
            compatibility_hint: "review import behavior".to_string(),
            evidence_refs: Vec::new(),
        }],
    };

    let summary = anki_forge::diff::summarize_writer_diff(&writer);
    assert_eq!(summary.summary_counts.removed, 1);
    assert_eq!(summary.semantic_changes.len(), 1);
    assert_eq!(
        summary.semantic_changes[0].category,
        SemanticDiffCategory::Template
    );
    assert_eq!(
        summary.semantic_changes[0].risk_codes,
        vec!["RISK.TEMPLATE_REMOVED".to_string()]
    );
}

#[test]
fn project_diff_report_roundtrips_through_json() {
    let report = ProjectDiffReport {
        status: BuildStatus::Success,
        comparison: ComparisonStatus::Complete,
        diagnostics: Vec::new(),
        current_inspect: None,
        previous_inspect: None,
        update_safety: None,
        diff: None,
        risk: None,
        metrics: anki_forge::diff::ComparisonMetrics { duration_ms: 7 },
    };

    let json = serde_json::to_string(&report).expect("serialize project diff report");
    let decoded: ProjectDiffReport =
        serde_json::from_str(&json).expect("deserialize project diff report");

    assert_eq!(decoded, report);
}

#[test]
fn writer_diff_summary_does_not_map_generic_template_modified_diff_to_reorder_risk() {
    let writer = anki_forge::writer::DiffReport {
        kind: "inspect-diff".to_string(),
        comparison_status: "complete".to_string(),
        left_fingerprint: "left".to_string(),
        right_fingerprint: "right".to_string(),
        left_observation_model_version: "v1".to_string(),
        right_observation_model_version: "v1".to_string(),
        summary: "1 change".to_string(),
        uncompared_domains: Vec::new(),
        comparison_limitations: Vec::new(),
        changes: vec![anki_forge::writer::DiffChange {
            category: "modified".to_string(),
            domain: "templates".to_string(),
            severity: "medium".to_string(),
            selector: "notetype[id='jp']::template[Recognition]".to_string(),
            message: "notetype[id='jp']::template[Recognition] changed".to_string(),
            compatibility_hint: "review import behavior".to_string(),
            evidence_refs: Vec::new(),
        }],
    };

    let summary = anki_forge::diff::summarize_writer_diff(&writer);

    assert_eq!(summary.semantic_changes, Vec::<SemanticDiffChange>::new());
}

#[test]
fn writer_diff_summary_maps_template_modified_ord_evidence_to_reorder_risk() {
    let writer = anki_forge::writer::DiffReport {
        kind: "inspect-diff".to_string(),
        comparison_status: "complete".to_string(),
        left_fingerprint: "left".to_string(),
        right_fingerprint: "right".to_string(),
        left_observation_model_version: "v1".to_string(),
        right_observation_model_version: "v1".to_string(),
        summary: "1 change".to_string(),
        uncompared_domains: Vec::new(),
        comparison_limitations: Vec::new(),
        changes: vec![anki_forge::writer::DiffChange {
            category: "modified".to_string(),
            domain: "templates".to_string(),
            severity: "medium".to_string(),
            selector: "notetype[id='jp']::template[Recognition]::ord".to_string(),
            message: "template ord changed".to_string(),
            compatibility_hint: "review import behavior".to_string(),
            evidence_refs: Vec::new(),
        }],
    };

    let summary = anki_forge::diff::summarize_writer_diff(&writer);

    assert_eq!(summary.summary_counts.reordered, 1);
    assert_eq!(summary.semantic_changes.len(), 1);
    assert_eq!(
        summary.semantic_changes[0].change_kind,
        SemanticDiffChangeKind::Reordered
    );
    assert_eq!(
        summary.semantic_changes[0].risk_codes,
        vec!["RISK.TEMPLATE_REORDER".to_string()]
    );
}

#[test]
fn writer_diff_summary_maps_real_counts_metadata_diff_to_card_count_risk() {
    let writer = anki_forge::writer::DiffReport {
        kind: "inspect-diff".to_string(),
        comparison_status: "complete".to_string(),
        left_fingerprint: "left".to_string(),
        right_fingerprint: "right".to_string(),
        left_observation_model_version: "v1".to_string(),
        right_observation_model_version: "v1".to_string(),
        summary: "1 change".to_string(),
        uncompared_domains: Vec::new(),
        comparison_limitations: Vec::new(),
        changes: vec![anki_forge::writer::DiffChange {
            category: "modified".to_string(),
            domain: "metadata".to_string(),
            severity: "medium".to_string(),
            selector: "counts".to_string(),
            message: "counts changed".to_string(),
            compatibility_hint: "review import behavior".to_string(),
            evidence_refs: Vec::new(),
        }],
    };

    let summary = anki_forge::diff::summarize_writer_diff(&writer);

    assert_eq!(summary.semantic_changes.len(), 1);
    assert_eq!(
        summary.semantic_changes[0].category,
        SemanticDiffCategory::CardCount
    );
    assert_eq!(
        summary.semantic_changes[0].risk_codes,
        vec!["RISK.CARD_COUNT_CHANGED".to_string()]
    );
}
