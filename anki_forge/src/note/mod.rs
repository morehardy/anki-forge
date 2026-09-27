//! Notes retain a validated model and typed content until added to a project.

mod content;
mod error;
mod occlusion;
mod occlusion_error;
pub use content::Content;
pub use error::{AddError, AddErrorKind};
pub use occlusion::{ImageOcclusionBuilder, Mask, OcclusionMode};
pub use occlusion_error::{ImageOcclusionError, ImageOcclusionErrorKind};

use crate::{schema::FieldKey, NoteType};
use std::collections::BTreeMap;

/// A note value owning its model and all typed field dependencies.
///
/// Use [`NoteType::note`] for custom models. The stable note key is supplied
/// when adding the value to a project, independently of its body or display name.
#[derive(Debug, Clone, PartialEq)]
#[must_use = "call Project::add(key, note) to add this note to a project"]
pub struct Note {
    pub(crate) model: NoteType,
    pub(crate) fields: BTreeMap<FieldKey, Content>,
    pub(crate) deck: Option<String>,
    pub(crate) tags: Vec<String>,
    pub(crate) occlusion: Option<occlusion::Occlusion>,
}

impl Note {
    pub(crate) fn new(model: NoteType) -> Self {
        Self {
            model,
            fields: BTreeMap::new(),
            deck: None,
            tags: Vec::new(),
            occlusion: None,
        }
    }

    /// Creates a Basic note. Ordinary strings are text, on both sides.
    pub fn basic(front: impl Into<Content>, back: impl Into<Content>) -> Self {
        crate::schema::stock::basic()
            .note()
            .field("front", front)
            .field("back", back)
    }

    /// Creates a Cloze note. Cloze syntax is preserved while literal HTML in
    /// ordinary strings is escaped. Use `.field("back_extra", ...)` for extras.
    pub fn cloze(text: impl Into<Content>) -> Self {
        crate::schema::stock::cloze()
            .note()
            .field("text", text)
            .field("back_extra", "")
    }

    /// Assigns content by stable field key. References are checked when adding
    /// the note to a project; assigning a key again replaces that field's value.
    pub fn field(mut self, key: impl Into<FieldKey>, value: impl Into<Content>) -> Self {
        self.fields.insert(key.into(), value.into());
        self
    }

    /// Overrides the project's default deck for this note.
    pub fn deck(mut self, name: impl Into<String>) -> Self {
        self.deck = Some(name.into());
        self
    }

    /// Adds a tag. Tag syntax and duplicates are checked when adding the note.
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Adds tags from an iterator.
    pub fn tags<I, T>(mut self, tags: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        self.tags.extend(tags.into_iter().map(Into::into));
        self
    }

    /// Returns the validated model owned by this note.
    pub fn note_type(&self) -> &NoteType {
        &self.model
    }
}
