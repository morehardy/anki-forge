use crate::build::{BuildPolicyResult, RiskLevel};
use crate::risk::ImportRiskReport;

pub fn policy_from_risk_report(
    threshold: Option<RiskLevel>,
    risk: Option<&ImportRiskReport>,
) -> BuildPolicyResult {
    let highest = risk.and_then(|report| report.highest_level);
    let blocking_codes = match (threshold, risk) {
        (Some(threshold), Some(report)) => report.blocking_codes_at_or_above(threshold),
        _ => Vec::new(),
    };
    BuildPolicyResult::evaluate(threshold, highest, blocking_codes)
}
