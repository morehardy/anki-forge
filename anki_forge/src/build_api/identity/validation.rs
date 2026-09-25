use std::collections::{BTreeMap, BTreeSet};

use anyhow::ensure;

use super::{IdentityEnvelope, ModelIdentity, NoteIdentity, SymbolIdentity};

#[derive(Debug)]
pub(crate) struct EvidenceError {
    pub code: &'static str,
    cause: Option<Box<dyn std::error::Error + Send + Sync>>,
}
impl EvidenceError {
    pub(crate) fn missing() -> Self {
        Self {
            code: "UPDATE.EVIDENCE_MISSING",
            cause: None,
        }
    }
    pub(crate) fn invalid(cause: anyhow::Error) -> Self {
        Self {
            code: "UPDATE.EVIDENCE_INVALID",
            cause: Some(cause.reallocate_into_boxed_dyn_error_without_backtrace()),
        }
    }
}
impl std::fmt::Display for EvidenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: complete package identity evidence is required",
            self.code
        )?;
        if let Some(cause) = &self.cause {
            write!(f, ": {cause}")?;
        }
        Ok(())
    }
}
impl std::error::Error for EvidenceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.cause.as_deref().map(|c| c as _)
    }
}

impl IdentityEnvelope {
    pub(crate) fn validate(&self, collection: &std::path::Path) -> anyhow::Result<()> {
        ensure!(
            self.format_version == "ankiforge-identity-v1",
            "unsupported identity format"
        );
        ensure!(
            self.collection_blake3 == super::file_hash(collection)?,
            "collection digest does not match identity evidence"
        );
        let actual = super::identity_checksum(&self.identity)?;
        ensure!(
            actual == self.identity_blake3,
            "identity payload digest mismatch"
        );
        ensure!(
            !self.identity.namespace.trim().is_empty(),
            "missing namespace"
        );
        let db = rusqlite::Connection::open_with_flags(
            collection,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )?;
        let count: usize = db.query_row("SELECT COUNT(*) FROM notetypes", [], |row| row.get(0))?;
        ensure!(
            count == self.identity.models.values().filter(|m| m.active).count(),
            "incomplete model mapping"
        );
        let mut model_ids = BTreeSet::new();
        let mut io_models = BTreeSet::new();
        for (key, model) in &self.identity.models {
            ensure!(
                !key.trim().is_empty() && model.id > 0 && model_ids.insert(model.id),
                "invalid or duplicate model identity"
            );
            ensure!(
                model.id == super::numeric_id(&self.identity.namespace, "model", key, ""),
                "model ID disagrees with namespace and stable key"
            );
            ensure!(
                model.mtime_secs > 0 && valid_hash(&model.content_hash),
                "invalid model revision"
            );
            validate_symbols(
                &self.identity.namespace,
                key,
                "field",
                &model.fields,
                model.field_high_water,
            )?;
            validate_symbols(
                &self.identity.namespace,
                key,
                "template",
                &model.templates,
                model.template_high_water,
            )?;
            ensure!(
                matches!(model.kind.as_str(), "normal" | "cloze"),
                "unknown model kind"
            );
            let sort_ordinal = model
                .fields
                .get(&model.sort_field)
                .and_then(|field| field.ordinal)
                .ok_or_else(|| anyhow::anyhow!("sort field is not active"))?;
            if !model.active {
                continue;
            }
            let (mtime, bytes): (i64, Vec<u8>) = db.query_row(
                "SELECT mtime_secs, config FROM notetypes WHERE id = ?1",
                [model.id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )?;
            let config = crate::writer_core::anki_proto::decode_notetype_config(&bytes)?;
            ensure!(
                config.kind == i32::from(model.kind == "cloze"),
                "model kind disagrees with collection"
            );
            ensure!(
                config.sort_field_idx == sort_ordinal,
                "sort field disagrees with collection"
            );
            if config.original_stock_kind == 6 {
                io_models.insert(model.id);
            }
            ensure!(
                mtime == model.mtime_secs,
                "model revision disagrees with collection"
            );
            // Protobuf field and template config IDs are checked by the ordinary
            // model reader; here each physical row must match the package map.
            validate_rows(&db, model.id, "fields", &model.fields)?;
            validate_rows(&db, model.id, "templates", &model.templates)?;
            ensure!(
                model.content_hash == super::content::model_from_collection(&db, model.id)?,
                "model content fingerprint disagrees with collection"
            );
        }
        let count: usize = db.query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))?;
        ensure!(
            count == self.identity.notes.values().filter(|n| n.active).count(),
            "incomplete note mapping"
        );
        let mut guids = BTreeSet::new();
        let mut active = BTreeMap::new();
        for (key, note) in &self.identity.notes {
            ensure!(
                !key.trim().is_empty() && !note.guid.is_empty() && guids.insert(&note.guid),
                "invalid or duplicate note identity"
            );
            ensure!(
                note.guid == super::note_guid(&self.identity.namespace, key),
                "note GUID disagrees with namespace and stable key"
            );
            ensure!(
                note.mtime_secs > 0 && valid_hash(&note.content_hash),
                "invalid note revision"
            );
            let model = self
                .identity
                .models
                .get(&note.model)
                .ok_or_else(|| anyhow::anyhow!("note refers to unknown model"))?;
            let mut ordinals = BTreeSet::new();
            ensure!(
                note.mask_high_water <= 500,
                "mask numbering exceeds Anki's limit"
            );
            for (mask, value) in &note.masks {
                ensure!(
                    !mask.trim().is_empty()
                        && value.ordinal < note.mask_high_water
                        && ordinals.insert(value.ordinal),
                    "invalid mask identity"
                );
            }
            ensure!(
                ordinals.len() == usize::from(note.mask_high_water),
                "mask history is incomplete"
            );
            validate_cards(note, model)?;
            if !note.active {
                continue;
            }
            ensure!(model.active, "active note refers to a retired model");
            active.insert(note.guid.as_str(), note);
        }
        let mut statement = db.prepare("SELECT id, guid, mid, mod FROM notes")?;
        let mut rows = statement.query([])?;
        let mut seen = BTreeSet::new();
        while let Some(row) = rows.next()? {
            let id: i64 = row.get(0)?;
            let guid: String = row.get(1)?;
            let mid: i64 = row.get(2)?;
            let mtime: i64 = row.get(3)?;
            ensure!(seen.insert(guid.clone()), "duplicate GUID in collection");
            let note = active
                .get(guid.as_str())
                .ok_or_else(|| anyhow::anyhow!("unmapped note GUID"))?;
            let model = &self.identity.models[&note.model];
            let has_active_masks = note.has_active_masks();
            ensure!(
                has_active_masks == io_models.contains(&model.id),
                "occlusion structure disagrees with model kind"
            );
            ensure!(
                mid == model.id && mtime == note.mtime_secs,
                "note mapping disagrees with collection"
            );
            ensure!(
                note.content_hash
                    == super::content::note_from_collection(&db, id, mid, model.kind == "cloze")?,
                "note content fingerprint disagrees with collection"
            );
            let actual = db
                .prepare("SELECT ord FROM cards WHERE nid = ?1")?
                .query_map([id], |row| row.get::<_, u32>(0))?
                .collect::<Result<Vec<_>, _>>()?;
            let expected: BTreeSet<_> = note.cards.values().copied().collect();
            ensure!(
                actual.len() == expected.len()
                    && actual.iter().copied().collect::<BTreeSet<_>>() == expected,
                "card mapping disagrees with actual cards"
            );
        }
        let actual_cards: usize =
            db.query_row("SELECT COUNT(*) FROM cards", [], |row| row.get(0))?;
        ensure!(
            actual_cards == active.values().map(|note| note.cards.len()).sum::<usize>(),
            "unmapped collection cards"
        );
        Ok(())
    }
}

