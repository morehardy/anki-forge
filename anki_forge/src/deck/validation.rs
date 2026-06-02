use crate::diagnostics::{ErrorCode, ErrorCodeExt, Severity};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ValidationCode {
    MissingStableId,
    DuplicateStableId,
    BlankStableId,
    EmptyIoMasks,
    UnknownMediaRef,
    NoteLevelIdentityOverrideUsed,
    IdentityDuplicatePayload,
    IdentityCollision,
    StableIdDuplicate,
}

impl ValidationCode {
    pub fn code(&self) -> ErrorCode {
        match self {
            Self::MissingStableId => ErrorCode::DeckMissingStableId,
            Self::DuplicateStableId => ErrorCode::DeckDuplicateStableId,
            Self::BlankStableId => ErrorCode::StableIdBlank,
            Self::EmptyIoMasks => ErrorCode::ImageOcclusionEmptyMasks,
            Self::UnknownMediaRef => ErrorCode::ImageOcclusionUnknownMedia,
            Self::NoteLevelIdentityOverrideUsed => ErrorCode::DeckNoteLevelIdentityOverrideUsed,
            Self::IdentityDuplicatePayload => ErrorCode::IdentityDuplicatePayload,
            Self::IdentityCollision => ErrorCode::IdentityCollision,
            Self::StableIdDuplicate => ErrorCode::StableIdDuplicate,
        }
    }
}

impl ErrorCodeExt for ValidationCode {
    fn code(&self) -> ErrorCode {
        ValidationCode::code(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ValidationDiagnostic {
    pub code: ValidationCode,
    pub message: String,
    pub severity: Severity,
}

impl ErrorCodeExt for ValidationDiagnostic {
    fn code(&self) -> ErrorCode {
        self.code.code()
    }
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
        self.diagnostics
            .iter()
            .any(|item| item.severity == Severity::Error)
    }
}
