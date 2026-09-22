use anki_forge::prelude::{MediaRef, Note, ProjectAddError};
use pyo3::prelude::*;
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NoteInput {
    note_type_id: String,
    stable_id: Option<String>,
    deck_name: Option<String>,
    fields: BTreeMap<String, ContentInput>,
    tags: Vec<String>,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum ContentInput {
    Text { value: String },
    Html { value: String },
    Sound { value: String },
    Image { value: String },
}

impl NoteInput {
    pub fn into_note(self, media: &BTreeMap<String, MediaRef>) -> PyResult<Note> {
        let mut note = Note::new(self.note_type_id);
        if let Some(id) = self.stable_id {
            note = note.stable_id(id);
        }
        if let Some(deck) = self.deck_name {
            note = note.deck(deck);
        }
        for tag in self.tags {
            note = note.tag(tag);
        }
        for (key, value) in self.fields {
            note = match value {
                ContentInput::Text { value } => note.text(key, value),
                ContentInput::Html { value } => note.html(key, value),
                ContentInput::Sound { value } => note.sound(key, reference(media, &value)?),
                ContentInput::Image { value } => note.image(key, reference(media, &value)?),
            };
        }
        Ok(note)
    }
}

fn reference(media: &BTreeMap<String, MediaRef>, filename: &str) -> PyResult<MediaRef> {
    media.get(filename).cloned().ok_or_else(|| {
        crate::domain_error(
            "note",
            "BINDING.MEDIA_REF_INVALID",
            "expected a registered MediaRef",
            serde_json::Value::Null,
        )
    })
}

pub fn add_error(error: ProjectAddError) -> PyErr {
    crate::domain_error(
        "add",
        error.code().as_str(),
        &error.to_string(),
        serde_json::json!({
            "diagnostic": anki_forge::build::json_report::DiagnosticJson::from(error.diagnostic())
        }),
    )
}
