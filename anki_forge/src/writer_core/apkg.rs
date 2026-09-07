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
use crate::writer_core::deck_name::DeckRegistry;
use crate::writer_core::model::{WriterGuidAssignment, WriterGuidPlan};
use crate::writer_core::staging::{
    load_staging_manifest, resolve_deck_registry, staging_notetype_ids, BuildArtifactTarget,
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

type GuidAssignments<'a> = std::collections::BTreeMap<&'a str, &'a WriterGuidAssignment>;

fn validate_guid_plan<'a>(
    normalized_ir: &NormalizedIr,
    guid_plan: Option<&'a WriterGuidPlan>,
) -> anyhow::Result<GuidAssignments<'a>> {
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
        by_note.insert(assignment.normalized_note_id.as_str(), assignment);
    }

    let actual: std::collections::BTreeSet<_> = by_note.keys().copied().collect();
    if expected != actual {
        anyhow::bail!(
            "UPDATE.WRITER_GUID_PLAN_MISMATCH: plan ids {:?} did not match normalized note ids {:?}",
            actual,
            expected
        );
    }

    Ok(by_note)
}

pub fn emit_apkg(
    materialized: &MaterializedStaging,
    artifact_target: &BuildArtifactTarget,
    guid_plan: Option<&WriterGuidPlan>,
) -> Result<ApkgMaterialization> {
    let (normalized_ir, staged_ids) = load_staging_manifest(&materialized.manifest_path)?;
    let guid_assignments = validate_guid_plan(&normalized_ir, guid_plan)?;
    let notetype_ids = staging_notetype_ids(&normalized_ir, staged_ids)?;
    emit_apkg_with_plans(
        &normalized_ir,
        &notetype_ids,
        &guid_assignments,
        artifact_target,
    )
}

pub(crate) fn emit_apkg_from_normalized(
    normalized_ir: &NormalizedIr,
    notetype_ids: &std::collections::BTreeMap<String, i64>,
    artifact_target: &BuildArtifactTarget,
    guid_plan: Option<&WriterGuidPlan>,
) -> Result<ApkgMaterialization> {
    let guid_assignments = validate_guid_plan(normalized_ir, guid_plan)?;
    let notetype_ids =
        crate::writer_core::identity::resolve_notetype_ids(normalized_ir, Some(notetype_ids))?;
    emit_apkg_with_plans(
        normalized_ir,
        &notetype_ids,
        &guid_assignments,
        artifact_target,
    )
}

