//! Structured observations shared by public build and comparison operations.

use serde::Serialize;

/// Severity of an observation. It is not an operation's success status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// A condition preventing the requested operation.
    Error,
    /// A nonfatal concern that the caller should review.
    Warning,
    /// An informational observation.
    Info,
}

/// Location in an authored value or input file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SourceLocation {
    /// Logical authoring path or filesystem path.
    pub path: String,
    /// Exact UTF-8 byte range in the original source, when available.
    pub byte_range: Option<std::ops::Range<usize>>,
}

/// A structured observation with a registered, machine-readable code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Diagnostic {
    /// Stable machine identifier, independent of the message.
    pub code: String,
    /// Observation severity.
    pub severity: Severity,
    /// Human-readable explanation.
    pub message: String,
    /// Related authored value or input.
    pub source: Option<SourceLocation>,
    /// Suggested corrective action, if available.
    pub help: Option<String>,
}

impl Diagnostic {
    pub(crate) fn from_backend(value: crate::diagnostics_backend::Diagnostic) -> Self {
        Self {
            code: value.code.as_str().to_owned(),
            severity: match value.severity {
                crate::diagnostics_backend::Severity::Error => Severity::Error,
                crate::diagnostics_backend::Severity::Warning => Severity::Warning,
                crate::diagnostics_backend::Severity::Info => Severity::Info,
            },
            message: value.message,
            source: value.source.map(|source| SourceLocation {
                path: source.as_str().to_owned(),
                byte_range: None,
            }),
            help: value.help,
        }
    }
}
