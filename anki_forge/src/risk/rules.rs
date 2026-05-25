use crate::build::{ComparisonStatus, RiskLevel};
use crate::diagnostics::Severity;
use crate::diff::{EvidenceRef, EvidenceRefKind, SemanticDiffCategory};
use crate::risk::{ImportRiskFinding, ImportRiskReport};

#[derive(Debug, Clone)]
pub struct RiskInput<'a> {
    pub diagnostics: &'a [crate::diagnostics::Diagnostic],
    pub comparison: ComparisonStatus,
    pub diff: Option<&'a crate::diff::BuildDiffSummary>,
    pub current_inspect: Option<&'a crate::build::InspectSummary>,
    pub previous_inspect: Option<&'a crate::build::InspectSummary>,
    pub update_safety: Option<&'a crate::build::UpdateSafetySummary>,
}

pub fn classify_import_risk(input: RiskInput<'_>) -> ImportRiskReport {
    let mut findings = Vec::new();
    let _update_safety_evidence_is_carried_by_diagnostics = input.update_safety;

    if matches!(input.comparison, ComparisonStatus::Unavailable) {
        findings.push(finding(
            "RISK.BASELINE_UNAVAILABLE",
            RiskLevel::High,
            "baseline",
            "compare_to was requested, but the previous APKG could not be inspected completely",
            vec![EvidenceRef {
                kind: EvidenceRefKind::Oracle,
                ref_id: "manual-doc:docs-api-design-phase4-baseline".to_string(),
            }],
            "verify the previous APKG path and rebuild with a readable baseline",
        ));
    }

    for (index, diagnostic) in input.diagnostics.iter().enumerate() {
        let code = diagnostic.code.as_str();
        if matches!(code, "MEDIA.MISSING_REFERENCE" | "MEDIA.UNSAFE_REFERENCE")
            && diagnostic.severity == Severity::Error
        {
            let mut item = finding(
                "RISK.MEDIA_REFERENCE_BROKEN",
                RiskLevel::High,
                "media",
                "current project contains a broken or unsafe media reference",
                vec![EvidenceRef {
                    kind: EvidenceRefKind::Diagnostic,
                    ref_id: format!("diagnostic:{index}:{code}"),
                }],
                "register the media file or update the Product content reference",
            );
            item.source = diagnostic.source.clone();
            findings.push(item);
        }

        if matches!(
            code,
            "UPDATE.GUID_DERIVATION_DRIFT"
                | "UPDATE.BASELINE_CONFLICT_GUID"
                | "UPDATE.GUID_DUPLICATE_AT_RECONCILE"
                | "UPDATE.GUID_DUPLICATE_IN_BASELINE"
        ) {
            let mut item = finding(
                "RISK.NOTE_GUID_DRIFT",
                RiskLevel::High,
                "identity",
                "same stable note identity maps to a different Anki GUID",
                vec![EvidenceRef {
                    kind: EvidenceRefKind::Diagnostic,
                    ref_id: format!("diagnostic:{index}:{code}"),
                }],
                "restore the previous stable id mapping or review the intentional GUID migration",
            );
            item.source = diagnostic.source.clone();
            findings.push(item);
        }

        if matches!(
            code,
            "UPDATE.FIELD_MERGE_ID_CHANGED"
                | "UPDATE.TEMPLATE_MERGE_ID_CHANGED"
                | "UPDATE.FIELD_ORD_CHANGED"
                | "UPDATE.TEMPLATE_ORD_CHANGED"
                | "UPDATE.NOTETYPE_SET_CHANGED"
                | "UPDATE.TEMPLATE_SET_CHANGED"
        ) {
            let mut item = finding(
                "RISK.NOTETYPE_CONFIG_ID_DRIFT",
                RiskLevel::High,
                "notetype",
                "notetype, field, or template merge identity changed unexpectedly",
                vec![EvidenceRef {
                    kind: EvidenceRefKind::Diagnostic,
                    ref_id: format!("diagnostic:{index}:{code}"),
                }],
                "restore stable field/template keys or regenerate the previous package intentionally",
            );
            item.source = diagnostic.source.clone();
            findings.push(item);
        }
    }

    if let Some(diff) = input.diff {
        for (index, change) in diff.semantic_changes.iter().enumerate() {
            for code in &change.risk_codes {
                let level = level_for_semantic_code(code);
                let evidence = vec![EvidenceRef {
                    kind: EvidenceRefKind::DiffChange,
                    ref_id: format!("semantic:{index}:{}", change.selector),
                }];
                findings.push(ImportRiskFinding {
                    code: code.clone(),
                    level,
                    category: semantic_category_name(change.category).to_string(),
                    message: change.message.clone(),
                    source: change.source.clone(),
                    evidence_refs: evidence,
                    suggested_action: suggested_action_for_code(code).map(str::to_string),
                });
            }
        }
    }

    promote_card_count_with_template_removed(&mut findings);
    ImportRiskReport::from_findings(findings)
}

