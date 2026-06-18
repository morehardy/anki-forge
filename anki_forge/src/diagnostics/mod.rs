use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    StableIdBlank,
    StableIdDuplicate,
    IdentityDuplicatePayload,
    IdentityCollision,
    IdentitySnapshotMissing,
    IdentitySnapshotNoteIdMismatch,
    IdentitySnapshotHashMismatch,
    IdentitySnapshotIncomplete,
    IdentityComponentEmpty,
    IdentityFieldsEmpty,
    ReservedAfidNamespace,
    NoteLevelIdentityOverrideReasonRequired,
    ClozeMalformed,
    ClozeOrdInvalid,
    ClozeNestedUnsupported,
    ImageOcclusionImageDimensionsMissing,
    ImageOcclusionRectEmpty,
    ImageOcclusionRectOutOfBounds,
    ImageOcclusionRectDuplicate,
    ImageOcclusionUnknownMedia,
    ImageOcclusionEmptyMasks,
    DeckMissingStableId,
    DeckDuplicateStableId,
    DeckNoteLevelIdentityOverrideUsed,
    MediaUnsafeFilename,
    MediaDuplicateFilenameConflict,
    MediaSourceMissing,
    MediaSourceNotRegularFile,
    MediaSourceReadFailed,
    MediaSourceChanged,
    MediaEmptySource,
    MediaInvalidSourceLabel,
    MediaInlineTooLarge,
    MediaCasWriteFailed,
    MediaCasObjectMissing,
    ProjectBuildMissingArtifact,
    ProjectBuildDiagnostics,
    ProjectBuildPolicyBlocked,
    ProjectBuildInvalid,
    ProjectBuildIo,
    ProjectBuildInternal,
    ProjectBuildStatusFailed,
    ProjectProductDocumentSourceMixed,
    ProjectProductMediaFailed,
    ProjectProductMediaStagingCollision,
    ProjectLowerFailed,
    ProjectNormalizeFailed,
    ProjectWriterFailed,
    Unknown(String),
}

