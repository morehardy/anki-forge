# APKG v3 Format SQLite Convergence Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Align the APKG v3 writer with Anki's current package protobuf shape, latest SQLite schema expectations, and note storage semantics for `sfld`, `csum`, and `mod`.

**Architecture:** Keep this pass scoped to `writer_core` plus the shared normalized note model. First lock behavior with failing archive/SQLite tests, then make narrow writer changes: protobuf field tag/type correction, schema17/V18 tag-table convergence, and Anki-like note storage derivation from notetype field order.

**Tech Stack:** Rust, `prost`, `rusqlite` with bundled SQLite, `zip`, `zstd`, `sha1`, existing `html-escape`, Cargo test runner.

---

## Scope Check

This plan covers one subsystem: APKG v3 materialization and inspection in `writer_core`.

Included:

1. `MediaEntries.MediaEntry.legacy_zip_filename` tag/type alignment with Anki's `import_export.proto`.
2. `collection.anki21b` schema construction that reaches a V18-compatible tag table shape through schema17.
3. Note row derivation for ordered `flds`, stripped `sfld`, first-field `csum`, and non-zero/explicit `mod`.
4. Focused regression tests that inspect the APKG zip, protobuf media map, and unpacked SQLite database.

Excluded:

1. Replacing all hand-written storage protobuf structs with generated upstream protobuf types.
2. Running real Anki Desktop import automation in CI.
3. Changing legacy `collection.anki2` beyond keeping it importable as the existing dummy collection lane.
4. Adding user-facing authoring APIs for note modified timestamps.

## File Structure Map

- Modify: `writer_core/src/apkg.rs` - APKG media map proto shape, schema execution path, latest collection population, note storage helpers.
- Modify: `writer_core/src/inspect.rs` - APKG media map decode shape and note mtime extraction for inspected packages.
- Create: `writer_core/assets/rslib/storage/upgrades/schema17_upgrade.sql` - vendored Anki schema17 tag-table migration.
- Modify: `writer_core/tests/build_tests.rs` - protobuf wire, SQLite schema, note storage derivation, and explicit mtime tests.
- Modify: `authoring_core/src/model.rs` - add optional normalized note `mtime_secs` for APKG import update semantics.
- Modify: `authoring_core/src/normalize.rs` - populate `mtime_secs: None` when lowering authoring notes.
- Modify: `contracts/schema/normalized-ir.schema.json` - allow optional `notes[].mtime_secs`.
- Modify: `contracts/semantics/build.md` - document that fallback note mtime is deterministic and explicit `mtime_secs` is required for chronological APKG update behavior.

## Reference Anchors

- Anki package proto: `docs/source/anki/proto/anki/import_export.proto`, `MediaEntry.legacy_zip_filename = optional uint32 tag 255`.
- Anki package metadata: `docs/source/anki/rslib/src/import_export/package/meta.rs`, latest packages use `collection.anki21b`, schema V18, zstd compression.
- Anki export path: `docs/source/anki/rslib/src/import_export/package/colpkg/export.rs`, latest packages write `meta`, latest collection, dummy legacy collection, and media map.
- Anki schema upgrade path: `docs/source/anki/rslib/src/storage/upgrades/mod.rs`, upgrades include schema16 and schema17 before schema18.
- Anki note storage derivation: `docs/source/anki/rslib/src/notes/mod.rs`, `prepare_for_update()` strips field 0 for checksum and uses `sort_field_idx` for `sfld`.

## Implementation Notes

- Keep package fingerprint tests deterministic. Do not use wall-clock time as a fallback `mod`.
- Use `mtime_secs` when present. Use fallback `1` when absent, and document that chronological APKG update behavior requires an explicit timestamp from the caller.
- Keep schema16 as an explicit marker step in this writer because newly-created deck config blobs are already written in schema16-compatible `initial_ease = 2.5` form.
- Keep `legacy_zip_filename` unset on export. The field still needs the correct tag/type so the local decoder and tests match upstream wire shape.
- For `csum`, match Anki's rule: strip HTML while preserving media filenames, SHA1 the stripped field 0 string, and take the first four bytes as big-endian `u32`.

### Task 1: Align MediaEntry Protobuf Wire Shape

**Files:**
- Modify: `writer_core/src/apkg.rs:62-72`
- Modify: `writer_core/src/inspect.rs:48-58`
- Modify: `writer_core/tests/build_tests.rs:1067-1081`

