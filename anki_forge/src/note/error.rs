use std::fmt;

/// The reason a note or explicit asset could not be added to a project.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddErrorKind {
    /// The stable note key is blank or contains invalid control characters.
    InvalidKey,
    /// A note with this exact stable key already exists in the project.
    DuplicateKey,
    /// The note assigns a field absent from its model.
    UnknownField,
    /// A required field has no nonempty content.
    RequiredField,
    /// The model's stable key is already bound to a different definition.
    ModelConflict,
    /// Media names or their bound contents conflict.
    MediaConflict,
    /// A tag is blank, duplicated, reserved or contains whitespace.
    InvalidTag,
    /// The destination deck name contains a blank component or control character.
    InvalidDeck,
    /// A structured image-occlusion note is missing its masks or overrides a generated field.
    InvalidOcclusion,
}

/// An addition failed without changing the project's notes, models or assets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddError {
    kind: AddErrorKind,
    code: &'static str,
    message: String,
}

impl AddError {
    pub(crate) fn new(kind: AddErrorKind, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            kind,
            code,
            message: message.into(),
        }
    }

    /// Returns the structured category independent of human wording.
    pub fn kind(&self) -> AddErrorKind {
        self.kind
    }

    /// Returns the registered machine code assigned when the operation failed.
    pub fn code(&self) -> &str {
        self.code
    }
}

impl fmt::Display for AddError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for AddError {}
