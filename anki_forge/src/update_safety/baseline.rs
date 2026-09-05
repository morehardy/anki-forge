use std::path::Path;

use anyhow::{Context, Result};

use super::model::{
    field_merge_key, template_merge_key, FieldMergeEntry, IdentityIndex, NoteIdentityEntry,
    NotetypeIdentityEntry, TemplateMergeEntry,
};

pub fn load_previous_apkg_identity_index(
    path: impl AsRef<Path>,
    current: Option<&IdentityIndex>,
    lockfile: Option<&IdentityIndex>,
) -> Result<IdentityIndex> {
    let path = path.as_ref();
    let inspect = crate::writer_core::inspect_apkg(path)
        .with_context(|| format!("inspect previous APKG {}", path.display()))?;
    identity_index_from_inspect(path, &inspect, current, lockfile)
}

pub(crate) fn identity_index_from_inspect(
    path: &Path,
    inspect: &crate::writer_core::InspectReport,
    current: Option<&IdentityIndex>,
    lockfile: Option<&IdentityIndex>,
) -> Result<IdentityIndex> {
    let mut index = IdentityIndex {
        schema_version: "identity-index-v1".into(),
        source_kind: "previous_apkg".into(),
        source_ref: "baseline.previous_apkg.primary".into(),
        writer_policy_ref: "unknown@unknown".into(),
        project_stable_id: None,
        notes: vec![],
        notetypes: vec![],
        limitations: vec![],
    };

    for metadata in &inspect.observations.metadata {
        if metadata
            .get("schema_version")
            .and_then(|value| value.as_str())
            != Some("identity-note-v1")
        {
            continue;
        }
        let Some(stable_id) = metadata.get("stable_id").and_then(|value| value.as_str()) else {
            index.limitations.push("identity_metadata_malformed".into());
            continue;
        };
        let selected = metadata
            .get("selected_anki_guid")
            .and_then(|value| value.as_str())
            .unwrap_or(stable_id);
        let provenance = metadata
            .get("provenance")
            .and_then(|value| value.as_str())
            .unwrap_or("unknown_baseline");
        if provenance == "unknown_baseline" {
            index.limitations.push("unknown_baseline_provenance".into());
        }
        index.notes.push(NoteIdentityEntry {
            stable_id: stable_id.into(),
            normalized_note_id: None,
            anki_guid: selected.into(),
            current_guid_candidate: metadata
                .get("current_guid_candidate")
                .and_then(|value| value.as_str())
                .unwrap_or(stable_id)
                .into(),
            guid_derivation_version: metadata
                .get("guid_derivation_version")
                .and_then(|value| value.as_str())
                .unwrap_or("guid.raw-stable-id.v1")
                .into(),
            note_type_id: "unknown".into(),
            revision: None,
            recipe_id: metadata
                .get("recipe_id")
                .and_then(|value| value.as_str())
                .unwrap_or("unknown")
                .into(),
            canonical_payload_hash: metadata
                .get("canonical_payload_hash")
                .and_then(|value| value.as_str())
                .map(str::to_string),
            provenance: provenance.into(),
            used_override: metadata
                .get("used_override")
                .and_then(|value| value.as_bool())
                .unwrap_or(false),
            entry_lifecycle: "active".into(),
            source_path: path.display().to_string(),
            recovery_method: "embedded_metadata".into(),
        });
    }

    if index.notes.is_empty() {
        recover_guid_equals_stable_id(&mut index, inspect, current, lockfile);
    }
    recover_notetype_merge_metadata(&mut index, inspect);
    recover_note_revisions(&mut index, inspect)?;
    super::notetype_ids::validate_baseline_model_ids(&index)?;

    index.limitations.sort();
    index.limitations.dedup();
    Ok(index)
}

fn recover_note_revisions(
    index: &mut IdentityIndex,
    inspect: &crate::writer_core::InspectReport,
) -> Result<()> {
    let by_guid: std::collections::BTreeMap<_, _> = inspect
        .observations
        .references
        .iter()
        .filter_map(|entry| Some((entry.get("id")?.as_str()?, entry.get("revision")?)))
        .collect();
    for note in &mut index.notes {
        if let Some(value) = by_guid.get(note.anki_guid.as_str()) {
            let revision: super::model::NoteRevision = serde_json::from_value((*value).clone())?;
            revision.validate()?;
            note.revision = Some(revision);
        }
    }
    Ok(())
}

