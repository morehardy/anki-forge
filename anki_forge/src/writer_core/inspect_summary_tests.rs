use super::*;
use std::io::Read;

use crate::{BuildOptions, Note, Project};
use zip::{write::SimpleFileOptions, ZipArchive, ZipWriter};

#[test]
fn collection_hash_tracks_accepted_short_writes_and_preserves_io_failures() {
    struct ShortWriter {
        bytes: Vec<u8>,
        fail_after: usize,
    }
    impl Write for ShortWriter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if self.bytes.len() == self.fail_after {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "collection write failed",
                ));
            }
            let count = bytes.len().min(37).min(self.fail_after - self.bytes.len());
            self.bytes.extend_from_slice(&bytes[..count]);
            Ok(count)
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Err(std::io::Error::other("collection flush failed"))
        }
    }

    for size in [0, 1, 65535, 65536, 65537, 3 * 65536 + 19] {
        let bytes: Vec<u8> = (0..size).map(|index| (index * 31) as u8).collect();
        let mut writer = CollectionWriter::new(
            ShortWriter {
                bytes: Vec::new(),
                fail_after: usize::MAX,
            },
            true,
        );
        for chunk in bytes.chunks(777) {
            writer.write_all(chunk).unwrap();
        }
        assert_eq!(writer.inner.bytes, bytes);
        assert_eq!(writer.digest(), Some(blake3::hash(&bytes)));
    }

    let mut writer = CollectionWriter::new(
        ShortWriter {
            bytes: Vec::new(),
            fail_after: 101,
        },
        true,
    );
    let error = writer.write_all(&[7; 200]).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
    assert_eq!(error.to_string(), "collection write failed");
    assert_eq!(writer.inner.bytes, vec![7; 101]);
    assert_eq!(writer.digest(), Some(blake3::hash(&[7; 101])));
    assert_eq!(
        writer.flush().unwrap_err().to_string(),
        "collection flush failed"
    );

    let mut unneeded = CollectionWriter::new(Vec::new(), false);
    unneeded.write_all(b"ordinary observations").unwrap();
    assert_eq!(unneeded.inner, b"ordinary observations");
    assert_eq!(unneeded.digest(), None);
}

#[test]
fn native_collection_hash_covers_all_frames_with_unchanged_decoded_budgets() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("native-frames.apkg");
    let mut project = Project::new("native-collection-frames").unwrap();
    project
        .add("note", Note::basic("q".repeat(90000), "answer"))
        .unwrap();
    project.build(BuildOptions::to(&path)).unwrap();
    let original = entries(&path);
    let entry = |name: &str| {
        original
            .iter()
            .find(|(candidate, _)| candidate == name)
            .unwrap()
            .1
            .as_slice()
    };
    let sqlite = zstd::decode_all(entry("collection.anki21b")).unwrap();
    let sqlite_path = root.path().join("reference.sqlite");
    fs::write(&sqlite_path, &sqlite).unwrap();
    let expected_digest = crate::build::identity::file_hash(&sqlite_path).unwrap();
    let total = (entry("ankiforge-identity.json").len()
        + entry("meta").len()
        + zstd::decode_all(entry("media")).unwrap().len()
        + sqlite.len()) as u64;
    let (first, second) = sqlite.split_at(sqlite.len() / 2);
    let first = zstd::encode_all(first, 0).unwrap();
    let second = zstd::encode_all(second, 0).unwrap();
    let mut skippable = 0x184d_2a50_u32.to_le_bytes().to_vec();
    skippable.extend_from_slice(&7_u32.to_le_bytes());
    skippable.extend_from_slice(b"skipped");
    for encoded in [
        entry("collection.anki21b").to_vec(),
        [first.as_slice(), second.as_slice()].concat(),
        [first.as_slice(), skippable.as_slice(), second.as_slice()].concat(),
    ] {
        replace_entry(&path, "collection.anki21b", Some(encoded));
        let exact = InspectLimits {
            max_collection_bytes: sqlite.len() as u64,
            max_decoded_total_bytes: total,
            ..InspectLimits::default()
        };
        let (summary, identity) = inspect_native_package(&path, &exact).unwrap();
        assert_eq!((summary.notes, summary.cards), (1, 1));
        assert_eq!(identity.collection_blake3, expected_digest);
        for (limits, resource) in [
            (
                InspectLimits {
                    max_collection_bytes: exact.max_collection_bytes - 1,
                    ..exact.clone()
                },
                "collection_bytes",
            ),
            (
                InspectLimits {
                    max_decoded_total_bytes: total - 1,
                    ..exact.clone()
                },
                "decoded_total_bytes",
            ),
        ] {
            let error = inspect_native_package(&path, &limits).unwrap_err();
            let limit = error.limit_exceeded().unwrap();
            assert_eq!(limit.resource, resource);
            assert_eq!(limit.entry.as_deref(), Some("collection.anki21b"));
        }
    }
}

