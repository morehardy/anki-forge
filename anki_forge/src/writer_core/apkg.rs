use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::authoring_core::stock::resolve_stock_notetype;
use crate::authoring_core::{AuthoringNotetype, NormalizedIr, NormalizedNote, NormalizedNotetype};
use anyhow::{Context, Result};
use prost::Message;
use rusqlite::Connection;
use sha1::{Digest, Sha1};
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::writer_core::anki_proto::{
    default_deck_common_bytes, default_deck_config_bytes, default_deck_kind_bytes,
    encode_field_config, encode_notetype_config, encode_template_config,
};
use crate::writer_core::card_plan::plan_cards;
use crate::writer_core::compat_schema::{
    SCHEMA11_SQL, SCHEMA14_UPGRADE_SQL, SCHEMA15_UPGRADE_SQL, SCHEMA17_UPGRADE_SQL,
    SCHEMA18_UPGRADE_SQL,
};
use crate::writer_core::model::{NoteIdentityMetadata, WriterGuidAssignment, WriterGuidPlan};
use crate::writer_core::staging::{
    load_normalized_ir_from_staging_manifest, resolve_deck_ids, BuildArtifactTarget,
    MaterializedStaging,
};

pub struct ApkgMaterialization {
    pub apkg_ref: String,
    pub apkg_path: PathBuf,
    pub package_fingerprint: String,
}

#[derive(Clone, PartialEq, Message)]
struct PackageMetadata {
    #[prost(int32, tag = "1")]
    version: i32,
}

#[derive(Clone, PartialEq, Message)]
struct MediaEntries {
    #[prost(message, repeated, tag = "1")]
    entries: Vec<MediaEntry>,
}

#[derive(Clone, PartialEq, Message)]
struct MediaEntry {
    #[prost(string, tag = "1")]
    name: String,
    #[prost(uint32, tag = "2")]
    size: u32,
    #[prost(bytes, tag = "3")]
    sha1: Vec<u8>,
    #[prost(uint32, optional, tag = "255")]
    legacy_zip_filename: Option<u32>,
}

fn validate_guid_plan(
    normalized_ir: &NormalizedIr,
    guid_plan: Option<&WriterGuidPlan>,
) -> anyhow::Result<std::collections::BTreeMap<String, WriterGuidAssignment>> {
    let Some(plan) = guid_plan else {
        return Ok(Default::default());
    };

    let expected: std::collections::BTreeSet<_> = normalized_ir
        .notes
        .iter()
        .map(|note| note.id.as_str())
        .collect();
    let mut seen = std::collections::BTreeSet::new();
    let mut by_note = std::collections::BTreeMap::new();

    for assignment in &plan.assignments {
        if !seen.insert(assignment.normalized_note_id.as_str()) {
            anyhow::bail!(
                "UPDATE.WRITER_GUID_PLAN_MISMATCH: duplicate assignment for {}",
                assignment.normalized_note_id
            );
        }
        by_note.insert(assignment.normalized_note_id.clone(), assignment.clone());
    }

    let actual: std::collections::BTreeSet<_> = by_note.keys().map(String::as_str).collect();
    if expected != actual {
        anyhow::bail!(
            "UPDATE.WRITER_GUID_PLAN_MISMATCH: plan ids {:?} did not match normalized note ids {:?}",
            actual,
            expected
        );
    }

    Ok(by_note)
}

fn note_identity_metadata_for_assignment(
    assignment: Option<&WriterGuidAssignment>,
    note: &NormalizedNote,
) -> NoteIdentityMetadata {
    let selected = assignment
        .map(|a| a.selected_anki_guid.clone())
        .unwrap_or_else(|| note.id.clone());
    let source = assignment
        .map(|a| a.source.clone())
        .unwrap_or_else(|| "current_derivation".into());

    NoteIdentityMetadata {
        schema_version: "identity-note-v1".into(),
        stable_id: assignment
            .map(|a| a.stable_id.clone())
            .unwrap_or_else(|| note.id.clone()),
        recipe_id: assignment
            .map(|a| a.recipe_id.clone())
            .unwrap_or_else(|| "product.explicit-or-normalized.v1".into()),
        canonical_payload_hash: assignment.and_then(|a| a.canonical_payload_hash.clone()),
        current_guid_candidate: assignment
            .map(|a| a.current_guid_candidate.clone())
            .unwrap_or_else(|| note.id.clone()),
        selected_anki_guid: selected,
        guid_derivation_version: assignment
            .map(|a| a.guid_derivation_version.clone())
            .unwrap_or_else(|| "guid.raw-stable-id.v1".into()),
        guid_source: source,
        recovery_method: "current_resolution".into(),
        provenance: assignment
            .map(|a| a.provenance.clone())
            .unwrap_or_else(|| "ExplicitStableId".into()),
        used_override: assignment.map(|a| a.used_override).unwrap_or(false),
    }
}