fn emit_apkg_with_plans(
    normalized_ir: &NormalizedIr,
    notetype_ids: &std::collections::BTreeMap<String, i64>,
    guid_assignments: &GuidAssignments<'_>,
    artifact_target: &BuildArtifactTarget,
) -> Result<ApkgMaterialization> {
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
    let latest_collection = create_latest_collection_file(
        &artifact_target.root_dir,
        normalized_ir,
        guid_assignments,
        notetype_ids,
    )?;
    write_zstd_collection_entry(&mut zip, latest_collection.path())?;
    drop(latest_collection);
    let legacy_collection = create_legacy_collection_bytes(&artifact_target.root_dir)?;
    write_stored_entry(&mut zip, "collection.anki2", &legacy_collection)?;

    write_media_payloads_and_map(&mut zip, normalized_ir, &artifact_target.media_store_dir)?;

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

fn write_zstd_collection_entry(zip: &mut ZipWriter<File>, path: &Path) -> Result<()> {
    let input = File::open(path).with_context(|| format!("read collection {}", path.display()))?;
    zip.start_file(
        "collection.anki21b",
        FileOptions::<'static, ()>::default().compression_method(CompressionMethod::Stored),
    )?;
    // encode_all used this same encoder with an unknown source size. Do not
    // pledge the file length: that changes the frame header and compression.
    zstd::stream::copy_encode(input, zip, 0)
        .with_context(|| format!("compress collection {}", path.display()))
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

fn create_latest_collection_file(
    root_dir: &Path,
    normalized_ir: &NormalizedIr,
    guid_assignments: &GuidAssignments<'_>,
    notetype_ids: &std::collections::BTreeMap<String, i64>,
) -> Result<tempfile::NamedTempFile> {
    let path = root_dir.join(".collection.anki21b.sqlite.tmp");
    let _ = fs::remove_file(&path);
    let conn = Connection::open(&path)
        .with_context(|| format!("open collection database {}", path.display()))?;
    {
        let transaction = conn.unchecked_transaction()?;
        execute_source_schema(&transaction, SCHEMA11_SQL)?;
        execute_source_schema(&transaction, SCHEMA14_UPGRADE_SQL)?;
        execute_source_schema(&transaction, SCHEMA15_UPGRADE_SQL)?;
        execute_schema16_marker(&transaction)?;
        execute_source_schema(&transaction, SCHEMA17_UPGRADE_SQL)?;
        execute_source_schema(&transaction, SCHEMA18_UPGRADE_SQL)?;
        transaction.commit()?;
    }
    populate_latest_collection(&conn, normalized_ir, guid_assignments, notetype_ids)?;

    // Keep compaction without rewriting and journaling the source database.
    // An empty, uniquely reserved file also cleans up on any error path.
    let compacted =
        tempfile::NamedTempFile::new_in(root_dir).context("create compacted collection file")?;
    // Match Connection::open's native Unix filename handling instead of using
    // lossy UTF-8 conversion. Binding the name also preserves quotes in paths.
    #[cfg(unix)]
    let compacted_path = {
        use std::os::unix::ffi::OsStrExt;
        compacted.path().as_os_str().as_bytes()
    };
    #[cfg(not(unix))]
    let compacted_path = compacted
        .path()
        .to_str()
        .context("compacted collection path is not UTF-8")?
        .as_bytes();
    conn.execute(
        "VACUUM INTO ?1",
        [rusqlite::types::ToSqlOutput::Borrowed(
            rusqlite::types::ValueRef::Text(compacted_path),
        )],
    )?;
    drop(conn);
    let _ = fs::remove_file(&path);
    Ok(compacted)
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
    guid_assignments: &GuidAssignments<'_>,
    notetype_ids: &std::collections::BTreeMap<String, i64>,
) -> Result<()> {
    let transaction = conn.unchecked_transaction()?;
    populate_latest_collection_rows(&transaction, normalized_ir, guid_assignments, notetype_ids)?;
    transaction.commit()?;
    Ok(())
}

fn populate_latest_collection_rows(
    conn: &Connection,
    normalized_ir: &NormalizedIr,
    guid_assignments: &GuidAssignments<'_>,
    notetype_ids: &std::collections::BTreeMap<String, i64>,
) -> Result<()> {
    let default_deck_config_id = 1_i64;
    let deck_registry = resolve_deck_registry(normalized_ir);

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
    for deck in deck_registry.rows() {
        if deck.id == 1 {
            continue;
        }
        conn.execute(
            "insert into decks (id, name, mtime_secs, usn, common, kind) values (?1, ?2, 0, 0, ?3, ?4)",
            rusqlite::params![
                deck.id,
                deck.native_name.as_str(),
                default_deck_common_bytes(),
                default_deck_kind_bytes(default_deck_config_id)
            ],
        )?;
    }

    for notetype in &normalized_ir.notetypes {
        let ntid = notetype_ids[&notetype.id];
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
            let target_deck_id = resolve_template_target_deck_id(template, &deck_registry, 0_i64);
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

    let mut prepared_notetypes = std::collections::BTreeMap::new();
    for notetype in &normalized_ir.notetypes {
        prepared_notetypes
            .entry(notetype.id.as_str())
            .or_insert_with(|| PreparedNotetype::new(notetype));
    }
    let mut insert_note = conn.prepare(
        "insert into notes (id, guid, mid, mod, usn, tags, flds, sfld, csum, flags, data) values (?1, ?2, ?3, ?4, 0, ?5, ?6, ?7, ?8, 0, ?9)",
    )?;
    let mut insert_card = conn.prepare(
        "insert into cards (id, nid, did, ord, mod, usn, type, queue, due, ivl, factor, reps, lapses, left, odue, odid, flags, data) values (?1, ?2, ?3, ?4, 0, 0, 0, 0, ?5, 0, 0, 0, 0, 0, 0, 0, 0, ?6)",
    )?;
    let mut note_row_id = 1_i64;
    let mut card_row_id = 1_i64;
    let mut normalized_tags = std::collections::BTreeSet::new();
    for note in &normalized_ir.notes {
        let ntid = notetype_ids
            .get(&note.notetype_id)
            .copied()
            .unwrap_or(1_i64);
        let prepared = prepared_notetypes
            .get(note.notetype_id.as_str())
            .expect("normalized note should reference a known notetype");
        let notetype = prepared.notetype;
        let mut stripped_fields = StrippedNoteFields::new(note);
        let storage = note_storage_values(note, prepared, &mut stripped_fields)?;
        let note_row = note_row_id;
        let guid = guid_assignments
            .get(note.id.as_str())
            .map(|assignment| assignment.selected_anki_guid.as_str())
            .unwrap_or(note.id.as_str());
        insert_note.execute(rusqlite::params![
            note_row,
            guid,
            ntid,
            storage.mtime_secs,
            note.tags.join(" "),
            storage.flds,
            storage.sfld,
            storage.csum,
            super::note_data::fresh_identity_note_data(
                guid_assignments.get(note.id.as_str()).copied(),
                note,
            ),
        ])?;
        for tag in &note.tags {
            normalized_tags.insert(tag.clone());
        }
        for planned_card in plan_cards(note, notetype) {
            let template = &notetype.templates[planned_card.template_index];
            let target_deck_id = resolve_card_deck_id(note, template, &deck_registry);
            insert_card.execute(rusqlite::params![
                card_row_id,
                note_row,
                target_deck_id,
                planned_card.card_ord as i64,
                note_row,
                "{}"
            ])?;
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
    deck_registry: &DeckRegistry,
) -> i64 {
    let deck_name = template
        .target_deck_name
        .as_deref()
        .unwrap_or(note.deck_name.as_str());
    deck_registry.id_for_human_name(deck_name).unwrap_or(1_i64)
}

fn resolve_template_target_deck_id(
    template: &crate::authoring_core::NormalizedTemplate,
    deck_registry: &DeckRegistry,
    default_id: i64,
) -> i64 {
    template
        .target_deck_name
        .as_deref()
        .map(|deck_name| {
            deck_registry
                .id_for_human_name(deck_name)
                .unwrap_or(default_id)
        })
        .unwrap_or(default_id)
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

struct PreparedNotetype<'a> {
    notetype: &'a NormalizedNotetype,
    fields: Vec<&'a crate::authoring_core::NormalizedField>,
    first_field: Option<&'a crate::authoring_core::NormalizedField>,
    sort_field_index: usize,
}

impl<'a> PreparedNotetype<'a> {
    fn new(notetype: &'a NormalizedNotetype) -> Self {
        let fields = ordered_notetype_fields(notetype);
        let first_field = fields.first().copied();
        let sort_field_index = fields.iter().position(|field| field.sort).unwrap_or(0);
        Self {
            notetype,
            fields,
            first_field,
            sort_field_index,
        }
    }
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
    prepared: &PreparedNotetype<'_>,
    stripped_fields: &mut StrippedNoteFields<'_>,
) -> Result<NoteStorageValues> {
    let fields = &prepared.fields;
    let values = ordered_field_values(note, fields);
    let sort_field_index = prepared.sort_field_index;
    let first_field_checksum = prepared
        .first_field
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
        mtime_secs: note
            .mtime_secs
            .unwrap_or(super::note_revision::INITIAL_MTIME_SECS),
    })
}

fn ordered_field_values<'a>(
    note: &'a NormalizedNote,
    fields: &[&crate::authoring_core::NormalizedField],
) -> Vec<&'a str> {
    fields
        .iter()
        .map(|field| {
            note.fields
                .get(&field.name)
                .map(String::as_str)
                .unwrap_or("")
        })
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

    fn assert_streamed_collection_matches_buffered_zip(path: &Path) {
        use std::io::Read;

        let bytes = fs::read(path).unwrap();
        let compressed = zstd::stream::encode_all(bytes.as_slice(), 0).unwrap();
        let root = tempfile::tempdir().unwrap();
        let mut archives = Vec::new();
        for streamed in [false, true] {
            let output = root.path().join(format!("{streamed}.apkg"));
            let mut zip = ZipWriter::new(File::create(&output).unwrap());
            write_meta(&mut zip).unwrap();
            if streamed {
                write_zstd_collection_entry(&mut zip, path).unwrap();
            } else {
                write_stored_entry(&mut zip, "collection.anki21b", &compressed).unwrap();
            }
            write_stored_entry(&mut zip, "collection.anki2", b"legacy placeholder").unwrap();
            write_stored_entry(&mut zip, "media", b"media map").unwrap();
            zip.finish().unwrap();
            archives.push(fs::read(output).unwrap());
        }
        assert!(
            archives[0] == archives[1],
            "buffered/streamed ZIP bytes differ for input length {}",
            bytes.len()
        );

        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(&archives[1])).unwrap();
        let names = (0..archive.len())
            .map(|index| archive.by_index(index).unwrap().name().to_owned())
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            ["meta", "collection.anki21b", "collection.anki2", "media"]
        );
        let mut entry = archive.by_name("collection.anki21b").unwrap();
        assert_eq!(entry.compression(), CompressionMethod::Stored);
        let mut actual = Vec::new();
        entry.read_to_end(&mut actual).unwrap();
        assert_eq!(
            zstd::zstd_safe::get_frame_content_size(&actual).unwrap(),
            zstd::zstd_safe::get_frame_content_size(&compressed).unwrap()
        );
        assert_eq!(zstd::stream::decode_all(actual.as_slice()).unwrap(), bytes);
    }

    #[test]
    fn streamed_collection_preserves_buffered_zip_bytes_across_block_boundaries() {
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("collection.sqlite");
        let mut state = 42_u32;
        let mixed = (0..2 * 1024 * 1024 + 17)
            .map(|index| {
                state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                if index % 4096 < 2048 {
                    0
                } else {
                    (state >> 24) as u8
                }
            })
            .collect::<Vec<_>>();
        for length in [0, 1, 8191, 8192, 8193, 131071, 131072, 131073, mixed.len()] {
            fs::write(&source, &mixed[..length]).unwrap();
            assert_streamed_collection_matches_buffered_zip(&source);
        }
    }

    #[test]
    fn streamed_collection_keeps_compacted_sqlite_bytes_and_removes_its_tempfile() {
        let root = tempfile::tempdir().unwrap();
        let normalized = two_basic_notes();
        let ids = crate::writer_core::identity::resolve_notetype_ids(&normalized, None).unwrap();
        let collection =
            create_latest_collection_file(root.path(), &normalized, &Default::default(), &ids)
                .unwrap();
        let path = collection.path().to_owned();
        assert_streamed_collection_matches_buffered_zip(&path);
        let conn = Connection::open(&path).unwrap();
        assert_eq!(
            conn.query_row("pragma integrity_check", [], |row| row.get::<_, String>(0))
                .unwrap(),
            "ok"
        );
        assert_eq!(
            conn.query_row("pragma freelist_count", [], |row| row.get::<_, i64>(0))
                .unwrap(),
            0
        );
        assert_eq!(
            conn.query_row("select count(*) from notes", [], |row| row.get::<_, i64>(0))
                .unwrap(),
            2
        );
        drop(conn);
        assert!(!root.path().join(".collection.anki21b.sqlite.tmp").exists());
        drop(collection);
        assert!(!path.exists());
    }

    #[test]
    fn failure_after_streaming_collection_keeps_previous_package_and_cleans_compaction() {
        let root = tempfile::tempdir().unwrap();
        let target = BuildArtifactTarget::new(root.path(), "artifacts");
        let output = root.path().join("package.apkg");
        fs::write(&output, b"previous package").unwrap();
        fs::create_dir(root.path().join(".collection.anki2.sqlite.tmp")).unwrap();
        let normalized = two_basic_notes();
        let ids = crate::writer_core::identity::resolve_notetype_ids(&normalized, None).unwrap();

        let error = emit_apkg_from_normalized(&normalized, &ids, &target, None)
            .err()
            .expect("the legacy database path is a directory");

        assert!(error
            .to_string()
            .contains("open legacy collection database"));
        assert_eq!(fs::read(output).unwrap(), b"previous package");
        let mut names = fs::read_dir(root.path())
            .unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .collect::<Vec<_>>();
        names.sort();
        assert_eq!(
            names,
            [
                ".collection.anki2.sqlite.tmp",
                ".package.apkg.tmp",
                "package.apkg"
            ]
        );
    }

    #[test]
    fn streamed_collection_propagates_input_and_output_io_errors() {
        let root = tempfile::tempdir().unwrap();
        let output = root.path().join("package.apkg");
        let mut zip = ZipWriter::new(File::create(&output).unwrap());
        let missing = root.path().join("missing.sqlite");
        let error = write_zstd_collection_entry(&mut zip, &missing).unwrap_err();
        assert_eq!(
            error.downcast_ref::<std::io::Error>().unwrap().kind(),
            std::io::ErrorKind::NotFound
        );
        zip.finish().unwrap();

        let mut read_only_zip = ZipWriter::new(File::open(&output).unwrap());
        assert!(write_zstd_collection_entry(&mut read_only_zip, &output).is_err());
    }

    fn previous_note_storage_values(
        note: &NormalizedNote,
        notetype: &NormalizedNotetype,
    ) -> NoteStorageValues {
        let mut fields = notetype.fields.iter().enumerate().collect::<Vec<_>>();
        fields.sort_by_key(|(index, field)| (field.ord.unwrap_or(*index as u32), *index));
        let fields = fields
            .into_iter()
            .map(|(_, field)| field)
            .collect::<Vec<_>>();
        let values = fields
            .iter()
            .map(|field| note.fields.get(&field.name).cloned().unwrap_or_default())
            .collect::<Vec<_>>();
        let sort_index = fields.iter().position(|field| field.sort).unwrap_or(0);
        let mut stripped = StrippedNoteFields::new(note);
        let csum = fields
            .first()
            .and_then(|field| stripped.get(&field.name))
            .map(field_checksum)
            .unwrap_or_else(|| field_checksum(""));
        let sfld = values
            .get(sort_index)
            .and_then(|_| {
                fields
                    .get(sort_index)
                    .and_then(|field| stripped.get(&field.name))
            })
            .unwrap_or("")
            .to_string();
        NoteStorageValues {
            flds: values.join("\u{1f}"),
            sfld,
            csum,
            mtime_secs: note
                .mtime_secs
                .unwrap_or(super::super::note_revision::INITIAL_MTIME_SECS),
        }
    }

    #[test]
    fn storage_values_keep_field_order_sort_checksum_and_empty_values() {
        let normalized = two_basic_notes();
        let mut notetype = normalized.notetypes[0].clone();
        let mut note = normalized.notes[0].clone();
        for width in [0, 1, 2, 32] {
            for sort_index in [None, Some(0), Some(1), Some(31)] {
                notetype.fields = (0..width)
                    .map(|index| crate::authoring_core::NormalizedField {
                        name: format!("字段 {index}"),
                        // Sparse and equal ordinals retain their input-position tie break.
                        ord: (index % 3 != 0).then_some((width - index) / 2 + 7),
                        config_id: None,
                        tag: None,
                        prevent_deletion: false,
                        sort: sort_index == Some(index),
                    })
                    .collect();
                note.fields = notetype
                    .fields
                    .iter()
                    .enumerate()
                    .filter(|(index, _)| index % 5 != 0)
                    .map(|(index, field)| {
                        let value = if index % 3 == 0 {
                            String::new()
                        } else {
                            format!("<b>中文 &amp; é🦀</b>\"\n\\\u{1f}{index}")
                        };
                        (field.name.clone(), value)
                    })
                    .collect();
                note.mtime_secs = (width % 2 == 0).then_some(123_456);
                let expected = previous_note_storage_values(&note, &notetype);
                let actual = note_storage_values(
                    &note,
                    &PreparedNotetype::new(&notetype),
                    &mut StrippedNoteFields::new(&note),
                )
                .unwrap();
                assert_eq!(actual.flds, expected.flds);
                assert_eq!(actual.sfld, expected.sfld);
                assert_eq!(actual.csum, expected.csum);
                assert_eq!(actual.mtime_secs, expected.mtime_secs);
            }
        }
    }

    #[test]
    fn guid_plan_validation_keeps_assignments_and_exact_mismatch_errors() {
        let normalized = two_basic_notes();
        let mut plan = WriterGuidPlan {
            assignments: normalized
                .notes
                .iter()
                .map(|note| WriterGuidAssignment {
                    normalized_note_id: note.id.clone(),
                    stable_id: format!("稳定:{}", note.id),
                    selected_anki_guid: format!("selected:{}", note.id),
                    current_guid_candidate: format!("current:{}", note.id),
                    guid_derivation_version: "guid.raw-stable-id.v1".into(),
                    recipe_id: "recipe".into(),
                    canonical_payload_hash: Some("payload".into()),
                    provenance: "ExplicitStableId".into(),
                    used_override: true,
                    source: "previous_apkg".into(),
                })
                .collect(),
        };
        let expected = plan
            .assignments
            .iter()
            .map(|assignment| (assignment.normalized_note_id.clone(), assignment.clone()))
            .collect::<std::collections::BTreeMap<_, _>>();
        plan.assignments.reverse();
        assert_eq!(
            serde_json::to_value(validate_guid_plan(&normalized, Some(&plan)).unwrap()).unwrap(),
            serde_json::to_value(expected).unwrap(),
        );
        assert!(validate_guid_plan(&normalized, None).unwrap().is_empty());
        plan.assignments.push(plan.assignments[0].clone());
        assert_eq!(
            validate_guid_plan(&normalized, Some(&plan))
                .unwrap_err()
                .to_string(),
            "UPDATE.WRITER_GUID_PLAN_MISMATCH: duplicate assignment for note-2",
        );
        plan.assignments.truncate(1);
        assert_eq!(
            validate_guid_plan(&normalized, Some(&plan))
                .unwrap_err()
                .to_string(),
            r#"UPDATE.WRITER_GUID_PLAN_MISMATCH: plan ids {"note-2"} did not match normalized note ids {"note-1", "note-2"}"#,
        );
    }

    #[test]
    fn collection_population_rolls_back_after_a_note_and_card_were_written() {
        let root = tempfile::tempdir().unwrap();
        let conn = Connection::open(root.path().join("collection.sqlite")).unwrap();
        execute_source_schema(&conn, SCHEMA11_SQL).unwrap();
        execute_source_schema(&conn, SCHEMA14_UPGRADE_SQL).unwrap();
        execute_source_schema(&conn, SCHEMA15_UPGRADE_SQL).unwrap();
        execute_schema16_marker(&conn).unwrap();
        execute_source_schema(&conn, SCHEMA17_UPGRADE_SQL).unwrap();
        execute_source_schema(&conn, SCHEMA18_UPGRADE_SQL).unwrap();
        conn.execute_batch(
            "create trigger fail_second_note before insert on notes
             when new.id = 2
               and (select count(*) from notes) = 1
               and (select count(*) from cards) = 1
             begin select raise(abort, 'injected failure after note and card'); end;",
        )
        .unwrap();
        let original_conf: String = conn
            .query_row("select conf from col where id = 1", [], |row| row.get(0))
            .unwrap();
        let normalized = two_basic_notes();
        let ids = crate::writer_core::identity::resolve_notetype_ids(&normalized, None).unwrap();

        // Exercise the same transaction boundary used by create_latest_collection_file.
        let error =
            populate_latest_collection(&conn, &normalized, &Default::default(), &ids).unwrap_err();

        assert!(error
            .to_string()
            .contains("injected failure after note and card"));
        assert!(conn.is_autocommit());
        for table in [
            "notes",
            "cards",
            "notetypes",
            "fields",
            "templates",
            "decks",
            "deck_config",
        ] {
            let count: i64 = conn
                .query_row(&format!("select count(*) from {table}"), [], |row| {
                    row.get(0)
                })
                .unwrap();
            assert_eq!(count, 0, "partial writes remain in {table}");
        }
        let conf: String = conn
            .query_row("select conf from col where id = 1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(conf, original_conf);

        // A failed build must not leave the connection in a poisoned transaction.
        conn.execute_batch("drop trigger fail_second_note").unwrap();
        populate_latest_collection(&conn, &normalized, &Default::default(), &ids).unwrap();
        assert!(conn.is_autocommit());
        for table in ["notes", "cards"] {
            let count: i64 = conn
                .query_row(&format!("select count(*) from {table}"), [], |row| {
                    row.get(0)
                })
                .unwrap();
            assert_eq!(count, 2);
        }
    }

    #[test]
    fn typed_writer_rejects_invalid_model_and_guid_plans_before_replacing_output() {
        let root = tempfile::tempdir().unwrap();
        let target = BuildArtifactTarget::new(root.path(), "artifacts");
        let output = root.path().join("package.apkg");
        fs::write(&output, b"previous package").unwrap();
        let normalized = two_basic_notes();
        let empty_ids = Default::default();

        let error = emit_apkg_from_normalized(&normalized, &empty_ids, &target, None)
            .err()
            .expect("empty model plan must fail");
        assert!(error
            .to_string()
            .contains("UPDATE.WRITER_NOTETYPE_ID_PLAN_MISMATCH"));
        let error = emit_apkg_from_normalized(
            &normalized,
            &empty_ids,
            &target,
            Some(&WriterGuidPlan {
                assignments: vec![],
            }),
        )
        .err()
        .expect("empty GUID plan must fail before the model plan");
        assert!(error
            .to_string()
            .contains("UPDATE.WRITER_GUID_PLAN_MISMATCH"));
        assert_eq!(fs::read(output).unwrap(), b"previous package");
    }

    fn two_basic_notes() -> NormalizedIr {
        let notetype = resolve_stock_notetype(&AuthoringNotetype {
            id: "basic".into(),
            kind: "basic".into(),
            name: Some("Basic".into()),
            original_stock_kind: None,
            original_id: None,
            fields: None,
            templates: None,
            css: None,
            field_metadata: vec![],
        })
        .unwrap();
        NormalizedIr {
            kind: "normalized-ir".into(),
            schema_version: "0.1.0".into(),
            document_id: "transaction-test".into(),
            resolved_identity: "document:transaction-test".into(),
            notes: (1..=2)
                .map(|index| NormalizedNote {
                    id: format!("note-{index}"),
                    notetype_id: notetype.id.clone(),
                    deck_name: "Default".into(),
                    fields: std::collections::BTreeMap::from([
                        ("Front".into(), format!("front-{index}")),
                        ("Back".into(), format!("back-{index}")),
                    ]),
                    tags: vec![],
                    mtime_secs: None,
                })
                .collect(),
            notetypes: vec![notetype],
            media_objects: vec![],
            media_bindings: vec![],
            media_references: vec![],
        }
    }

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
