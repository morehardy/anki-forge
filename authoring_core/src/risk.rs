use crate::model::{ComparisonContext, MergeRiskReport, NormalizedIr};

const BASELINE_UNAVAILABLE: &str = "BASELINE_UNAVAILABLE";
const BASELINE_IDENTITY_INDEX_ONLY: &str = "BASELINE_IDENTITY_INDEX_ONLY";

pub fn assess_risk(
    current: &NormalizedIr,
    comparison: Option<&ComparisonContext>,
) -> Option<MergeRiskReport> {
    comparison.map(|context| {
        if context.comparison_mode == "strict"
            && context.baseline_artifact_fingerprint.trim().is_empty()
        {
            report(
                context,
                current.resolved_identity.clone(),
                "unavailable",
                "high",
                vec![BASELINE_UNAVAILABLE.into()],
            )
        } else if context.baseline_kind == "identity_index" {
            report(
                context,
                current.resolved_identity.clone(),
                "partial",
                "medium",
                vec![BASELINE_IDENTITY_INDEX_ONLY.into()],
            )
        } else {
            report(
                context,
                current.resolved_identity.clone(),
                "complete",
                "low",
                Vec::new(),
            )
        }
    })
}

pub fn unavailable_report(
    comparison: Option<&ComparisonContext>,
    current_artifact_fingerprint: String,
    comparison_reason: String,
) -> Option<MergeRiskReport> {
    comparison.map(|context| {
        let mut comparison_reasons = Vec::new();
        if context.comparison_mode == "strict"
            && context.baseline_artifact_fingerprint.trim().is_empty()
        {
            comparison_reasons.push(BASELINE_UNAVAILABLE.into());
        }
        comparison_reasons.push(comparison_reason);

        report(
            context,
            current_artifact_fingerprint,
            "unavailable",
            "unknown",
            comparison_reasons,
        )
    })
}

fn report(
    context: &ComparisonContext,
    current_artifact_fingerprint: String,
    comparison_status: &str,
    overall_level: &str,
    comparison_reasons: Vec<String>,
) -> MergeRiskReport {
    MergeRiskReport {
        kind: "merge-risk-report".into(),
        comparison_status: comparison_status.into(),
        overall_level: overall_level.into(),
        policy_version: context.risk_policy_ref.clone(),
        baseline_artifact_fingerprint: context.baseline_artifact_fingerprint.clone(),
        current_artifact_fingerprint,
        comparison_reasons,
    }
}