- [ ] **Step 1: Write the failing media map wire-shape tests**

Add this unit test module at the bottom of `writer_core/src/apkg.rs`:

```rust
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
```

Add this helper struct near the existing `TestMediaEntries` definitions in `writer_core/tests/build_tests.rs`:

```rust
#[derive(Clone, PartialEq, Message)]
struct RemovedTag4MediaEntries {
    #[prost(message, repeated, tag = "1")]
    entries: Vec<RemovedTag4MediaEntry>,
}

#[derive(Clone, PartialEq, Message)]
struct RemovedTag4MediaEntry {
    #[prost(string, tag = "1")]
    name: String,
    #[prost(uint32, tag = "2")]
    size: u32,
    #[prost(bytes, tag = "3")]
    sha1: Vec<u8>,
    #[prost(string, optional, tag = "4")]
    legacy_zip_filename: Option<String>,
}
```

Add this test near the existing APKG media tests:

```rust
#[test]
fn exported_apkg_media_entries_do_not_emit_removed_tag4_legacy_filename() {
    let root = unique_artifact_root("media-map-wire-shape");
    let target = BuildArtifactTarget::new(root.clone(), "artifacts/phase3/media-map-wire-shape");

    build(
        &sample_basic_normalized_ir_with_media(),
        &sample_writer_policy(),
        &sample_build_context(true),
        &target,
    )
    .unwrap();

    let mut archive = open_zip(&root.join("package.apkg"));
    let media_map =
        zstd::stream::decode_all(read_zip_entry_bytes(&mut archive, "media").as_slice()).unwrap();
    let decoded = RemovedTag4MediaEntries::decode(media_map.as_slice()).unwrap();

    assert_eq!(decoded.entries.len(), 1);
    assert_eq!(decoded.entries[0].legacy_zip_filename, None);
}
```

Update `TestMediaEntry` in `writer_core/tests/build_tests.rs` to include the upstream-shaped optional field:

```rust
#[derive(Clone, PartialEq, Message)]
struct TestMediaEntry {
    #[prost(string, tag = "1")]
    name: String,
    #[prost(uint32, tag = "2")]
    size: u32,
    #[prost(bytes, tag = "3")]
    sha1: Vec<u8>,
    #[prost(uint32, optional, tag = "255")]
    legacy_zip_filename: Option<u32>,
}
```

- [ ] **Step 2: Run the focused failing tests**

Run: `cargo test -p writer_core apkg::tests::media_entry_legacy_zip_filename_uses_upstream_tag_255_uint32 -v`

Expected: FAIL to compile because the current writer `MediaEntry.legacy_zip_filename` field expects `Option<String>`, not `Option<u32>`.

Run: `cargo test -p writer_core --test build_tests exported_apkg_media_entries_do_not_emit_removed_tag4_legacy_filename -v`

Expected: PASS before implementation if `None` is still omitted, and remain PASS after implementation.

- [ ] **Step 3: Correct writer and inspector protobuf structs**

Change `writer_core/src/apkg.rs`:

```rust
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
```

Change `writer_core/src/inspect.rs`:

```rust
#[derive(Clone, PartialEq, Message)]
struct ArchiveMediaEntry {
    #[prost(string, tag = "1")]
    name: String,
    #[prost(uint32, tag = "2")]
    size: u32,
    #[prost(bytes, tag = "3")]
    sha1: Vec<u8>,
    #[prost(uint32, optional, tag = "255")]
    legacy_zip_filename: Option<u32>,
}
```

The existing export construction in `write_media_payloads_and_map()` remains:

```rust
legacy_zip_filename: None,
```

- [ ] **Step 4: Run the media map tests**

Run: `cargo test -p writer_core apkg::tests::media_entry_legacy_zip_filename_uses_upstream_tag_255_uint32 -v`

Expected: PASS.

Run: `cargo test -p writer_core --test build_tests exported_apkg_media_entries_do_not_emit_removed_tag4_legacy_filename -v`

Expected: PASS.

- [ ] **Step 5: Commit Task 1**

```bash
git add writer_core/src/apkg.rs writer_core/src/inspect.rs writer_core/tests/build_tests.rs
git commit -m "fix: align apkg media entry protobuf shape"
```

### Task 2: Bring Latest Collection Schema To V18-Compatible Tag Shape

