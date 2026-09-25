use super::{FieldKey, TemplateKey};

/// A field declaration with a stable key and a separately editable display name.
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use = "pass this field to NoteTypeBuilder::field"]
pub struct Field {
    pub(crate) key: FieldKey,
    pub(crate) name: String,
    pub(crate) required: bool,
    pub(crate) sort: bool,
}

impl Field {
    /// Declares a field. Its display name initially equals its key.
    pub fn new(key: impl Into<FieldKey>) -> Self {
        let key = key.into();
        Self {
            name: key.to_string(),
            key,
            required: false,
            sort: false,
        }
    }

    /// Sets the Anki display name without changing the stable key.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Requires a nonempty value when a note is added to a project.
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Selects this field as the model's sort field. At most one may be selected.
    pub fn sort(mut self) -> Self {
        self.sort = true;
        self
    }

    /// Returns the stable symbol used in notes, templates and generation rules.
    pub fn key(&self) -> &FieldKey {
        &self.key
    }

    /// Returns the field name shown by Anki.
    pub fn display_name(&self) -> &str {
        &self.name
    }

    /// Whether adding a note requires this field to be nonempty.
    pub fn is_required(&self) -> bool {
        self.required
    }

    /// Whether this field was explicitly selected for sorting.
    pub fn is_sort(&self) -> bool {
        self.sort
    }
}

/// The field-presence condition for generating a normal card.
///
/// Cloze generation is selected on the model with
/// [`NoteTypeBuilder::cloze_field`](super::NoteTypeBuilder::cloze_field).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum GenerationRule {
    /// Infer the condition from the front template, as Anki does.
    #[default]
    AnkiDefault,
    /// Generate only if all listed fields are nonempty.
    All(Vec<FieldKey>),
    /// Generate if at least one listed field is nonempty.
    Any(Vec<FieldKey>),
}

impl GenerationRule {
    /// Requires all the supplied field keys.
    pub fn all<I, K>(fields: I) -> Self
    where
        I: IntoIterator<Item = K>,
        K: Into<FieldKey>,
    {
        Self::All(fields.into_iter().map(Into::into).collect())
    }

    /// Requires any of the supplied field keys.
    pub fn any<I, K>(fields: I) -> Self
    where
        I: IntoIterator<Item = K>,
        K: Into<FieldKey>,
    {
        Self::Any(fields.into_iter().map(Into::into).collect())
    }
}

/// A card template written using stable field keys, such as `{{front}}`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use = "pass this template to NoteTypeBuilder::template"]
pub struct Template {
    pub(crate) key: TemplateKey,
    pub(crate) name: String,
    pub(crate) front: String,
    pub(crate) back: String,
    pub(crate) browser_front: Option<String>,
    pub(crate) browser_back: Option<String>,
    pub(crate) target_deck: Option<String>,
    pub(crate) generation: GenerationRule,
}

impl Template {
    /// Declares a template. Its display name initially equals its key.
    pub fn new(key: impl Into<TemplateKey>) -> Self {
        let key = key.into();
        Self {
            name: key.to_string(),
            key,
            front: String::new(),
            back: String::new(),
            browser_front: None,
            browser_back: None,
            target_deck: None,
            generation: GenerationRule::default(),
        }
    }

    /// Sets the card's display name without changing its identity.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Sets the question template, referencing stable field keys.
    pub fn front(mut self, source: impl Into<String>) -> Self {
        self.front = source.into();
        self
    }

    /// Sets the answer template; `{{FrontSide}}` remains a system expression.
    pub fn back(mut self, source: impl Into<String>) -> Self {
        self.back = source.into();
        self
    }

    /// Sets the optional question template used by Anki's browser.
    pub fn browser_front(mut self, source: impl Into<String>) -> Self {
        self.browser_front = Some(source.into());
        self
    }

    /// Sets the optional answer template used by Anki's browser.
    pub fn browser_back(mut self, source: impl Into<String>) -> Self {
        self.browser_back = Some(source.into());
        self
    }

    /// Overrides the destination deck for cards created by this template.
    pub fn target_deck(mut self, deck: impl Into<String>) -> Self {
        self.target_deck = Some(deck.into());
        self
    }

    /// Sets an explicit field-presence condition for this template.
    pub fn generate_when(mut self, rule: GenerationRule) -> Self {
        self.generation = rule;
        self
    }

    /// Returns the stable template symbol.
    pub fn key(&self) -> &TemplateKey {
        &self.key
    }

    /// Returns the card name shown by Anki.
    pub fn display_name(&self) -> &str {
        &self.name
    }

    /// Returns the original question source using stable field keys.
    pub fn front_source(&self) -> &str {
        &self.front
    }

    /// Returns the original answer source using stable field keys.
    pub fn back_source(&self) -> &str {
        &self.back
    }

    /// Returns the optional original browser question source.
    pub fn browser_front_source(&self) -> Option<&str> {
        self.browser_front.as_deref()
    }

    /// Returns the optional original browser answer source.
    pub fn browser_back_source(&self) -> Option<&str> {
        self.browser_back.as_deref()
    }

    /// Returns the explicit generation rule.
    pub fn generation_rule(&self) -> &GenerationRule {
        &self.generation
    }

    /// Returns the optional card destination override.
    pub fn target_deck_name(&self) -> Option<&str> {
        self.target_deck.as_deref()
    }
}
