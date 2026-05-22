use std::path::Path;

use anyhow::{Context, Result};

use super::model::{IdentityIndex, NoteIdentityEntry};

pub fn load_previous_apkg_identity_index(
    path: impl AsRef<Path>,
    current: Option<&IdentityIndex>,
    lockfile: Option<&IdentityIndex>,
) -> Result<IdentityIndex> {
    let path = path.as_ref();
    let inspect = writer_core::inspect_apkg(path)
        .with_context(|| format!("inspect previous APKG {}", path.display()))?;
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
        if metadata.get("schema_version").and_then(|value| value.as_str()) != Some("identity-note-v1") {
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
        index.limitations.push("unknown_baseline_provenance".into());
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
            recipe_id: metadata
                .get("recipe_id")
                .and_then(|value| value.as_str())
                .unwrap_or("unknown")
                .into(),
            canonical_payload_hash: metadata
                .get("canonical_payload_hash")
                .and_then(|value| value.as_str())
                .map(str::to_string),
            provenance: "unknown_baseline".into(),
            used_override: false,
            entry_lifecycle: "active".into(),
            source_path: path.display().to_string(),
            recovery_method: "embedded_metadata".into(),
        });
    }

    if index.notes.is_empty() {
        recover_guid_equals_stable_id(&mut index, &inspect, current, lockfile);
    }

    index.limitations.sort();
    index.limitations.dedup();
    Ok(index)
}

fn recover_guid_equals_stable_id(
    index: &mut IdentityIndex,
    inspect: &writer_core::InspectReport,
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