**Files:**
- Create: `writer_core/assets/rslib/storage/upgrades/schema17_upgrade.sql`
- Modify: `writer_core/src/apkg.rs:27-42`
- Modify: `writer_core/src/apkg.rs:211-215`
- Modify: `writer_core/src/apkg.rs:365-369`
- Modify: `writer_core/tests/build_tests.rs:251-258`
- Modify: `writer_core/tests/build_tests.rs:985-1058`

- [ ] **Step 1: Write the failing schema17/V18 tests**

Add `schema17_upgrade.sql` to the tracked snapshot test list in `writer_core/tests/build_tests.rs`:

```rust
for relative in [
    "assets/rslib/storage/schema11.sql",
    "assets/rslib/storage/upgrades/schema14_upgrade.sql",
    "assets/rslib/storage/upgrades/schema15_upgrade.sql",
    "assets/rslib/storage/upgrades/schema17_upgrade.sql",
    "assets/rslib/storage/upgrades/schema18_upgrade.sql",
] {
```

Extend `assert_latest_collection_has_required_system_tables()` with these checks after the table name assertions:

```rust
let schema_version: i64 = conn
    .query_row("select ver from col where id = 1", [], |row| row.get(0))
    .unwrap();
assert_eq!(schema_version, 18, "latest collection should advertise schema V18");

let tag_columns: std::collections::BTreeSet<String> = conn
    .prepare("pragma table_info(tags)")
    .unwrap()
    .query_map([], |row| row.get::<_, String>(1))
    .unwrap()
    .map(|row| row.unwrap())
    .collect();
for expected in ["tag", "usn", "collapsed", "config"] {
    assert!(
        tag_columns.contains(expected),
        "schema17 tags table should contain `{expected}`: {tag_columns:?}"
    );
}

let tag_row: (i64, i64, Option<Vec<u8>>) = conn
    .query_row(
        "select usn, collapsed, config from tags where tag = 'demo'",
        [],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )
    .unwrap();
assert_eq!(tag_row, (0, 0, None));
```

- [ ] **Step 2: Run the failing schema tests**

Run: `cargo test -p writer_core --test build_tests tracked_rslib_storage_sql_snapshots_exist -v`

Expected: FAIL because `writer_core/assets/rslib/storage/upgrades/schema17_upgrade.sql` does not exist.

Run: `cargo test -p writer_core --test build_tests build_materializes_image_occlusion_apkg_into_caller_owned_root -v`

Expected: FAIL after the schema assertions are added because the current `tags` table has only `tag` and `usn`.

- [ ] **Step 3: Vendor schema17 and execute schema16/schema17 before schema18**

Create `writer_core/assets/rslib/storage/upgrades/schema17_upgrade.sql` with the current Anki schema17 migration:

```sql
DROP TABLE tags;
CREATE TABLE tags (
  tag text NOT NULL PRIMARY KEY COLLATE unicase,
  usn integer NOT NULL,
  collapsed boolean NOT NULL,
  config blob NULL
) without rowid;
```

Add the schema17 constant in `writer_core/src/apkg.rs`:

```rust
const SCHEMA17_UPGRADE_SQL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/rslib/storage/upgrades/schema17_upgrade.sql"
));
```

Add an explicit schema16 step:

```rust
fn execute_schema16_marker(conn: &Connection) -> Result<()> {
    conn.execute_batch("update col set ver = 16;")?;
    Ok(())
}
```

Update `create_latest_collection_bytes()`:

```rust
execute_source_schema(&conn, SCHEMA11_SQL)?;
execute_source_schema(&conn, SCHEMA14_UPGRADE_SQL)?;
execute_source_schema(&conn, SCHEMA15_UPGRADE_SQL)?;
execute_schema16_marker(&conn)?;
execute_source_schema(&conn, SCHEMA17_UPGRADE_SQL)?;
execute_source_schema(&conn, SCHEMA18_UPGRADE_SQL)?;
populate_latest_collection(&conn, normalized_ir)?;
```

- [ ] **Step 4: Insert tags with schema17 columns**

Change the tag insert in `populate_latest_collection()`:

```rust
for tag in normalized_tags {
    conn.execute(
        "insert into tags (tag, usn, collapsed, config) values (?1, 0, 0, null)",
        rusqlite::params![tag],
    )?;
}
```

