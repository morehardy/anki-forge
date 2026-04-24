use crate::deck::model::{BasicIdentityField, BasicNote, Deck, DeckNote, IdentityProvenance};
use serde::Serialize;
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedIdentity {
    pub stable_id: String,
    pub recipe_id: String,
    pub canonical_payload: String,
    pub provenance: IdentityProvenance,
    pub used_override: bool,
}

#[derive(Debug, Serialize)]
struct CanonicalIdentityPayload<'a, T: Serialize> {
    algo_version: u8,
    recipe_id: &'a str,
    notetype_family: &'a str,
    notetype_key: &'a str,
    components: T,
}

#[derive(Debug, Serialize)]
struct BasicFieldComponent {
    name: &'static str,
    value: String,
}

#[derive(Debug, Serialize)]
struct BasicComponents {
    selected_fields: Vec<BasicFieldComponent>,
}

const BASIC_RECIPE_ID: &str = "basic.core.v1";
const BASIC_DEFAULT_FIELDS: [BasicIdentityField; 1] = [BasicIdentityField::Front];

pub(crate) fn normalize_field_text_for_identity(value: &str) -> String {
    value
        .nfc()
        .collect::<String>()
        .replace("\r\n", "\n")
        .replace('\r', "\n")
}

pub(crate) fn hash_payload<T: Serialize>(
    recipe_id: &str,
    notetype_family: &str,
    notetype_key: &str,
    components: T,
) -> anyhow::Result<(String, String)> {
    let payload = CanonicalIdentityPayload {
        algo_version: 1,
        recipe_id,
        notetype_family,
        notetype_key,
        components,
    };
    let canonical_payload = authoring_core::to_canonical_json(&payload)?;
    let stable_id = format!(
        "afid:v1:{}",
        blake3::hash(canonical_payload.as_bytes()).to_hex()
    );
    Ok((stable_id, canonical_payload))
}

pub(crate) fn resolve_inferred_identity(
    deck: &Deck,
    note: &DeckNote,
) -> anyhow::Result<ResolvedIdentity> {
    match note {
        DeckNote::Basic(note) => resolve_basic_identity(deck, note),
        DeckNote::Cloze(_) => {
            anyhow::bail!("AFID.IDENTITY_COMPONENT_EMPTY: cloze resolver not implemented")
        }
        DeckNote::ImageOcclusion(_) => {
            anyhow::bail!("AFID.IDENTITY_COMPONENT_EMPTY: image occlusion resolver not implemented")
        }
    }
}

fn resolve_basic_identity(deck: &Deck, note: &BasicNote) -> anyhow::Result<ResolvedIdentity> {
    let (fields, provenance, used_override) =
        if let Some(override_cfg) = note.identity_override_config() {
            (
                override_cfg.fields(),
                IdentityProvenance::InferredFromNoteFields,
                true,
            )
        } else if let Some(selection) = deck.identity_policy().basic.as_ref() {
            (
                selection.as_slice(),
                IdentityProvenance::InferredFromNotetypeFields,
                false,
            )
        } else {
            (
                BASIC_DEFAULT_FIELDS.as_slice(),
                IdentityProvenance::InferredFromStockRecipe,
                false,
            )
        };

    let components = BasicComponents {
        selected_fields: fields
            .iter()
            .map(|field| match field {
                BasicIdentityField::Front => BasicFieldComponent {
                    name: "front",
                    value: normalize_field_text_for_identity(&note.front),
                },
                BasicIdentityField::Back => BasicFieldComponent {
                    name: "back",
                    value: normalize_field_text_for_identity(&note.back),
                },
            })
            .collect(),
    };
    let (stable_id, canonical_payload) =
        hash_payload(BASIC_RECIPE_ID, "stock", "basic", components)?;

    Ok(ResolvedIdentity {
        stable_id,
        recipe_id: BASIC_RECIPE_ID.to_string(),
        canonical_payload,
        provenance,
        used_override,
    })
}