// Retired notes still drive comparison and revival, even though they have no
// SQLite rows. Check their intrinsic card history before trusting that history.
// Their model can have evolved while they were absent, so only active notes
// must match the model's current template ordinals and stock IO subtype.
fn validate_cards(note: &NoteIdentity, model: &ModelIdentity) -> anyhow::Result<()> {
    let ordinals: BTreeSet<_> = note.cards.values().copied().collect();
    ensure!(
        ordinals.len() == note.cards.len(),
        "duplicate card ordinal mapping"
    );
    ensure!(
        note.masks.is_empty() || model.kind == "cloze",
        "mask history requires a cloze model"
    );
    if note.has_active_masks() {
        for (key, ordinal) in &note.cards {
            let mask = key
                .strip_prefix("mask:")
                .and_then(|key| note.masks.get(key))
                .filter(|mask| mask.active)
                .ok_or_else(|| anyhow::anyhow!("card refers to an unknown mask"))?;
            ensure!(
                *ordinal == u32::from(mask.ordinal),
                "mask card identity disagrees with its ordinal"
            );
        }
        let masks: BTreeSet<_> = note
            .masks
            .values()
            .filter(|mask| mask.active)
            .map(|mask| u32::from(mask.ordinal))
            .collect();
        ensure!(
            masks == ordinals,
            "mask mapping disagrees with card history"
        );
    } else if model.kind == "cloze" {
        for (key, ordinal) in &note.cards {
            ensure!(
                *ordinal < 500 && *key == format!("cloze:{}", ordinal + 1),
                "cloze card identity is invalid or unsupported"
            );
        }
    } else {
        let mut history = Vec::new();
        for (key, ordinal) in &note.cards {
            let template = key
                .strip_prefix("template:")
                .and_then(|key| model.templates.get(key))
                .ok_or_else(|| anyhow::anyhow!("card refers to an unknown template"))?;
            if note.active {
                ensure!(
                    template.ordinal == Some(*ordinal),
                    "template card identity disagrees with its ordinal"
                );
            } else {
                // Slot order never changes, but retired templates can disappear
                // from the contiguous physical ordinals in later model revisions.
                ensure!(
                    *ordinal <= template.slot,
                    "historical template card ordinal exceeds its slot"
                );
                history.push((template.slot, *ordinal));
            }
        }
        history.sort_unstable();
        for pair in history.windows(2) {
            let [(left_slot, left_ordinal), (right_slot, right_ordinal)] = pair else {
                unreachable!("windows contain exactly two card histories")
            };
            ensure!(
                left_ordinal < right_ordinal
                    && right_ordinal - left_ordinal <= right_slot - left_slot,
                "historical template card ordinals disagree with slot order"
            );
        }
    }
    Ok(())
}

