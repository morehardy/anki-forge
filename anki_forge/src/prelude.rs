//! Common imports for the supported 0.1 consumer interface.
//!
//! This facade intentionally excludes normalization IR, contract loading,
//! writer internals, and artifact inspection. Those implementation interfaces
//! may change without notice before 1.0.

/// Build configuration and observable build results.
pub use crate::build::{BuildOptions, BuildReport, MediaSummary, UpdateSafetyMode};
/// High-level deck construction, image-occlusion modes, and deck media inputs.
pub use crate::deck::{Deck, IoMode, MediaSource};
/// Structured diagnostics returned by validation and build operations.
pub use crate::diagnostics::{
    Diagnostic, DiagnosticCode, DiagnosticDomain, DiagnosticStage, ErrorCode, ErrorCodeExt,
    Severity, SourcePath, ValidationError, ValidationReport,
};
/// Typed project, note type, template, note, and media interfaces.
pub use crate::product::{
    Content, Field, FieldKey, GenerationRule, IdentityRecipe, MediaRef, Note, NoteType,
    NoteTypeKind, ProductNoteError, Project, ProjectAddError, Template, TemplateKey,
};