- [ ] **Step 5: Run the schema tests**

Run: `cargo test -p writer_core --test build_tests tracked_rslib_storage_sql_snapshots_exist -v`

Expected: PASS.

Run: `cargo test -p writer_core --test build_tests build_materializes_image_occlusion_apkg_into_caller_owned_root -v`

Expected: PASS.

- [ ] **Step 6: Commit Task 2**

```bash
git add writer_core/src/apkg.rs writer_core/assets/rslib/storage/upgrades/schema17_upgrade.sql writer_core/tests/build_tests.rs
git commit -m "fix: build latest collections with schema17 tag shape"
```

### Task 3: Derive Note Storage Fields From Notetype Order

**Files:**
- Modify: `authoring_core/src/model.rs:213-220`
- Modify: `authoring_core/src/normalize.rs:199-205`
- Modify: `writer_core/src/inspect.rs:846-876`
- Modify: `writer_core/src/apkg.rs:317-343`
- Modify: `writer_core/src/apkg.rs:478-486`
- Modify: `contracts/schema/normalized-ir.schema.json`
- Modify: `writer_core/tests/build_tests.rs`

- [ ] **Step 1: Add failing note storage tests**

Add this helper to `writer_core/tests/build_tests.rs`:

```rust
fn latest_collection_from_built_apkg(root: &PathBuf) -> Connection {
    let mut archive = open_zip(&root.join("package.apkg"));
    let latest_collection = zstd::stream::decode_all(
        read_zip_entry_bytes(&mut archive, "collection.anki21b").as_slice(),
    )
    .unwrap();

    let db_root = unique_artifact_root("latest-note-storage-db");
    let db_path = db_root.join("collection.anki21b");
    fs::write(&db_path, latest_collection).unwrap();
    Connection::open(db_path).unwrap()
}
```

Add these tests:

```rust
#[test]
fn latest_collection_derives_sfld_and_csum_from_first_notetype_field() {
    let root = unique_artifact_root("note-storage-first-field");
    let target = BuildArtifactTarget::new(root.clone(), "artifacts/phase3/note-storage-first-field");

    build(
        &sample_basic_normalized_ir(),
        &sample_writer_policy(),
        &sample_build_context(true),
        &target,
    )
    .unwrap();

    let conn = latest_collection_from_built_apkg(&root);
    let row: (String, String, u32) = conn
        .query_row("select flds, cast(sfld as text), csum from notes where guid = 'note-1'", [], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })
        .unwrap();

    assert_eq!(row.0, "front\u{1f}back");
    assert_eq!(row.1, "front");
    assert_eq!(row.2, 460_909_371);
}

#[test]
fn latest_collection_strips_html_when_deriving_sort_field_and_checksum() {
    let root = unique_artifact_root("note-storage-html");
    let target = BuildArtifactTarget::new(root.clone(), "artifacts/phase3/note-storage-html");
    let mut normalized = sample_basic_normalized_ir();
    normalized.notes[0]
        .fields
        .insert("Front".into(), "<b>front</b>".into());

    build(
        &normalized,
        &sample_writer_policy(),
        &sample_build_context(true),
        &target,
    )
    .unwrap();

    let conn = latest_collection_from_built_apkg(&root);
    let row: (String, u32) = conn
        .query_row("select cast(sfld as text), csum from notes where guid = 'note-1'", [], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })
        .unwrap();

    assert_eq!(row.0, "front");
    assert_eq!(row.1, 460_909_371);
}
```

- [ ] **Step 2: Run the failing note storage tests**

Run: `cargo test -p writer_core --test build_tests latest_collection_derives_sfld_and_csum_from_first_notetype_field -v`

Expected: FAIL because `sfld` currently comes from `BTreeMap::values().next()` and `csum` is written as `0`.

Run: `cargo test -p writer_core --test build_tests latest_collection_strips_html_when_deriving_sort_field_and_checksum -v`

Expected: FAIL because `sfld` currently comes from `BTreeMap::values().next()` and `csum` is written as `0`.

- [ ] **Step 3: Add optional normalized note mtime**

Change `authoring_core/src/model.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedNote {
    pub id: String,
    pub notetype_id: String,
    pub deck_name: String,
    pub fields: BTreeMap<String, String>,
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mtime_secs: Option<i64>,
}
```

Change `authoring_core/src/normalize.rs`:

