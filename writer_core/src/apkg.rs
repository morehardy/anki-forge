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
    load_normalized_ir_from_staging_manifest, resolve_deck_ids, BuildArtifactTarget,
    MaterializedStaging,
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
const SCHEMA17_UPGRADE_SQL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/rslib/storage/upgrades/schema17_upgrade.sql"
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
    #[prost(uint32, optional, tag = "255")]
    legacy_zip_filename: Option<u32>,
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
    execute_schema16_marker(&conn)?;
    execute_source_schema(&conn, SCHEMA17_UPGRADE_SQL)?;
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

fn execute_schema16_marker(conn: &Connection) -> Result<()> {
    conn.execute_batch("update col set ver = 16;")?;
    Ok(())
}

fn populate_latest_collection(conn: &Connection, normalized_ir: &NormalizedIr) -> Result<()> {
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
        let storage = note_storage_values(note, notetype)?;
        let note_row = note_row_id;
        conn.execute(
            "insert into notes (id, guid, mid, mod, usn, tags, flds, sfld, csum, flags, data) values (?1, ?2, ?3, ?4, 0, ?5, ?6, ?7, ?8, 0, ?9)",
            rusqlite::params![
                note_row,
                note.id,
                ntid,
                storage.mtime_secs,
                note.tags.join(" "),
                storage.flds,
                storage.sfld,
                storage.csum,
                "{}"
            ],
        )?;
        for tag in &note.tags {
            normalized_tags.insert(tag.clone());
        }
        for (template_ord, template) in notetype.templates.iter().enumerate() {
            let target_deck_id = resolve_card_deck_id(note, template, &deck_ids);
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
            "insert into tags (tag, usn, collapsed, config) values (?1, 0, 0, null)",
            rusqlite::params![tag],
        )?;
    }

    Ok(())
}

fn resolve_card_deck_id(
    note: &NormalizedNote,
    template: &authoring_core::NormalizedTemplate,
    deck_ids: &std::collections::BTreeMap<String, i64>,
) -> i64 {
    let deck_name = template
        .target_deck_name
        .as_deref()
        .unwrap_or(note.deck_name.as_str());
    resolve_deck_id(deck_name, deck_ids, 1_i64)
}

