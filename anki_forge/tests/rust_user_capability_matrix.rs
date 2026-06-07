use std::path::{Path, PathBuf};

use anki_forge::build::{BuildError, BuildReport};
use anki_forge::prelude::*;
use anki_forge::update_safety::lockfile::write_lockfile_atomic;
use anki_forge::update_safety::model::{
    FieldMergeEntry, GeneratedBy, IdentityIndex, IdentityLockfile, NotetypeIdentityEntry,
    TemplateMergeEntry,
};
use anki_forge::writer::{inspect_apkg, InspectReport};
use anki_forge::Deck;
use rusqlite::Connection;
use serde_json::Value;

fn scenario_dir() -> PathBuf {
    let value = std::env::var("ANKI_FORGE_CAPABILITY_ARTIFACT_DIR").expect(
        "set ANKI_FORGE_CAPABILITY_ARTIFACT_DIR by running scripts/run_rust_user_capabilities.sh",
    );
    assert!(
        !value.trim().is_empty(),
        "ANKI_FORGE_CAPABILITY_ARTIFACT_DIR must not be empty; run scripts/run_rust_user_capabilities.sh"
    );
    let path = PathBuf::from(value);
    std::fs::create_dir_all(&path).expect("create scenario artifact dir");
    path
}

fn capability_mode() -> String {
    std::env::var("ANKI_FORGE_CAPABILITY_MODE").unwrap_or_else(|_| "automated".into())
}

fn write_inspect_json(path: &Path, inspect: &InspectReport) {
    let json = serde_json::to_string_pretty(inspect).expect("serialize inspect");
    std::fs::write(path, json).expect("write inspect json");
}

fn write_manual_checklist(
    scenario: &str,
    package: &Path,
    inspect: &InspectReport,
    diagnostics: &[anki_forge::diagnostics::Diagnostic],
) {
    // Keep `scenario` equal to the ignored Rust test function name.
    if capability_mode() != "manual-desktop" {
        return;
    }
    let root = package.parent().expect("package parent");
    let diagnostic_codes = if diagnostics.is_empty() {
        "N/A".to_string()
    } else {
        diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.as_str().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    };
    let body = format!(
        "# Manual Desktop Check: {scenario}\n\n\
- Date:\n\
- Platform:\n\
- Anki version:\n\
- anki-forge commit:\n\
- Package path: {}\n\
- Package SHA-256: ANKI_FORGE_SHA256_PENDING\n\
- Import action: file_import | double_click_apkg\n\
- Notes before import:\n\
- Notes after import:\n\
- Cards before import:\n\
- Cards after import:\n\
- GUID/update result: N/A\n\
- Duplicate note result: N/A\n\
- Media rendering result:\n\
- Media files verified:\n\
- Relevant diagnostics: {diagnostic_codes}\n\
- Pass/fail:\n\
- Notes:\n",
        package.display()
    );
    std::fs::write(root.join("manual-checklist.md"), body).expect("write checklist");
    write_inspect_json(&root.join("apkg.inspect.json"), inspect);
}

fn expect_error_report(result: Result<BuildReport, BuildError>, code: &str) -> BuildReport {
    let error = result.expect_err("scenario should return BuildError");
    assert!(
        error.report.diagnostic_codes().contains(&code.to_string()),
        "missing diagnostic {code}; got {:?}",
        error.report.diagnostic_codes()
    );
    assert!(
        error.report.ensure_success().is_err(),
        "error report must not ensure_success"
    );
    *error.report
}

fn assert_diagnostic_severity(report: &BuildReport, code: &str, severity: anki_forge::Severity) {
    let diagnostic = report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code.as_str() == code)
        .unwrap_or_else(|| panic!("missing diagnostic {code}"));
    assert_eq!(diagnostic.severity, severity);
}

fn inspect_complete(path: &Path) -> InspectReport {
    let report = inspect_apkg(path).expect("inspect generated APKG");
    assert_eq!(report.source_kind, "apkg");
    assert_eq!(report.observation_status, "complete");
    assert!(
        report.missing_domains.is_empty(),
        "inspect missing domains: {:?}",
        report.missing_domains
    );
    assert!(
        report.degradation_reasons.is_empty(),
        "inspect degradation reasons: {:?}",
        report.degradation_reasons
    );
    report
}

