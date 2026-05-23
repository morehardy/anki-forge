use anki_forge::build::BuildOptions;
use anki_forge::prelude::*;
use rusqlite::Connection;

#[test]
fn update_safety_project_build_compare_to_preserves_previous_guid() {
    let root = tempfile::tempdir().expect("tempdir");
    let previous = root.path().join("previous.apkg");
    let updated = root.path().join("updated.apkg");

    let mut first = Project::new("Spanish").stable_id("spanish");
    first
        .add_note(Note::basic("hola", "hello").stable_id("es:hola"))
        .expect("add first note");
    first
        .build(BuildOptions::new().output(&previous))
        .expect("first build");

    rewrite_single_note_guid(&previous, "legacy-guid");

    let mut second = Project::new("Spanish").stable_id("spanish");
    second
        .add_note(Note::basic("hola", "hello updated").stable_id("es:hola"))
        .expect("add second note");
    let report = second
        .build(BuildOptions::new().output(&updated).compare_to(&previous))
        .expect("update-safe build");

    assert_eq!(report.update_safety.as_ref().unwrap().notes_preserved, 1);
    let baseline = &report.update_safety.as_ref().unwrap().baseline_sources[0];
    assert_eq!(baseline.source_kind, "previous_apkg");
    assert_eq!(baseline.status, "loaded");
    assert!(baseline.used_for_reconcile);
    assert_eq!(read_single_guid(&updated), "legacy-guid");
}

#[test]
fn strict_compare_to_unreadable_previous_apkg_blocks_writer() {
    let root = tempfile::tempdir().expect("tempdir");
    let previous = root.path().join("missing.apkg");
    let output = root.path().join("updated.apkg");

    let mut project = Project::new("Spanish").stable_id("spanish");
    project
        .add_note(Note::basic("hola", "hello").stable_id("es:hola"))
        .expect("add note");

    let err = project
        .build(BuildOptions::new().output(&output).compare_to(&previous))
        .expect_err("strict compare_to should block on unreadable APKG");

    assert!(err
        .report
        .diagnostic_codes()
        .contains(&"UPDATE.BASELINE_APKG_UNREADABLE".into()));
    assert!(!output.exists());
}

#[test]
fn update_safety_compare_to_previous_apkg_reports_field_rename_by_stable_merge_id() {
    let root = tempfile::tempdir().expect("tempdir");
    let previous = root.path().join("previous.apkg");
    let updated = root.path().join("updated.apkg");

    let mut first = Project::new("Japanese").stable_id("jp-core");
    first
        .add_notetype(vocab_notetype("Expression", "Recognition"))
        .expect("add first notetype");
    first
        .add_note(
            Note::new("jp-vocab")
                .stable_id("jp:taberu")
                .text("expr", "食べる")
                .text("meaning", "to eat"),
        )
        .expect("add first note");
    first
        .build(BuildOptions::new().output(&previous))
        .expect("first build");

    let mut second = Project::new("Japanese").stable_id("jp-core");
    second
        .add_notetype(vocab_notetype("Prompt", "Recognition"))
        .expect("add second notetype");
    second
        .add_note(
            Note::new("jp-vocab")
                .stable_id("jp:taberu")
                .text("expr", "食べる")
                .text("meaning", "to eat"),
        )
        .expect("add second note");
    let report = second
        .build(BuildOptions::new().output(&updated).compare_to(&previous))
        .expect("field rename with stable key should not block");

    assert!(report
        .diagnostic_codes()
        .contains(&"UPDATE.FIELD_RENAMED".into()));
    assert!(!report
        .diagnostic_codes()
        .contains(&"UPDATE.FIELD_MERGE_ID_CHANGED".into()));
}

fn vocab_notetype(expression_name: &str, template_name: &str) -> NoteType {
    NoteType::custom("jp-vocab")
        .field(Field::new(expression_name).key("expr"))
        .field(Field::new("Meaning").key("meaning"))
        .template(
            Template::new(template_name)
                .key("recognition")
                .front(format!("{{{{{expression_name}}}}}"))
                .back("{{Meaning}}")
                .generate_when(GenerationRule::all(["expr"])),
        )
}

fn read_single_guid(path: &std::path::Path) -> String {
    let tmp = tempfile::tempdir().expect("tempdir");
    let collection = read_latest_collection_bytes(path);
    let db_path = tmp.path().join("collection.sqlite");
    std::fs::write(&db_path, collection).expect("write sqlite");
    let conn = Connection::open(db_path).expect("open sqlite");
    conn.query_row("select guid from notes", [], |row| row.get(0))
        .expect("guid")
}

fn rewrite_single_note_guid(path: &std::path::Path, guid: &str) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let file = std::fs::File::open(path).expect("open apkg");
    let mut zip = zip::ZipArchive::new(file).expect("zip");
    let mut entries = std::collections::BTreeMap::<String, Vec<u8>>::new();
    for index in 0..zip.len() {
        let mut entry = zip.by_index(index).expect("entry");
        let mut bytes = Vec::new();
        std::io::Read::read_to_end(&mut entry, &mut bytes).expect("read entry");
        entries.insert(entry.name().to_string(), bytes);
    }

    let compressed = entries
        .get("collection.anki21b")
        .expect("latest collection")
        .clone();
    let decoded = zstd::stream::decode_all(compressed.as_slice()).expect("decode collection");
    let collection = tmp.path().join("collection.sqlite");
    std::fs::write(&collection, decoded).expect("write sqlite");
    let conn = Connection::open(&collection).expect("open sqlite");
    let data: String = conn
        .query_row("select data from notes", [], |row| row.get(0))
        .expect("read notes.data");
    let mut data_json: serde_json::Value =
        serde_json::from_str(&data).unwrap_or_else(|_| serde_json::json!({}));
    data_json["anki_forge_identity"]["selected_anki_guid"] = serde_json::json!(guid);
    conn.execute(
        "update notes set guid = ?1, data = ?2",
        rusqlite::params![
            guid,
            serde_json::to_string(&data_json).expect("serialize data")
        ],
    )
    .expect("update guid and metadata");
    drop(conn);
    let updated = std::fs::read(&collection).expect("read sqlite");
    entries.insert(
        "collection.anki21b".into(),
        zstd::stream::encode_all(updated.as_slice(), 0).expect("encode collection"),
    );

    let output = std::fs::File::create(path).expect("replace apkg");
    let mut writer = zip::ZipWriter::new(output);
    let options: zip::write::FileOptions<'_, ()> =
        zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for (entry, bytes) in entries {
        writer.start_file(entry, options).expect("start file");
        std::io::Write::write_all(&mut writer, &bytes).expect("write entry");
    }
    writer.finish().expect("finish");
}

fn read_latest_collection_bytes(path: &std::path::Path) -> Vec<u8> {
    let file = std::fs::File::open(path).expect("open apkg");
    let mut zip = zip::ZipArchive::new(file).expect("zip");
    let mut entry = zip
        .by_name("collection.anki21b")
        .expect("latest collection");
    let mut compressed = Vec::new();
    std::io::Read::read_to_end(&mut entry, &mut compressed).expect("read collection");
    zstd::stream::decode_all(compressed.as_slice()).expect("decode collection")
}
