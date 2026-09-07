use super::*;
use std::io::Read;

use crate::product::{Note, Project};
use zip::{write::SimpleFileOptions, ZipArchive, ZipWriter};

fn package(path: &Path) {
    let mut project = Project::new("Summary").stable_id("summary");
    for id in ["note", "note-prefix"] {
        project
            .add_note(Note::basic(id, "answer & 中文").stable_id(id))
            .unwrap();
    }
    project.write_apkg(path).unwrap().ensure_success().unwrap();
}

fn entries(path: &Path) -> Vec<(String, Vec<u8>)> {
    let mut archive = ZipArchive::new(fs::File::open(path).unwrap()).unwrap();
    (0..archive.len())
        .map(|index| {
            let mut entry = archive.by_index(index).unwrap();
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes).unwrap();
            (entry.name().to_owned(), bytes)
        })
        .collect()
}

fn replace_entry(path: &Path, name: &str, bytes: Option<Vec<u8>>) {
    let mut entries = entries(path);
    entries.retain(|(entry, _)| entry != name);
    if let Some(bytes) = bytes {
        entries.push((name.to_owned(), bytes));
    }
    let mut archive = ZipWriter::new(fs::File::create(path).unwrap());
    for (name, bytes) in entries {
        archive
            .start_file(name, SimpleFileOptions::default())
            .unwrap();
        archive.write_all(&bytes).unwrap();
    }
    archive.finish().unwrap();
}

fn mutate_collection(path: &Path, sql: &str) {
    mutate_collection_with(path, |conn| conn.execute_batch(sql).unwrap());
}

fn mutate_collection_with(path: &Path, mutate: impl FnOnce(&Connection)) {
    let encoded = entries(path)
        .into_iter()
        .find(|(name, _)| name == "collection.anki21b")
        .unwrap()
        .1;
    let root = tempfile::tempdir().unwrap();
    let collection = root.path().join("collection.sqlite");
    fs::write(
        &collection,
        zstd::stream::decode_all(encoded.as_slice()).unwrap(),
    )
    .unwrap();
    let conn = Connection::open(&collection).unwrap();
    mutate(&conn);
    drop(conn);
    let bytes = fs::read(collection).unwrap();
    replace_entry(
        path,
        "collection.anki21b",
        Some(zstd::stream::encode_all(bytes.as_slice(), 0).unwrap()),
    );
}

fn assert_summary_matches_full(path: &Path, limits: &InspectLimits) -> ApkgInspectSummary {
    let full = inspect_apkg_with_limits(path, limits).expect("complete read");
    let counts = &full.observations.metadata[0];
    let summary = inspect_apkg_summary_with_limits(path, limits).expect("summary read");
    assert_eq!(
        summary,
        ApkgInspectSummary {
            observation_status: full.observation_status,
            notes: counts["note_count"].as_u64().unwrap() as usize,
            cards: counts["card_count"].as_u64().unwrap() as usize,
            notetypes: full.observations.notetypes.len(),
            templates: full.observations.templates.len(),
            fields: full.observations.fields.len(),
            media: full.observations.media.len(),
        }
    );
    summary
}

fn assert_same_error(path: &Path, limits: &InspectLimits) -> InspectError {
    let full = inspect_apkg_with_limits(path, limits).expect_err("full must fail");
    let summary = inspect_apkg_summary_with_limits(path, limits).expect_err("summary must fail");
    assert_eq!(summary, full);
    summary
}

#[test]
fn summary_counts_actual_cards_with_sparse_ordinals_and_prefix_guids() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("cards.apkg");
    package(&path);
    let limits = InspectLimits::default();
    let summary = assert_summary_matches_full(&path, &limits);
    assert_eq!((summary.notes, summary.cards), (2, 2));

    // Empty Fronts produce no cards under the current template plan. Existing
    // cards, including an ordinal with no template, remain actual evidence.
    mutate_collection(
        &path,
        "UPDATE notes SET flds = char(31) || 'answer';
         INSERT INTO cards SELECT id+100, nid, did, 7, mod, usn, type, queue,
             due, ivl, factor, reps, lapses, left, odue, odid, flags, data
             FROM cards WHERE nid = (SELECT id FROM notes WHERE guid = 'note');",
    );
    let summary = assert_summary_matches_full(&path, &limits);
    assert_eq!((summary.notes, summary.cards), (2, 3));
    let report = inspect_apkg(&path).unwrap();
    let card = report
        .observations
        .references
        .iter()
        .find(|entry| entry["selector"] == "card[note_id='note'][ord=7]")
        .unwrap();
    assert_eq!(card["template_name"], "<missing template>");
    assert_eq!(card["deck_name"], "Summary");

    mutate_collection(&path, "DELETE FROM cards;");
    let summary = assert_summary_matches_full(&path, &limits);
    assert_eq!((summary.notes, summary.cards), (2, 0));
    mutate_collection(&path, "DELETE FROM notes;");
    let summary = assert_summary_matches_full(&path, &limits);
    assert_eq!((summary.notes, summary.cards), (0, 0));
}