fn merge_identity_note_data(
    existing: &str,
    metadata: &NoteIdentityMetadata,
) -> anyhow::Result<String> {
    let mut value = if existing.trim().is_empty() {
        serde_json::json!({})
    } else {
        serde_json::from_str(existing).map_err(|err| {
            anyhow::anyhow!("UPDATE.NOTE_DATA_METADATA_UNMERGEABLE: invalid notes.data JSON: {err}")
        })?
    };

    let Some(object) = value.as_object_mut() else {
        anyhow::bail!("UPDATE.NOTE_DATA_METADATA_UNMERGEABLE: notes.data must be a JSON object");
    };

    object.insert(
        "anki_forge_identity".into(),
        serde_json::to_value(metadata).expect("identity metadata serializes"),
    );
    Ok(serde_json::to_string(&value).expect("identity note data serializes"))
}

pub fn emit_apkg(
    materialized: &MaterializedStaging,
    artifact_target: &BuildArtifactTarget,
    guid_plan: Option<&WriterGuidPlan>,
) -> Result<ApkgMaterialization> {
    let normalized_ir = load_normalized_ir_from_staging_manifest(&materialized.manifest_path)?;

    let guid_assignments = validate_guid_plan(&normalized_ir, guid_plan)?;

    fs::create_dir_all(&artifact_target.root_dir).with_context(|| {
        format!(
            "create artifact root {}",
            artifact_target.root_dir.display()
        )
    })?;

    let apkg_path = artifact_target.root_dir.join("package.apkg");
    let temp_path = artifact_target.root_dir.join(".package.apkg.tmp");
    let _ = fs::remove_file(&temp_path);

    let file = File::create(&temp_path)
        .with_context(|| format!("create package {}", temp_path.display()))?;
    let mut zip = ZipWriter::new(file);

    write_meta(&mut zip)?;
    let latest_collection = create_latest_collection_bytes(
        &artifact_target.root_dir,
        &normalized_ir,
        &guid_assignments,
    )?;
    write_zstd_stored_entry(&mut zip, "collection.anki21b", &latest_collection)?;
    let legacy_collection = create_legacy_collection_bytes(&artifact_target.root_dir)?;
    write_stored_entry(&mut zip, "collection.anki2", &legacy_collection)?;

    write_media_payloads_and_map(&mut zip, &normalized_ir, &artifact_target.media_store_dir)?;

    zip.finish()?;
    fs::rename(&temp_path, &apkg_path).with_context(|| {
        format!(
            "move package {} into {}",
            temp_path.display(),
            apkg_path.display()
        )
    })?;

    let package_bytes =
        fs::read(&apkg_path).with_context(|| format!("read package {}", apkg_path.display()))?;

    Ok(ApkgMaterialization {
        apkg_ref: package_ref(artifact_target),
        apkg_path,
        package_fingerprint: package_fingerprint(&package_bytes),
    })
}

fn write_meta(zip: &mut ZipWriter<File>) -> Result<()> {
    write_stored_entry(
        zip,
        "meta",
        &PackageMetadata {
            version: latest_package_version(),
        }
        .encode_to_vec(),
    )
}

fn latest_package_version() -> i32 {
    3
}

fn package_ref(target: &BuildArtifactTarget) -> String {
    format!(
        "{}/package.apkg",
        target.stable_ref_prefix.trim_end_matches('/')
    )
}

fn package_fingerprint(bytes: &[u8]) -> String {
    let digest = Sha1::digest(bytes);
    format!("package:{}", hex::encode(digest))
}

