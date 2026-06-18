pub use crate::build::{BuildOptions, BuildReport, MediaSummary, UpdateSafetyMode};
pub use crate::deck::{Deck, IoMode};
pub use crate::diagnostics::{
    Diagnostic, DiagnosticCode, DiagnosticDomain, DiagnosticStage, ErrorCode, ErrorCodeExt,
    Severity, SourcePath, ValidationError, ValidationReport,
};
pub use crate::product::{
    Content, Field, FieldKey, GenerationRule, IdentityRecipe, MediaRef, Note, NoteType,
    ProductNoteError, Project, ProjectAddError, Template, TemplateKey,
};
