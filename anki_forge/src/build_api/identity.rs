//! Complete, versioned identities owned by a native build. This is not the
//! legacy lockfile/recovery protocol and never infers keys from display names.
use std::{collections::BTreeMap, path::Path};

use serde::{Deserialize, Serialize};

use super::{BuildError, BuildErrorKind as Kind};
use crate::{authoring_core::NormalizedIr, Project};

mod reconcile;
mod validation;
pub(crate) use validation::EvidenceError;

pub(crate) const EVIDENCE_ENTRY: &str = "ankiforge-identity.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PackageIdentity {
    pub namespace: String,
    pub models: BTreeMap<String, ModelIdentity>,
    pub notes: BTreeMap<String, NoteIdentity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ModelIdentity {
    pub id: i64,
    pub kind: String,
    pub sort_field: String,
    pub active: bool,
    pub mtime_secs: i64,
    pub content_hash: String,
    pub fields: BTreeMap<String, SymbolIdentity>,
    pub field_high_water: u32,
    pub templates: BTreeMap<String, SymbolIdentity>,
    pub template_high_water: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SymbolIdentity {
    pub id: i64,
    pub slot: u32,
    pub ordinal: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NoteIdentity {
    pub guid: String,
    pub model: String,
    pub active: bool,
    pub mtime_secs: i64,
    pub content_hash: String,
    pub masks: BTreeMap<String, MaskIdentity>,
    pub mask_high_water: u16,
    pub cards: BTreeMap<String, u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MaskIdentity {
    pub ordinal: u16,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct IdentityEnvelope {
    pub format_version: String,
    pub collection_blake3: String,
    pub identity_blake3: String,
    pub media: BTreeMap<String, MediaIdentity>,
    pub identity: PackageIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MediaIdentity {
    pub size: u64,
    pub sha1: String,
}

impl PackageIdentity {
    pub(crate) fn create(project: &Project) -> Result<Self, BuildError> {
        let now = timestamp()?;
        let mut models = BTreeMap::new();
        for (key, model) in &project.models {
            let fields = symbols(
                &project.namespace,
                key,
                "field",
                model.fields().iter().map(|f| f.key().as_str()),
            );
            let templates = symbols(
                &project.namespace,
                key,
                "template",
                model.templates().iter().map(|t| t.key().as_str()),
            );
            models.insert(
                key.clone(),
                ModelIdentity {
                    id: numeric_id(&project.namespace, "model", key, ""),
                    sort_field: model
                        .fields()
                        .iter()
                        .find(|f| f.is_sort())
                        .unwrap_or(&model.fields()[0])
                        .key()
                        .to_string(),
                    kind: if model.cloze_field().is_some() {
                        "cloze"
                    } else {
                        "normal"
                    }
                    .into(),
                    active: true,
                    mtime_secs: now,
                    content_hash: String::new(),
                    field_high_water: fields.len() as u32,
                    fields,
                    template_high_water: templates.len() as u32,
                    templates,
                },
            );
        }
        let notes = project
            .notes
            .iter()
            .map(|(key, note)| {
                let masks: BTreeMap<_, _> = note
                    .occlusion
                    .as_ref()
                    .map(|io| {
                        io.masks
                            .iter()
                            .enumerate()
                            .map(|(ordinal, mask)| {
                                (
                                    mask.key().into(),
                                    MaskIdentity {
                                        ordinal: ordinal as u16,
                                        active: true,
                                    },
                                )
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                (
                    key.clone(),
                    NoteIdentity {
                        guid: digest(&[&project.namespace, "note", key])[..32].into(),
                        model: note.model.key().into(),
                        active: true,
                        mtime_secs: now,
                        content_hash: String::new(),
                        mask_high_water: masks.len() as u16,
                        masks,
                        cards: BTreeMap::new(),
                    },
                )
            })
            .collect();
        Ok(Self {
            namespace: project.namespace.clone(),
            models,
            notes,
        })
    }

    /// Apply stable config identities before card planning or collection writes.
    pub(crate) fn bind(
        &mut self,
        project: &Project,
        normalized: &mut NormalizedIr,
    ) -> Result<(), BuildError> {
        for model in &mut normalized.notetypes {
            let key = model.id.strip_prefix("model:").ok_or_else(invalid_plan)?;
            let identity = self.models.get_mut(key).ok_or_else(invalid_plan)?;
            let declaration = project.models.get(key).ok_or_else(invalid_plan)?;
            if let Some(field) = declaration.fields().iter().find(|field| field.is_sort()) {
                identity.sort_field = field.key().to_string();
            } else if identity
                .fields
                .get(&identity.sort_field)
                .is_none_or(|field| field.ordinal.is_none())
            {
                identity.sort_field = identity
                    .fields
                    .iter()
                    .filter_map(|(key, field)| field.ordinal.map(|ord| (ord, key)))
                    .min()
                    .ok_or_else(invalid_plan)?
                    .1
                    .clone();
            }
            for field in &mut model.fields {
                let key = declaration
                    .fields()
                    .iter()
                    .find(|f| f.display_name() == field.name)
                    .ok_or_else(invalid_plan)?
                    .key()
                    .as_str();
                let symbol = &identity.fields[key];
                field.config_id = Some(symbol.id);
                field.ord = symbol.ordinal;
                field.sort = key == identity.sort_field;
            }
            for template in &mut model.templates {
                let key = declaration
                    .templates()
                    .iter()
                    .find(|t| t.display_name() == template.name)
                    .ok_or_else(invalid_plan)?
                    .key()
                    .as_str();
                let symbol = &identity.templates[key];
                template.config_id = Some(symbol.id);
                template.ord = symbol.ordinal;
            }
            model.fields.sort_by_key(|f| f.ord);
            model.templates.sort_by_key(|t| t.ord);
            let hash = value_hash(model)?;
            reconcile::advance_revision(
                &mut identity.content_hash,
                &mut identity.mtime_secs,
                hash,
            )?;
        }
        let models: BTreeMap<_, _> = normalized
            .notetypes
            .iter()
            .map(|model| (&model.id, model))
            .collect();
        for note in &mut normalized.notes {
            let identity = self.notes.get_mut(&note.id).ok_or_else(invalid_plan)?;
            let model = models[&note.notetype_id];
            let symbols = &self.models[&identity.model].templates;
            identity.cards = crate::writer_core::card_plan::plan_cards(note, model)
                .iter()
                .map(|card| {
                    if model.kind == "cloze" && card.card_ord >= 500 {
                        return Err(BuildError::new(
                            Kind::Validation,
                            "NOTE.CLOZE_ORDINAL_EXCEEDED",
                            "cloze numbers must be between 1 and 500; split this note",
                        ));
                    }
                    let key = if !identity.masks.is_empty() {
                        let mask = identity
                            .masks
                            .iter()
                            .find(|(_, mask)| {
                                mask.active && u32::from(mask.ordinal) == card.card_ord
                            })
                            .map(|(key, _)| key)
                            .ok_or_else(invalid_plan)?;
                        format!("mask:{mask}")
                    } else if model.kind == "cloze" {
                        format!("cloze:{}", card.card_ord + 1)
                    } else {
                        let template = symbols
                            .iter()
                            .find(|(_, symbol)| symbol.ordinal == Some(card.card_ord))
                            .map(|(key, _)| key)
                            .ok_or_else(invalid_plan)?;
                        format!("template:{template}")
                    };
                    Ok((key, card.card_ord))
                })
                .collect::<Result<_, BuildError>>()?;
            let hash = value_hash(&(
                &note.notetype_id,
                &note.deck_name,
                &note.fields,
                &note.tags,
                &identity.cards,
            ))?;
            reconcile::advance_revision(
                &mut identity.content_hash,
                &mut identity.mtime_secs,
                hash,
            )?;
            note.mtime_secs = Some(identity.mtime_secs);
        }
        Ok(())
    }

    pub(crate) fn model_ids(&self) -> BTreeMap<String, i64> {
        self.models
            .iter()
            .filter(|(_, m)| m.active)
            .map(|(key, model)| (format!("model:{key}"), model.id))
            .collect()
    }

    pub(crate) fn guid_plan(&self) -> crate::writer_core::WriterGuidPlan {
        crate::writer_core::WriterGuidPlan {
            assignments: self
                .notes
                .iter()
                .filter(|(_, n)| n.active)
                .map(|(key, note)| crate::writer_core::WriterGuidAssignment {
                    normalized_note_id: key.clone(),
                    stable_id: key.clone(),
                    selected_anki_guid: note.guid.clone(),
                    current_guid_candidate: note.guid.clone(),
                    guid_derivation_version: "namespace-key-v1".into(),
                    recipe_id: "explicit-key-v1".into(),
                    canonical_payload_hash: None,
                    provenance: "ExplicitStableId".into(),
                    used_override: false,
                    source: "package_identity".into(),
                })
                .collect(),
        }
    }

    pub(crate) fn envelope(
        &self,
        collection: &Path,
        normalized: &NormalizedIr,
    ) -> anyhow::Result<Vec<u8>> {
        let objects: BTreeMap<_, _> = normalized
            .media_objects
            .iter()
            .map(|object| (&object.id, object))
            .collect();
        let media = normalized
            .media_bindings
            .iter()
            .map(|binding| {
                let object = objects[&binding.object_id];
                (
                    binding.export_filename.clone(),
                    MediaIdentity {
                        size: object.size_bytes,
                        sha1: object.sha1.clone(),
                    },
                )
            })
            .collect();
        Ok(serde_json::to_vec(&IdentityEnvelope {
            format_version: "ankiforge-identity-v1".into(),
            collection_blake3: file_hash(collection)?,
            identity_blake3: identity_checksum(self)?,
            media,
            identity: self.clone(),
        })?)
    }
}

pub(crate) fn identity_checksum(identity: &PackageIdentity) -> anyhow::Result<String> {
    let json = crate::writer_core::canonical_json::to_canonical_json(identity)?;
    Ok(blake3::hash(json.as_bytes()).to_hex().to_string())
}

fn symbols<'a>(
    namespace: &str,
    model: &str,
    kind: &str,
    keys: impl Iterator<Item = &'a str>,
) -> BTreeMap<String, SymbolIdentity> {
    keys.enumerate()
        .map(|(ordinal, key)| {
            (
                key.into(),
                SymbolIdentity {
                    id: numeric_id(namespace, kind, model, key),
                    slot: ordinal as u32,
                    ordinal: Some(ordinal as u32),
                },
            )
        })
        .collect()
}

pub(crate) fn file_hash(path: &Path) -> std::io::Result<String> {
    let mut hash = blake3::Hasher::new();
    std::io::copy(&mut std::fs::File::open(path)?, &mut hash)?;
    Ok(hash.finalize().to_hex().to_string())
}

fn digest(parts: &[&str]) -> String {
    let mut hash = blake3::Hasher::new_derive_key("ankiforge.package-identity.v1");
    for part in parts {
        hash.update(&(part.len() as u64).to_be_bytes());
        hash.update(part.as_bytes());
    }
    hash.finalize().to_hex().to_string()
}

fn numeric_id(namespace: &str, kind: &str, model: &str, key: &str) -> i64 {
    let hash = digest(&[namespace, kind, model, key]);
    (i64::from_str_radix(&hash[..13], 16).expect("BLAKE3 is hexadecimal") & ((1_i64 << 52) - 1))
        .max(1)
}

fn value_hash(value: &impl Serialize) -> Result<String, BuildError> {
    serde_json::to_vec(value)
        .map(|bytes| blake3::hash(&bytes).to_hex().to_string())
        .map_err(|cause| {
            BuildError::new(
                Kind::Internal,
                "BUILD.IDENTITY_ENCODING_FAILED",
                "encode identity facts",
            )
            .caused_by(cause)
        })
}

fn timestamp() -> Result<i64, BuildError> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs().min(i64::MAX as u64) as i64)
        .map_err(|cause| {
            BuildError::new(
                Kind::Configuration,
                "BUILD.CLOCK_INVALID",
                "system clock predates the Unix epoch",
            )
            .caused_by(cause)
        })
}

fn invalid_plan() -> BuildError {
    BuildError::new(
        Kind::Internal,
        "BUILD.IDENTITY_PLAN_INVALID",
        "normalized content does not match the authored identity plan",
    )
}