fn valid_hash(hash: &str) -> bool {
    hash.len() == 64 && hash.bytes().all(|c| c.is_ascii_hexdigit())
}

fn validate_symbols(
    namespace: &str,
    model_key: &str,
    kind: &str,
    symbols: &std::collections::BTreeMap<String, SymbolIdentity>,
    high_water: u32,
) -> anyhow::Result<()> {
    let mut ids = BTreeSet::new();
    let mut slots = BTreeSet::new();
    let mut ordinals = Vec::new();
    for (key, value) in symbols {
        ensure!(
            !key.trim().is_empty() && value.id > 0 && ids.insert(value.id),
            "invalid or duplicate config ID"
        );
        ensure!(
            value.id == super::numeric_id(namespace, kind, model_key, key),
            "config ID disagrees with namespace, model and stable key"
        );
        ensure!(
            value.slot < high_water && slots.insert(value.slot),
            "invalid symbol history"
        );
        if let Some(ordinal) = value.ordinal {
            ordinals.push(ordinal);
        }
    }
    ensure!(
        slots.len() == high_water as usize,
        "symbol history is incomplete"
    );
    ordinals.sort_unstable();
    ensure!(
        ordinals.iter().copied().eq(0..ordinals.len() as u32),
        "physical ordinals must be contiguous"
    );
    Ok(())
}

// Only the two literal table names above reach this query.
fn validate_rows(
    db: &rusqlite::Connection,
    id: i64,
    table: &str,
    symbols: &std::collections::BTreeMap<String, SymbolIdentity>,
) -> anyhow::Result<()> {
    let expected: BTreeMap<_, _> = symbols
        .values()
        .filter_map(|s| s.ordinal.map(|ord| (ord, s.id)))
        .collect();
    let rows = db
        .prepare(&format!(
            "SELECT ord, config FROM {table} WHERE ntid = ?1 ORDER BY ord"
        ))?
        .query_map([id], |row| {
            Ok((row.get::<_, u32>(0)?, row.get::<_, Vec<u8>>(1)?))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    ensure!(
        rows.len() == symbols.values().filter(|s| s.ordinal.is_some()).count(),
        "incomplete config mapping"
    );
    for (ord, bytes) in rows {
        let actual = if table == "fields" {
            crate::writer_core::anki_proto::decode_field_config(&bytes)?.id
        } else {
            crate::writer_core::anki_proto::decode_template_config(&bytes)?.id
        };
        let expected = expected.get(&ord).copied();
        ensure!(
            actual.is_some() && actual == expected,
            "config ID mapping disagrees with collection"
        );
    }
    Ok(())
}
