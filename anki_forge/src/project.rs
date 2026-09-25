use std::collections::{BTreeMap, BTreeSet};

use crate::{
    media::Assets,
    note::{AddError, AddErrorKind as Kind},
    schema::{SchemaError, SchemaErrorKind},
    Media, Note, NoteType,
};

/// One authored publication with a stable namespace, notes and owned assets.
///
/// Adding a note atomically collects its model and all media dependencies. A
/// project can contain several decks; the namespace is independent of names.
#[derive(Debug, Clone)]
pub struct Project {
    pub(crate) namespace: String,
    pub(crate) name: String,
    pub(crate) default_deck: String,
    pub(crate) models: BTreeMap<String, NoteType>,
    pub(crate) notes: BTreeMap<String, Note>,
    pub(crate) assets: Assets,
}

impl Project {
    /// Creates a project with a nonempty stable namespace. The namespace cannot
    /// have surrounding whitespace, path separators or control characters.
    pub fn new(namespace: impl Into<String>) -> Result<Self, SchemaError> {
        let namespace = namespace.into();
        if namespace.is_empty()
            || namespace.trim() != namespace
            || namespace
                .chars()
                .any(|c| c.is_control() || "/\\".contains(c))
            || matches!(namespace.as_str(), "." | "..")
        {
            return Err(SchemaError::new(SchemaErrorKind::InvalidKey, "SCHEMA.NAMESPACE_INVALID", "choose a nonempty stable namespace without surrounding whitespace, control characters or path separators"));
        }
        Ok(Self {
            name: namespace.clone(),
            namespace,
            default_deck: "Default".into(),
            models: BTreeMap::new(),
            notes: BTreeMap::new(),
            assets: Assets::default(),
        })
    }

    /// Sets the human-readable project title without changing its namespace.
    #[must_use = "use the returned project with its updated display name"]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Sets the destination of notes without an explicit deck. Deck names are
    /// validated when adding notes and again before building.
    #[must_use = "use the returned project with its updated default deck"]
    pub fn default_deck(mut self, name: impl Into<String>) -> Self {
        self.default_deck = name.into();
        self
    }

    /// Adds a note under an explicit stable key. Any error leaves all project
    /// state unchanged. Reusing a model key requires an identical definition.
    pub fn add(&mut self, key: impl Into<String>, note: Note) -> Result<(), AddError> {
        let key = key.into();
        if key.trim().is_empty() || key.trim() != key || key.chars().any(char::is_control) {
            return Err(AddError::new(
                Kind::InvalidKey,
                "NOTE.KEY_INVALID",
                "note key must be nonempty without surrounding whitespace or control characters",
            ));
        }
        if self.notes.contains_key(&key) {
            return Err(AddError::new(
                Kind::DuplicateKey,
                "NOTE.KEY_DUPLICATE",
                format!("note key {key:?} already exists"),
            ));
        }
        if note.model.stock_kind() == Some("image_occlusion") && note.occlusion.is_none() {
            return Err(AddError::new(
                Kind::InvalidOcclusion,
                "NOTE.IO_STRUCTURE_REQUIRED",
                "create image-occlusion notes with Note::image_occlusion and stable mask keys",
            ));
        }
        if note.occlusion.is_some()
            && note
                .fields
                .keys()
                .any(|field| matches!(field.as_str(), "occlusion" | "image"))
        {
            return Err(AddError::new(
                Kind::InvalidOcclusion,
                "NOTE.IO_FIELD_RESERVED",
                "image and occlusion fields are generated from the structured image and masks",
            ));
        }
        if self
            .models
            .get(note.model.key())
            .is_some_and(|model| model != &note.model)
        {
            return Err(AddError::new(
                Kind::ModelConflict,
                "NOTE.MODEL_CONFLICT",
                format!(
                    "model key {:?} already names a different definition",
                    note.model.key()
                ),
            ));
        }
        for field in note.fields.keys() {
            if !note
                .model
                .fields()
                .iter()
                .any(|declared| declared.key() == field)
            {
                return Err(AddError::new(
                    Kind::UnknownField,
                    "NOTE.FIELD_UNKNOWN",
                    format!(
                        "field key {field:?} is absent from model {:?}",
                        note.model.key()
                    ),
                ));
            }
        }
        for field in note
            .model
            .fields()
            .iter()
            .filter(|field| field.is_required())
        {
            if !note
                .fields
                .get(field.key())
                .is_some_and(crate::Content::has_value)
            {
                return Err(AddError::new(
                    Kind::RequiredField,
                    "NOTE.FIELD_REQUIRED",
                    format!("required field {:?} is empty", field.key()),
                ));
            }
        }
        validate_deck(note.deck.as_deref().unwrap_or(&self.default_deck))?;
        let mut tags = BTreeSet::new();
        for tag in &note.tags {
            if tag.is_empty()
                || tag.chars().any(|c| c.is_control() || c.is_whitespace())
                || tag.starts_with("afid::")
                || !tags.insert(tag)
            {
                return Err(AddError::new(
                    Kind::InvalidTag,
                    "NOTE.TAG_INVALID",
                    format!("invalid, reserved or duplicate tag {tag:?}"),
                ));
            }
        }
        let mut dependencies = note.model.assets().to_vec();
        if let Some(occlusion) = &note.occlusion {
            dependencies.push(occlusion.image.clone());
        }
        for content in note.fields.values() {
            content.visit_media(&mut |media| dependencies.push(media.clone()));
        }
        let mut additions = Assets::default();
        for media in dependencies {
            if self.assets.check(&media).map_err(media_conflict)? {
                additions.add(media).map_err(media_conflict)?;
            }
        }
        // All fallible work is complete. No reader can observe a partial add.
        self.assets.merge(additions);
        self.models
            .entry(note.model.key().to_owned())
            .or_insert_with(|| note.model.clone());
        self.notes.insert(key, note);
        Ok(())
    }

    /// Includes an asset used by raw HTML, CSS or scripts. Identical name/content
    /// pairs are idempotent; conflicts fail without changing project state.
    pub fn add_asset(&mut self, media: Media) -> Result<(), AddError> {
        self.assets.add(media).map_err(media_conflict)
    }

    /// Returns the number of notes currently registered in this project.
    pub fn len(&self) -> usize {
        self.notes.len()
    }

    /// Whether this project contains no notes.
    pub fn is_empty(&self) -> bool {
        self.notes.is_empty()
    }

    /// Returns the stable namespace, independent of all display names.
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Returns the human-readable project title.
    pub fn display_name(&self) -> &str {
        &self.name
    }
}

pub(crate) fn validate_deck(name: &str) -> Result<(), AddError> {
    if !crate::writer_core::deck_name::valid_authored_deck_name(name) {
        Err(AddError::new(Kind::InvalidDeck, "NOTE.DECK_INVALID", "deck names need nonempty components without surrounding whitespace, leading/trailing colons or control characters"))
    } else {
        Ok(())
    }
}

fn media_conflict(error: crate::media::assets::AssetConflict) -> AddError {
    AddError::new(Kind::MediaConflict, error.code, error.message)
}
