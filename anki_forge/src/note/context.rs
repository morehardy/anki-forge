use crate::{
    media::{MediaConflictKind, MediaUsage},
    schema::FieldKey,
};
use std::ops::Range;

/// The authored location responsible for an unsuccessful addition.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AddTarget {
    /// The attempted note key.
    NoteKey,
    /// The note's overall structure.
    Note,
    /// The model carried by the note.
    Model,
    /// A field declaration or a node in its original content tree.
    #[non_exhaustive]
    Field {
        /// Stable field key, including unknown keys supplied by the caller.
        field_key: FieldKey,
        /// Zero-based sequence indices; None is the field, Some([]) its root.
        content_path: Option<Vec<usize>>,
        /// UTF-8 half-open range in the original Text/Html leaf, when applicable.
        byte_range: Option<Range<usize>>,
    },
    /// A tag at its original insertion position.
    #[non_exhaustive]
    Tag {
        /// Zero-based insertion index.
        index: usize,
        /// Original tag value.
        value: String,
    },
    /// The destination selected for a note.
    #[non_exhaustive]
    Deck {
        /// Original destination name.
        name: String,
        /// Whether this name came from the project's default deck.
        inherited: bool,
    },
    /// An asset carried by the note's validated model.
    #[non_exhaustive]
    ModelAsset {
        /// Export name in the completed model, after deduplication.
        media_name: String,
    },
    /// The structured image-occlusion note's main image.
    #[non_exhaustive]
    OcclusionImage {
        /// Captured export name.
        media_name: String,
    },
    /// An asset passed directly to Project::add_asset.
    #[non_exhaustive]
    ExplicitAsset {
        /// Attempted export name.
        media_name: String,
    },
    /// The project default deck checked before building, without a note context.
    #[non_exhaustive]
    ProjectDefaultDeck {
        /// Original default deck name.
        name: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Scope {
    Note { note_key: String, model_key: String },
    ExplicitAsset,
    ProjectDefaultDeck,
}

/// Owned identifiers and a location, without retaining any authoring resources.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddContext {
    scope: Scope,
    target: AddTarget,
}

impl AddContext {
    pub(crate) fn note(note_key: &str, model_key: &str, target: AddTarget) -> Self {
        Self {
            scope: Scope::Note {
                note_key: note_key.into(),
                model_key: model_key.into(),
            },
            target,
        }
    }

    pub(crate) fn asset(name: &str) -> Self {
        Self {
            scope: Scope::ExplicitAsset,
            target: AddTarget::ExplicitAsset {
                media_name: name.into(),
            },
        }
    }

    pub(crate) fn default_deck(name: &str) -> Self {
        Self {
            scope: Scope::ProjectDefaultDeck,
            target: AddTarget::ProjectDefaultDeck { name: name.into() },
        }
    }

    /// Original attempted note key, even when invalid; absent outside an addition.
    pub fn note_key(&self) -> Option<&str> {
        match &self.scope {
            Scope::Note { note_key, .. } => Some(note_key),
            _ => None,
        }
    }

    /// Key of the attempted note's model; present exactly when note_key is present.
    pub fn model_key(&self) -> Option<&str> {
        match &self.scope {
            Scope::Note { model_key, .. } => Some(model_key),
            _ => None,
        }
    }

    /// Returns the precise authored target of the error.
    pub fn target(&self) -> &AddTarget {
        &self.target
    }
}

/// Extra facts needed to distinguish or repair a conflict or media misuse.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AddDetail {
    /// The attempted model key already names a different definition.
    ModelDefinitionConflict,
    /// Another model key already uses this displayed name.
    #[non_exhaustive]
    ModelNameConflict {
        /// Stable key of the previously registered model.
        existing_model_key: String,
        /// Trimmed name used by the existing collision rule.
        conflicting_name: String,
    },
    /// Two media values cannot share the project's portable filename space.
    #[non_exhaustive]
    MediaConflict {
        /// Which existing collision rule failed.
        kind: MediaConflictKind,
        /// Original filename already registered or staged in this addition.
        existing_name: String,
        /// Original filename of the incoming value.
        incoming_name: String,
    },
    /// The retained media type does not support the requested content usage.
    #[non_exhaustive]
    MediaUsage {
        /// The content node the caller constructed.
        requested: MediaUsage,
        /// Filename captured by the content node.
        media_name: String,
        /// MIME retained at media import, including parameters if supplied.
        media_type: String,
    },
}