#[test]
fn summary_counts_mixed_notetypes_and_sparse_cloze_cards() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("cloze.apkg");
    let mut project = Project::new("Mixed summary").stable_id("mixed-summary");
    project
        .add_note(Note::basic("front", "back").stable_id("basic"))
        .unwrap();
    project
        .add_note(Note::cloze("{{c1::one}} and {{c3::three}}").stable_id("cloze"))
        .unwrap();
    project.write_apkg(&path).unwrap().ensure_success().unwrap();
    let summary = assert_summary_matches_full(&path, &InspectLimits::default());
    assert_eq!((summary.notes, summary.cards, summary.notetypes), (2, 3, 2));
}

#[test]
fn summary_preserves_existing_duplicate_guid_and_ordinal_observations() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("duplicate.apkg");
    package(&path);
    mutate_collection(
        &path,
        "UPDATE notes SET guid = 'note';
         INSERT INTO cards SELECT id+100, nid, did, ord, mod, usn, type, queue,
             due, ivl, factor, reps, lapses, left, odue, odid, flags, data
             FROM cards WHERE id = (SELECT min(id) FROM cards);",
    );
    // Full inspection collapses equal (GUID, ord) keys, then observes that key
    // for each matching note. Summary must preserve that existing behavior.
    let summary = assert_summary_matches_full(&path, &InspectLimits::default());
    assert_eq!((summary.notes, summary.cards), (2, 2));
}

#[test]
fn summary_keeps_missing_and_malformed_media_degradation() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("missing.apkg");
    package(&path);
    replace_entry(&path, "media", None);
    let summary = assert_summary_matches_full(&path, &InspectLimits::default());
    assert_eq!(summary.observation_status, "degraded");
    assert_eq!((summary.notes, summary.cards, summary.media), (2, 2, 0));

    replace_entry(&path, "media", Some(b"bad zstd".to_vec()));
    let summary = assert_summary_matches_full(&path, &InspectLimits::default());
    assert_eq!(summary.observation_status, "degraded");
    replace_entry(&path, "collection.anki21b", None);
    let summary = assert_summary_matches_full(&path, &InspectLimits::default());
    assert_eq!(summary.observation_status, "unavailable");
    assert_eq!((summary.notes, summary.cards), (0, 0));
}

#[test]
fn summary_keeps_archive_sqlite_and_protobuf_errors() {
    let root = tempfile::tempdir().unwrap();
    let original = root.path().join("original.apkg");
    package(&original);
    let path = root.path().join("corrupt.apkg");
    for (name, bytes) in [
        ("meta", b"bad protobuf".to_vec()),
        ("collection.anki21b", b"bad zstd".to_vec()),
        (
            "collection.anki21b",
            zstd::stream::encode_all(b"not a database".as_slice(), 0).unwrap(),
        ),
    ] {
        fs::copy(&original, &path).unwrap();
        replace_entry(&path, name, Some(bytes));
        assert!(matches!(
            assert_same_error(&path, &InspectLimits::default()),
            InspectError::Read(_)
        ));
    }
    fs::copy(&original, &path).unwrap();
    mutate_collection(&path, "UPDATE notetypes SET config = X'80';");
    assert_same_error(&path, &InspectLimits::default());
    fs::copy(&original, &path).unwrap();
    mutate_collection(&path, "UPDATE notes SET mid = 999;");
    assert_same_error(&path, &InspectLimits::default());
    fs::write(&path, b"not a ZIP").unwrap();
    assert_same_error(&path, &InspectLimits::default());
}

#[test]
fn summary_decodes_unused_sqlite_columns_and_keeps_error_precedence() {
    let root = tempfile::tempdir().unwrap();
    let original = root.path().join("original.apkg");
    package(&original);
    let path = root.path().join("invalid-column.apkg");
    for sql in [
        "UPDATE notes SET guid = X'80';",
        "UPDATE notes SET mid = 'not an integer';",
        "UPDATE notes SET mod = X'80';",
        "UPDATE notes SET tags = X'80';",
        "UPDATE notes SET flds = X'80';",
        "UPDATE notes SET data = X'80';",
        "UPDATE notes SET tags = CAST(X'80' AS TEXT);",
        "UPDATE notes SET flds = CAST(X'80' AS TEXT);",
        "UPDATE notes SET data = CAST(X'80' AS TEXT);",
        "UPDATE cards SET ord = X'80';",
        "UPDATE decks SET name = X'80' WHERE id = (SELECT min(id) FROM decks);",
        "UPDATE fields SET config = X'80';",
        "UPDATE templates SET config = X'80';",
        // A bad field column must still be decoded before rejecting mid.
        "UPDATE notes SET mid = 999, flds = X'80';",
        // Card decoding precedes note decoding in both projections.
        "UPDATE cards SET ord = X'80'; UPDATE notes SET data = X'80';",
        "CREATE TABLE untyped_notes AS SELECT * FROM notes;
         DROP TABLE notes;
         ALTER TABLE untyped_notes RENAME TO notes;
         UPDATE notes SET data = NULL;",
    ] {
        fs::copy(&original, &path).unwrap();
        mutate_collection(&path, sql);
        assert!(matches!(
            assert_same_error(&path, &InspectLimits::default()),
            InspectError::Read(_)
        ));
    }
}