fn finding(
    code: &str,
    level: RiskLevel,
    category: &str,
    message: &str,
    evidence_refs: Vec<EvidenceRef>,
    suggested_action: &str,
) -> ImportRiskFinding {
    ImportRiskFinding {
        code: code.to_string(),
        level,
        category: category.to_string(),
        message: message.to_string(),
        source: None,
        evidence_refs,
        suggested_action: Some(suggested_action.to_string()),
    }
}

fn level_for_semantic_code(code: &str) -> RiskLevel {
    match code {
        "RISK.TEMPLATE_REMOVED" => RiskLevel::Critical,
        "RISK.TEMPLATE_REORDER" => RiskLevel::High,
        "RISK.FIELD_REMOVED_OR_RENAMED" => RiskLevel::Medium,
        "RISK.CARD_COUNT_CHANGED" => RiskLevel::Medium,
        "RISK.MEDIA_REMOVED" => RiskLevel::Medium,
        "RISK.NOTE_GUID_DRIFT" => RiskLevel::High,
        "RISK.NOTETYPE_CONFIG_ID_DRIFT" => RiskLevel::High,
        "RISK.MEDIA_REFERENCE_BROKEN" => RiskLevel::High,
        "RISK.BASELINE_UNAVAILABLE" => RiskLevel::High,
        _ => RiskLevel::Low,
    }
}

fn suggested_action_for_code(code: &str) -> Option<&'static str> {
    match code {
        "RISK.TEMPLATE_REMOVED" => Some("restore the template or document the card migration"),
        "RISK.TEMPLATE_REORDER" => Some("preserve template keys and ordinals for existing cards"),
        "RISK.FIELD_REMOVED_OR_RENAMED" => {
            Some("preserve the field key/config id or confirm the migration")
        }
        "RISK.CARD_COUNT_CHANGED" => {
            Some("review expected card generation changes before importing")
        }
        "RISK.MEDIA_REMOVED" => Some("restore removed media or verify no notes reference it"),
        _ => None,
    }
}

fn semantic_category_name(category: SemanticDiffCategory) -> &'static str {
    match category {
        SemanticDiffCategory::Notetype => "notetype",
        SemanticDiffCategory::Field => "field",
        SemanticDiffCategory::Template => "template",
        SemanticDiffCategory::NoteIdentity => "note_identity",
        SemanticDiffCategory::CardCount => "card_count",
        SemanticDiffCategory::Media => "media",
        SemanticDiffCategory::Baseline => "baseline",
    }
}

fn promote_card_count_with_template_removed(findings: &mut [ImportRiskFinding]) {
    let template_removed = findings
        .iter()
        .any(|finding| finding.code == "RISK.TEMPLATE_REMOVED");
    if !template_removed {
        return;
    }

    for finding in findings {
        if finding.code == "RISK.CARD_COUNT_CHANGED" {
            finding.level = RiskLevel::High;
            finding.evidence_refs.push(EvidenceRef {
                kind: EvidenceRefKind::DiffChange,
                ref_id: "linked:RISK.TEMPLATE_REMOVED".to_string(),
            });
        }
    }
}
