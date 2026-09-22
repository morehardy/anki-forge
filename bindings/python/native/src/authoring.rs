use anki_forge::prelude::{
    Content, Field, FieldKey, GenerationRule, IdentityRecipe, MediaRef, Note, NoteType,
    ProjectAddError, Template,
};
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
    identity: Option<Vec<String>>,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ContentInput {
    Text { value: String },
    Html { value: String },
    Sound { value: String },
    Image { value: String },
}

impl ContentInput {
    pub fn into_content(self, media: &BTreeMap<String, MediaRef>) -> PyResult<Content> {
        Ok(match self {
            Self::Text { value } => Content::text(value),
            Self::Html { value } => Content::html(value),
            Self::Image { value } => reference(media, &value)?.image(),
            Self::Sound { value } => reference(media, &value)?.sound(),
        })
    }
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
        if let Some(identity) = self.identity {
            note = note.identity(identity);
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

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IoModeInput {
    HideAllGuessOne,
    HideOneGuessOne,
}

impl From<IoModeInput> for anki_forge::prelude::IoMode {
    fn from(value: IoModeInput) -> Self {
        match value {
            IoModeInput::HideAllGuessOne => Self::HideAllGuessOne,
            IoModeInput::HideOneGuessOne => Self::HideOneGuessOne,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImageOcclusionInput {
    stable_id: Option<String>,
    deck_name: Option<String>,
    mode: IoModeInput,
    rects: Vec<(u32, u32, u32, u32)>,
    header: String,
    back_extra: String,
    comments: String,
    tags: Vec<String>,
}

impl ImageOcclusionInput {
    pub fn into_note(self, image: MediaRef) -> PyResult<Note> {
        let mut builder = Note::image_occlusion(image)
            .mode(self.mode.into())
            .header(self.header)
            .back_extra(self.back_extra)
            .comments(self.comments)
            .tags(self.tags);
        if let Some(id) = self.stable_id {
            builder = builder.stable_id(id);
        }
        if let Some(deck) = self.deck_name {
            builder = builder.deck(deck);
        }
        for (x, y, width, height) in self.rects {
            builder = builder.rect(x, y, width, height);
        }
        builder.build().map_err(|error| {
            crate::domain_error(
                "note",
                error.code().as_str(),
                &error.to_string(),
                serde_json::Value::Null,
            )
        })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FieldInput {
    name: String,
    key: Option<String>,
    identity: bool,
    sort: bool,
    required: bool,
    optional: bool,
    key_auto_derived: bool,
}

impl FieldInput {
    fn into_field(self) -> Field {
        let mut field = Field::new(self.name);
        if !self.key_auto_derived {
            if let Some(key) = self.key {
                field = field.key(key);
            }
        }
        if self.identity {
            field = field.identity();
        }
        if self.sort {
            field = field.sort();
        }
        if self.required {
            field = field.required();
        }
        if self.optional {
            field = field.optional();
        }
        field
    }
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum RuleInput {
    AnkiDefault,
    All { fields: Vec<String> },
    Any { fields: Vec<String> },
    Cloze { field: String },
}

impl From<RuleInput> for GenerationRule {
    fn from(value: RuleInput) -> Self {
        match value {
            RuleInput::AnkiDefault => Self::AnkiDefault,
            RuleInput::All { fields } => Self::all(fields),
            RuleInput::Any { fields } => Self::any(fields),
            RuleInput::Cloze { field } => Self::Cloze {
                field: FieldKey::new(field),
            },
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TemplateInput {
    name: String,
    key: Option<String>,
    front: String,
    back: String,
    browser_front: Option<String>,
    browser_back: Option<String>,
    target_deck: Option<String>,
    generate_when: RuleInput,
}

impl TemplateInput {
    fn into_template(self) -> Template {
        let mut template = Template::new(self.name)
            .front(self.front)
            .back(self.back)
            .generate_when(self.generate_when.into());
        if let Some(key) = self.key {
            template = template.key(key);
        }
        if let Some(value) = self.browser_front {
            template = template.browser_front(value);
        }
        if let Some(value) = self.browser_back {
            template = template.browser_back(value);
        }
        if let Some(value) = self.target_deck {
            template = template.target_deck(value);
        }
        template
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NoteTypeInput {
    id: String,
    name: Option<String>,
    cloze_field: Option<String>,
    fields: Vec<FieldInput>,
    templates: Vec<TemplateInput>,
    css: Option<String>,
    identity: Option<Vec<String>>,
}

impl NoteTypeInput {
    pub fn into_notetype(self) -> NoteType {
        let mut note_type = match self.cloze_field {
            Some(field) => NoteType::custom_cloze(self.id, field),
            None => NoteType::custom(self.id),
        };
        if let Some(name) = self.name {
            note_type = note_type.name(name);
        }
        if let Some(css) = self.css {
            note_type = note_type.css(css);
        }
        let fields: Vec<_> = self
            .fields
            .into_iter()
            .map(FieldInput::into_field)
            .collect();
        let identity: Vec<_> = fields
            .iter()
            .filter(|field| field.is_identity())
            .map(|field| field.key_ref().as_str().to_string())
            .collect();
        // The 0.1 Python flag implied a type recipe. Keep that authoring convention;
        // the core still resolves and hashes the recipe itself.
        if let Some(explicit) = self.identity {
            note_type = note_type.identity(IdentityRecipe::fields(explicit));
        } else if !identity.is_empty() {
            note_type = note_type.identity(IdentityRecipe::fields(identity));
        }
        for field in fields {
            note_type = note_type.field(field);
        }
        for template in self.templates {
            note_type = note_type.template(template.into_template());
        }
        note_type
    }
}
