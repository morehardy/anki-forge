use std::collections::{BTreeMap, BTreeSet};

use crate::{
    media::Assets,
    note::{AddContext, AddDetail, AddError, AddErrorKind as Kind, AddTarget},
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
            namespace,
            default_deck: "Default".into(),
            models: BTreeMap::new(),
            notes: BTreeMap::new(),
            assets: Assets::default(),
        })
    }

    /// Sets the destination of notes without an explicit deck. Deck names are
    /// validated when adding notes and again before building.
    #[must_use = "use the returned project with its updated default deck"]
    pub fn default_deck(mut self, name: impl Into<String>) -> Self {
        self.default_deck = name.into();
        self
    }

    /// Adds a note under an explicit stable key. Any error leaves all project
    /// state unchanged. Reusing a model key requires an identical definition;
    /// distinct models must have distinct display names after trimming whitespace.
    /// Field content must not contain the raw U+001F Anki field separator.
    /// Typed image content requires image/* MIME; sound accepts audio/* or video/*.
    /// Errors identify the original note, field/content path and conflict facts.
    pub fn add(&mut self, key: impl Into<String>, note: Note) -> Result<(), AddError> {
        let key = key.into();
        let context = |target| AddContext::note(&key, note.model.key(), target);
        let error = |kind, code, message: String, target| {
            AddError::new(kind, code, message, context(target))
        };
        if key.trim().is_empty() || key.trim() != key || key.chars().any(char::is_control) {
            return Err(error(
                Kind::InvalidKey,
                "NOTE.KEY_INVALID",
                "note key must be nonempty without surrounding whitespace or control characters"
                    .into(),
                AddTarget::NoteKey,
            ));
        }
        if self.notes.contains_key(&key) {
            return Err(error(
                Kind::DuplicateKey,
                "NOTE.KEY_DUPLICATE",
                format!("note key {key:?} already exists"),
                AddTarget::NoteKey,
            ));
        }
        if note.model.stock_kind() == Some("image_occlusion") && note.occlusion.is_none() {
            return Err(error(
                Kind::InvalidOcclusion,
                "NOTE.IO_STRUCTURE_REQUIRED",
                "create image-occlusion notes with Note::image_occlusion and stable mask keys"
                    .into(),
                AddTarget::Note,
            ));
        }
        if note.occlusion.is_some() {
            if let Some(field) = note
                .fields
                .keys()
                .find(|field| matches!(field.as_str(), "occlusion" | "image"))
            {
                return Err(error(
                    Kind::InvalidOcclusion,
                    "NOTE.IO_FIELD_RESERVED",
                    "image and occlusion fields are generated from the structured image and masks"
                        .into(),
                    field_target(field),
                ));
            }
        }
        if self
            .models
            .get(note.model.key())
            .is_some_and(|model| model != &note.model)
        {
            return Err(error(
                Kind::ModelConflict,
                "NOTE.MODEL_CONFLICT",
                format!(
                    "model key {:?} already names a different definition",
                    note.model.key()
                ),
                AddTarget::Model,
            )
            .with_detail(AddDetail::ModelDefinitionConflict));
        }
        // Lowering trims model names; Anki compares them using BINARY collation.
        let name = note.model.display_name().trim();
        if let Some(existing) = self
            .models
            .values()
            .find(|model| model.key() != note.model.key() && model.display_name().trim() == name)
        {
            return Err(error(
                Kind::ModelConflict,
                "NOTE.MODEL_CONFLICT",
                format!(
                    "model name {name:?} is already used by model key {:?}",
                    existing.key()
                ),
                AddTarget::Model,
            )
            .with_detail(AddDetail::ModelNameConflict {
                existing_model_key: existing.key().into(),
                conflicting_name: name.into(),
            }));
        }
        for (field, content) in &note.fields {
            if !note
                .model
                .fields()
                .iter()
                .any(|declared| declared.key() == field)
            {
                return Err(error(
                    Kind::UnknownField,
                    "NOTE.FIELD_UNKNOWN",
                    format!(
                        "field key {field:?} is absent from model {:?}",
                        note.model.key()
                    ),
                    field_target(field),
                ));
            }
            content.validate().map_err(|issue| {
                let mut failure = error(
                    issue.kind,
                    issue.code,
                    issue.message,
                    AddTarget::Field {
                        field_key: field.clone(),
                        content_path: Some(issue.path),
                        byte_range: issue.byte_range,
                    },
                );
                if let Some(detail) = issue.detail {
                    failure = failure.with_detail(*detail);
                }
                failure
            })?;
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
                return Err(error(
                    Kind::RequiredField,
                    "NOTE.FIELD_REQUIRED",
                    format!("required field {:?} is empty", field.key()),
                    field_target(field.key()),
                ));
            }
        }
        let deck = note.deck.as_deref().unwrap_or(&self.default_deck);
        validate_deck(deck).map_err(|error| {
            error.with_context(context(AddTarget::Deck {
                name: deck.into(),
                inherited: note.deck.is_none(),
            }))
        })?;
        let mut tags = BTreeSet::new();
        for (index, tag) in note.tags.iter().enumerate() {
            if tag.is_empty()
                || tag.chars().any(|c| c.is_control() || c.is_whitespace())
                || tag.starts_with("afid::")
                || !tags.insert(tag)
            {
                return Err(error(
                    Kind::InvalidTag,
                    "NOTE.TAG_INVALID",
                    format!("invalid, reserved or duplicate tag {tag:?}"),
                    AddTarget::Tag {
                        index,
                        value: tag.clone(),
                    },
                ));
            }
        }
        let mut additions = Assets::default();
        let mut collect = |media: &Media, target: &dyn Fn() -> AddTarget| -> Result<(), AddError> {
            if self
                .assets
                .check(media)
                .map_err(|e| media_conflict(e, context(target())))?
            {
                additions
                    .add(media.clone())
                    .map_err(|e| media_conflict(e, context(target())))?;
            }
            Ok(())
        };
        for media in note.model.assets() {
            collect(media, &|| AddTarget::ModelAsset {
                media_name: media.filename().into(),
            })?;
        }
        if let Some(occlusion) = &note.occlusion {
            collect(&occlusion.image, &|| AddTarget::OcclusionImage {
                media_name: occlusion.image.filename().into(),
            })?;
        }
        for (field, content) in &note.fields {
            content.try_visit_media(&mut |media, path| {
                collect(media, &|| AddTarget::Field {
                    field_key: field.clone(),
                    content_path: Some(path.to_vec()),
                    byte_range: None,
                })
            })?;
        }
        // No fallible work remains: commit all dependencies with the note.
        self.assets.merge(additions);
        self.models
            .entry(note.model.key().into())
            .or_insert_with(|| note.model.clone());
        self.notes.insert(key, note);
        Ok(())
    }

    /// Includes an asset used by raw HTML, CSS or scripts. Identical name/content
    /// pairs are idempotent; conflicts fail without changing project state.
    pub fn add_asset(&mut self, media: Media) -> Result<(), AddError> {
        self.assets.add(media).map_err(|error| {
            let context = AddContext::asset(&error.incoming_name);
            media_conflict(error, context)
        })
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
}

pub(crate) fn validate_deck(name: &str) -> Result<(), AddError> {
    if !crate::writer_core::deck_name::valid_authored_deck_name(name) {
        Err(AddError::new(Kind::InvalidDeck, "NOTE.DECK_INVALID", "deck names need nonempty components without surrounding whitespace, leading/trailing colons or control characters", AddContext::default_deck(name)))
    } else {
        Ok(())
    }
}

fn field_target(field: &crate::schema::FieldKey) -> AddTarget {
    AddTarget::Field {
        field_key: field.clone(),
        content_path: None,
        byte_range: None,
    }
}

fn media_conflict(error: crate::media::assets::AssetConflict, context: AddContext) -> AddError {
    AddError::new(
        Kind::MediaConflict,
        error.code(),
        error.to_string(),
        context,
    )
    .with_detail(AddDetail::MediaConflict {
        kind: error.kind,
        existing_name: error.existing_name,
        incoming_name: error.incoming_name,
    })
}