fn counts(report: &InspectReport) -> &Value {
    report
        .observations
        .metadata
        .iter()
        .find(|value| value["selector"] == "counts")
        .expect("counts metadata observation")
}

fn has_observation(values: &[Value], key: &str, expected: &str) -> bool {
    values
        .iter()
        .any(|value| value[key].as_str() == Some(expected))
}

fn read_latest_collection_bytes(path: &Path) -> Vec<u8> {
    let file = std::fs::File::open(path).expect("open apkg");
    let mut zip = zip::ZipArchive::new(file).expect("zip");
    let mut entry = zip
        .by_name("collection.anki21b")
        .expect("latest collection");
    let mut compressed = Vec::new();
    std::io::Read::read_to_end(&mut entry, &mut compressed).expect("read collection");
    zstd::stream::decode_all(compressed.as_slice()).expect("decode collection")
}

fn read_single_guid(path: &Path) -> String {
    let tmp = tempfile::tempdir().expect("tempdir");
    let db_path = tmp.path().join("collection.sqlite");
    std::fs::write(&db_path, read_latest_collection_bytes(path)).expect("write sqlite");
    let conn = Connection::open(db_path).expect("open sqlite");
    conn.query_row("select guid from notes", [], |row| row.get(0))
        .expect("guid")
}

