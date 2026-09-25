use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    path::Path,
};

use ankiforge::{update::CompareOptions, BuildOptions, Note, Project};
use contract_tools::{
    contract_manifest_path,
    manifest::{load_manifest, resolve_asset_path},
    schema::load_schema,
};
use serde_json::{json, Value};

fn package_entries(path: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut archive = zip::ZipArchive::new(fs::File::open(path).unwrap()).unwrap();
    (0..archive.len())
        .map(|index| {
            let mut entry = archive.by_index(index).unwrap();
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes).unwrap();
            (entry.name().to_owned(), bytes)
        })
        .collect()
}

#[test]
fn media_history_schema_covers_active_and_omitted_release_descriptors() {
    let manifest = load_manifest(contract_manifest_path()).unwrap();
    let schema =
        load_schema(resolve_asset_path(&manifest, "identity_evidence_schema").unwrap()).unwrap();
    let mut omitted = Project::new("media-history-schema").unwrap();
    omitted.add("one", Note::basic("Front", "Back")).unwrap();
    let mut published = omitted.clone();
    published
        .add_asset(
            ankiforge::Media::bytes(b"content".to_vec(), "application/octet-stream")
                .unwrap()
                .with_export_name("Café.bin")
                .unwrap(),
        )
        .unwrap();
    let first = published.build(BuildOptions::temporary()).unwrap();
    let next = omitted
        .build(BuildOptions::temporary().update_from(first.artifact().path()))
        .unwrap();
    let first: Value = serde_json::from_slice(
        &package_entries(first.artifact().path())["ankiforge-identity.json"],
    )
    .unwrap();
    let next: Value =
        serde_json::from_slice(&package_entries(next.artifact().path())["ankiforge-identity.json"])
            .unwrap();
    assert_eq!(first["identity"]["media_history"], first["media"]);
    assert_eq!(next["identity"]["media_history"], first["media"]);
    assert_eq!(next["media"], json!({}));
    for envelope in [first, next] {
        assert!(schema.is_valid(&envelope));
        for mutation in [
            "missing",
            "empty-filename",
            "negative-size",
            "invalid-digest",
            "unknown-property",
        ] {
            let mut invalid = envelope.clone();
            let history = &mut invalid["identity"]["media_history"];
            match mutation {
                "missing" => {
                    invalid["identity"]
                        .as_object_mut()
                        .unwrap()
                        .remove("media_history");
                }
                "empty-filename" => {
                    let content = history.as_object_mut().unwrap().remove("Café.bin").unwrap();
                    history[""] = content;
                }
                "negative-size" => history["Café.bin"]["size"] = json!(-1),
                "invalid-digest" => history["Café.bin"]["sha1"] = json!("invalid"),
                "unknown-property" => history["Café.bin"]["unknown"] = json!(true),
                _ => unreachable!(),
            }
            assert!(!schema.is_valid(&invalid), "{mutation}");
        }
    }
}

#[test]
fn identity_revision_timestamps_match_schema_and_verified_package_validation() {
    let manifest = load_manifest(contract_manifest_path()).unwrap();
    let schema =
        load_schema(resolve_asset_path(&manifest, "identity_evidence_schema").unwrap()).unwrap();
    let mut project = Project::new("timestamp-parity").unwrap();
    project.add("one", Note::basic("Front", "Back")).unwrap();
    let built = project.build(BuildOptions::temporary()).unwrap();
    let entries = package_entries(built.artifact().path());
    let original: Value = serde_json::from_slice(&entries["ankiforge-identity.json"]).unwrap();
    assert!(schema.is_valid(&original));

    for entity in ["models", "notes"] {
        for timestamp in [1, 0] {
            let root = tempfile::tempdir().unwrap();
            let db_path = root.path().join("collection.sqlite");
            fs::write(
                &db_path,
                zstd::decode_all(entries["collection.anki21b"].as_slice()).unwrap(),
            )
            .unwrap();
            let db = rusqlite::Connection::open(&db_path).unwrap();
            let mut envelope = original.clone();
            let revision = envelope["identity"][entity]
                .as_object_mut()
                .unwrap()
                .values_mut()
                .next()
                .unwrap();
            revision["mtime_secs"] = json!(timestamp);
            if entity == "models" {
                let model_id = revision["id"].as_i64().unwrap();
                assert_eq!(
                    db.execute(
                        "UPDATE notetypes SET mtime_secs = ?1 WHERE id = ?2",
                        [timestamp, model_id]
                    )
                    .unwrap(),
                    1
                );
                db.execute(
                    "UPDATE templates SET mtime_secs = ?1 WHERE ntid = ?2",
                    [timestamp, model_id],
                )
                .unwrap();
            } else {
                assert_eq!(
                    db.execute(
                        "UPDATE notes SET mod = ?1 WHERE guid = ?2",
                        rusqlite::params![timestamp, revision["guid"].as_str().unwrap()]
                    )
                    .unwrap(),
                    1
                );
            }
            drop(db);
            let collection = fs::read(&db_path).unwrap();
            // Refresh both digests and the real SQLite timestamp so runtime
            // failure cannot be attributed to a stale envelope or DB mismatch.
            envelope["collection_blake3"] = json!(blake3::hash(&collection).to_hex().to_string());
            envelope["identity"].sort_all_objects();
            envelope["identity_blake3"] = json!(blake3::hash(
                &serde_json::to_vec(&envelope["identity"]).unwrap()
            )
            .to_hex()
            .to_string());
            let mut changed = entries.clone();
            changed.insert(
                "collection.anki21b".into(),
                zstd::encode_all(collection.as_slice(), 0).unwrap(),
            );
            changed.insert(
                "ankiforge-identity.json".into(),
                serde_json::to_vec(&envelope).unwrap(),
            );
            let package = root.path().join("timestamp.apkg");
            let mut writer = zip::ZipWriter::new(fs::File::create(&package).unwrap());
            for (name, data) in changed {
                writer
                    .start_file(name, zip::write::SimpleFileOptions::default())
                    .unwrap();
                writer.write_all(&data).unwrap();
            }
            writer.finish().unwrap();
            let result = project.compare(CompareOptions::against(&package));
            assert_eq!(
                result.is_ok(),
                timestamp > 0,
                "runtime {entity} timestamp {timestamp}: {result:?}"
            );
            if let Err(error) = result {
                assert_eq!(error.code(), "UPDATE.EVIDENCE_INVALID");
                let cause = if entity == "models" {
                    "invalid model revision"
                } else {
                    "invalid note revision"
                };
                assert!(
                    anyhow::Error::new(error)
                        .chain()
                        .any(|error| error.to_string().contains(cause)),
                    "expected timestamp validation for {entity}"
                );
            }
            assert_eq!(
                schema.is_valid(&envelope),
                timestamp > 0,
                "schema {entity} timestamp {timestamp}"
            );
        }
    }
}
