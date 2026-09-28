//! Language adapter for the Rust addition-error interface.
use ankiforge::media::{MediaConflictKind, MediaUsage};
use ankiforge::note::{AddDetail, AddError, AddTarget};
use serde_json::{json, Value};

pub(crate) fn details(error: &AddError) -> Value {
    let context = error.context();
    let target = match context.target() {
        AddTarget::NoteKey => json!({"type":"note_key"}),
        AddTarget::Note => json!({"type":"note"}),
        AddTarget::Model => json!({"type":"model"}),
        AddTarget::Field {
            field_key,
            content_path,
            byte_range,
            ..
        } => {
            json!({"type":"field", "field_key":field_key.as_str(), "content_path":content_path, "byte_range":byte_range.as_ref().map(|r|json!({"start":r.start,"end":r.end}))})
        }
        AddTarget::Tag { index, value, .. } => json!({"type":"tag", "index":index, "value":value}),
        AddTarget::Deck {
            name, inherited, ..
        } => json!({"type":"deck", "name":name, "inherited":inherited}),
        AddTarget::ModelAsset { media_name, .. } => {
            json!({"type":"model_asset", "media_name":media_name})
        }
        AddTarget::OcclusionImage { media_name, .. } => {
            json!({"type":"occlusion_image", "media_name":media_name})
        }
        AddTarget::ExplicitAsset { media_name, .. } => {
            json!({"type":"explicit_asset", "media_name":media_name})
        }
        AddTarget::ProjectDefaultDeck { name, .. } => {
            json!({"type":"project_default_deck", "name":name})
        }
        _ => json!({"type":"unknown"}),
    };
    let detail = error.detail().map(|detail| match detail {
        AddDetail::ModelDefinitionConflict => json!({"type":"model_definition_conflict"}),
        AddDetail::ModelNameConflict { existing_model_key, conflicting_name, .. } => json!({"type":"model_name_conflict", "existing_model_key":existing_model_key, "conflicting_name":conflicting_name}),
        AddDetail::MediaConflict { kind, existing_name, incoming_name, .. } => json!({"type":"media_conflict", "kind":match kind {
            MediaConflictKind::PortableNameCollision => "portable_name_collision",
            MediaConflictKind::DifferentContent => "different_content",
            _ => "unknown",
        }, "existing_name":existing_name, "incoming_name":incoming_name}),
        AddDetail::MediaUsage { requested, media_name, media_type, .. } => json!({"type":"media_usage", "requested":match requested {
            MediaUsage::Image => "image", MediaUsage::Sound => "sound", _ => "unknown",
        }, "media_name":media_name, "media_type":media_type}),
        _ => json!({"type":"unknown"}),
    });
    json!({"context":{"note_key":context.note_key(),"model_key":context.model_key(),"target":target},"detail":detail})
}

pub(crate) fn source(error: &AddError) -> Value {
    let mut value = details(error);
    value["type"] = json!("add");
    value["kind"] = json!(format!("{:?}", error.kind()));
    value["code"] = json!(error.code());
    value
}