impl ErrorCode {
    pub fn as_str(&self) -> &str {
        match self {
            Self::StableIdBlank => "DECK.BLANK_STABLE_ID",
            Self::StableIdDuplicate => "AFID.STABLE_ID_DUPLICATE",
            Self::IdentityDuplicatePayload => "AFID.IDENTITY_DUPLICATE_PAYLOAD",
            Self::IdentityCollision => "AFID.IDENTITY_COLLISION",
            Self::IdentitySnapshotMissing => "AFID.IDENTITY_SNAPSHOT_MISSING",
            Self::IdentitySnapshotNoteIdMismatch => "AFID.IDENTITY_SNAPSHOT_NOTE_ID_MISMATCH",
            Self::IdentitySnapshotHashMismatch => "AFID.IDENTITY_SNAPSHOT_HASH_MISMATCH",
            Self::IdentitySnapshotIncomplete => "AFID.IDENTITY_SNAPSHOT_INCOMPLETE",
            Self::IdentityComponentEmpty => "AFID.IDENTITY_COMPONENT_EMPTY",
            Self::IdentityFieldsEmpty => "AFID.IDENTITY_FIELDS_EMPTY",
            Self::ReservedAfidNamespace => "DECK.RESERVED_AFID_NAMESPACE",
            Self::NoteLevelIdentityOverrideReasonRequired => {
                "AFID.NOTE_LEVEL_IDENTITY_OVERRIDE_REASON_REQUIRED"
            }
            Self::ClozeMalformed => "AFID.CLOZE_MALFORMED",
            Self::ClozeOrdInvalid => "AFID.CLOZE_ORD_INVALID",
            Self::ClozeNestedUnsupported => "AFID.CLOZE_NESTED_UNSUPPORTED",
            Self::ImageOcclusionImageDimensionsMissing => "AFID.IO_IMAGE_DIMENSIONS_MISSING",
            Self::ImageOcclusionRectEmpty => "AFID.IO_RECT_EMPTY",
            Self::ImageOcclusionRectOutOfBounds => "AFID.IO_RECT_OUT_OF_BOUNDS",
            Self::ImageOcclusionRectDuplicate => "AFID.IO_RECT_DUPLICATE",
            Self::ImageOcclusionUnknownMedia => "DECK.UNKNOWN_MEDIA_REF",
            Self::ImageOcclusionEmptyMasks => "DECK.EMPTY_IO_MASKS",
            Self::DeckMissingStableId => "DECK.MISSING_STABLE_ID",
            Self::DeckDuplicateStableId => "DECK.DUPLICATE_STABLE_ID",
            Self::DeckNoteLevelIdentityOverrideUsed => "DECK.NOTE_LEVEL_IDENTITY_OVERRIDE_USED",
            Self::MediaUnsafeFilename => "MEDIA.UNSAFE_FILENAME",
            Self::MediaDuplicateFilenameConflict => "MEDIA.DUPLICATE_FILENAME_CONFLICT",
            Self::MediaSourceMissing => "MEDIA.SOURCE_MISSING",
            Self::MediaSourceNotRegularFile => "MEDIA.SOURCE_NOT_REGULAR_FILE",
            Self::MediaSourceReadFailed => "MEDIA.SOURCE_READ_FAILED",
            Self::MediaSourceChanged => "MEDIA.SOURCE_CHANGED",
            Self::MediaEmptySource => "MEDIA.EMPTY_SOURCE",
            Self::MediaInvalidSourceLabel => "MEDIA.INVALID_SOURCE_LABEL",
            Self::MediaInlineTooLarge => "MEDIA.INLINE_TOO_LARGE",
            Self::MediaCasWriteFailed => "MEDIA.CAS_WRITE_FAILED",
            Self::MediaCasObjectMissing => "MEDIA.CAS_OBJECT_MISSING",
            Self::ProjectBuildMissingArtifact => "PROJECT.BUILD_MISSING_ARTIFACT",
            Self::ProjectBuildDiagnostics => "PROJECT.BUILD_DIAGNOSTICS",
            Self::ProjectBuildPolicyBlocked => "PROJECT.BUILD_POLICY_BLOCKED",
            Self::ProjectBuildInvalid => "PROJECT.BUILD_INVALID",
            Self::ProjectBuildIo => "PROJECT.BUILD_IO",
            Self::ProjectBuildInternal => "PROJECT.BUILD_INTERNAL",
            Self::ProjectBuildStatusFailed => "PROJECT.BUILD_STATUS_FAILED",
            Self::ProjectProductDocumentSourceMixed => "PROJECT.PRODUCT_DOCUMENT_SOURCE_MIXED",
            Self::ProjectProductMediaFailed => "PROJECT.PRODUCT_MEDIA_FAILED",
            Self::ProjectProductMediaStagingCollision => "PROJECT.PRODUCT_MEDIA_STAGING_COLLISION",
            Self::ProjectLowerFailed => "PROJECT.LOWER_FAILED",
            Self::ProjectNormalizeFailed => "PROJECT.NORMALIZE_FAILED",
            Self::ProjectWriterFailed => "PROJECT.WRITER_FAILED",
            Self::Unknown(code) => code.as_str(),
        }
    }