```rust
normalized_notes.push(NormalizedNote {
    id: note.id.clone(),
    notetype_id: note.notetype_id.clone(),
    deck_name: note.deck_name.clone(),
    fields: note.fields.clone(),
    tags: note.tags.clone(),
    mtime_secs: None,
});
```

Change `writer_core/src/inspect.rs` to read and preserve note mtime:

```rust
let mut note_rows =
    conn.prepare("select id, guid, mid, mod, tags, flds from notes order by id")?;
```

Inside the row mapper:

```rust
let _id: i64 = row.get(0)?;
let guid: String = row.get(1)?;
let mid: i64 = row.get(2)?;
let mtime_secs: i64 = row.get(3)?;
let tags: String = row.get(4)?;
let flds: String = row.get(5)?;
```

And construct:

```rust
NormalizedNote {
    id: guid,
    notetype_id: notetype.id.clone(),
    deck_name: "Default".into(),
    fields,
    tags: if tags.is_empty() {
        vec![]
    } else {
        tags.split(' ').map(|tag| tag.to_string()).collect()
    },
    mtime_secs: Some(mtime_secs),
}
```

Update each direct `NormalizedNote` struct initializer returned by `rg -n "NormalizedNote \\{"` to set `mtime_secs: None` unless the test explicitly needs a timestamp.

Add `mtime_secs` to `contracts/schema/normalized-ir.schema.json` under `normalized_note.properties`:

```json
"mtime_secs": {
  "type": "integer",
  "minimum": 1
}
```

- [ ] **Step 4: Implement note storage derivation helpers**

Replace the existing `serialize_fields()` helper in `writer_core/src/apkg.rs` with these helpers:

```rust
struct NoteStorageValues {
    flds: String,
    sfld: String,
    csum: u32,
    mtime_secs: i64,
}

fn note_storage_values(note: &NormalizedNote, notetype: &NormalizedNotetype) -> Result<NoteStorageValues> {
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
```

Add stripping helpers in the same file:

```rust
fn strip_html_preserving_media_filenames(input: &str) -> String {
    let preserved = replace_html_media_tags_with_filenames(input);
    let stripped = strip_html_tags(&preserved);
    html_escape::decode_html_entities(&stripped).into_owned()
}

fn replace_html_media_tags_with_filenames(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut remaining = input;

    while let Some(start) = remaining.find('<') {
        output.push_str(&remaining[..start]);
        let after_lt = &remaining[start..];
        let Some(end) = after_lt.find('>') else {
            output.push_str(after_lt);
            return output;
        };
        let tag = &after_lt[..=end];
        if let Some(filename) = media_filename_from_tag(tag) {
            output.push(' ');
            output.push_str(&filename);
            output.push(' ');
        } else {
            output.push_str(tag);
        }
        remaining = &after_lt[end + 1..];
    }

    output.push_str(remaining);
    output
}

fn media_filename_from_tag(tag: &str) -> Option<String> {
    let lower = tag.to_ascii_lowercase();
    if !(lower.starts_with("<img")
        || lower.starts_with("<audio")
        || lower.starts_with("<video")
        || lower.starts_with("<source")
        || lower.starts_with("<object"))
    {
        return None;
    }

    extract_html_attr(tag, "src").or_else(|| extract_html_attr(tag, "data"))
}

fn extract_html_attr(tag: &str, attr: &str) -> Option<String> {
    let marker = format!("{attr}=");
    let start = tag.find(&marker)?;
    let after = &tag[start + marker.len()..];
    let first = after.chars().next()?;
    let raw = match first {
        '"' | '\'' => {
            let content = &after[first.len_utf8()..];
            let end = content.find(first)?;
            &content[..end]
        }
        _ => {
            let end = after
                .find(|ch: char| ch.is_whitespace() || ch == '>')
                .unwrap_or(after.len());
            &after[..end]
        }
    };

    Some(html_escape::decode_html_entities(raw).into_owned())
}

fn strip_html_tags(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut in_tag = false;

    for ch in input.chars() {
        match ch {
            '<' => in_tag = true,
            '>' if in_tag => in_tag = false,
            _ if !in_tag => output.push(ch),
            _ => {}
        }
    }

    output
}
```

- [ ] **Step 5: Use derived values when inserting notes**

Change note insertion in `populate_latest_collection()`:

```rust
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
```

- [ ] **Step 6: Add explicit mtime regression test**

