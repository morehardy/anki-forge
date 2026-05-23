use crate::diagnostics::Severity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceCondition {
    StrictCompareOnly,
    LockfileRequired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateDiagnosticClass {
    pub limitation: Option<&'static str>,
    pub diagnostic_code: Option<&'static str>,
    pub severity: Severity,
}

pub fn classify_project_stable_id_missing(condition: EvidenceCondition) -> UpdateDiagnosticClass {
    match condition {
        EvidenceCondition::StrictCompareOnly => UpdateDiagnosticClass {
            limitation: Some("project_stable_id_missing"),
            diagnostic_code: Some("UPDATE.PROJECT_STABLE_ID_MISSING"),
            severity: Severity::Warning,
        },
        EvidenceCondition::LockfileRequired => UpdateDiagnosticClass {
            limitation: Some("project_stable_id_missing"),
            diagnostic_code: Some("UPDATE.PROJECT_STABLE_ID_MISSING"),
            severity: Severity::Error,
        },
    }
}
