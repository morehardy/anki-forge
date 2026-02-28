#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Note {
    pub fields: Vec<String>,
    pub tags: Vec<String>,
    pub guid_override: Option<String>,
}

impl Note {
    #[must_use]
    pub fn new<I, S>(fields: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            fields: fields.into_iter().map(Into::into).collect(),
            tags: Vec::new(),
            guid_override: None,
        }
    }

    #[must_use]
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    #[must_use]
    pub fn with_guid_override(mut self, guid: impl Into<String>) -> Self {
        self.guid_override = Some(guid.into());
        self
    }
}
