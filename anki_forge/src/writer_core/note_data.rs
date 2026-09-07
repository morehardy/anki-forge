use std::sync::OnceLock;

use serde::{ser::SerializeMap, Serialize, Serializer};

use crate::authoring_core::NormalizedNote;

use super::WriterGuidAssignment;

// Keep declaration order aligned with NoteIdentityMetadata. serde_json's
// preserve_order feature uses this order when serializing that model to Value.
#[derive(Serialize)]
struct IdentityMetadata<'a> {
    schema_version: &'static str,
    stable_id: &'a str,
    recipe_id: &'a str,
    canonical_payload_hash: Option<&'a str>,
    current_guid_candidate: &'a str,
    selected_anki_guid: &'a str,
    guid_derivation_version: &'a str,
    guid_source: &'a str,
    recovery_method: &'static str,
    provenance: &'a str,
    used_override: bool,
}

struct OrderedMetadata<'a>(IdentityMetadata<'a>);

impl Serialize for OrderedMetadata<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let metadata = &self.0;
        if json_preserves_insertion_order() {
            return metadata.serialize(serializer);
        }
        // The default Value::Object representation orders keys lexically.
        let mut map = serializer.serialize_map(Some(11))?;
        map.serialize_entry("canonical_payload_hash", &metadata.canonical_payload_hash)?;
        map.serialize_entry("current_guid_candidate", metadata.current_guid_candidate)?;
        map.serialize_entry("guid_derivation_version", metadata.guid_derivation_version)?;
        map.serialize_entry("guid_source", metadata.guid_source)?;
        map.serialize_entry("provenance", metadata.provenance)?;
        map.serialize_entry("recipe_id", metadata.recipe_id)?;
        map.serialize_entry("recovery_method", metadata.recovery_method)?;
        map.serialize_entry("schema_version", metadata.schema_version)?;
        map.serialize_entry("selected_anki_guid", metadata.selected_anki_guid)?;
        map.serialize_entry("stable_id", metadata.stable_id)?;
        map.serialize_entry("used_override", &metadata.used_override)?;
        map.end()
    }
}

fn json_preserves_insertion_order() -> bool {
    // Dependency features can be unified by a consumer without setting a
    // feature on this crate. Inspect the actual Map behavior once per process.
    static PRESERVES_ORDER: OnceLock<bool> = OnceLock::new();
    *PRESERVES_ORDER.get_or_init(|| {
        let mut map = serde_json::Map::new();
        map.insert("z".into(), serde_json::Value::Null);
        map.insert("a".into(), serde_json::Value::Null);
        map.keys().next().is_some_and(|key| key == "z")
    })
}

