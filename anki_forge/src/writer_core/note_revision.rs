use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::authoring_core::NormalizedNote;

const HASH_PREFIX: &str = "note-content.v1:blake3:";
pub(crate) const INITIAL_MTIME_SECS: i64 = 1;

/// Full note content evidence, independent of the fields used to derive identity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NoteRevision {
    pub content_hash: String,
    pub mtime_secs: i64,
}

impl NoteRevision {
    pub(crate) fn from_note(note: &NormalizedNote) -> Self {
        // Match the writer's space-separated tag storage, without order/duplicates.
        let tags: BTreeSet<_> = note
            .tags
            .iter()
            .flat_map(|tag| tag.split(' '))
            .filter(|tag| !tag.is_empty())
            .collect();
        let payload = super::to_canonical_json(&json!({
            "notetype_id": note.notetype_id,
            "fields": note.fields,
            "tags": tags,
        }))
        .expect("note revision payload contains only JSON-compatible strings and maps");
        Self {
            content_hash: format!("{HASH_PREFIX}{}", blake3::hash(payload.as_bytes()).to_hex()),
            mtime_secs: note.mtime_secs.unwrap_or(INITIAL_MTIME_SECS),
        }
    }

    pub(crate) fn validate(&self) -> anyhow::Result<()> {
        let hash = self.content_hash.strip_prefix(HASH_PREFIX).unwrap_or("");
        anyhow::ensure!(self.mtime_secs > 0 && hash.len() == 64
            && hash.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)),
            "UPDATE.NOTE_REVISION_INVALID: expected a positive time and a supported full-content hash");
        Ok(())
    }
}