#[test]
fn summary_preserves_tolerated_note_content_and_identity_json() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("tolerated-content.apkg");
    package(&path);
    for data in [
        "not JSON",
        "null",
        "[]",
        r#"{"anki_forge_identity":null}"#,
        r#"{"anki_forge_identity":{"unknown":[1,2]}}"#,
    ] {
        mutate_collection(
            &path,
            &format!("UPDATE notes SET flds = '', tags = ' repeated  repeated ', data = '{data}';"),
        );
        let summary = assert_summary_matches_full(&path, &InspectLimits::default());
        assert_eq!((summary.notes, summary.cards), (2, 2));
        assert_eq!(summary.observation_status, "complete");
    }
}

#[test]
fn summary_keeps_signed_ordinals_orphan_cards_and_missing_decks() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("unusual-cards.apkg");
    package(&path);
    mutate_collection(
        &path,
        "INSERT INTO cards SELECT id+100, nid, did, -1, mod, usn, type, queue,
             due, ivl, factor, reps, lapses, left, odue, odid, flags, data
             FROM cards WHERE id = (SELECT min(id) FROM cards);
         INSERT INTO cards SELECT id+200, 999, did, ord, mod, usn, type, queue,
             due, ivl, factor, reps, lapses, left, odue, odid, flags, data
             FROM cards WHERE id = (SELECT min(id) FROM cards);
         UPDATE cards SET did = 999;",
    );
    let summary = assert_summary_matches_full(&path, &InspectLimits::default());
    assert_eq!((summary.notes, summary.cards), (2, 3));
}

#[test]
fn summary_keeps_duplicate_notetype_metadata_ids_and_names() {
    let root = tempfile::tempdir().unwrap();
    let original = root.path().join("original-models.apkg");
    let path = root.path().join("duplicate-models.apkg");
    let mut project = Project::new("Duplicate models").stable_id("duplicate-models");
    project
        .add_note(Note::basic("front", "back").stable_id("basic"))
        .unwrap();
    project
        .add_note(Note::cloze("{{c1::one}} and {{c3::three}}").stable_id("cloze"))
        .unwrap();
    project
        .write_apkg(&original)
        .unwrap()
        .ensure_success()
        .unwrap();

    for (duplicate_id, duplicate_name) in [(true, false), (false, true), (true, true)] {
        fs::copy(&original, &path).unwrap();
        mutate_collection_with(&path, |conn| {
            if duplicate_name {
                // Inspection also reads external SQLite files without the
                // writer's uniqueness constraint; names do not identify notes.
                conn.execute_batch(
                    "DROP INDEX idx_notetypes_name;
                     UPDATE notetypes SET name = 'Same model name';",
                )
                .unwrap();
            }
            if duplicate_id {
                let mut statement = conn.prepare("SELECT id, config FROM notetypes").unwrap();
                let configs = statement
                    .query_map([], |row| {
                        Ok((row.get::<_, i64>(0)?, row.get::<_, Vec<u8>>(1)?))
                    })
                    .unwrap()
                    .collect::<rusqlite::Result<Vec<_>>>()
                    .unwrap();
                for (id, bytes) in configs {
                    let mut config = decode_notetype_config(&bytes).unwrap();
                    let mut metadata = decode_notetype_metadata(&config.other).unwrap().unwrap();
                    metadata.anki_forge_notetype_id = "shared-metadata-id".into();
                    config.other = serde_json::to_vec(&metadata).unwrap();
                    conn.execute(
                        "UPDATE notetypes SET config = ?1 WHERE id = ?2",
                        rusqlite::params![config.encode_to_vec(), id],
                    )
                    .unwrap();
                }
            }
        });
        let summary = assert_summary_matches_full(&path, &InspectLimits::default());
        assert_eq!((summary.notes, summary.cards, summary.notetypes), (2, 3, 2));
        assert_eq!((summary.fields, summary.templates), (4, 2));
    }
}