fn write_media_payloads_and_map(
    zip: &mut ZipWriter<File>,
    normalized_ir: &NormalizedIr,
    media_store_dir: &Path,
) -> Result<()> {
    let mut entries = Vec::new();
    let objects_by_id = normalized_ir
        .media_objects
        .iter()
        .map(|object| (object.id.as_str(), object))
        .collect::<std::collections::BTreeMap<_, _>>();

    for (index, binding) in normalized_ir.media_bindings.iter().enumerate() {
        let object = objects_by_id
            .get(binding.object_id.as_str())
            .with_context(|| {
                format!(
                    "binding {} references missing object {}",
                    binding.id, binding.object_id
                )
            })?;
        let source =
            crate::writer_core::media::verify_cas_object_streaming(media_store_dir, object)?;
        write_zstd_file_entry(zip, &index.to_string(), &source)?;
        let size = apkg_media_size(object.size_bytes, &object.id)?;
        entries.push(MediaEntry {
            name: binding.export_filename.clone(),
            size,
            sha1: hex::decode(&object.sha1)
                .with_context(|| format!("decode sha1 for media object {}", object.id))?,
            legacy_zip_filename: None,
        });
    }

    let media_map = MediaEntries { entries }.encode_to_vec();
    let encoded_media_map =
        zstd::stream::encode_all(media_map.as_slice(), 0).context("compress apkg media map")?;
    write_stored_entry(zip, "media", &encoded_media_map)?;

    Ok(())
}

fn apkg_media_size(size_bytes: u64, object_id: &str) -> Result<u32> {
    u32::try_from(size_bytes).with_context(|| {
        format!("media object {object_id} size {size_bytes} exceeds APKG uint32 media map limit")
    })
}

fn write_stored_entry(zip: &mut ZipWriter<File>, name: &str, bytes: &[u8]) -> Result<()> {
    zip.start_file(
        name,
        FileOptions::<'static, ()>::default().compression_method(CompressionMethod::Stored),
    )?;
    zip.write_all(bytes)?;
    Ok(())
}

fn write_zstd_stored_entry(zip: &mut ZipWriter<File>, name: &str, bytes: &[u8]) -> Result<()> {
    let compressed = zstd::stream::encode_all(bytes, 0)?;
    write_stored_entry(zip, name, &compressed)
}

fn write_zstd_file_entry(zip: &mut ZipWriter<File>, name: &str, path: &Path) -> Result<()> {
    zip.start_file(
        name,
        FileOptions::<'static, ()>::default().compression_method(CompressionMethod::Stored),
    )?;
    let mut input =
        File::open(path).with_context(|| format!("open media object {}", path.display()))?;
    let mut encoder =
        zstd::stream::Encoder::new(zip, 0).context("create zstd encoder for media payload")?;
    std::io::copy(&mut input, &mut encoder)
        .with_context(|| format!("stream-compress media object {}", path.display()))?;
    encoder.finish().context("finish zstd media payload")?;
    Ok(())
}

fn create_latest_collection_bytes(
    root_dir: &Path,
    normalized_ir: &NormalizedIr,
    guid_assignments: &std::collections::BTreeMap<String, WriterGuidAssignment>,
) -> Result<Vec<u8>> {
    let path = root_dir.join(".collection.anki21b.sqlite.tmp");
    let _ = fs::remove_file(&path);
    let conn = Connection::open(&path)
        .with_context(|| format!("open collection database {}", path.display()))?;
    execute_source_schema(&conn, SCHEMA11_SQL)?;
    execute_source_schema(&conn, SCHEMA14_UPGRADE_SQL)?;
    execute_source_schema(&conn, SCHEMA15_UPGRADE_SQL)?;
    execute_schema16_marker(&conn)?;
    execute_source_schema(&conn, SCHEMA17_UPGRADE_SQL)?;
    execute_source_schema(&conn, SCHEMA18_UPGRADE_SQL)?;
    populate_latest_collection(&conn, normalized_ir, guid_assignments)?;
    conn.execute_batch("VACUUM;")?;
    drop(conn);
    let bytes = fs::read(&path).with_context(|| format!("read collection {}", path.display()))?;
    let _ = fs::remove_file(&path);
    Ok(bytes)
}

fn create_legacy_collection_bytes(root_dir: &Path) -> Result<Vec<u8>> {
    let path = root_dir.join(".collection.anki2.sqlite.tmp");
    let _ = fs::remove_file(&path);
    let conn = Connection::open(&path)
        .with_context(|| format!("open legacy collection database {}", path.display()))?;
    execute_source_schema(&conn, SCHEMA11_SQL)?;
    populate_legacy_collection(&conn)?;
    conn.execute_batch("VACUUM;")?;
    drop(conn);
    let bytes = fs::read(&path).with_context(|| format!("read collection {}", path.display()))?;
    let _ = fs::remove_file(&path);
    Ok(bytes)
}

