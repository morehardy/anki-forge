#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ValidationCode {
    MissingStableId,
    DuplicateStableId,
    BlankStableId,
    EmptyIoMasks,
    UnknownMediaRef,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ValidationDiagnostic {
    pub code: ValidationCode,
    pub message: String,
    pub severity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub struct ValidationReport {
    diagnostics: Vec<ValidationDiagnostic>,
}

impl ValidationReport {
    pub fn new(diagnostics: Vec<ValidationDiagnostic>) -> Self {
        Self { diagnostics }
    }

    pub fn diagnostics(&self) -> &[ValidationDiagnostic] {
        &self.diagnostics
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|item| item.severity == "error")
    }
}