#[test]
fn native_collection_hash_keeps_archive_and_identity_error_precedence() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("native-priority.apkg");
    let mut project = Project::new("native-collection-priority").unwrap();
    project
        .add("note", Note::basic("question", "answer"))
        .unwrap();
    project.build(BuildOptions::to(&path)).unwrap();
    let original = entries(&path);
    let identity_bytes = &original
        .iter()
        .find(|(name, _)| name == "ankiforge-identity.json")
        .unwrap()
        .1;
    let collection = &original
        .iter()
        .find(|(name, _)| name == "collection.anki21b")
        .unwrap()
        .1;
    let mut identity: Value = serde_json::from_slice(identity_bytes).unwrap();
    identity["format_version"] = "future-format".into();
    identity["collection_blake3"] = "0".repeat(64).into();
    identity["identity_blake3"] = "0".repeat(64).into();
    replace_entry(
        &path,
        "ankiforge-identity.json",
        Some(serde_json::to_vec(&identity).unwrap()),
    );
    let error = inspect_native_package(&path, &InspectLimits::default()).unwrap_err();
    assert!(error.to_string().contains("unsupported identity format"));

    let error = inspect_native_package(
        &path,
        &InspectLimits {
            max_collection_bytes: 0,
            ..InspectLimits::default()
        },
    )
    .unwrap_err();
    assert_eq!(error.limit_exceeded().unwrap().resource, "collection_bytes");

    replace_entry(&path, "collection.anki21b", Some(b"not zstd".to_vec()));
    let error = inspect_native_package(&path, &InspectLimits::default()).unwrap_err();
    assert!(error.to_string().contains("invalid zstd frame magic"));

    replace_entry(&path, "collection.anki21b", Some(collection.clone()));
    identity["format_version"] = "ankiforge-identity-v1".into();
    replace_entry(
        &path,
        "ankiforge-identity.json",
        Some(serde_json::to_vec(&identity).unwrap()),
    );
    let error = inspect_native_package(&path, &InspectLimits::default()).unwrap_err();
    assert!(error
        .to_string()
        .contains("collection digest does not match identity evidence"));
}

#[test]
fn media_hash_streams_bounded_blocks_and_resets_between_files() {
    let (sender, jobs) = std::sync::mpsc::sync_channel(1);
    let (done, receiver) = std::sync::mpsc::sync_channel(1);
    std::thread::scope(|scope| {
        let worker = scope.spawn(move || hash_media_chunks(jobs, done));
        for size in [
            0,
            31,
            HASH_BUFFER_BYTES - 1,
            HASH_BUFFER_BYTES,
            HASH_BUFFER_BYTES + 1,
            6 * HASH_BUFFER_BYTES + 13,
            0,
        ] {
            let bytes: Vec<u8> = (0..size).map(|i| (i * 31) as u8).collect();
            let mut sink = MediaHash::new(Some(&sender));
            // Exercise short writes as well as blocks spanning many decoder reads.
            for chunk in bytes.chunks(777) {
                sink.write_all(chunk).unwrap();
                assert!(sink.pending.capacity() <= HASH_BUFFER_BYTES);
            }
            assert_eq!(sink.finish().unwrap(), None);
            assert_eq!(
                receiver.recv().unwrap(),
                hex::encode(sha1::Sha1::digest(&bytes))
            );
        }
        drop(sender);
        worker.join().unwrap();
    });
}

