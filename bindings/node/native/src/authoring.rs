use anki_forge::prelude::{
    Field, FieldKey, GenerationRule, IdentityRecipe, Note, NoteType, Template,
};
use anki_forge::product::MediaRef;
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct NoteOptions {
    stable_id: Option<String>,
    deck_name: Option<String>,
    tags: Vec<String>,
    back_extra: Option<String>,
    identity: Option<Vec<String>>,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum NoteKind {
    Basic {
        front: String,
        back: String,
    },
    Cloze {
        text: String,
    },
    Custom {
        id: String,
    },
    ImageOcclusion {
        image: String,
        mode: IoModeInput,
        rects: Vec<RectInput>,
        header: Option<String>,
        comments: Option<String>,
    },
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum IoModeInput {
    HideAllGuessOne,
    HideOneGuessOne,
}
impl From<IoModeInput> for anki_forge::deck::IoMode {
    fn from(mode: IoModeInput) -> Self {
        match mode {
            IoModeInput::HideAllGuessOne => Self::HideAllGuessOne,
            IoModeInput::HideOneGuessOne => Self::HideOneGuessOne,
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RectInput {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentInput {
    pub kind: ContentKind,
    pub value: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentKind {
    Text,
    Html,
    Image,
    Sound,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NoteInput {
    pub source: NoteKind,
    pub options: NoteOptions,
    pub fields: BTreeMap<String, ContentInput>,
}

impl NoteInput {
    pub fn into_note(self, media: &BTreeMap<String, MediaRef>) -> anyhow::Result<Note> {
        let get_media = |name: &str| {
            media.get(name).cloned().ok_or_else(|| {
                anyhow::anyhow!("BINDING.MEDIA_REF_INVALID: unregistered media {name}")
            })
        };
        let mut note = match self.source {
            NoteKind::Basic { front, back } => Note::basic(front, back),
            NoteKind::Cloze { text } => Note::cloze(text),
            NoteKind::Custom { id } => Note::new(id),
            NoteKind::ImageOcclusion {
                image,
                mode,
                rects,
                header,
                comments,
            } => {
                let mut builder = Note::image_occlusion(get_media(&image)?).mode(mode.into());
                if let Some(id) = &self.options.stable_id {
                    builder = builder.stable_id(id);
                }
                if let Some(header) = header {
                    builder = builder.header(header);
                }
                if let Some(comments) = comments {
                    builder = builder.comments(comments);
                }
                for rect in rects {
                    builder = builder.rect(rect.x, rect.y, rect.width, rect.height);
                }
                builder.build()?
            }
        };
        if let Some(id) = self.options.stable_id {
            note = note.stable_id(id);
        }
        if let Some(deck) = self.options.deck_name {
            note = note.deck(deck);
        }
        if let Some(extra) = self.options.back_extra {
            note = note.extra(extra);
        }
        if let Some(identity) = self.options.identity {
            note = note.identity(identity);
        }
        for tag in self.options.tags {
            note = note.tag(tag);
        }
        for (field, content) in self.fields {
            note = match content.kind {
                ContentKind::Text => note.text(field, content.value),
                ContentKind::Html => note.html(field, content.value),
                ContentKind::Image => note.image(field, get_media(&content.value)?),
                ContentKind::Sound => note.sound(field, get_media(&content.value)?),
            };
        }
        Ok(note)
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FieldInput {
    name: String,
    key: Option<String>,
    identity: Option<bool>,
    sort: Option<bool>,
    required: Option<bool>,
    optional: Option<bool>,
}
impl FieldInput {
    fn into_field(self) -> Field {
        let mut field = Field::new(self.name);
        if let Some(key) = self.key {
            field = field.key(key);
        }
        if self.identity == Some(true) {
            field = field.identity();
        }
        if self.sort == Some(true) {
            field = field.sort();
        }
        if self.required == Some(true) {
            field = field.required();
        }
        if self.optional == Some(true) {
            field = field.optional();
        }
        field
    }
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RuleInput {
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
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TemplateInput {
    name: String,
    key: Option<String>,
    front: String,
    back: String,
    browser_front: Option<String>,
    browser_back: Option<String>,
    target_deck: Option<String>,
    generate_when: Option<RuleInput>,
}
impl TemplateInput {
    fn into_template(self) -> Template {
        let mut template = Template::new(self.name).front(self.front).back(self.back);
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
        if let Some(value) = self.generate_when {
            template = template.generate_when(value.into());
        }
        template
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NoteTypeInput {
    id: String,
    cloze_field: Option<String>,
    name: Option<String>,
    fields: Vec<FieldInput>,
    templates: Vec<TemplateInput>,
    css: Option<String>,
    identity: Option<Vec<String>>,
}
impl NoteTypeInput {
    pub fn into_notetype(self) -> NoteType {
        let mut note_type = if let Some(field) = self.cloze_field {
            NoteType::custom_cloze(self.id, field)
        } else {
            NoteType::custom(self.id)
        };
        if let Some(name) = self.name {
            note_type = note_type.name(name);
        }
        if let Some(css) = self.css {
            note_type = note_type.css(css);
        }
        if let Some(identity) = self.identity {
            note_type = note_type.identity(IdentityRecipe::fields(identity));
        }
        for field in self.fields {
            note_type = note_type.field(field.into_field());
        }
        for template in self.templates {
            note_type = note_type.template(template.into_template());
        }
        note_type
    }
}
