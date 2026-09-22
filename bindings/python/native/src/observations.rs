//! On-demand views of core state; the adapter does not retain a second model.
use anki_forge::prelude::{Content, GenerationRule, Note, NoteType, NoteTypeKind};
use serde_json::{json, Value};

pub fn note(note: &Note) -> Value {
    let fields: serde_json::Map<_, _> = note
        .fields_ref()
        .iter()
        .map(|(key, content)| {
            let value = match content {
                Content::Text(value) => json!({"kind": "text", "value": value}),
                _ => json!({"kind": "html", "value": content.render()}),
            };
            (key.clone(), value)
        })
        .collect();
    json!({
        "note_type_id": note.note_type_id(), "stable_id": note.stable_id_ref(),
        "deck_name": note.deck_name(), "tags": note.tags(), "fields": fields,
        "identity": note.identity_ref().map(|recipe| recipe.field_keys().iter()
            .map(|key| key.as_str().to_string()).collect::<Vec<_>>()),
    })
}

fn rule(rule: &GenerationRule) -> Value {
    match rule {
        GenerationRule::AnkiDefault => json!({"kind": "anki_default"}),
        GenerationRule::All(fields) => {
            json!({"kind": "all", "fields": fields.iter().map(|key| key.as_str()).collect::<Vec<_>>()})
        }
        GenerationRule::Any(fields) => {
            json!({"kind": "any", "fields": fields.iter().map(|key| key.as_str()).collect::<Vec<_>>()})
        }
        GenerationRule::Cloze { field } => json!({"kind": "cloze", "field": field.as_str()}),
    }
}

pub fn notetype(note_type: &NoteType) -> Value {
    let (kind, cloze_field) = match note_type.kind() {
        NoteTypeKind::Normal => ("normal", None),
        NoteTypeKind::Cloze { field } => ("cloze", Some(field.as_str())),
    };
    let fields: Vec<_> = note_type.fields().iter().map(|field| json!({
        "name": field.name(), "key": field.key_ref().as_str(),
        "identity": field.is_identity(), "sort": field.is_sort(), "required": field.is_required(),
        "optional": field.is_optional(),
        "key_auto_derived": field.key_auto_derived(),
    })).collect();
    let templates: Vec<_> = note_type.templates().iter().map(|template| json!({
        "name": template.name(), "key": template.key_ref().as_str(),
        "front": template.front_source().as_str(), "back": template.back_source().as_str(),
        "browser_front": template.browser_front_source().map(|source| source.as_str()),
        "browser_back": template.browser_back_source().map(|source| source.as_str()),
        "target_deck": template.target_deck_name(), "generate_when": rule(template.generation_rule()),
    })).collect();
    json!({
        "id": note_type.id(), "name": note_type.name_ref(), "kind_value": kind,
        "cloze_field": cloze_field, "css_value": note_type.css_ref(),
        "fields": fields, "templates": templates,
        "identity": note_type.identity_ref().map(|recipe| recipe.field_keys().iter()
            .map(|key| key.as_str().to_string()).collect::<Vec<_>>()),
    })
}
