use crate::{core_error, mapped, NativeMedia};
use ankiforge::schema::GenerationRule;
use ankiforge::{Field, NoteType, Template};
use pyo3::prelude::*;
use serde::Deserialize;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelInput {
    key: String,
    name: Option<String>,
    fields: Vec<FieldInput>,
    templates: Vec<TemplateInput>,
    css: String,
    cloze_field: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FieldInput {
    key: String,
    name: Option<String>,
    required: bool,
    sort: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TemplateInput {
    key: String,
    name: Option<String>,
    front: String,
    back: String,
    browser_front: Option<String>,
    browser_back: Option<String>,
    target_deck: Option<String>,
    generation: RuleInput,
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum RuleInput {
    AnkiDefault,
    All { fields: Vec<String> },
    Any { fields: Vec<String> },
}
impl ModelInput {
    pub fn build(self, assets: Vec<Py<NativeMedia>>) -> PyResult<NoteType> {
        let mut b = NoteType::builder(self.key).css(self.css);
        if let Some(name) = self.name {
            b = b.name(name);
        }
        if let Some(key) = self.cloze_field {
            b = b.cloze_field(key);
        }
        for f in self.fields {
            let mut field = Field::new(f.key);
            if let Some(name) = f.name {
                field = field.name(name);
            }
            if f.required {
                field = field.required();
            }
            if f.sort {
                field = field.sort();
            }
            b = b.field(field);
        }
        for t in self.templates {
            let mut template = Template::new(t.key).front(t.front).back(t.back);
            if let Some(v) = t.name {
                template = template.name(v);
            }
            if let Some(v) = t.browser_front {
                template = template.browser_front(v);
            }
            if let Some(v) = t.browser_back {
                template = template.browser_back(v);
            }
            if let Some(v) = t.target_deck {
                template = template.target_deck(v);
            }
            template = template.generate_when(match t.generation {
                RuleInput::AnkiDefault => GenerationRule::AnkiDefault,
                RuleInput::All { fields } => GenerationRule::all(fields),
                RuleInput::Any { fields } => GenerationRule::any(fields),
            });
            b = b.template(template);
        }
        for asset in assets {
            b = b.asset(asset.get().inner.get()?.clone());
        }
        b.build().map_err(mapped!("schema"))
    }
}