fn resolve_template_target_deck_id(
    template: &authoring_core::NormalizedTemplate,
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

fn note_storage_values(
    note: &NormalizedNote,
    notetype: &NormalizedNotetype,
) -> Result<NoteStorageValues> {
    let values = ordered_field_values(note, notetype);
    let first_field = values.first().map(String::as_str).unwrap_or("");
    let first_field_stripped = strip_html_preserving_media_filenames(first_field);
    let sort_field = values
        .first()
        .map(|field| strip_html_preserving_media_filenames(field))
        .unwrap_or_default();

    Ok(NoteStorageValues {
        flds: values.join("\u{1f}"),
        sfld: sort_field,
        csum: field_checksum(&first_field_stripped),
        mtime_secs: note.mtime_secs.unwrap_or(1),
    })
}

fn ordered_field_values(note: &NormalizedNote, notetype: &NormalizedNotetype) -> Vec<String> {
    ordered_notetype_fields(notetype)
        .into_iter()
        .map(|field| note.fields.get(&field.name).cloned().unwrap_or_default())
        .collect()
}

fn ordered_notetype_fields(notetype: &NormalizedNotetype) -> Vec<&authoring_core::NormalizedField> {
    let mut fields = notetype.fields.iter().enumerate().collect::<Vec<_>>();
    fields.sort_by_key(|(index, field)| (field.ord.unwrap_or(*index as u32), *index));
    fields.into_iter().map(|(_, field)| field).collect()
}

fn field_checksum(text: &str) -> u32 {
    let digest = Sha1::digest(text.as_bytes());
    u32::from_be_bytes(digest[..4].try_into().expect("sha1 digest has four bytes"))
}

fn strip_html_preserving_media_filenames(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut index = 0;

    while index < input.len() {
        if input[index..].starts_with("<!--") {
            let Some(end) = input[index + 4..].find("-->") else {
                break;
            };
            index += 4 + end + 3;
            continue;
        }

        let ch = input[index..]
            .chars()
            .next()
            .expect("index is within string bounds");
        if ch == '<' {
            let Some(tag_end) = find_html_tag_end(input, index) else {
                break;
            };
            let tag = &input[index..=tag_end];
            if let Some((tag_name, closing)) = html_tag_name(tag) {
                if !closing && is_raw_text_html_tag(tag_name) {
                    index = find_raw_text_html_tag_end(input, tag_end + 1, tag_name)
                        .unwrap_or(input.len());
                    continue;
                }
                if !closing {
                    if let Some(filename) = media_filename_from_tag(tag) {
                        output.push(' ');
                        output.push_str(&filename);
                        output.push(' ');
                    }
                }
            }
            index = tag_end + 1;
        } else {
            output.push(ch);
            index += ch.len_utf8();
        }
    }

    html_escape::decode_html_entities(&output).into_owned()
}

fn find_html_tag_end(input: &str, start: usize) -> Option<usize> {
    let mut quote = None;
    let mut index = start + 1;

    while index < input.len() {
        let ch = input[index..].chars().next()?;
        match quote {
            Some(active_quote) if ch == active_quote => quote = None,
            Some(_) => {}
            None if ch == '"' || ch == '\'' => quote = Some(ch),
            None if ch == '>' => return Some(index),
            None => {}
        }
        index += ch.len_utf8();
    }

    None
}

fn find_raw_text_html_tag_end(input: &str, from: usize, tag_name: &str) -> Option<usize> {
    let closing_prefix = format!("</{}", tag_name.to_ascii_lowercase());
    let mut search_from = from;

    while search_from < input.len() {
        let lower_remaining = input[search_from..].to_ascii_lowercase();
        let Some(relative_start) = lower_remaining.find(&closing_prefix) else {
            break;
        };
        let close_start = search_from + relative_start;
        let Some(close_end) = find_html_tag_end(input, close_start) else {
            break;
        };
        let closing_tag = &input[close_start..=close_end];
        if let Some((closing_name, true)) = html_tag_name(closing_tag) {
            if closing_name.eq_ignore_ascii_case(tag_name) {
                return Some(close_end + 1);
            }
        }
        search_from = close_start + 2;
    }

    None
}

fn html_tag_name(tag: &str) -> Option<(&str, bool)> {
    if !tag.starts_with('<') {
        return None;
    }

    let mut index = skip_html_whitespace(tag, 1);
    let closing = tag[index..].starts_with('/');
    if closing {
        index += 1;
        index = skip_html_whitespace(tag, index);
    }

    let name_start = index;
    while index < tag.len() {
        let ch = tag[index..].chars().next()?;
        if ch.is_whitespace() || matches!(ch, '>' | '/') {
            break;
        }
        index += ch.len_utf8();
    }

    if name_start == index {
        None
    } else {
        Some((&tag[name_start..index], closing))
    }
}

fn media_filename_from_tag(tag: &str) -> Option<String> {
    let Some((tag_name, false)) = html_tag_name(tag) else {
        return None;
    };
    if !is_media_html_tag(tag_name) {
        return None;
    }

    extract_html_attr(tag, "src").or_else(|| extract_html_attr(tag, "data"))
}

fn is_media_html_tag(tag_name: &str) -> bool {
    tag_name.eq_ignore_ascii_case("img")
        || tag_name.eq_ignore_ascii_case("audio")
        || tag_name.eq_ignore_ascii_case("video")
        || tag_name.eq_ignore_ascii_case("source")
        || tag_name.eq_ignore_ascii_case("object")
}

fn is_raw_text_html_tag(tag_name: &str) -> bool {
    tag_name.eq_ignore_ascii_case("script") || tag_name.eq_ignore_ascii_case("style")
}

fn extract_html_attr(tag: &str, attr: &str) -> Option<String> {
    let mut index = 0;
    while index < tag.len() {
        index = skip_html_whitespace(tag, index);
        if index >= tag.len() || tag.as_bytes()[index] == b'>' {
            break;
        }

        let name_start = index;
        while index < tag.len() {
            let ch = tag[index..].chars().next()?;
            if ch.is_whitespace() || matches!(ch, '=' | '>' | '/') {
                break;
            }
            index += ch.len_utf8();
        }
        if name_start == index {
            index += tag[index..].chars().next()?.len_utf8();
            continue;
        }
        let name = &tag[name_start..index];

        index = skip_html_whitespace(tag, index);
        if index >= tag.len() || tag.as_bytes()[index] != b'=' {
            continue;
        }
        index += 1;
        index = skip_html_whitespace(tag, index);
        if index >= tag.len() {
            break;
        }

        let first = tag[index..].chars().next()?;
        let raw = match first {
            '"' | '\'' => {
                let content_start = index + first.len_utf8();
                let end = tag[content_start..].find(first)?;
                index = content_start + end + first.len_utf8();
                &tag[content_start..content_start + end]
            }
            _ => {
                let value_start = index;
                while index < tag.len() {
                    let ch = tag[index..].chars().next()?;
                    if ch.is_whitespace() || ch == '>' {
                        break;
                    }
                    index += ch.len_utf8();
                }
                &tag[value_start..index]
            }
        };

        if name.eq_ignore_ascii_case(attr) {
            return Some(html_escape::decode_html_entities(raw).into_owned());
        }
    }

    None
}

fn skip_html_whitespace(input: &str, mut index: usize) -> usize {
    while index < input.len() {
        let ch = input[index..]
            .chars()
            .next()
            .expect("index is within string bounds");
        if !ch.is_whitespace() {
            break;
        }
        index += ch.len_utf8();
    }
    index
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