fn rewrite_single_note_guid(path: &Path, guid: &str) {
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
    drop(zip);
    let decoded = zstd::stream::decode_all(
        entries
            .get("collection.anki21b")
            .expect("collection")
            .as_slice(),
    )
    .expect("decode collection");
    let collection = tmp.path().join("collection.sqlite");
    std::fs::write(&collection, decoded).expect("write sqlite");
    let conn = Connection::open(&collection).expect("open sqlite");
    let data: String = conn
        .query_row("select data from notes", [], |row| row.get(0))
        .expect("data");
    let mut data_json: serde_json::Value = serde_json::from_str(&data).expect("identity json");
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

fn stable_project(front: &str) -> Project {
    let mut project = Project::new("Update")
        .stable_id("update")
        .default_deck("Update");
    project
        .add_note(Note::basic(front, "back").stable_id("update:one"))
        .expect("add note");
    project
}

fn write_drift_lockfile(
    path: &Path,
    fields: Vec<FieldMergeEntry>,
    templates: Vec<TemplateMergeEntry>,
) {
    let mut index = IdentityIndex::empty_lockfile("jp-core", "writer-policy.default@1.0.0");
    index.notetypes.push(NotetypeIdentityEntry {
        note_type_id: "jp-vocab".into(),
        anki_model_id: None,
        name: "jp-vocab".into(),
        fields,
        templates,
    });
    let lockfile = IdentityLockfile {
        schema_version: "identity-lockfile-v1".into(),
        project_stable_id: "jp-core".into(),
        writer_policy_ref: "writer-policy.default@1.0.0".into(),
        identity_index: index,
        generated_by: GeneratedBy {
            tool: "anki-forge".into(),
            tool_version: env!("CARGO_PKG_VERSION").into(),
            writer_policy_ref: "writer-policy.default@1.0.0".into(),
        },
    };
    write_lockfile_atomic(path, &lockfile).expect("write lockfile");
}

fn write_field_config_drift_lockfile(path: &Path) {
    let current = anki_forge::product::stable_config_id("field", "jp-vocab", "expr");
    write_drift_lockfile(
        path,
        vec![FieldMergeEntry {
            field_key: format!("field:config:{current}"),
            field_name: "Expression".into(),
            ord: 0,
            config_id: current + 1,
            tag: 0,
        }],
        vec![],
    );
}

fn write_template_config_drift_lockfile(path: &Path) {
    let current = anki_forge::product::stable_config_id("template", "jp-vocab", "recognition");
    write_drift_lockfile(
        path,
        vec![],
        vec![TemplateMergeEntry {
            template_key: format!("template:config:{current}"),
            template_name: "Recognition".into(),
            ord: 0,
            config_id: current + 1,
        }],
    );
}

fn drift_project() -> Project {
    let mut project = Project::new("Japanese")
        .stable_id("jp-core")
        .default_deck("Japanese");
    project.add_notetype(vocab_notetype()).expect("notetype");
    project
        .add_note(
            Note::new("jp-vocab")
                .stable_id("jp:taberu")
                .text("expr", "taberu")
                .text("meaning", "to eat"),
        )
        .expect("note");
    project
}

fn two_template_notetype(order: [&str; 2]) -> NoteType {
    let mut notetype = NoteType::custom("jp-vocab")
        .field(Field::new("Expression").key("expr"))
        .field(Field::new("Meaning").key("meaning"));
    for key in order {
        let name = if key == "recognition" {
            "Recognition"
        } else {
            "Production"
        };
        notetype = notetype.template(
            Template::new(name)
                .key(key)
                .front("{{Expression}}")
                .back("{{Meaning}}")
                .generate_when(GenerationRule::all(["expr"])),
        );
    }
    notetype
}

const PNG_1X1: &[u8] = &[
    137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1, 8, 6, 0,
    0, 0, 31, 21, 196, 137, 0, 0, 0, 12, 73, 68, 65, 84, 120, 156, 99, 248, 15, 4, 0, 9, 251, 3,
    253, 167, 102, 129, 94, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
];

const MP3_BYTES: &[u8] = b"fake-mp3-bytes-for-capability-test";

#[ignore]
#[test]
fn duplicate_stable_id() {
    let root = scenario_dir();
    let mut project = Project::new("Duplicate")
        .stable_id("dup")
        .default_deck("Duplicate");
    project
        .add_note(Note::basic("one", "one").stable_id("dup-note"))
        .expect("add first");
    project
        .add_note(Note::basic("two", "two").stable_id("dup-note"))
        .expect("add second");
    let report = expect_error_report(
        project.build(BuildOptions::new().output(root.join("package.apkg"))),
        "AFID.STABLE_ID_DUPLICATE",
    );
    assert_diagnostic_severity(
        &report,
        "AFID.STABLE_ID_DUPLICATE",
        anki_forge::Severity::Error,
    );
}

#[ignore]
#[test]
fn blank_stable_id() {
    let mut deck = Deck::new("Blank");
    let error = deck
        .basic()
        .note("front", "back")
        .stable_id(" ")
        .add()
        .expect_err("blank stable id should fail");
    assert!(error.to_string().contains("DECK.BLANK_STABLE_ID"));
}

#[ignore]
#[test]
fn cloze_inferred_identity_requires_marker() {
    let mut deck = Deck::new("Cloze");
    let error = deck
        .cloze()
        .note("plain text")
        .add()
        .expect_err("markerless inferred cloze should fail");
    assert!(error.to_string().contains("AFID.IDENTITY_COMPONENT_EMPTY"));
}

#[ignore]
#[test]
fn missing_media_source() {
    let root = scenario_dir();
    let source = root.join("source.bin");
    std::fs::write(&source, b"original bytes").expect("write source");
    let mut project = Project::new("Media")
        .stable_id("missing-media")
        .default_deck("Media");
    let media = project
        .media_mut()
        .add_file(&source)
        .expect("file media")
        .export_as("source.bin")
        .expect("export");
    std::fs::remove_file(&source).expect("delete source");
    project
        .add_note(
            Note::basic("source", "")
                .stable_id("media:source")
                .sound("Back", media),
        )
        .expect("add note");
    let report = expect_error_report(
        project.build(BuildOptions::new().output(root.join("package.apkg"))),
        "MEDIA.SOURCE_MISSING",
    );
    assert_diagnostic_severity(&report, "MEDIA.SOURCE_MISSING", anki_forge::Severity::Error);
}

#[ignore]
#[test]
fn missing_media_reference() {
    let root = scenario_dir();
    let mut project = Project::new("Missing Ref")
        .stable_id("missing-ref")
        .default_deck("Missing Ref");
    project
        .add_note(
            Note::new("basic")
                .stable_id("media:missing")
                .text("Front", "front")
                .html("Back", r#"<img src="missing.png">"#),
        )
        .expect("add note");
    let report = expect_error_report(
        project.build(BuildOptions::new().output(root.join("package.apkg"))),
        "MEDIA.MISSING_REFERENCE",
    );
    assert_diagnostic_severity(
        &report,
        "MEDIA.MISSING_REFERENCE",
        anki_forge::Severity::Error,
    );
}

#[ignore]
#[test]
fn unused_media_binding() {
    let root = scenario_dir();
    let mut project = Project::new("Unused Media")
        .stable_id("unused-media")
        .default_deck("Unused Media");
    project
        .media_mut()
        .add_bytes("unused.bin", MP3_BYTES.to_vec())
        .expect("bytes")
        .export_as("unused.mp3")
        .expect("export");
    project
        .add_note(Note::basic("front", "back").stable_id("unused:note"))
        .expect("add note");
    let report = project
        .build(BuildOptions::new().output(root.join("package.apkg")))
        .expect("warning-only build");
    report.ensure_success().expect("unused binding is warning");
    assert_eq!(report.media.unused_bindings, 1);
    assert_diagnostic_severity(
        &report,
        "MEDIA.UNUSED_BINDING",
        anki_forge::Severity::Warning,
    );
    assert!(root.join("package.apkg").is_file());
}

#[ignore]
#[test]
fn unsafe_media_reference() {
    let root = scenario_dir();
    let mut project = Project::new("Unsafe Ref")
        .stable_id("unsafe-ref")
        .default_deck("Unsafe Ref");
    project
        .add_note(
            Note::new("basic")
                .stable_id("unsafe:note")
                .text("Front", "front")
                .html("Back", r#"<img src="bad%2Fname.png">"#),
        )
        .expect("add note");
    let report = expect_error_report(
        project.build(BuildOptions::new().output(root.join("package.apkg"))),
        "MEDIA.UNSAFE_REFERENCE",
    );
    assert_eq!(report.media.unsafe_references, 1);
    assert_diagnostic_severity(
        &report,
        "MEDIA.UNSAFE_REFERENCE",
        anki_forge::Severity::Error,
    );
}

#[ignore]
#[test]
fn unsafe_media_export_filename() {
    let mut project = Project::new("Unsafe Filename");
    let error = project
        .media_mut()
        .add_bytes("raw-image.bin", PNG_1X1.to_vec())
        .expect("bytes")
        .export_as("../chart.png")
        .expect_err("unsafe export filename fails");
    assert!(error.to_string().contains("MEDIA.UNSAFE_FILENAME"));
}

#[ignore]
#[test]
fn mime_mismatch() {
    let root = scenario_dir();
    let mut project = Project::new("Mime").stable_id("mime").default_deck("Mime");
    let media = project
        .media_mut()
        .add_bytes("raw-image.bin", PNG_1X1.to_vec())
        .expect("bytes")
        .export_as("chart.mp3")
        .expect("export");
    project
        .add_note(
            Note::basic("chart", "")
                .stable_id("mime:note")
                .sound("Back", media),
        )
        .expect("add note");
    let report = expect_error_report(
        project.build(BuildOptions::new().output(root.join("package.apkg"))),
        "MEDIA.DECLARED_MIME_MISMATCH",
    );
    assert_diagnostic_severity(
        &report,
        "MEDIA.DECLARED_MIME_MISMATCH",
        anki_forge::Severity::Error,
    );
}

#[ignore]
#[test]
fn baseline_apkg_unreadable() {
    let root = scenario_dir();
    let missing = root.join("missing.apkg");
    let output = root.join("package.apkg");
    let mut project = Project::new("Baseline")
        .stable_id("baseline")
        .default_deck("Baseline");
    project
        .add_note(Note::basic("front", "back").stable_id("baseline:note"))
        .expect("add note");
    let report = expect_error_report(
        project.build(BuildOptions::new().output(&output).compare_to(&missing)),
        "UPDATE.BASELINE_APKG_UNREADABLE",
    );
    assert!(!output.exists());
    assert_diagnostic_severity(
        &report,
        "UPDATE.BASELINE_APKG_UNREADABLE",
        anki_forge::Severity::Error,
    );
}

#[ignore]
#[test]
fn update_preserves_guid() {
    let root = scenario_dir();
    let previous = root.join("previous.apkg");
    let updated = root.join("updated.apkg");
    stable_project("front")
        .build(BuildOptions::new().output(&previous))
        .expect("previous build");
    rewrite_single_note_guid(&previous, "legacy-guid");
    let report = stable_project("front updated")
        .build(BuildOptions::new().output(&updated).compare_to(&previous))
        .expect("updated build");
    report.ensure_success().expect("successful update");
    assert_eq!(
        report
            .update_safety
            .as_ref()
            .expect("update")
            .notes_preserved,
        1
    );
    assert!(report
        .diagnostic_codes()
        .contains(&"UPDATE.GUID_PRESERVED_FROM_PREVIOUS".into()));
    assert_eq!(read_single_guid(&updated), "legacy-guid");
}

#[ignore]
#[test]
fn update_adds_new_note() {
    let root = scenario_dir();
    let previous = root.join("previous.apkg");
    let updated = root.join("updated.apkg");
    stable_project("front")
        .build(BuildOptions::new().output(&previous))
        .expect("previous build");
    let mut project = stable_project("front");
    project
        .add_note(Note::basic("new", "back").stable_id("update:two"))
        .expect("add new");
    let report = project
        .build(BuildOptions::new().output(&updated).compare_to(&previous))
        .expect("updated build");
    report.ensure_success().expect("successful update");
    assert_eq!(report.counts.notes, 2);
    assert_eq!(
        report
            .update_safety
            .as_ref()
            .expect("update")
            .notes_preserved,
        1
    );
    assert!(report
        .diagnostic_codes()
        .contains(&"UPDATE.GUID_DERIVED_FOR_NEW_NOTE".into()));
    inspect_complete(&updated);
}

#[ignore]
#[test]
fn field_rename_stable_key_safe() {
    let root = scenario_dir();
    let previous = root.join("previous.apkg");
    let updated = root.join("updated.apkg");
    let mut first = Project::new("Japanese")
        .stable_id("jp-core")
        .default_deck("Japanese");
    first.add_notetype(vocab_notetype()).expect("notetype");
    first
        .add_note(
            Note::new("jp-vocab")
                .stable_id("jp:taberu")
                .text("expr", "taberu")
                .text("meaning", "to eat"),
        )
        .expect("note");
    first
        .build(BuildOptions::new().output(&previous))
        .expect("previous build");

    let renamed = NoteType::custom("jp-vocab")
        .field(Field::new("Prompt").key("expr"))
        .field(Field::new("Meaning").key("meaning"))
        .template(
            Template::new("Recognition")
                .key("recognition")
                .front("{{Prompt}}")
                .back("{{Meaning}}")
                .generate_when(GenerationRule::all(["expr"])),
        );
    let mut second = Project::new("Japanese")
        .stable_id("jp-core")
        .default_deck("Japanese");
    second.add_notetype(renamed).expect("notetype");
    second
        .add_note(
            Note::new("jp-vocab")
                .stable_id("jp:taberu")
                .text("expr", "taberu")
                .text("meaning", "to eat"),
        )
        .expect("note");
    let report = second
        .build(BuildOptions::new().output(&updated).compare_to(&previous))
        .expect("updated build");
    report.ensure_success().expect("field rename is safe");
    assert!(report
        .diagnostic_codes()
        .contains(&"UPDATE.FIELD_RENAMED".into()));
    assert!(!report
        .diagnostic_codes()
        .contains(&"UPDATE.FIELD_MERGE_ID_CHANGED".into()));
    inspect_complete(&updated);
}

#[ignore]
#[test]
fn template_reorder_risk() {
    let root = scenario_dir();
    let previous = root.join("previous.apkg");
    let updated = root.join("updated.apkg");
    let mut first = Project::new("Japanese")
        .stable_id("jp-core")
        .default_deck("Japanese");
    first
        .add_notetype(two_template_notetype(["recognition", "production"]))
        .expect("notetype");
    first
        .add_note(
            Note::new("jp-vocab")
                .stable_id("jp:taberu")
                .text("expr", "taberu")
                .text("meaning", "to eat"),
        )
        .expect("note");
    first
        .build(BuildOptions::new().output(&previous))
        .expect("previous build");

    let mut second = Project::new("Japanese")
        .stable_id("jp-core")
        .default_deck("Japanese");
    second
        .add_notetype(two_template_notetype(["production", "recognition"]))
        .expect("notetype");
    second
        .add_note(
            Note::new("jp-vocab")
                .stable_id("jp:taberu")
                .text("expr", "taberu")
                .text("meaning", "to eat"),
        )
        .expect("note");
    let report = second
        .build(BuildOptions::new().output(&updated).compare_to(&previous))
        .expect("updated build");
    report
        .ensure_success()
        .expect("template reorder is warning without fail_on");
    assert_diagnostic_severity(
        &report,
        "UPDATE.TEMPLATE_ORD_CHANGED",
        anki_forge::Severity::Warning,
    );
    assert!(report
        .risk
        .as_ref()
        .expect("risk")
        .findings
        .iter()
        .any(|finding| finding.code == "RISK.TEMPLATE_REORDER"));
    inspect_complete(&updated);
}

#[ignore]
#[test]
fn field_config_id_drift_blocks() {
    let root = scenario_dir();
    let lockfile = root.join("identity-lockfile.json");
    let output = root.join("package.apkg");
    write_field_config_drift_lockfile(&lockfile);
    let report = expect_error_report(
        drift_project().build(
            BuildOptions::new()
                .output(&output)
                .identity_lockfile(&lockfile),
        ),
        "UPDATE.FIELD_MERGE_ID_CHANGED",
    );
    assert_eq!(report.status, anki_forge::build::BuildStatus::Invalid);
    assert!(!output.exists());
    assert_diagnostic_severity(
        &report,
        "UPDATE.FIELD_MERGE_ID_CHANGED",
        anki_forge::Severity::Error,
    );
}

#[ignore]
#[test]
fn template_config_id_drift_blocks() {
    let root = scenario_dir();
    let lockfile = root.join("identity-lockfile.json");
    let output = root.join("package.apkg");
    write_template_config_drift_lockfile(&lockfile);
    let report = expect_error_report(
        drift_project().build(
            BuildOptions::new()
                .output(&output)
                .identity_lockfile(&lockfile),
        ),
        "UPDATE.TEMPLATE_MERGE_ID_CHANGED",
    );
    assert_eq!(report.status, anki_forge::build::BuildStatus::Invalid);
    assert!(!output.exists());
    assert_diagnostic_severity(
        &report,
        "UPDATE.TEMPLATE_MERGE_ID_CHANGED",
        anki_forge::Severity::Error,
    );
}

fn io_fixture_image_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../contracts/fixtures/phase3/manual-desktop-v1/S03_io_minimal/assets/occlusion-heart.png",
    )
}

fn has_selector(values: &[Value], expected: &str) -> bool {
    values
        .iter()
        .any(|value| value["selector"].as_str() == Some(expected))
}

fn vocab_notetype() -> NoteType {
    NoteType::custom("jp-vocab")
        .field(Field::new("Expression").key("expr"))
        .field(Field::new("Meaning").key("meaning"))
        .template(
            Template::new("Recognition")
                .key("recognition")
                .front("{{Expression}}")
                .back("{{Meaning}}")
                .generate_when(GenerationRule::all(["expr"])),
        )
}

#[ignore]
#[test]
fn deck_basic_apkg() {
    let root = scenario_dir();
    let apkg = root.join("package.apkg");
    let mut deck = Deck::builder("Spanish").stable_id("cap-deck-basic").build();

    deck.basic()
        .note("hola", "hello")
        .stable_id("es:hola")
        .add()
        .expect("add basic note");

    let report = deck.write_apkg(&apkg).expect("write apkg");
    report.ensure_success().expect("successful report");
    assert_eq!(report.counts.notes, 1);
    assert_eq!(report.counts.cards, 1);
    assert_eq!(report.counts.media, 0);
    assert!(apkg.is_file());

    let inspected = inspect_complete(&apkg);
    write_manual_checklist("deck_basic_apkg", &apkg, &inspected, &report.diagnostics);
    let counts = counts(&inspected);
    assert_eq!(counts["note_count"], 1);
    assert_eq!(counts["card_count"], 1);
    assert!(has_observation(
        &inspected.observations.notetypes,
        "name",
        "Basic"
    ));
}

#[ignore]
#[test]
fn deck_cloze_apkg() {
    let root = scenario_dir();
    let apkg = root.join("package.apkg");
    let mut deck = Deck::builder("Cloze").stable_id("cap-deck-cloze").build();
    deck.cloze()
        .note("A {{c1::cloze}} fact")
        .stable_id("cloze:one")
        .add()
        .expect("add cloze");

    let report = deck.write_apkg(&apkg).expect("write apkg");
    report.ensure_success().expect("successful report");
    assert_eq!(report.counts.notes, 1);
    assert_eq!(report.counts.cards, 1);
    let inspected = inspect_complete(&apkg);
    write_manual_checklist("deck_cloze_apkg", &apkg, &inspected, &report.diagnostics);
    assert_eq!(counts(&inspected)["note_count"], 1);
    assert_eq!(counts(&inspected)["card_count"], 1);
    assert!(inspected
        .observations
        .notetypes
        .iter()
        .any(|value| value["name"]
            .as_str()
            .is_some_and(|name| name.contains("Cloze"))));
}

#[ignore]
#[test]
fn deck_image_occlusion_apkg() {
    let root = scenario_dir();
    let apkg = root.join("package.apkg");
    let mut deck = Deck::builder("IO").stable_id("cap-deck-io").build();
    let image = deck
        .media()
        .add(anki_forge::MediaSource::from_file(io_fixture_image_path()))
        .expect("image media");
    deck.image_occlusion()
        .note(image)
        .mode(anki_forge::IoMode::HideAllGuessOne)
        .rect(0, 0, 50, 50)
        .stable_id("io:one")
        .add()
        .expect("add image occlusion");

    let report = deck.write_apkg(&apkg).expect("write apkg");
    report.ensure_success().expect("successful report");
    assert_eq!(report.counts.notes, 1);
    assert_eq!(report.counts.cards, 1);
    assert_eq!(report.counts.media, 1);
    let inspected = inspect_complete(&apkg);
    write_manual_checklist(
        "deck_image_occlusion_apkg",
        &apkg,
        &inspected,
        &report.diagnostics,
    );
    assert_eq!(counts(&inspected)["card_count"], 1);
    assert!(has_selector(
        &inspected.observations.notetypes,
        "notetype[id='image_occlusion']"
    ));
}

#[ignore]
#[test]
fn deck_bytes_export() {
    let root = scenario_dir();
    let apkg = root.join("package.apkg");
    let mut deck = Deck::builder("Bytes").stable_id("cap-deck-bytes").build();
    deck.basic()
        .note("front", "back")
        .stable_id("bytes:one")
        .add()
        .expect("add basic");
    let bytes = deck.to_apkg_bytes().expect("apkg bytes");
    assert!(!bytes.is_empty());
    std::fs::write(&apkg, bytes).expect("write bytes");
    let inspected = inspect_complete(&apkg);
    write_manual_checklist("deck_bytes_export", &apkg, &inspected, &[]);
    assert_eq!(counts(&inspected)["note_count"], 1);
    assert_eq!(counts(&inspected)["card_count"], 1);
}

#[ignore]
#[test]
fn project_stock_notes_apkg() {
    let root = scenario_dir();
    let apkg = root.join("package.apkg");
    let mut project = Project::new("Stock")
        .stable_id("cap-project-stock")
        .default_deck("Stock");
    project
        .add_note(Note::basic("front", "back").stable_id("stock:basic"))
        .expect("add basic");
    project
        .add_note(Note::cloze("A {{c1::cloze}} fact").stable_id("stock:cloze"))
        .expect("add cloze");
    let report = project.write_apkg(&apkg).expect("write apkg");
    report.ensure_success().expect("successful report");
    assert_eq!(report.counts.notes, 2);
    assert_eq!(report.counts.cards, 2);
    let inspected = inspect_complete(&apkg);
    write_manual_checklist(
        "project_stock_notes_apkg",
        &apkg,
        &inspected,
        &report.diagnostics,
    );
    assert_eq!(counts(&inspected)["note_count"], 2);
    assert_eq!(counts(&inspected)["card_count"], 2);
}

#[ignore]
#[test]
fn project_custom_notetype_apkg() {
    let root = scenario_dir();
    let apkg = root.join("package.apkg");
    let mut project = Project::new("Custom")
        .stable_id("cap-project-custom")
        .default_deck("Custom");
    project
        .add_notetype(vocab_notetype())
        .expect("add notetype");
    project
        .add_note(
            Note::new("jp-vocab")
                .stable_id("jp:taberu")
                .text("expr", "taberu")
                .text("meaning", "to eat"),
        )
        .expect("add note");
    let report = project.write_apkg(&apkg).expect("write apkg");
    report.ensure_success().expect("successful report");
    assert_eq!(report.counts.notes, 1);
    assert_eq!(report.counts.cards, 1);
    let inspected = inspect_complete(&apkg);
    write_manual_checklist(
        "project_custom_notetype_apkg",
        &apkg,
        &inspected,
        &report.diagnostics,
    );
    assert!(has_observation(
        &inspected.observations.notetypes,
        "id",
        "jp-vocab"
    ));
}

#[ignore]
#[test]
fn project_media_references_apkg() {
    let root = scenario_dir();
    let apkg = root.join("package.apkg");
    let mut project = Project::new("Media")
        .stable_id("cap-project-media")
        .default_deck("Media");
    let audio = project
        .media_mut()
        .add_bytes("raw-audio.bin", MP3_BYTES.to_vec())
        .expect("audio bytes")
        .export_as("voice.mp3")
        .expect("audio export");
    let image = project
        .media_mut()
        .add_bytes("raw-image.bin", PNG_1X1.to_vec())
        .expect("image bytes")
        .export_as("chart.png")
        .expect("image export");
    let back = format!("{}{}", audio.sound().render(), image.image().render());
    project
        .add_note(
            Note::basic("media", "")
                .stable_id("media:one")
                .html("Back", back),
        )
        .expect("add media note");
    let report = project.write_apkg(&apkg).expect("write apkg");
    report.ensure_success().expect("successful report");
    assert_eq!(report.counts.notes, 1);
    assert_eq!(report.counts.cards, 1);
    assert_eq!(report.counts.media, 2);
    assert_eq!(report.media.objects, 2);
    assert_eq!(report.media.bindings, 2);
    assert_eq!(report.media.references, 2);
    assert_eq!(report.media.missing_references, 0);
    assert_eq!(report.media.unsafe_references, 0);
    assert_eq!(report.media.unused_bindings, 0);
    let inspected = inspect_complete(&apkg);
    write_manual_checklist(
        "project_media_references_apkg",
        &apkg,
        &inspected,
        &report.diagnostics,
    );
    assert!(inspected
        .observations
        .media
        .iter()
        .any(|value| value["filename"].as_str() == Some("voice.mp3")));
    assert!(inspected
        .observations
        .media
        .iter()
        .any(|value| value["filename"].as_str() == Some("chart.png")));
    let note = inspected
        .observations
        .references
        .iter()
        .find(|value| value["selector"].as_str() == Some("note[id='media:one']"))
        .expect("media note observation");
    let back = note["fields"]["Back"]
        .as_str()
        .expect("media note Back field");
    assert!(back.contains("[sound:voice.mp3]"));
    assert!(back.contains("src=\"chart.png\""));
}
