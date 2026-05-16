use std::collections::BTreeMap;

use super::{Content, MediaRef};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    note_type_id: String,
    stable_id: Option<String>,
    deck_name: Option<String>,
    fields: BTreeMap<String, Content>,
    tags: Vec<String>,
}

impl Note {
    pub fn new(note_type_id: impl Into<String>) -> Self {
        Self {
            note_type_id: note_type_id.into(),
            stable_id: None,
            deck_name: None,
            fields: BTreeMap::new(),
            tags: Vec::new(),
        }
    }

    pub fn basic(front: impl Into<String>, back: impl Into<String>) -> Self {
        Self::new("basic").text("Front", front).text("Back", back)
    }

    pub fn cloze(text: impl Into<String>) -> Self {
        Self::new("cloze").html("Text", text).text("Back Extra", "")
    }

    pub fn stable_id(mut self, stable_id: impl Into<String>) -> Self {
        self.stable_id = Some(stable_id.into());
        self
    }

    pub fn deck(mut self, deck_name: impl Into<String>) -> Self {
        self.deck_name = Some(deck_name.into());
        self
    }

    pub fn text(mut self, field: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.insert(field.into(), Content::text(value));
        self
    }

    pub fn html(mut self, field: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.insert(field.into(), Content::html(value));
        self
    }

    pub fn sound(mut self, field: impl Into<String>, media: MediaRef) -> Self {
        self.fields.insert(field.into(), media.sound());
        self
    }

    pub fn image(mut self, field: impl Into<String>, media: MediaRef) -> Self {
        self.fields.insert(field.into(), media.image());
        self
    }

    pub fn extra(mut self, extra: impl Into<String>) -> Self {
        self.fields
            .insert("Back Extra".into(), Content::text(extra));
        self
    }

    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    pub fn note_type_id(&self) -> &str {
        &self.note_type_id
    }

    pub fn stable_id_ref(&self) -> Option<&str> {
        self.stable_id.as_deref()
    }

    pub fn deck_name(&self) -> Option<&str> {
        self.deck_name.as_deref()
    }

    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    pub fn rendered_fields(&self) -> BTreeMap<String, String> {
        self.fields
            .iter()
            .map(|(field, content)| (field.clone(), content.render()))
            .collect()
    }
}
