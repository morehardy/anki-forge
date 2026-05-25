use crate::diff::{
    ArtifactDiffChange, ArtifactDiffSummary, BuildDiffSummary, DiffSummaryCounts, EvidenceRef,
    EvidenceRefKind, SemanticDiffCategory, SemanticDiffChange, SemanticDiffChangeKind,
};

pub fn summarize_writer_diff(report: &writer_core::DiffReport) -> BuildDiffSummary {
    let changes = report
        .changes
        .iter()
        .enumerate()
        .map(|(index, change)| ArtifactDiffChange {
            category: change.category.clone(),
            domain: change.domain.clone(),
            severity: change.severity.clone(),
            selector: change.selector.clone(),
            message: change.message.clone(),
            evidence_refs: vec![EvidenceRef {
                kind: EvidenceRefKind::DiffChange,
                ref_id: format!("diff:{}:{index}", change.domain),
            }],
        })
        .collect::<Vec<_>>();

    let mut counts = DiffSummaryCounts {
        uncompared_domains: report.uncompared_domains.len(),
        ..DiffSummaryCounts::default()
    };
    for change in &changes {
        match change.category.as_str() {
            "added" => counts.added += 1,
            "removed" => counts.removed += 1,
            "modified" => counts.modified += 1,
            _ => counts.modified += 1,
        }
    }

    let semantic_changes = report
        .changes
        .iter()
        .filter_map(semantic_change_from_writer_change)
        .collect::<Vec<_>>();
    let mut limitations = report.comparison_limitations.clone();
    if semantic_changes.iter().any(|change| {
        change
            .risk_codes
            .iter()
            .any(|code| code == "RISK.FIELD_REMOVED_OR_RENAMED")
    }) {
        limitations.push(
            "field_safe_rename_proof_not_applied: inspect exposes field config_id, but the first Product diff adapter does not receive paired before/after field payloads; field removal or rename is treated as unsafe"
                .to_string(),
        );
    }

    BuildDiffSummary {
        artifact_diff: Some(ArtifactDiffSummary {
            changes,
            limitations: limitations.clone(),
        }),
        semantic_changes,
        summary_counts: counts,
        limitations,
    }
}

fn semantic_change_from_writer_change(
    change: &writer_core::DiffChange,
) -> Option<SemanticDiffChange> {
    let (category, change_kind, risk_code) =
        match (change.domain.as_str(), change.category.as_str()) {
            ("templates", "removed") => (
                SemanticDiffCategory::Template,
                SemanticDiffChangeKind::Removed,
                "RISK.TEMPLATE_REMOVED",
            ),
            ("templates", "modified")
                if change.message.contains("ord") || change.selector.contains("ord") =>
            {
                (
                    SemanticDiffCategory::Template,
                    SemanticDiffChangeKind::Reordered,
                    "RISK.TEMPLATE_REORDER",
                )
            }
            ("fields", "removed") => (
                SemanticDiffCategory::Field,
                SemanticDiffChangeKind::Removed,
                "RISK.FIELD_REMOVED_OR_RENAMED",
            ),
            ("media", "removed") => (
                SemanticDiffCategory::Media,
                SemanticDiffChangeKind::Removed,
                "RISK.MEDIA_REMOVED",
            ),
            ("metadata", "modified")
                if change.selector == "counts" || change.message.contains("counts") =>
            {
                (
                    SemanticDiffCategory::CardCount,
                    SemanticDiffChangeKind::Modified,
                    "RISK.CARD_COUNT_CHANGED",
                )
            }
            _ => return None,
        };

    Some(SemanticDiffChange {
        category,
        selector: change.selector.clone(),
        change_kind,
        risk_codes: vec![risk_code.to_string()],
        message: change.message.clone(),
        source: None,
    })
}