fn execute_source_schema(conn: &Connection, sql: &str) -> Result<()> {
    let sql = sql.replace("COLLATE unicase", "");
    conn.execute_batch(&sql)?;
    Ok(())
}

fn execute_schema16_marker(conn: &Connection) -> Result<()> {
    conn.execute_batch("update col set ver = 16;")?;
    Ok(())
}

fn populate_latest_collection(
    conn: &Connection,
    normalized_ir: &NormalizedIr,
    guid_assignments: &std::collections::BTreeMap<String, WriterGuidAssignment>,
) -> Result<()> {
    let default_deck_config_id = 1_i64;
    let deck_ids = resolve_deck_ids(normalized_ir);

    conn.execute(
        "update col set conf = ?, models = ?, decks = ?, dconf = ?, tags = ? where id = 1",
        rusqlite::params!["{}", "{}", "{}", "{}", "{}"],
    )?;
    conn.execute(
        "insert into deck_config (id, name, mtime_secs, usn, config) values (?1, ?2, 0, 0, ?3)",
        rusqlite::params![
            default_deck_config_id,
            "Default",
            default_deck_config_bytes()
        ],
    )?;
    conn.execute(
        "insert into decks (id, name, mtime_secs, usn, common, kind) values (?1, ?2, 0, 0, ?3, ?4)",
        rusqlite::params![
            1_i64,
            "Default",
            default_deck_common_bytes(),
            default_deck_kind_bytes(default_deck_config_id)
        ],
    )?;
    for (deck_name, deck_id) in &deck_ids {
        if deck_name == "Default" {
            continue;
        }
        conn.execute(
            "insert into decks (id, name, mtime_secs, usn, common, kind) values (?1, ?2, 0, 0, ?3, ?4)",
            rusqlite::params![
                deck_id,
                deck_name,
                default_deck_common_bytes(),
                default_deck_kind_bytes(default_deck_config_id)
            ],
        )?;
    }

    let mut notetype_ids = std::collections::BTreeMap::new();
    for (index, notetype) in normalized_ir.notetypes.iter().enumerate() {
        let ntid = (index + 1) as i64;
        notetype_ids.insert(notetype.id.clone(), ntid);
        conn.execute(
            "insert into notetypes (id, name, mtime_secs, usn, config) values (?1, ?2, 0, 0, ?3)",
            rusqlite::params![ntid, notetype.name, encode_notetype_config(notetype)?],
        )?;
        for (field_ord, field) in notetype.fields.iter().enumerate() {
            conn.execute(
                "insert into fields (ntid, ord, name, config) values (?1, ?2, ?3, ?4)",
                rusqlite::params![
                    ntid,
                    field.ord.unwrap_or(field_ord as u32) as i64,
                    field.name,
                    encode_field_config(field)
                ],
            )?;
        }
        for (template_ord, template) in notetype.templates.iter().enumerate() {
            let target_deck_id = resolve_template_target_deck_id(template, &deck_ids, 0_i64);
            conn.execute(
                "insert into templates (ntid, ord, name, mtime_secs, usn, config) values (?1, ?2, ?3, 0, 0, ?4)",
                rusqlite::params![
                    ntid,
                    template.ord.unwrap_or(template_ord as u32) as i64,
                    template.name,
                    encode_template_config(template, target_deck_id)
                ],
            )?;
        }
    }

    let mut note_row_id = 1_i64;
    let mut card_row_id = 1_i64;
    let mut normalized_tags = std::collections::BTreeSet::new();
    for note in &normalized_ir.notes {
        let ntid = notetype_ids
            .get(&note.notetype_id)
            .copied()
            .unwrap_or(1_i64);
        let notetype = normalized_ir
            .notetypes
            .iter()
            .find(|candidate| candidate.id == note.notetype_id)
            .expect("normalized note should reference a known notetype");
        let mut stripped_fields = StrippedNoteFields::new(note);
        let storage = note_storage_values(note, notetype, &mut stripped_fields)?;
        let note_row = note_row_id;
        let guid = guid_assignments
            .get(&note.id)
            .map(|assignment| assignment.selected_anki_guid.as_str())
            .unwrap_or(note.id.as_str());
        conn.execute(
            "insert into notes (id, guid, mid, mod, usn, tags, flds, sfld, csum, flags, data) values (?1, ?2, ?3, ?4, 0, ?5, ?6, ?7, ?8, 0, ?9)",
            rusqlite::params![
                note_row,
                guid,
                ntid,
                storage.mtime_secs,
                note.tags.join(" "),
                storage.flds,
                storage.sfld,
                storage.csum,
                merge_identity_note_data(
                    "{}",
                    &note_identity_metadata_for_assignment(
                        guid_assignments.get(&note.id),
                        note,
                    ),
                )?,
            ],
        )?;
        for tag in &note.tags {
            normalized_tags.insert(tag.clone());
        }
        for planned_card in plan_cards(note, notetype) {
            let template = &notetype.templates[planned_card.template_index];
            let target_deck_id = resolve_card_deck_id(note, template, &deck_ids);
            conn.execute(
                "insert into cards (id, nid, did, ord, mod, usn, type, queue, due, ivl, factor, reps, lapses, left, odue, odid, flags, data) values (?1, ?2, ?3, ?4, 0, 0, 0, 0, ?5, 0, 0, 0, 0, 0, 0, 0, 0, ?6)",
                rusqlite::params![
                    card_row_id,
                    note_row,
                    target_deck_id,
                    planned_card.card_ord as i64,
                    note_row,
                    "{}"
                ],
            )?;
            card_row_id += 1;
        }
        note_row_id += 1;
    }

    for tag in normalized_tags {
        conn.execute(
            "insert into tags (tag, usn, collapsed, config) values (?1, 0, 0, null)",
            rusqlite::params![tag],
        )?;
    }

    Ok(())
}

