use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

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
        let payload = revision_payload_json(note);
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

fn revision_payload_json(note: &NormalizedNote) -> String {
    // This fixed payload contains only strings, a BTreeMap and sorted tags.
    // Declare struct fields in canonical order so direct serialization keeps
    // identical bytes even if a consumer enables serde_json/preserve_order.
    #[derive(Serialize)]
    struct Payload<'a> {
        fields: &'a BTreeMap<String, String>,
        notetype_id: &'a str,
        tags: BTreeSet<&'a str>,
    }
    // Match the writer's space-separated tag storage, without order/duplicates.
    let tags = note
        .tags
        .iter()
        .flat_map(|tag| tag.split(' '))
        .filter(|tag| !tag.is_empty())
        .collect();
    serde_json::to_string(&Payload {
        fields: &note.fields,
        notetype_id: &note.notetype_id,
        tags,
    })
    .expect("note revision payload contains only JSON-compatible strings and maps")
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use serde_json::json;

    fn previous_payload(note: &NormalizedNote) -> String {
        let tags: BTreeSet<_> = note
            .tags
            .iter()
            .flat_map(|tag| tag.split(' '))
            .filter(|tag| !tag.is_empty())
            .collect();
        crate::writer_core::to_canonical_json(&json!({
            "notetype_id": note.notetype_id,
            "fields": note.fields,
            "tags": tags,
        }))
        .unwrap()
    }

    fn previous_revision(note: &NormalizedNote) -> NoteRevision {
        let payload = previous_payload(note);
        NoteRevision {
            content_hash: format!("{HASH_PREFIX}{}", blake3::hash(payload.as_bytes()).to_hex()),
            mtime_secs: note.mtime_secs.unwrap_or(INITIAL_MTIME_SECS),
        }
    }

    #[test]
    fn note_revision_keeps_canonical_hashes_for_empty_wide_and_escaped_fields() {
        let mut note = NormalizedNote {
            id: "note".into(),
            notetype_id: "型\"\n\\".into(),
            deck_name: "Default".into(),
            fields: BTreeMap::new(),
            tags: vec![" z  a ".into(), "a".into(), "中文\t标签".into()],
            mtime_secs: None,
        };
        assert_eq!(revision_payload_json(&note), previous_payload(&note));
        assert_eq!(NoteRevision::from_note(&note), previous_revision(&note));
        note.fields = (0..128)
            .rev()
            .map(|index| {
                (
                    format!("字段 {index:03}"),
                    format!("<b>中文 &amp; é🦀</b>\"\n\\\0\u{1f}{index}"),
                )
            })
            .collect();
        note.mtime_secs = Some(123_456);
        assert_eq!(revision_payload_json(&note), previous_payload(&note));
        assert_eq!(NoteRevision::from_note(&note), previous_revision(&note));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]
        #[test]
        fn note_revision_matches_previous_canonical_serialization(
            fields in prop::collection::btree_map(".{0,16}", ".{0,80}", 0..12),
            tags in prop::collection::vec(".{0,20}", 0..8),
            notetype_id in ".{0,24}",
            mtime_secs in prop::option::of(1..i64::MAX),
        ) {
            let note = NormalizedNote {
                id: "note".into(),
                notetype_id,
                deck_name: "Default".into(),
                fields,
                tags,
                mtime_secs,
            };
            prop_assert_eq!(revision_payload_json(&note), previous_payload(&note));
            prop_assert_eq!(NoteRevision::from_note(&note), previous_revision(&note));
        }
    }
}
