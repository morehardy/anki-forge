use std::collections::{BTreeMap, BTreeSet};

use crate::authoring_core::NormalizedIr;

/// Persistent new-model identity, independent of declaration order and display names.
pub(crate) fn derived_notetype_id(document_id: &str, notetype_id: &str) -> i64 {
    let mut hasher = blake3::Hasher::new_derive_key("anki-forge.notetype-id.v1");
    for value in [document_id, notetype_id] {
        hasher.update(&(value.len() as u64).to_be_bytes());
        hasher.update(value.as_bytes());
    }
    let mut bytes = [0_u8; 8];
    bytes.copy_from_slice(&hasher.finalize().as_bytes()[..8]);
    // Keep generated IDs exact in JSON consumers that use IEEE-754 numbers.
    (i64::from_be_bytes(bytes) & ((1_i64 << 53) - 1)).max(1)
}

pub(crate) fn resolve_notetype_ids(
    normalized: &NormalizedIr,
    selected: Option<&BTreeMap<String, i64>>,
) -> anyhow::Result<BTreeMap<String, i64>> {
    let expected: BTreeSet<_> = normalized
        .notetypes
        .iter()
        .map(|notetype| &notetype.id)
        .collect();
    if let Some(selected) = selected {
        anyhow::ensure!(
            selected.keys().collect::<BTreeSet<_>>() == expected,
            "UPDATE.WRITER_NOTETYPE_ID_PLAN_MISMATCH: model assignments must match normalized note types"
        );
    }
    let mut ids = BTreeMap::new();
    let mut occupied = BTreeMap::new();
    for notetype in &normalized.notetypes {
        let id = selected
            .map(|ids| ids[&notetype.id])
            .unwrap_or_else(|| derived_notetype_id(&normalized.document_id, &notetype.id));
        anyhow::ensure!(
            id > 0,
            "UPDATE.WRITER_NOTETYPE_ID_PLAN_MISMATCH: model IDs must be positive"
        );
        if let Some(other) = occupied.insert(id, &notetype.id) {
            anyhow::bail!(
                "UPDATE.NOTETYPE_MODEL_ID_COLLISION: {} and {} share model ID {}",
                other,
                notetype.id,
                id
            );
        }
        ids.insert(notetype.id.clone(), id);
    }
    Ok(ids)
}