    pub fn from_code(code: impl Into<String>) -> Self {
        let code = code.into();
        match code.as_str() {
            "DECK.BLANK_STABLE_ID" | "AFID.STABLE_ID_BLANK" => Self::StableIdBlank,
            "AFID.STABLE_ID_DUPLICATE" | "DECK.STABLE_ID_DUPLICATE" => Self::StableIdDuplicate,
            "AFID.IDENTITY_DUPLICATE_PAYLOAD" | "DECK.IDENTITY_DUPLICATE_PAYLOAD" => {
                Self::IdentityDuplicatePayload
            }
            "AFID.IDENTITY_COLLISION" | "DECK.IDENTITY_COLLISION" => Self::IdentityCollision,
            "AFID.IDENTITY_SNAPSHOT_MISSING" => Self::IdentitySnapshotMissing,
            "AFID.IDENTITY_SNAPSHOT_NOTE_ID_MISMATCH" => Self::IdentitySnapshotNoteIdMismatch,
            "AFID.IDENTITY_SNAPSHOT_HASH_MISMATCH" => Self::IdentitySnapshotHashMismatch,
            "AFID.IDENTITY_SNAPSHOT_INCOMPLETE" => Self::IdentitySnapshotIncomplete,
            "AFID.IDENTITY_COMPONENT_EMPTY" => Self::IdentityComponentEmpty,
            "AFID.IDENTITY_FIELDS_EMPTY" => Self::IdentityFieldsEmpty,
            "DECK.RESERVED_AFID_NAMESPACE" => Self::ReservedAfidNamespace,
            "AFID.NOTE_LEVEL_IDENTITY_OVERRIDE_REASON_REQUIRED" => {
                Self::NoteLevelIdentityOverrideReasonRequired
            }
            "AFID.CLOZE_MALFORMED" => Self::ClozeMalformed,
            "AFID.CLOZE_ORD_INVALID" => Self::ClozeOrdInvalid,
            "AFID.CLOZE_NESTED_UNSUPPORTED" => Self::ClozeNestedUnsupported,
            "AFID.IO_IMAGE_DIMENSIONS_MISSING" => Self::ImageOcclusionImageDimensionsMissing,
            "AFID.IO_RECT_EMPTY" => Self::ImageOcclusionRectEmpty,
            "AFID.IO_RECT_OUT_OF_BOUNDS" => Self::ImageOcclusionRectOutOfBounds,
            "AFID.IO_RECT_DUPLICATE" => Self::ImageOcclusionRectDuplicate,
            "DECK.UNKNOWN_MEDIA_REF" => Self::ImageOcclusionUnknownMedia,
            "DECK.EMPTY_IO_MASKS" => Self::ImageOcclusionEmptyMasks,
            "DECK.MISSING_STABLE_ID" => Self::DeckMissingStableId,
            "DECK.DUPLICATE_STABLE_ID" => Self::DeckDuplicateStableId,
            "DECK.NOTE_LEVEL_IDENTITY_OVERRIDE_USED" => Self::DeckNoteLevelIdentityOverrideUsed,
            "MEDIA.UNSAFE_FILENAME"
            | "MEDIA.EXPORT_NAME_EMPTY"
            | "MEDIA.EXPORT_NAME_CONTAINS_SEPARATOR"
            | "MEDIA.EXPORT_NAME_NOT_BARE_FILENAME"
            | "MEDIA.EXPORT_NAME_UNSAFE_CHARACTER" => Self::MediaUnsafeFilename,
            "MEDIA.DUPLICATE_FILENAME_CONFLICT" => Self::MediaDuplicateFilenameConflict,
            "MEDIA.SOURCE_MISSING" => Self::MediaSourceMissing,
            "MEDIA.SOURCE_NOT_REGULAR_FILE" => Self::MediaSourceNotRegularFile,
            "MEDIA.SOURCE_READ_FAILED" => Self::MediaSourceReadFailed,
            "MEDIA.SOURCE_CHANGED" => Self::MediaSourceChanged,
            "MEDIA.EMPTY_SOURCE" => Self::MediaEmptySource,
            "MEDIA.INVALID_SOURCE_LABEL" => Self::MediaInvalidSourceLabel,
            "MEDIA.INLINE_TOO_LARGE" => Self::MediaInlineTooLarge,
            "MEDIA.CAS_WRITE_FAILED" => Self::MediaCasWriteFailed,
            "MEDIA.CAS_OBJECT_MISSING" => Self::MediaCasObjectMissing,
            "PROJECT.BUILD_MISSING_ARTIFACT" => Self::ProjectBuildMissingArtifact,
            "PROJECT.BUILD_DIAGNOSTICS" => Self::ProjectBuildDiagnostics,
            "PROJECT.BUILD_POLICY_BLOCKED" => Self::ProjectBuildPolicyBlocked,
            "PROJECT.BUILD_INVALID" => Self::ProjectBuildInvalid,
            "PROJECT.BUILD_IO" => Self::ProjectBuildIo,
            "PROJECT.BUILD_INTERNAL" => Self::ProjectBuildInternal,
            "PROJECT.BUILD_STATUS_FAILED" => Self::ProjectBuildStatusFailed,
            "PROJECT.PRODUCT_DOCUMENT_SOURCE_MIXED" => Self::ProjectProductDocumentSourceMixed,
            "PROJECT.PRODUCT_MEDIA_FAILED" => Self::ProjectProductMediaFailed,
            "PROJECT.PRODUCT_MEDIA_STAGING_COLLISION" => Self::ProjectProductMediaStagingCollision,
            "PROJECT.LOWER_FAILED" => Self::ProjectLowerFailed,
            "PROJECT.NORMALIZE_FAILED" => Self::ProjectNormalizeFailed,
            "PROJECT.WRITER_FAILED" => Self::ProjectWriterFailed,
            _ => Self::Unknown(code),
        }
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

pub trait ErrorCodeExt {
    fn code(&self) -> ErrorCode;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticCode(String);

impl DiagnosticCode {
    pub fn new(code: impl Into<String>) -> Self {
        Self(code.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn error_code(&self) -> ErrorCode {
        ErrorCode::from_code(self.0.clone())
    }
}

impl std::fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<ErrorCode> for DiagnosticCode {
    fn from(code: ErrorCode) -> Self {
        Self(code.as_str().to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourcePath(String);

impl SourcePath {
    pub fn new(path: impl Into<String>) -> Self {
        Self(path.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticDomain(String);

impl DiagnosticDomain {
    pub fn new(domain: impl Into<String>) -> Self {
        Self(domain.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for DiagnosticDomain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticStage(String);

impl DiagnosticStage {
    pub fn new(stage: impl Into<String>) -> Self {
        Self(stage.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for DiagnosticStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub severity: Severity,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<DiagnosticDomain>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage: Option<DiagnosticStage>,
    #[serde(
        rename = "path",
        alias = "source",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub source: Option<SourcePath>,
    pub message: String,
    #[serde(
        rename = "suggested_fix",
        alias = "help",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub help: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValidationReport {
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    report: ValidationReport,
    primary_code: DiagnosticCode,
}

impl ValidationError {
    pub fn report(&self) -> &ValidationReport {
        &self.report
    }

    pub fn primary_code(&self) -> &DiagnosticCode {
        &self.primary_code
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(diagnostic) = self
            .report
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.severity == Severity::Error)
        {
            write!(f, "{}: {}", diagnostic.code, diagnostic.message)
        } else {
            write!(f, "{}: validation failed", self.primary_code)
        }
    }
}

impl std::error::Error for ValidationError {}

impl ErrorCodeExt for ValidationError {
    fn code(&self) -> ErrorCode {
        self.primary_code.error_code()
    }
}

impl ValidationReport {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == Severity::Error)
    }

    pub fn ensure_success(&self) -> Result<(), ValidationError> {
        if let Some(diagnostic) = self
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.severity == Severity::Error)
        {
            Err(ValidationError {
                report: self.clone(),
                primary_code: diagnostic.code.clone(),
            })
        } else {
            Ok(())
        }
    }
}

impl ErrorCodeExt for anyhow::Error {
    fn code(&self) -> ErrorCode {
        if let Some(error) = self.downcast_ref::<crate::deck::DeckError>() {
            return error.code();
        }
        if let Some(error) = self.downcast_ref::<authoring_core::MediaFilenameError>() {
            return error.code();
        }
        if let Some(error) = self.downcast_ref::<crate::deck::MediaError>() {
            return error.code();
        }
        if let Some(error) = self.downcast_ref::<crate::build::BuildError>() {
            return error.code();
        }
        if let Some(error) = self.downcast_ref::<crate::product::ProductLoweringError>() {
            return error.code();
        }
        if let Some(error) =
            self.downcast_ref::<crate::product::project::ProductMediaPrepareError>()
        {
            return error.code();
        }
        if let Some(error) = self.downcast_ref::<crate::product::ProjectAddError>() {
            return error.code();
        }

        code_from_message(&self.to_string())
    }
}

impl ErrorCodeExt for authoring_core::MediaFilenameError {
    fn code(&self) -> ErrorCode {
        ErrorCode::MediaUnsafeFilename
    }
}

fn code_from_message(message: &str) -> ErrorCode {
    let candidate = message
        .split_once(':')
        .map(|(code, _)| code)
        .unwrap_or(message)
        .trim();
    if candidate.contains('.') && candidate.chars().all(is_code_char) {
        ErrorCode::from_code(candidate)
    } else {
        ErrorCode::Unknown(candidate.to_string())
    }
}

fn is_code_char(ch: char) -> bool {
    ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_' || ch == '.'
}
