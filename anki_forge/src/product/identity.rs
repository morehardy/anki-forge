use super::FieldKey;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityRecipe {
    field_keys: Vec<FieldKey>,
}

impl IdentityRecipe {
    pub fn fields<I, S>(fields: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut field_keys = fields
            .into_iter()
            .map(|field| FieldKey::new(field.into()))
            .collect::<Vec<_>>();
        field_keys.sort();
        field_keys.dedup();
        Self { field_keys }
    }

    pub fn field_keys(&self) -> Vec<FieldKey> {
        self.field_keys.clone()
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct CustomIdentityComponents {
    pub(crate) selected_fields: Vec<CustomIdentityFieldComponent>,
}

#[derive(Debug, Serialize)]
pub(crate) struct CustomIdentityFieldComponent {
    pub(crate) key: String,
    pub(crate) name: String,
    pub(crate) value: String,
}

pub(crate) fn derive_custom_identity(
    note_type_id: &str,
    recipe_id: &str,
    provenance: &str,
    used_override: bool,
    selected_fields: Vec<CustomIdentityFieldComponent>,
) -> crate::update_safety::model::ResolvedNoteIdentity {
    let components = CustomIdentityComponents { selected_fields };
    let (stable_id, canonical_payload) =
        crate::deck::identity::hash_payload(recipe_id, "custom", note_type_id, components)
            .expect("product identity payload should serialize");
    let canonical_payload_hash = format!("blake3:{}", blake3::hash(canonical_payload.as_bytes()));

    crate::update_safety::model::ResolvedNoteIdentity {
        stable_id: stable_id.clone(),
        current_guid_candidate: stable_id,
        recipe_id: recipe_id.into(),
        canonical_payload_hash: Some(canonical_payload_hash),
        provenance: provenance.into(),
        used_override,
    }
}

pub(crate) fn derive_custom_notetype_identity(
    note_type_id: &str,
    selected_fields: Vec<CustomIdentityFieldComponent>,
) -> crate::update_safety::model::ResolvedNoteIdentity {
    derive_custom_identity(
        note_type_id,
        "custom.notetype.fields.v1",
        "InferredFromNotetypeFields",
        false,
        selected_fields,
    )
}