fn resolve_card_deck_id(
    note: &NormalizedNote,
    template: &crate::authoring_core::NormalizedTemplate,
    deck_ids: &std::collections::BTreeMap<String, i64>,
) -> i64 {
    let deck_name = template
        .target_deck_name
        .as_deref()
        .unwrap_or(note.deck_name.as_str());
    resolve_deck_id(deck_name, deck_ids, 1_i64)
}

fn resolve_template_target_deck_id(
    template: &crate::authoring_core::NormalizedTemplate,
    deck_ids: &std::collections::BTreeMap<String, i64>,
    default_id: i64,
) -> i64 {
    template
        .target_deck_name
        .as_deref()
        .map(|deck_name| resolve_deck_id(deck_name, deck_ids, default_id))
        .unwrap_or(default_id)
}

fn resolve_deck_id(
    deck_name: &str,
    deck_ids: &std::collections::BTreeMap<String, i64>,
    default_id: i64,
) -> i64 {
    deck_ids.get(deck_name).copied().unwrap_or(default_id)
}

fn populate_legacy_collection(conn: &Connection) -> Result<()> {
    let front_text = legacy_dummy_front_text();
    let fields = format!("{front_text}\u{1f}");

    conn.execute(
        "update col set conf = ?, models = ?, decks = ?, dconf = ?, tags = ? where id = 1",
        rusqlite::params!["{}", legacy_basic_models_json()?, "{}", "{}", "{}"],
    )?;
    conn.execute(
        "insert into notes (id, guid, mid, mod, usn, tags, flds, sfld, csum, flags, data) values (1, 'legacy-dummy', 1, 0, 0, '', ?1, ?2, 0, 0, '{}')",
        rusqlite::params![fields, front_text],
    )?;
    conn.execute(
        "insert into cards (id, nid, did, ord, mod, usn, type, queue, due, ivl, factor, reps, lapses, left, odue, odid, flags, data) values (1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, '{}')",
        [],
    )?;
    Ok(())
}

fn legacy_basic_models_json() -> Result<String> {
    let basic = resolve_stock_notetype(&AuthoringNotetype {
        id: "legacy-basic".into(),
        kind: "basic".into(),
        name: Some("Basic".into()),
        original_stock_kind: None,
        original_id: None,
        fields: None,
        templates: None,
        css: None,
        field_metadata: vec![],
    })
    .context("resolve source-grounded basic notetype for legacy dummy collection")?;

    let field_entries: Vec<_> = basic
        .fields
        .iter()
        .enumerate()
        .map(|(ord, field)| {
            serde_json::json!({
                "name": field.name,
                "ord": field.ord.unwrap_or(ord as u32),
                "sticky": false,
                "rtl": false,
                "font": "Arial",
                "size": 20
            })
        })
        .collect();
    let template_entries: Vec<_> = basic
        .templates
        .iter()
        .enumerate()
        .map(|(ord, template)| {
            serde_json::json!({
                "name": template.name,
                "ord": ord,
                "qfmt": template.question_format,
                "afmt": template.answer_format,
                "bqfmt": "",
                "bafmt": ""
            })
        })
        .collect();
    let models = serde_json::json!({
        "1": {
            "id": 1,
            "name": basic.name,
            "type": 0,
            "mod": 0,
            "usn": 0,
            "sortf": 0,
            "did": serde_json::Value::Null,
            "tmpls": template_entries,
            "flds": field_entries,
            "css": basic.css,
            "latexPre": "",
            "latexPost": "",
            "latexsvg": false,
            "req": [[0, "all", [0]]],
            "originalStockKind": 0
        }
    });

    serde_json::to_string(&models).context("serialize schema11 legacy models")
}

