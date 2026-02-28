use std::path::PathBuf;
use std::{error::Error, fmt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecError {
    EmptyDeckName {
        deck_index: usize,
    },
    EmptyModelName {
        model_index: usize,
    },
    ModelHasNoFields {
        model_index: usize,
    },
    EmptyModelFieldName {
        model_index: usize,
        field_index: usize,
    },
    ModelHasNoTemplates {
        model_index: usize,
    },
    EmptyTemplateName {
        model_index: usize,
        template_index: usize,
    },
    EmptyTemplateFront {
        model_index: usize,
        template_index: usize,
    },
    EmptyTemplateBack {
        model_index: usize,
        template_index: usize,
    },
    NotesRequireModel,
    NoteFieldCountMismatch {
        note_index: usize,
        expected: usize,
        actual: usize,
    },
    EmptyMediaLogicalName {
        media_index: usize,
    },
    EmptyMediaSourcePath {
        media_index: usize,
    },
}

#[derive(Debug)]
pub enum AnkiForgeError {
    Spec(SpecError),
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    CompatFeatureRequired,
}

pub type Result<T, E = AnkiForgeError> = std::result::Result<T, E>;

impl fmt::Display for AnkiForgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spec(error) => write!(f, "invalid package specification: {error}"),
            Self::Io { path, source } => write!(f, "io error at {}: {source}", path.display()),
            Self::CompatFeatureRequired => {
                write!(f, "compat feature is required for this operation")
            }
        }
    }
}

impl Error for AnkiForgeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Spec(error) => Some(error),
            Self::CompatFeatureRequired => None,
        }
    }
}

impl From<SpecError> for AnkiForgeError {
    fn from(error: SpecError) -> Self {
        Self::Spec(error)
    }
}

impl fmt::Display for SpecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyDeckName { deck_index } => {
                write!(f, "deck[{deck_index}] has an empty name")
            }
            Self::EmptyModelName { model_index } => {
                write!(f, "model[{model_index}] has an empty name")
            }
            Self::ModelHasNoFields { model_index } => {
                write!(f, "model[{model_index}] must define at least one field")
            }
            Self::EmptyModelFieldName {
                model_index,
                field_index,
            } => write!(
                f,
                "model[{model_index}] field[{field_index}] has an empty name"
            ),
            Self::ModelHasNoTemplates { model_index } => {
                write!(f, "model[{model_index}] must define at least one template")
            }
            Self::EmptyTemplateName {
                model_index,
                template_index,
            } => write!(
                f,
                "model[{model_index}] template[{template_index}] has an empty name"
            ),
            Self::EmptyTemplateFront {
                model_index,
                template_index,
            } => write!(
                f,
                "model[{model_index}] template[{template_index}] has an empty front"
            ),
            Self::EmptyTemplateBack {
                model_index,
                template_index,
            } => write!(
                f,
                "model[{model_index}] template[{template_index}] has an empty back"
            ),
            Self::NotesRequireModel => write!(f, "notes are present but no model is defined"),
            Self::NoteFieldCountMismatch {
                note_index,
                expected,
                actual,
            } => write!(
                f,
                "note[{note_index}] field count mismatch: expected {expected}, got {actual}"
            ),
            Self::EmptyMediaLogicalName { media_index } => {
                write!(f, "media[{media_index}] has an empty logical name")
            }
            Self::EmptyMediaSourcePath { media_index } => {
                write!(f, "media[{media_index}] has an empty source path")
            }
        }
    }
}

impl Error for SpecError {}