/// Serialize identity metadata for a newly inserted note with empty notes.data.
/// This helper does not merge or validate existing note data.
pub(crate) fn fresh_identity_note_data(
    assignment: Option<&WriterGuidAssignment>,
    note: &NormalizedNote,
) -> String {
    #[derive(Serialize)]
    struct NoteData<'a> {
        anki_forge_identity: OrderedMetadata<'a>,
    }

    let metadata = IdentityMetadata {
        schema_version: "identity-note-v1",
        stable_id: assignment.map_or(note.id.as_str(), |value| value.stable_id.as_str()),
        recipe_id: assignment.map_or("product.explicit-or-normalized.v1", |value| {
            value.recipe_id.as_str()
        }),
        canonical_payload_hash: assignment
            .and_then(|value| value.canonical_payload_hash.as_deref()),
        current_guid_candidate: assignment.map_or(note.id.as_str(), |value| {
            value.current_guid_candidate.as_str()
        }),
        selected_anki_guid: assignment
            .map_or(note.id.as_str(), |value| value.selected_anki_guid.as_str()),
        guid_derivation_version: assignment.map_or("guid.raw-stable-id.v1", |value| {
            value.guid_derivation_version.as_str()
        }),
        guid_source: assignment.map_or("current_derivation", |value| value.source.as_str()),
        recovery_method: "current_resolution",
        provenance: assignment.map_or("ExplicitStableId", |value| value.provenance.as_str()),
        used_override: assignment.is_some_and(|value| value.used_override),
    };
    serde_json::to_string(&NoteData {
        anki_forge_identity: OrderedMetadata(metadata),
    })
    .expect("identity note data serializes")
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use proptest::prelude::*;

    use super::*;
    use crate::writer_core::NoteIdentityMetadata;

    // Preserve the previous owned-model -> Value -> JSON route as the oracle.
    fn previous_data(assignment: Option<&WriterGuidAssignment>, note: &NormalizedNote) -> String {
        let metadata = NoteIdentityMetadata {
            schema_version: "identity-note-v1".into(),
            stable_id: assignment
                .map(|value| value.stable_id.clone())
                .unwrap_or_else(|| note.id.clone()),
            recipe_id: assignment
                .map(|value| value.recipe_id.clone())
                .unwrap_or_else(|| "product.explicit-or-normalized.v1".into()),
            canonical_payload_hash: assignment
                .and_then(|value| value.canonical_payload_hash.clone()),
            current_guid_candidate: assignment
                .map(|value| value.current_guid_candidate.clone())
                .unwrap_or_else(|| note.id.clone()),
            selected_anki_guid: assignment
                .map(|value| value.selected_anki_guid.clone())
                .unwrap_or_else(|| note.id.clone()),
            guid_derivation_version: assignment
                .map(|value| value.guid_derivation_version.clone())
                .unwrap_or_else(|| "guid.raw-stable-id.v1".into()),
            guid_source: assignment
                .map(|value| value.source.clone())
                .unwrap_or_else(|| "current_derivation".into()),
            recovery_method: "current_resolution".into(),
            provenance: assignment
                .map(|value| value.provenance.clone())
                .unwrap_or_else(|| "ExplicitStableId".into()),
            used_override: assignment.map(|value| value.used_override).unwrap_or(false),
        };
        let mut value: serde_json::Value = serde_json::from_str("{}").unwrap();
        value.as_object_mut().unwrap().insert(
            "anki_forge_identity".into(),
            serde_json::to_value(metadata).unwrap(),
        );
        serde_json::to_string(&value).unwrap()
    }

    fn note(id: String) -> NormalizedNote {
        NormalizedNote {
            id,
            notetype_id: "basic".into(),
            deck_name: "Default".into(),
            fields: BTreeMap::new(),
            tags: vec![],
            mtime_secs: None,
        }
    }

    #[test]
    fn fresh_data_matches_existing_bytes_for_fallback_and_assignment_variants() {
        let escaped = "\"\\\n\0\u{1f}é中文🦀";
        for id in ["", "simple", escaped] {
            let note = note(id.into());
            assert_eq!(
                fresh_identity_note_data(None, &note),
                previous_data(None, &note)
            );
            for hash in [None, Some(String::new()), Some(escaped.into())] {
                for source in ["current_derivation", "previous_apkg", "lockfile", escaped] {
                    for used_override in [false, true] {
                        let assignment = WriterGuidAssignment {
                            normalized_note_id: id.into(),
                            stable_id: format!("stable:{escaped}"),
                            selected_anki_guid: format!("selected:{escaped}"),
                            current_guid_candidate: format!("current:{escaped}"),
                            guid_derivation_version: format!("version:{escaped}"),
                            recipe_id: format!("recipe:{escaped}"),
                            canonical_payload_hash: hash.clone(),
                            provenance: format!("provenance:{escaped}"),
                            used_override,
                            source: source.into(),
                        };
                        assert_eq!(
                            fresh_identity_note_data(Some(&assignment), &note),
                            previous_data(Some(&assignment), &note),
                        );
                    }
                }
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]
        #[test]
        fn fresh_data_matches_existing_bytes_for_arbitrary_strings(
            values in prop::collection::vec(".{0,80}", 8),
            canonical_payload_hash in prop::option::of(".{0,80}"),
            used_override in any::<bool>(),
        ) {
            let note = note(values[0].clone());
            let assignment = WriterGuidAssignment {
                normalized_note_id: values[0].clone(),
                stable_id: values[1].clone(),
                selected_anki_guid: values[2].clone(),
                current_guid_candidate: values[3].clone(),
                guid_derivation_version: values[4].clone(),
                recipe_id: values[5].clone(),
                canonical_payload_hash,
                provenance: values[6].clone(),
                used_override,
                source: values[7].clone(),
            };
            prop_assert_eq!(fresh_identity_note_data(None, &note), previous_data(None, &note));
            prop_assert_eq!(
                fresh_identity_note_data(Some(&assignment), &note),
                previous_data(Some(&assignment), &note),
            );
        }
    }
}