fn legacy_dummy_front_text() -> &'static str {
    "This package requires a newer version of Anki."
}

struct NoteStorageValues {
    flds: String,
    sfld: String,
    csum: u32,
    mtime_secs: i64,
}

struct StrippedNoteFields<'a> {
    note: &'a NormalizedNote,
    values: std::collections::BTreeMap<String, String>,
}

impl<'a> StrippedNoteFields<'a> {
    fn new(note: &'a NormalizedNote) -> Self {
        Self {
            note,
            values: Default::default(),
        }
    }

    fn get(&mut self, field_name: &str) -> Option<&str> {
        if !self.values.contains_key(field_name) {
            let value = self.note.fields.get(field_name)?;
            self.values.insert(
                field_name.to_string(),
                strip_html_preserving_media_filenames(value),
            );
        }
        self.values.get(field_name).map(String::as_str)
    }
}

fn note_storage_values(
    note: &NormalizedNote,
    notetype: &NormalizedNotetype,
    stripped_fields: &mut StrippedNoteFields<'_>,
) -> Result<NoteStorageValues> {
    let fields = ordered_notetype_fields(notetype);
    let values = ordered_field_values(note, &fields);
    let sort_field_index = fields.iter().position(|field| field.sort).unwrap_or(0);
    let first_field_checksum = fields
        .first()
        .and_then(|field| stripped_fields.get(&field.name))
        .map(field_checksum)
        .unwrap_or_else(|| field_checksum(""));
    let sort_field_stripped = values
        .get(sort_field_index)
        .and_then(|_| {
            fields
                .get(sort_field_index)
                .and_then(|field| stripped_fields.get(&field.name))
        })
        .unwrap_or("")
        .to_string();

    Ok(NoteStorageValues {
        flds: values.join("\u{1f}"),
        sfld: sort_field_stripped,
        csum: first_field_checksum,
        mtime_secs: note.mtime_secs.unwrap_or(1),
    })
}

fn ordered_field_values(
    note: &NormalizedNote,
    fields: &[&crate::authoring_core::NormalizedField],
) -> Vec<String> {
    fields
        .iter()
        .map(|field| note.fields.get(&field.name).cloned().unwrap_or_default())
        .collect()
}

fn ordered_notetype_fields(
    notetype: &NormalizedNotetype,
) -> Vec<&crate::authoring_core::NormalizedField> {
    let mut fields = notetype.fields.iter().enumerate().collect::<Vec<_>>();
    fields.sort_by_key(|(index, field)| (field.ord.unwrap_or(*index as u32), *index));
    fields.into_iter().map(|(_, field)| field).collect()
}

fn field_checksum(text: &str) -> u32 {
    let digest = Sha1::digest(text.as_bytes());
    u32::from_be_bytes(digest[..4].try_into().expect("sha1 digest has four bytes"))
}

