use super::TemplateKey;
use std::{fmt, ops::Range};

/// The category of a failed model declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaErrorKind {
    /// A stable key is blank, reserved or cannot occur in a template expression.
    InvalidKey,
    /// A display name is blank, reserved or cannot be encoded as an Anki field.
    InvalidName,
    /// Two declarations have conflicting symbols or display names.
    Duplicate,
    /// A required declaration is absent or an incompatible option was supplied.
    InvalidStructure,
    /// A template expression is malformed or references an undeclared key.
    InvalidTemplate,
    /// Cloze declarations and their templates do not agree.
    InvalidCloze,
    /// A generation rule refers to an unknown field or is otherwise invalid.
    InvalidGeneration,
    /// Model assets contain conflicting export names or content.
    AssetConflict,
}

/// Identifies one of a template's four source strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateSide {
    /// Question template.
    Front,
    /// Answer template.
    Back,
    /// Optional question template in the browser.
    BrowserFront,
    /// Optional answer template in the browser.
    BrowserBack,
}

/// The exact byte range in the author's original template source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateLocation {
    /// Stable key of the offending template.
    pub template: TemplateKey,
    /// Source string containing the issue.
    pub side: TemplateSide,
    /// UTF-8 byte range before field-name compilation.
    pub byte_range: Range<usize>,
}

/// A model could not be completed. No partially validated model is returned.
///
/// Machine decisions should use [`Self::kind`] and [`Self::code`], not Display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaError {
    kind: SchemaErrorKind,
    code: &'static str,
    message: String,
    location: Option<TemplateLocation>,
}

impl SchemaError {
    pub(crate) fn new(
        kind: SchemaErrorKind,
        code: &'static str,
        message: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            code,
            message: message.into(),
            location: None,
        }
    }

    pub(crate) fn at(
        mut self,
        template: TemplateKey,
        side: TemplateSide,
        byte_range: Range<usize>,
    ) -> Self {
        self.location = Some(TemplateLocation {
            template,
            side,
            byte_range,
        });
        self
    }

    /// Returns the structural error category, independent of human wording.
    pub fn kind(&self) -> SchemaErrorKind {
        self.kind
    }

    /// Returns the machine code assigned when validation failed.
    pub fn code(&self) -> &str {
        self.code
    }

    /// Returns the offending original source range for a template error.
    pub fn location(&self) -> Option<&TemplateLocation> {
        self.location.as_ref()
    }
}

impl fmt::Display for SchemaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for SchemaError {}
