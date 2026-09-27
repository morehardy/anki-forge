//! Validated note types, stable field symbols, and card templates.
//!
//! Declare fields and templates with stable keys; use `.name(...)` for labels.
//! [`NoteTypeBuilder::build`] validates the complete declaration. The resulting
//! [`NoteType`] is immutable and cheap to clone across functions and projects.

mod bundle;
mod bundle_error;
mod bundle_manifest;
mod declarations;
mod error;
mod keys;
pub(crate) mod stock;
mod validation;

pub use bundle_error::{TemplateBundleError, TemplateBundleErrorKind, TemplateBundleLimitExceeded};
pub use declarations::{Field, GenerationRule, Template};
pub use error::{SchemaError, SchemaErrorKind, TemplateLocation, TemplateSide};
pub use keys::{FieldKey, TemplateKey};

use std::sync::Arc;

/// An immutable, validated note type shared by the notes that use it.
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use = "use this model to create notes, or save or return it for later use"]
pub struct NoteType(Arc<NoteTypeBuilder>);

impl NoteType {
    /// Creates a note that owns a shared reference to this complete model.
    pub fn note(&self) -> crate::Note {
        crate::Note::new(self.clone())
    }

    /// Begins a model with an explicit stable key, independent of its name.
    pub fn builder(key: impl Into<String>) -> NoteTypeBuilder {
        let key = key.into();
        NoteTypeBuilder {
            name: key.clone(),
            key,
            fields: Vec::new(),
            templates: Vec::new(),
            css: String::new(),
            cloze_field: None,
            assets: Vec::new(),
            stock_kind: None,
        }
    }

    /// Returns the stable model key.
    pub fn key(&self) -> &str {
        &self.0.key
    }

    pub(crate) fn stock_kind(&self) -> Option<&'static str> {
        self.0.stock_kind
    }

    /// Returns the note-type name displayed by Anki.
    pub fn display_name(&self) -> &str {
        &self.0.name
    }

    /// Returns the validated field declarations in author order.
    pub fn fields(&self) -> &[Field] {
        &self.0.fields
    }

    /// Returns the validated templates, retaining the author's original source.
    pub fn templates(&self) -> &[Template] {
        &self.0.templates
    }

    /// Returns the model's stylesheet.
    pub fn css(&self) -> &str {
        &self.0.css
    }

    /// Returns the cloze field key, or None for a normal model.
    pub fn cloze_field(&self) -> Option<&FieldKey> {
        self.0.cloze_field.as_ref()
    }

    /// Returns all explicitly declared assets, including those referenced only
    /// from raw HTML, CSS or scripts. Identical name/content pairs appear once.
    pub fn assets(&self) -> &[crate::Media] {
        &self.0.assets
    }
}

/// An unfinished model declaration. No registration occurs when it is dropped.
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use = "call .build() to validate and finish this note type"]
pub struct NoteTypeBuilder {
    key: String,
    name: String,
    fields: Vec<Field>,
    templates: Vec<Template>,
    css: String,
    cloze_field: Option<FieldKey>,
    assets: Vec<crate::Media>,
    stock_kind: Option<&'static str>,
}

impl NoteTypeBuilder {
    /// Sets the displayed model name without changing its stable key.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Adds a field declaration, validated together with the complete model.
    pub fn field(mut self, field: Field) -> Self {
        self.fields.push(field);
        self
    }

    /// Adds a card template written with stable field keys.
    pub fn template(mut self, template: Template) -> Self {
        self.templates.push(template);
        self
    }

    /// Sets the model's stylesheet.
    pub fn css(mut self, css: impl Into<String>) -> Self {
        self.css = css.into();
        self
    }

    /// Declares a cloze model whose single template renders this field.
    pub fn cloze_field(mut self, key: impl Into<FieldKey>) -> Self {
        self.cloze_field = Some(key.into());
        self
    }

    /// Includes an asset used by this model's HTML, CSS or scripts. The asset is
    /// retained even when its reference cannot be recognized by static analysis.
    pub fn asset(mut self, media: crate::Media) -> Self {
        self.assets.push(media);
        self
    }

    /// Validates the complete model and returns an immutable shared value.
    pub fn build(mut self) -> Result<NoteType, SchemaError> {
        validation::validate(&self)?;
        let mut assets = crate::media::Assets::default();
        for asset in self.assets.drain(..) {
            assets.add(asset).map_err(|error| {
                SchemaError::new(SchemaErrorKind::AssetConflict, error.code, error.message)
            })?;
        }
        self.assets = assets.into_values();
        Ok(NoteType(Arc::new(self)))
    }
}