pub(crate) fn strip_html_preserving_media_filenames(input: &str) -> String {
    crate::authoring_core::strip_html_preserving_media_filenames(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[derive(Clone, PartialEq, Message)]
    struct UpstreamShapeMediaEntry {
        #[prost(string, tag = "1")]
        name: String,
        #[prost(uint32, tag = "2")]
        size: u32,
        #[prost(bytes, tag = "3")]
        sha1: Vec<u8>,
        #[prost(uint32, optional, tag = "255")]
        legacy_zip_filename: Option<u32>,
    }

    #[test]
    fn media_entry_legacy_zip_filename_uses_upstream_tag_255_uint32() {
        let entry = MediaEntry {
            name: "sample.jpg".into(),
            size: 5,
            sha1: vec![1; 20],
            legacy_zip_filename: Some(7),
        };

        let decoded = UpstreamShapeMediaEntry::decode(entry.encode_to_vec().as_slice()).unwrap();

        assert_eq!(decoded.legacy_zip_filename, Some(7));
    }

    #[test]
    fn apkg_media_size_rejects_values_above_uint32_range() {
        let err = apkg_media_size(u64::from(u32::MAX) + 1, "object:too-large").unwrap_err();

        assert!(
            err.to_string().contains("object:too-large"),
            "error should identify the media object: {err:?}"
        );
    }

    #[test]
    fn apkg_media_size_accepts_uint32_max() {
        assert_eq!(
            apkg_media_size(u64::from(u32::MAX), "object:max").unwrap(),
            u32::MAX
        );
    }

    #[test]
    fn strip_html_preserving_media_filenames_handles_anki_boundary_vectors() {
        let cases = [
            ("plain text", "plain text"),
            ("AT&amp;T&nbsp;ok", "AT&T ok"),
            ("<b>front</b>", "front"),
            (
                "<script>ignored()</script><style>.ignored{}</style><!-- hidden --><b>front</b>",
                "front",
            ),
            ("a<!-- unclosed <b>front</b>", "afront"),
            ("before <b unclosed", "before <b unclosed"),
            (
                r#"<img data-note="1 > 0" src="sample.jpg">"#,
                " sample.jpg ",
            ),
            (
                r#"<img src="sample.jpg" data-note="1 > 0">tail"#,
                " sample.jpg tail",
            ),
            ("<IMG SRC = 'sample&#46;jpg'>", " sample.jpg "),
            ("<img src=sample.jpg>", " sample.jpg "),
            ("<video><source src=clip.webm></video>", " clip.webm "),
            ("<audio><source src='voice.mp3'></audio>", " voice.mp3 "),
            ("<object data=diagram.svg></object>", " diagram.svg "),
            (
                r#"<img src="data:image/png;base64,AAAA">"#,
                " data:image/png;base64,AAAA ",
            ),
            (
                r#"<svg><image href="ignored.png"></image><text>Label</text></svg>"#,
                "Label",
            ),
            (
                r#"<style>.card { background: url(bg.png); }</style>front"#,
                "front",
            ),
            ("{{c1::<b>front</b>}}", "{{c1::front}}"),
            (r#"<img alt="src=ghost.png" src="real.png">"#, " real.png "),
        ];

        for (input, expected) in cases {
            assert_eq!(
                strip_html_preserving_media_filenames(input),
                expected,
                "input: {input:?}"
            );
        }
    }

    #[test]
    fn field_checksum_uses_stripped_first_field_text_for_html_boundaries() {
        let cases = [
            ("AT&amp;T&nbsp;ok", 2_203_148_468),
            (r#"<img data-note="1 > 0" src="sample.jpg">"#, 1_786_670_956),
            ("{{c1::<b>front</b>}}", 2_031_771_444),
        ];

        for (input, expected_checksum) in cases {
            let stripped = strip_html_preserving_media_filenames(input);
            assert_eq!(
                field_checksum(&stripped),
                expected_checksum,
                "input: {input:?}, stripped: {stripped:?}"
            );
        }
    }

    proptest! {
        #[test]
        fn strip_html_preserving_media_filenames_never_panics(input in "\\PC*") {
            let stripped = strip_html_preserving_media_filenames(&input);
            prop_assert!(stripped.is_char_boundary(stripped.len()));
        }

        #[test]
        fn strip_html_preserving_media_filenames_preserves_generated_media_attrs(
            tag in prop::sample::select(vec!["img", "audio", "video", "source", "object"]),
            attr in prop::sample::select(vec!["src", "data"]),
            filename in "[A-Za-z0-9_.-]{1,32}",
            quote in prop::sample::select(vec!["\"", "'", ""]),
            before in "[A-Za-z0-9_-]{0,12}",
            after in "[A-Za-z0-9_ >-]{0,12}",
        ) {
            let html = if quote.is_empty() {
                format!(r#"<{tag} title="{before} >" {attr}={filename} data-note="{after}">"#)
            } else {
                format!(r#"<{tag} title="{before} >" {attr}={quote}{filename}{quote} data-note="{after}">"#)
            };

            prop_assert_eq!(
                strip_html_preserving_media_filenames(&html),
                format!(" {filename} ")
            );
        }
    }
}