fn recover_notetype_merge_metadata(
    index: &mut IdentityIndex,
    inspect: &crate::writer_core::InspectReport,
) {
    let mut fields_by_notetype = std::collections::BTreeMap::<String, Vec<FieldMergeEntry>>::new();
    for field in &inspect.observations.fields {
        let Some(notetype_id) = field.get("notetype_id").and_then(|value| value.as_str()) else {
            continue;
        };
        let Some(name) = field.get("name").and_then(|value| value.as_str()) else {
            continue;
        };
        let config_id = field
            .get("config_id")
            .and_then(|value| value.as_i64())
            .unwrap_or(0);
        fields_by_notetype
            .entry(notetype_id.to_string())
            .or_default()
            .push(FieldMergeEntry {
                field_key: field_merge_key(name, Some(config_id)),
                field_name: name.to_string(),
                ord: field
                    .get("ord")
                    .and_then(|value| value.as_u64())
                    .unwrap_or_default() as u32,
                config_id,
                tag: field
                    .get("tag")
                    .and_then(|value| value.as_i64())
                    .unwrap_or_default() as i32,
            });
    }

    let mut templates_by_notetype =
        std::collections::BTreeMap::<String, Vec<TemplateMergeEntry>>::new();
    for template in &inspect.observations.templates {
        let Some(notetype_id) = template.get("notetype_id").and_then(|value| value.as_str()) else {
            continue;
        };
        let Some(name) = template.get("name").and_then(|value| value.as_str()) else {
            continue;
        };
        let config_id = template
            .get("config_id")
            .and_then(|value| value.as_i64())
            .unwrap_or(0);
        templates_by_notetype
            .entry(notetype_id.to_string())
            .or_default()
            .push(TemplateMergeEntry {
                template_key: template_merge_key(name, Some(config_id)),
                template_name: name.to_string(),
                ord: template
                    .get("ord")
                    .and_then(|value| value.as_u64())
                    .unwrap_or_default() as u32,
                config_id,
            });
    }

    for notetype in &inspect.observations.notetypes {
        let Some(note_type_id) = notetype.get("id").and_then(|value| value.as_str()) else {
            continue;
        };
        let Some(name) = notetype.get("name").and_then(|value| value.as_str()) else {
            continue;
        };
        index.notetypes.push(NotetypeIdentityEntry {
            note_type_id: note_type_id.to_string(),
            anki_model_id: notetype
                .get("anki_model_id")
                .and_then(|value| value.as_i64()),
            name: name.to_string(),
            fields: fields_by_notetype.remove(note_type_id).unwrap_or_default(),
            templates: templates_by_notetype
                .remove(note_type_id)
                .unwrap_or_default(),
        });
    }
}

fn recover_guid_equals_stable_id(
    index: &mut IdentityIndex,
    inspect: &crate::writer_core::InspectReport,
    current: Option<&IdentityIndex>,
    lockfile: Option<&IdentityIndex>,
) {
    let mut stable_ids = std::collections::BTreeSet::new();
    if let Some(current) = current {
        for note in &current.notes {
            stable_ids.insert(note.stable_id.as_str());
        }
    }
    if let Some(lockfile) = lockfile {
        for note in &lockfile.notes {
            stable_ids.insert(note.stable_id.as_str());
        }
    }
    for note in &inspect.observations.references {
        let Some(selector) = note.get("selector").and_then(|value| value.as_str()) else {
            continue;
        };
        if !selector.starts_with("note[id='") {
            continue;
        }
        let Some(guid) = note.get("id").and_then(|value| value.as_str()) else {
            continue;
        };
        if stable_ids.contains(guid) {
            index.limitations.push("unknown_baseline_provenance".into());
            index.notes.push(NoteIdentityEntry {
                stable_id: guid.into(),
                normalized_note_id: None,
                anki_guid: guid.into(),
                current_guid_candidate: guid.into(),
                guid_derivation_version: "guid.raw-stable-id.v1".into(),
                note_type_id: "unknown".into(),
                revision: None,
                recipe_id: "unknown".into(),
                canonical_payload_hash: None,
                provenance: "unknown_baseline".into(),
                used_override: false,
                entry_lifecycle: "active".into(),
                source_path: "inspect.notes".into(),
                recovery_method: "guid_equals_stable_id".into(),
            });
        }
    }
}

pub fn lockfile_identity_index(lockfile: &super::model::IdentityLockfile) -> IdentityIndex {
    lockfile.identity_index.clone()
}
