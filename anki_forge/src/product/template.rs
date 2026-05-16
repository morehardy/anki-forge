use super::FieldKey;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TemplateKey(String);

impl TemplateKey {
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateSource(String);

impl TemplateSource {
    pub fn new(source: impl Into<String>) -> Self {
        Self(source.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenerationRule {
    AnkiDefault,
    All(Vec<FieldKey>),
    Any(Vec<FieldKey>),
    Cloze { field: FieldKey },
}

impl GenerationRule {
    pub fn all<I, S>(fields: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::All(
            fields
                .into_iter()
                .map(|field| FieldKey::new(field.into()))
                .collect(),
        )
    }

    pub fn any<I, S>(fields: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::Any(
            fields
                .into_iter()
                .map(|field| FieldKey::new(field.into()))
                .collect(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Template {
    key: TemplateKey,
    name: String,
    front: TemplateSource,
    back: TemplateSource,
    browser_front: Option<TemplateSource>,
    browser_back: Option<TemplateSource>,
    target_deck: Option<String>,
    generation_rule: GenerationRule,
}

impl Template {
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            key: TemplateKey::new(name.to_ascii_lowercase().replace(' ', "_")),
            name,
            front: TemplateSource::new(""),
            back: TemplateSource::new(""),
            browser_front: None,
            browser_back: None,
            target_deck: None,
            generation_rule: GenerationRule::AnkiDefault,
        }
    }

    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = TemplateKey::new(key);
        self
    }

    pub fn front(mut self, front: impl Into<String>) -> Self {
        self.front = TemplateSource::new(front);
        self
    }

    pub fn back(mut self, back: impl Into<String>) -> Self {
        self.back = TemplateSource::new(back);
        self
    }

    pub fn browser_front(mut self, source: impl Into<String>) -> Self {
        self.browser_front = Some(TemplateSource::new(source));
        self
    }

    pub fn browser_back(mut self, source: impl Into<String>) -> Self {
        self.browser_back = Some(TemplateSource::new(source));
        self
    }

    pub fn target_deck(mut self, deck_name: impl Into<String>) -> Self {
        self.target_deck = Some(deck_name.into());
        self
    }

    pub fn generate_when(mut self, rule: GenerationRule) -> Self {
        self.generation_rule = rule;
        self
    }

    pub fn key_ref(&self) -> &TemplateKey {
        &self.key
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn front_source(&self) -> &TemplateSource {
        &self.front
    }

    pub fn back_source(&self) -> &TemplateSource {
        &self.back
    }

    pub fn browser_front_source(&self) -> Option<&TemplateSource> {
        self.browser_front.as_ref()
    }

    pub fn browser_back_source(&self) -> Option<&TemplateSource> {
        self.browser_back.as_ref()
    }

    pub fn target_deck_name(&self) -> Option<&str> {
        self.target_deck.as_deref()
    }

    pub fn generation_rule(&self) -> &GenerationRule {
        &self.generation_rule
    }
}