#[test]
fn serial_media_hash_does_not_allocate_payload_buffers() {
    let bytes = vec![29; 3 * HASH_BUFFER_BYTES + 7];
    let mut sink = MediaHash::new(None);
    for chunk in bytes.chunks(8191) {
        sink.write_all(chunk).unwrap();
        assert_eq!(sink.pending.capacity(), 0);
    }
    assert_eq!(
        sink.finish().unwrap(),
        Some(hex::encode(sha1::Sha1::digest(&bytes)))
    );
}

#[test]
fn interrupted_media_hash_stream_finishes_without_reporting_a_partial_digest() {
    let (sender, jobs) = std::sync::mpsc::sync_channel(1);
    let (done, receiver) = std::sync::mpsc::sync_channel(1);
    std::thread::scope(|scope| {
        let worker = scope.spawn(move || hash_media_chunks(jobs, done));
        let mut sink = MediaHash::new(Some(&sender));
        sink.write_all(&vec![7; 4 * HASH_BUFFER_BYTES + 1]).unwrap();
        drop(sink);
        drop(sender);
        worker.join().unwrap();
        assert_eq!(
            receiver.try_recv(),
            Err(std::sync::mpsc::TryRecvError::Disconnected)
        );
    });
}

fn package(path: &Path) {
    let mut project = Project::new("summary").unwrap().default_deck("Summary");
    for id in ["note", "note-prefix"] {
        project.add(id, Note::basic(id, "answer & 中文")).unwrap();
    }
    project.build(BuildOptions::to(path)).unwrap();
    // Reader tests deliberately use external GUIDs with a shared prefix; native
    // namespace-derived GUIDs are verified by the build/update consumer tests.
    mutate_collection(
        path,
        "UPDATE notes SET guid = substr(flds, 1, instr(flds, char(31)) - 1);",
    );
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
    let mut project = Project::new("mixed-summary").unwrap();
    project.add("basic", Note::basic("front", "back")).unwrap();
    project
        .add("cloze", Note::cloze("{{c1::one}} and {{c3::three}}"))
        .unwrap();
    project.build(BuildOptions::to(&path)).unwrap();
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
    let mut project = Project::new("duplicate-models").unwrap();
    project.add("basic", Note::basic("front", "back")).unwrap();
    project
        .add("cloze", Note::cloze("{{c1::one}} and {{c3::three}}"))
        .unwrap();
    project.build(BuildOptions::to(&original)).unwrap();

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
            sha1: sha1::Sha1::digest(vec![b'a'; 1024]).to_vec(),
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
fn failed_current_summary_never_publishes() {
    let root = tempfile::tempdir().unwrap();
    let output = root.path().join("existing.apkg");
    let mut project = Project::new("summary-limits").unwrap();
    project.add("note", Note::basic("front", "back")).unwrap();
    fs::write(&output, b"previous artifact").unwrap();
    let error = project
        .build(BuildOptions::to(&output).inspect_limits(InspectLimits {
            max_collection_bytes: 0,
            ..InspectLimits::default()
        }))
        .expect_err("a failed current read must block publication");
    assert_eq!(error.kind(), crate::build::BuildErrorKind::ResourceLimit);
    assert_eq!(error.code(), "INSPECT.RESOURCE_LIMIT_EXCEEDED");
    assert!(error.publications().is_empty());
    assert_eq!(fs::read(&output).unwrap(), b"previous artifact");
}