#[cfg(target_pointer_width = "64")]
#[test]
fn summary_distinguishes_negative_and_u32_max_ordinals_before_deduplication() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("wide-ordinals.apkg");
    package(&path);
    mutate_collection(
        &path,
        "INSERT INTO cards SELECT id+100, nid, did, -1, mod, usn, type, queue,
             due, ivl, factor, reps, lapses, left, odue, odid, flags, data
             FROM cards WHERE id = (SELECT min(id) FROM cards);
         INSERT INTO cards SELECT id+200, nid, did, 4294967295, mod, usn, type, queue,
             due, ivl, factor, reps, lapses, left, odue, odid, flags, data
             FROM cards WHERE id = (SELECT min(id) FROM cards);
         INSERT INTO cards SELECT id+300, nid, did, ord, mod, usn, type, queue,
             due, ivl, factor, reps, lapses, left, odue, odid, flags, data
             FROM cards WHERE ord IN (-1, 4294967295);",
    );
    // Each unusual ordinal occurs twice for one note. On 64-bit platforms,
    // -1 converts to usize::MAX, not u32::MAX, so both remain distinct cards.
    let summary = assert_summary_matches_full(&path, &InspectLimits::default());
    assert_eq!((summary.notes, summary.cards), (2, 4));
}

#[test]
fn summary_enforces_identical_resource_limits_and_media_boundaries() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("limits.apkg");
    package(&path);
    let map = MediaEntries {
        entries: vec![ArchiveMediaEntry {
            name: "asset.bin".into(),
            size: 1024,
            sha1: vec![],
            legacy_zip_filename: None,
        }],
    }
    .encode_to_vec();
    replace_entry(
        &path,
        "media",
        Some(zstd::stream::encode_all(map.as_slice(), 0).unwrap()),
    );
    replace_entry(
        &path,
        "0",
        Some(zstd::stream::encode_all(vec![b'a'; 1024].as_slice(), 0).unwrap()),
    );
    let mut limits = InspectLimits {
        max_media_bytes: 1024,
        ..InspectLimits::default()
    };
    let summary = assert_summary_matches_full(&path, &limits);
    assert_eq!(summary.media, 1);
    limits.max_media_bytes = 1023;
    assert_eq!(
        assert_same_error(&path, &limits)
            .limit_exceeded()
            .unwrap()
            .resource,
        "media_bytes"
    );

    for limited in 0..11 {
        let mut limits = InspectLimits::default();
        match limited {
            0 => limits.max_archive_bytes = 0,
            1 => limits.max_entries = 0,
            2 => limits.max_central_directory_bytes = 0,
            3 => limits.max_zip_entry_bytes = 0,
            4 => limits.max_zip_total_bytes = 0,
            5 => limits.max_meta_bytes = 0,
            6 => limits.max_media_map_bytes = 0,
            7 => limits.max_collection_bytes = 0,
            8 => limits.max_media_bytes = 0,
            9 => limits.max_decoded_total_bytes = 0,
            10 => limits.max_zstd_window_bytes = 0,
            _ => unreachable!(),
        }
        assert!(assert_same_error(&path, &limits).limit_exceeded().is_some());
    }
}

#[test]
fn failed_current_summary_never_publishes_even_when_inspect_is_hidden() {
    use crate::build::{BuildOptions, UpdateSafetyMode};
    use crate::diagnostics::Severity;

    let root = tempfile::tempdir().unwrap();
    let output = root.path().join("existing.apkg");
    let mut project = Project::new("Summary limits").stable_id("summary-limits");
    project
        .add_note(Note::basic("front", "back").stable_id("note"))
        .unwrap();
    for mode in [
        UpdateSafetyMode::Disabled,
        UpdateSafetyMode::ReportOnly,
        UpdateSafetyMode::Strict,
    ] {
        for inspect in [false, true] {
            fs::write(&output, b"previous artifact").unwrap();
            let error = project
                .build(
                    BuildOptions::new()
                        .output(&output)
                        .inspect(inspect)
                        .update_safety(mode)
                        .inspect_limits(InspectLimits {
                            max_collection_bytes: 0,
                            ..InspectLimits::default()
                        }),
                )
                .expect_err("a failed current read must block publication");
            assert!(error.report.artifact.is_none());
            assert!(error.report.diagnostics.iter().any(|diagnostic| {
                diagnostic.code.as_str() == "INSPECT.RESOURCE_LIMIT_EXCEEDED"
                    && diagnostic.severity == Severity::Error
            }));
            let comparison_severity = if mode == UpdateSafetyMode::Strict {
                Severity::Error
            } else {
                Severity::Warning
            };
            assert!(error.report.diagnostics.iter().any(|diagnostic| {
                diagnostic.code.as_str() == "COMPARE.CURRENT_UNAVAILABLE"
                    && diagnostic.severity == comparison_severity
            }));
            assert_eq!(fs::read(&output).unwrap(), b"previous artifact");
        }
    }
}
