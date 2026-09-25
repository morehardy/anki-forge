#![allow(dead_code)]

use std::{fs, io::Read, path::Path};

pub fn entries(path: &Path) -> std::collections::BTreeMap<String, Vec<u8>> {
    let mut zip = zip::ZipArchive::new(fs::File::open(path).unwrap()).unwrap();
    (0..zip.len())
        .map(|i| {
            let mut entry = zip.by_index(i).unwrap();
            let name = entry.name().to_owned();
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes).unwrap();
            (name, bytes)
        })
        .collect()
}

pub fn collection(path: &Path) -> (tempfile::TempDir, rusqlite::Connection) {
    let entries = entries(path);
    let root = tempfile::tempdir().unwrap();
    let db_path = root.path().join("collection.sqlite");
    fs::write(
        &db_path,
        zstd::decode_all(entries["collection.anki21b"].as_slice()).unwrap(),
    )
    .unwrap();
    let db = rusqlite::Connection::open(db_path).unwrap();
    (root, db)
}

pub fn fields(path: &Path) -> Vec<String> {
    let (_root, db) = collection(path);
    let mut query = db.prepare("select flds from notes order by flds").unwrap();
    query
        .query_map([], |row| row.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

pub fn evidence(path: &Path) -> serde_json::Value {
    serde_json::from_slice(&entries(path)["ankiforge-identity.json"]).unwrap()
}

pub fn fixture(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/public-api")
        .join(name)
}
