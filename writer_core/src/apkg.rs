use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use authoring_core::stock::resolve_stock_notetype;
use authoring_core::{AuthoringNotetype, NormalizedIr, NormalizedNote, NormalizedNotetype};
use prost::Message;
use rusqlite::Connection;
use sha1::{Digest, Sha1};
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::anki_proto::{
    default_deck_common_bytes, default_deck_config_bytes, default_deck_kind_bytes,
    encode_field_config, encode_notetype_config, encode_template_config,
};
use crate::staging::{
    load_normalized_ir_from_staging_manifest, resolve_template_target_deck_ids,
    BuildArtifactTarget, MaterializedStaging,
};

// The local docs/source/rslib tree is an ignored reference mirror that CI does
// not receive, so writer_core snapshots the exact SQL anchors it needs under
// writer_core/assets/rslib/.
const SCHEMA11_SQL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/rslib/storage/schema11.sql"
));
const SCHEMA14_UPGRADE_SQL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/rslib/storage/upgrades/schema14_upgrade.sql"
));
const SCHEMA15_UPGRADE_SQL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/rslib/storage/upgrades/schema15_upgrade.sql"
));
const SCHEMA18_UPGRADE_SQL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/rslib/storage/upgrades/schema18_upgrade.sql"
));

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
    #[prost(string, optional, tag = "4")]
    legacy_zip_filename: Option<String>,
}

pub fn emit_apkg(
    materialized: &MaterializedStaging,
    artifact_target: &BuildArtifactTarget,
) -> Result<ApkgMaterialization> {
    let normalized_ir = load_normalized_ir_from_staging_manifest(&materialized.manifest_path)?;
    let staging_dir = materialized
        .manifest_path
        .parent()
        .context("materialized staging manifest should live under a staging directory")?;

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
    let latest_collection =
        create_latest_collection_bytes(&artifact_target.root_dir, &normalized_ir)?;
    write_zstd_stored_entry(&mut zip, "collection.anki21b", &latest_collection)?;
    let legacy_collection = create_legacy_collection_bytes(&artifact_target.root_dir)?;
    write_stored_entry(&mut zip, "collection.anki2", &legacy_collection)?;

    write_media_payloads_and_map(&mut zip, &normalized_ir, staging_dir)?;

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
    staging_dir: &Path,
) -> Result<()> {
    let mut entries = Vec::new();
    let media_dir = staging_dir.join("media");

    for (index, media) in normalized_ir.media.iter().enumerate() {
        let payload = read_media_payload(&media_dir, &media.filename)?;
        let sha1 = Sha1::digest(&payload).to_vec();
        let encoded = zstd::stream::encode_all(payload.as_slice(), 0)
            .context("compress media payload for apkg")?;
        write_stored_entry(zip, &index.to_string(), &encoded)?;
        entries.push(MediaEntry {
            name: media.filename.clone(),
            size: payload.len() as u32,
            sha1,
            legacy_zip_filename: None,
        });
    }

    let media_map = MediaEntries { entries }.encode_to_vec();
    let encoded_media_map =
        zstd::stream::encode_all(media_map.as_slice(), 0).context("compress apkg media map")?;
    write_stored_entry(zip, "media", &encoded_media_map)?;

    Ok(())
}

fn read_media_payload(media_dir: &Path, filename: &str) -> Result<Vec<u8>> {
    let path = media_dir.join(filename);
    fs::read(&path).with_context(|| format!("read materialized media {}", path.display()))
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

fn create_latest_collection_bytes(
    root_dir: &Path,
    normalized_ir: &NormalizedIr,
) -> Result<Vec<u8>> {
    let path = root_dir.join(".collection.anki21b.sqlite.tmp");
    let _ = fs::remove_file(&path);
    let conn = Connection::open(&path)
        .with_context(|| format!("open collection database {}", path.display()))?;
    execute_source_schema(&conn, SCHEMA11_SQL)?;
    execute_source_schema(&conn, SCHEMA14_UPGRADE_SQL)?;
    execute_source_schema(&conn, SCHEMA15_UPGRADE_SQL)?;
    execute_source_schema(&conn, SCHEMA18_UPGRADE_SQL)?;
    populate_latest_collection(&conn, normalized_ir)?;
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

fn populate_latest_collection(conn: &Connection, normalized_ir: &NormalizedIr) -> Result<()> {
    let default_deck_config_id = 1_i64;
    let template_target_deck_ids = resolve_template_target_deck_ids(normalized_ir);

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
    for (deck_name, deck_id) in &template_target_deck_ids {
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
            let target_deck_id = template
                .target_deck_name
                .as_ref()
                .and_then(|deck_name| template_target_deck_ids.get(deck_name))
                .copied()
                .unwrap_or(0_i64);
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
        let fields = serialize_fields(note, notetype)?;
        let sfld = note.fields.values().next().cloned().unwrap_or_default();
        let note_row = note_row_id;
        conn.execute(
            "insert into notes (id, guid, mid, mod, usn, tags, flds, sfld, csum, flags, data) values (?1, ?2, ?3, 0, 0, ?4, ?5, ?6, 0, 0, ?7)",
            rusqlite::params![
                note_row,
                note.id,
                ntid,
                note.tags.join(" "),
                fields,
                sfld,
                "{}"
            ],
        )?;
        for tag in &note.tags {
            normalized_tags.insert(tag.clone());
        }
        for (template_ord, template) in notetype.templates.iter().enumerate() {
            let target_deck_id = template
                .target_deck_name
                .as_ref()
                .and_then(|deck_name| template_target_deck_ids.get(deck_name))
                .copied()
                .unwrap_or(1_i64);
            conn.execute(
                "insert into cards (id, nid, did, ord, mod, usn, type, queue, due, ivl, factor, reps, lapses, left, odue, odid, flags, data) values (?1, ?2, ?3, ?4, 0, 0, 0, 0, ?5, 0, 0, 0, 0, 0, 0, 0, 0, ?6)",
                rusqlite::params![
                    note_row * 10 + template_ord as i64,
                    note_row,
                    target_deck_id,
                    template_ord as i64,
                    note_row,
                    "{}"
                ],
            )?;
        }
        note_row_id += 1;
    }

    for tag in normalized_tags {
        conn.execute(
            "insert into tags (tag, usn) values (?1, 0)",
            rusqlite::params![tag],
        )?;
    }

    Ok(())
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

fn serialize_fields(note: &NormalizedNote, notetype: &NormalizedNotetype) -> Result<String> {
    let mut values = Vec::with_capacity(notetype.fields.len());
    let mut ordered_fields = notetype.fields.iter().collect::<Vec<_>>();
    ordered_fields.sort_by_key(|field| field.ord.unwrap_or(u32::MAX));
    for field in ordered_fields {
        values.push(note.fields.get(&field.name).cloned().unwrap_or_default());
    }
    Ok(values.join("\u{1f}"))
}