Add this test:

```rust
#[test]
fn latest_collection_uses_explicit_normalized_note_mtime_when_present() {
    let root = unique_artifact_root("note-storage-mtime");
    let target = BuildArtifactTarget::new(root.clone(), "artifacts/phase3/note-storage-mtime");
    let mut normalized = sample_basic_normalized_ir();
    normalized.notes[0].mtime_secs = Some(1_777_777_777);

    build(
        &normalized,
        &sample_writer_policy(),
        &sample_build_context(true),
        &target,
    )
    .unwrap();

    let conn = latest_collection_from_built_apkg(&root);
    let mtime_secs: i64 = conn
        .query_row("select mod from notes where guid = 'note-1'", [], |row| row.get(0))
        .unwrap();

    assert_eq!(mtime_secs, 1_777_777_777);
}
```

- [ ] **Step 7: Run focused note storage tests**

Run: `cargo test -p writer_core --test build_tests latest_collection_derives_sfld_and_csum_from_first_notetype_field -v`

Expected: PASS.

Run: `cargo test -p writer_core --test build_tests latest_collection_strips_html_when_deriving_sort_field_and_checksum -v`

Expected: PASS.

Run: `cargo test -p writer_core --test build_tests latest_collection_uses_explicit_normalized_note_mtime_when_present -v`

Expected: PASS.

- [ ] **Step 8: Commit Task 3**

```bash
git add authoring_core/src/model.rs authoring_core/src/normalize.rs writer_core/src/apkg.rs writer_core/src/inspect.rs writer_core/tests/build_tests.rs contracts/schema/normalized-ir.schema.json
git commit -m "fix: derive apkg note storage fields from notetype order"
```

### Task 4: Document Remaining Oracle Boundary And Run Full Verification

**Files:**
- Modify: `contracts/semantics/build.md`
- Test: `writer_core/tests/build_tests.rs`
- Test: `writer_core/tests/inspect_tests.rs`
- Test: `authoring_core/tests/normalization_pipeline_tests.rs`

- [ ] **Step 1: Update build semantics documentation**

Add this paragraph to the APKG materialization section in `contracts/semantics/build.md`:

```markdown
For latest APKG output, the writer emits package metadata version 3, a zstd-compressed
`collection.anki21b`, a schema11 dummy `collection.anki2`, zstd-compressed media
payloads, and a zstd-compressed protobuf `MediaEntries` map. The latest collection is
constructed directly as a V18-compatible SQLite database using vendored schema anchors;
it is not a byte-for-byte replay of Anki's full Rust upgrade path. Note rows derive
`flds`, `sfld`, and `csum` from notetype field order. `notes.mod` uses explicit
`notes[].mtime_secs` when supplied and deterministic fallback `1` otherwise; callers
that need chronological APKG reimport updates must supply `mtime_secs`.
```

- [ ] **Step 2: Run focused crate verification**

Run: `cargo test -p writer_core -v`

Expected: PASS.

Run: `cargo test -p authoring_core -v`

Expected: PASS.

- [ ] **Step 3: Run workspace verification**

Run: `cargo test --workspace -v`

Expected: PASS.

- [ ] **Step 4: Commit Task 4**

```bash
git add contracts/semantics/build.md
git commit -m "docs: clarify apkg v3 writer schema and mtime semantics"
```

## Final Verification Checklist

- [ ] `cargo test -p writer_core -v` passes.
- [ ] `cargo test -p authoring_core -v` passes.
- [ ] `cargo test --workspace -v` passes.
- [ ] A generated APKG contains `meta`, `collection.anki21b`, `collection.anki2`, `media`, and numbered media payloads.
- [ ] The decoded media map accepts upstream tag 255 `legacy_zip_filename` and never emits old tag 4.
- [ ] The latest collection has `col.ver = 18`.
- [ ] The latest collection `tags` table has `tag`, `usn`, `collapsed`, and `config`.
- [ ] Basic note `Front/Back` writes `flds = "front\u{1f}back"`, `sfld = "front"`, and `csum = 460909371`.
- [ ] Explicit `NormalizedNote.mtime_secs` is written to `notes.mod`.

## Execution Handoff

Plan execution should happen task-by-task. The safest path is to run Task 1 and Task 2 before touching note model shape, because they are isolated and reduce APKG format risk without changing contract structs.
