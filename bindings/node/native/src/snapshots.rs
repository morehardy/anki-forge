//! Explicit projections of the supported product interface, never the normalization IR.
use anki_forge::prelude::{Field, GenerationRule, IdentityRecipe, NoteType, Template};
use anki_forge::product::NoteTypeKind;
use napi::Result;
use napi_derive::napi;
use serde_json::{json, Value};

use crate::authoring::{FieldInput, NoteTypeInput, RuleInput, TemplateInput};
use crate::{parse, reports};

fn field(value: &Field) -> Value {
    json!({"key": value.key_ref().as_str(), "name": value.name(),
        "identity": value.is_identity(), "sort": value.is_sort(),
        "required": value.is_required(), "optional": value.is_optional(),
        "keyAutoDerived": value.key_auto_derived()})
}
fn identity(value: &IdentityRecipe) -> Value {
    json!({"fieldKeys": value.field_keys().iter().map(|key| key.as_str()).collect::<Vec<_>>()})
}
fn rule(value: &GenerationRule) -> Value {
    match value {
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
fn template(value: &Template) -> Value {
    json!({"key": value.key_ref().as_str(), "name": value.name(),
        "front": value.front_source().as_str(), "back": value.back_source().as_str(),
        "browserFront": value.browser_front_source().map(|source| source.as_str()),
        "browserBack": value.browser_back_source().map(|source| source.as_str()),
        "targetDeck": value.target_deck_name(), "generateWhen": rule(value.generation_rule())})
}
fn notetype(value: &NoteType) -> Value {
    let (kind, cloze_field) = match value.kind() {
        NoteTypeKind::Normal => ("normal", None),
        NoteTypeKind::Cloze { field } => ("cloze", Some(field.as_str())),
    };
    json!({"id": value.id(), "kind": kind, "clozeField": cloze_field,
        "name": value.name_ref(), "fields": value.fields().iter().map(field).collect::<Vec<_>>(),
        "templates": value.templates().iter().map(template).collect::<Vec<_>>(),
        "css": value.css_ref(), "identity": value.identity_ref().map(identity)})
}

#[napi]
pub fn describe_field(input: String) -> Result<String> {
    Ok(reports::success(field(
        &parse::<FieldInput>(&input)?.into_field(),
    )))
}
#[napi]
pub fn describe_template(input: String) -> Result<String> {
    Ok(reports::success(template(
        &parse::<TemplateInput>(&input)?.into_template(),
    )))
}
#[napi]
pub fn describe_note_type(input: String) -> Result<String> {
    Ok(reports::success(notetype(
        &parse::<NoteTypeInput>(&input)?.into_notetype(),
    )))
}
#[napi]
pub fn describe_identity(fields: Vec<String>) -> String {
    reports::success(identity(&IdentityRecipe::fields(fields)))
}
#[napi]
pub fn describe_generation_rule(input: String) -> Result<String> {
    Ok(reports::success(rule(&parse::<RuleInput>(&input)?.into())))
}

#[napi]
pub fn describe_note(
    input: String,
    references: Vec<napi::bindgen_prelude::ClassInstance<crate::NativeMediaRef>>,
) -> Result<String> {
    let media = references
        .iter()
        .map(|reference| {
            (
                reference.inner.filename().to_string(),
                reference.inner.clone(),
            )
        })
        .collect();
    let note = match parse::<crate::authoring::NoteInput>(&input)?.into_note(&media) {
        Ok(note) => note,
        Err(error) => return Ok(reports::domain_failure("note", error)),
    };
    Ok(reports::success(json!({"noteTypeId": note.note_type_id(),
        "stableId": note.stable_id_ref(), "deckName": note.deck_name(),
        "tags": note.tags(), "identity": note.identity_ref().map(identity),
        "renderedFields": note.rendered_fields()})))
}

pub(crate) fn deck(value: &anki_forge::deck::Deck) -> Value {
    use anki_forge::deck::{DeckNote, IdentityProvenance};
    let notes = value
        .notes()
        .iter()
        .map(|note| {
            // Deck note fields have a public Serialize interface but no individual getters.
            // Select named fields here so changes to that representation cannot leak new IR.
            let (kind, source) = match note {
                DeckNote::Basic(note) => (
                    "basic",
                    serde_json::to_value(note).expect("BasicNote serializes"),
                ),
                DeckNote::Cloze(note) => (
                    "cloze",
                    serde_json::to_value(note).expect("ClozeNote serializes"),
                ),
                DeckNote::ImageOcclusion(note) => (
                    "image_occlusion",
                    serde_json::to_value(note).expect("IoNote serializes"),
                ),
            };
            let resolved = note.resolved_identity().map(|identity| {
                let provenance = match identity.provenance {
                    IdentityProvenance::ExplicitStableId => "explicit_stable_id",
                    IdentityProvenance::InferredFromNoteFields => "inferred_from_note_fields",
                    IdentityProvenance::InferredFromNotetypeFields => {
                        "inferred_from_notetype_fields"
                    }
                    IdentityProvenance::InferredFromStockRecipe => "inferred_from_stock_recipe",
                };
                json!({"stableId":identity.stable_id, "recipeId": identity.recipe_id,
                "provenance": provenance, "canonicalPayload": identity.canonical_payload,
                "usedOverride": identity.used_override})
            });
            let mut result = json!({"kind":kind, "id":note.id(), "stableId":source["stable_id"],
            "tags":source["tags"], "generated":source["generated"], "resolvedIdentity":resolved});
            let fields: &[(&str, &str)] = match note {
                DeckNote::Basic(_) => &[("front", "front"), ("back", "back")],
                DeckNote::Cloze(_) => &[("text", "text"), ("extra", "extra")],
                DeckNote::ImageOcclusion(_) => &[
                    ("image", "image"),
                    ("rects", "rects"),
                    ("header", "header"),
                    ("backExtra", "back_extra"),
                    ("comments", "comments"),
                ],
            };
            for (key, source_key) in fields {
                result[key] = source[source_key].clone();
            }
            if let DeckNote::Basic(note) = note {
                result["identityOverride"] = note.identity_override_config().map(|value|
                json!({"fields": value.fields(), "reasonCode": value.reason_code()})
            ).unwrap_or(Value::Null);
            }
            if matches!(note, DeckNote::ImageOcclusion(_)) {
                result["mode"] = json!(match source["mode"].as_str() {
                    Some("HideAllGuessOne") => "hide-all-guess-one",
                    Some("HideOneGuessOne") => "hide-one-guess-one",
                    _ => unreachable!("known IoMode"),
                });
            }
            result
        })
        .collect::<Vec<_>>();
    json!({"name": value.name(), "stableId":value.stable_id(),
        "identityPolicy": {"basic":value.identity_policy().basic.as_ref().map(|fields| fields.as_slice())},
        "notes":notes})
}
