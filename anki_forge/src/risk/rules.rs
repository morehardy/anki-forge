use crate::risk::ImportRiskReport;

#[derive(Debug, Clone)]
pub struct RiskInput<'a> {
    pub diagnostics: &'a [crate::diagnostics::Diagnostic],
    pub comparison: crate::build::ComparisonStatus,
    pub diff: Option<&'a crate::diff::BuildDiffSummary>,
    pub current_inspect: Option<&'a crate::build::InspectSummary>,
    pub previous_inspect: Option<&'a crate::build::InspectSummary>,
    pub update_safety: Option<&'a crate::build::UpdateSafetySummary>,
}

pub fn classify_import_risk(input: RiskInput<'_>) -> ImportRiskReport {
    let _ = input;
    ImportRiskReport::default()
}
