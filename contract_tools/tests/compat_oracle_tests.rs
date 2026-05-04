use authoring_core::NormalizedIr;
use contract_tools::{
    compat_oracle::{run_compat_oracle_gates, validate_supported_package},
    contract_manifest_path,
    manifest::{load_manifest, resolve_contract_relative_path},
    policies::{load_build_context_asset, load_writer_policy_asset},
};
use prost::Message;
use serde_json::Value;
use std::{
    fs,
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
use tempfile::TempDir;

#[test]
fn compat_oracle_gates_accept_bundled_writer_phase3_fixtures() {
    run_compat_oracle_gates(copied_bundled_manifest_path("compat-oracle"))
        .expect("bundled compat oracle gate should pass");
}

#[test]
fn compat_oracle_skips_when_catalog_has_no_phase3_writer_cases() {
    let manifest_path = copied_bundled_manifest_path("compat-oracle-no-writer");
    let catalog_path = manifest_path
        .parent()
        .expect("manifest parent")
        .join("fixtures/index.yaml");
    let raw = fs::read_to_string(&catalog_path).expect("read fixture catalog");
    let mut catalog: serde_yaml::Value =
        serde_yaml::from_str(&raw).expect("decode fixture catalog");
    let cases = catalog
        .get_mut("cases")
        .and_then(serde_yaml::Value::as_sequence_mut)
        .expect("fixture catalog should have cases");
    cases.retain(|entry| {
        entry.get("category").and_then(serde_yaml::Value::as_str) != Some("phase3-writer")
    });
    fs::write(
        &catalog_path,
        serde_yaml::to_string(&catalog).expect("encode fixture catalog"),
    )
    .expect("write fixture catalog");

    run_compat_oracle_gates(&manifest_path)
        .expect("compat oracle should skip when no phase3-writer fixtures exist");
}

#[test]
fn compat_oracle_rejects_basic_field_order_drift() {
    let (_artifact_root, apkg_path, mut inspect_report) = build_phase3_fixture_apkg(
        "fixtures/phase3/inputs/basic-normalized-ir.json",
        "field-order",
    );

    inspect_report.observations.fields.swap(0, 1);

    let err = validate_supported_package(&apkg_path, &inspect_report)
        .expect_err("field-order drift should be rejected");
    assert!(
        err.to_string().contains("fields must match stock order")
            || err.to_string().contains("must match stock"),
        "unexpected error: {err}"
    );
}

#[test]
fn compat_oracle_rejects_dangling_media_refs_derived_from_note_fields() {
    let (_artifact_root, apkg_path, mut inspect_report) = build_phase3_fixture_apkg(
        "fixtures/phase3/inputs/image-occlusion-normalized-ir.json",
        "dangling-media",
    );

    let note_entry = inspect_report
        .observations
        .references
        .iter_mut()
        .find(|entry| entry.get("selector").and_then(Value::as_str) == Some("note[id='note-io-1']"))
        .expect("image occlusion inspect report should include note entry");
    note_entry["fields"]["Image"] = Value::String("<img src=\"missing.png\">".to_string());

    let err = validate_supported_package(&apkg_path, &inspect_report)
        .expect_err("dangling media references should be rejected");
    assert!(
        err.to_string().contains("missing from media map")
            || err.to_string().contains("media reference"),
        "unexpected error: {err}"
    );
}

#[test]
fn compat_oracle_rejects_image_occlusion_template_drift() {
    let (_artifact_root, apkg_path, mut inspect_report) = build_phase3_fixture_apkg(
        "fixtures/phase3/inputs/image-occlusion-normalized-ir.json",
        "io-template",
    );

    let template = inspect_report
        .observations
        .templates
        .iter_mut()
        .find(|entry| {
            entry.get("notetype_id").and_then(Value::as_str) == Some("io-main")
                && entry.get("name").and_then(Value::as_str) == Some("Image Occlusion")
        })
        .expect("image occlusion inspect report should include io template");
    let existing = template
        .get("answer_format")
        .and_then(Value::as_str)
        .expect("template should include answer_format")
        .to_string();
    template["answer_format"] = Value::String(format!("{existing}\n<!-- drift -->"));

    let err = validate_supported_package(&apkg_path, &inspect_report)
        .expect_err("template drift should be rejected");
    assert!(
        err.to_string().contains("must match stock") || err.to_string().contains("template"),
        "unexpected error: {err}"
    );
}

#[test]
fn compat_oracle_accepts_numeric_html_entity_media_references() {
    let (_artifact_root, apkg_path, mut inspect_report) = build_phase3_fixture_apkg(
        "fixtures/phase3/inputs/image-occlusion-normalized-ir.json",
        "html-entity-media",
    );

    let note_entry = inspect_report
        .observations
        .references
        .iter_mut()
        .find(|entry| entry.get("selector").and_then(Value::as_str) == Some("note[id='note-io-1']"))
        .expect("image occlusion inspect report should include note entry");
    note_entry["fields"]["Image"] = Value::String("<img src=\"occlusion&#46;png\">".to_string());

    validate_supported_package(&apkg_path, &inspect_report)
        .expect("numeric html entities should resolve to the bundled media filename");
}

#[test]
fn compat_oracle_rejects_media_map_legacy_zip_filename_tag255() {
    let (_artifact_root, apkg_path, inspect_report) = build_phase3_fixture_apkg(
        "fixtures/phase3/inputs/image-occlusion-normalized-ir.json",
        "media-legacy-tag255",
    );
    let rewritten_apkg = apkg_path.with_file_name("package-with-legacy-media-tag.apkg");

    rewrite_media_map_with_legacy_zip_filename(&apkg_path, &rewritten_apkg);

    let err = validate_supported_package(&rewritten_apkg, &inspect_report)
        .expect_err("legacy zip filename tag should be rejected");
    assert!(
        err.to_string().contains("legacy_zip_filename")
            || err.to_string().contains("legacy zip filename"),
        "unexpected error: {err}"
    );
}

fn temp_contract_root(label: &str) -> PathBuf {
    static NEXT_TEMP_ROOT_ID: AtomicU64 = AtomicU64::new(0);
    let unique = NEXT_TEMP_ROOT_ID.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "anki-forge-contract-tools-{}-{}-{}",
        label,
        std::process::id(),
        unique
    ))
}

fn copy_tree(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).expect("create destination tree");
    for entry in fs::read_dir(src).expect("read source tree") {
        let entry = entry.expect("read source entry");
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_tree(&src_path, &dst_path);
        } else {
            fs::copy(&src_path, &dst_path).expect("copy source file");
        }
    }
}

fn copied_bundled_manifest_path(label: &str) -> PathBuf {
    let root = temp_contract_root(label);
    copy_tree(
        contract_manifest_path()
            .parent()
            .expect("contracts root for bundled manifest"),
        &root,
    );
    root.join("manifest.yaml")
}

fn build_phase3_fixture_apkg(
    normalized_fixture: &str,
    label: &str,
) -> (TempDir, PathBuf, writer_core::InspectReport) {
    let manifest = load_manifest(contract_manifest_path()).expect("load bundled manifest");
    let normalized_path =
        resolve_contract_relative_path(&manifest.contracts_root, normalized_fixture)
            .expect("resolve normalized fixture");
    let raw = fs::read_to_string(&normalized_path).expect("read normalized fixture");
    let normalized_ir: NormalizedIr =
        serde_json::from_str(&raw).expect("decode normalized fixture");

    let writer_policy = load_writer_policy_asset(&manifest, "default").expect("load writer policy");
    let build_context = load_build_context_asset(&manifest, "default").expect("load build context");
    let artifact_root = tempfile::tempdir().expect("temp artifact root");
    let media_store_dir = normalized_path
        .parent()
        .expect("normalized fixture has parent")
        .join(".anki-forge-media");
    let target = writer_core::BuildArtifactTarget::new(
        artifact_root.path().to_path_buf(),
        format!("artifacts/compat-oracle-tests/{label}"),
    )
    .with_media_store_dir(media_store_dir);

    let build_result = writer_core::build(&normalized_ir, &writer_policy, &build_context, &target)
        .expect("build fixture package");
    let apkg_ref = build_result
        .apkg_ref
        .as_deref()
        .expect("build should produce apkg_ref")
        .to_string();
    let apkg_path = artifact_path_from_ref(&target, &apkg_ref);
    let inspect_report = writer_core::inspect_apkg(&apkg_path).expect("inspect built package");

    (artifact_root, apkg_path, inspect_report)
}

fn rewrite_media_map_with_legacy_zip_filename(source: &Path, target: &Path) {
    let mut archive = zip::ZipArchive::new(File::open(source).unwrap()).unwrap();
    let mut writer = zip::ZipWriter::new(File::create(target).unwrap());

    for index in 0..archive.len() {
        let mut file = archive.by_index(index).unwrap();
        let name = file.name().to_string();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();

        if name == "media" {
            let media_map = zstd::stream::decode_all(bytes.as_slice()).unwrap();
            let mut media_entries =
                TestMediaEntries::decode(media_map.as_slice()).expect("decode media map");
            media_entries
                .entries
                .first_mut()
                .expect("fixture should have a media entry")
                .legacy_zip_filename = Some(7);
            bytes = zstd::stream::encode_all(media_entries.encode_to_vec().as_slice(), 0).unwrap();
        }

        writer
            .start_file(
                name,
                zip::write::FileOptions::<'static, ()>::default()
                    .compression_method(zip::CompressionMethod::Stored),
            )
            .unwrap();
        writer.write_all(&bytes).unwrap();
    }

    writer.finish().unwrap();
}

fn artifact_path_from_ref(target: &writer_core::BuildArtifactTarget, reference: &str) -> PathBuf {
    let prefix = target.stable_ref_prefix.trim_end_matches('/');
    let trimmed = reference
        .strip_prefix(prefix)
        .unwrap_or(reference)
        .trim_start_matches('/');
    if trimmed.is_empty() {
        target.root_dir.clone()
    } else {
        target.root_dir.join(trimmed)
    }
}

#[derive(Clone, PartialEq, prost::Message)]
struct TestMediaEntries {
    #[prost(message, repeated, tag = "1")]
    entries: Vec<TestMediaEntry>,
}

#[derive(Clone, PartialEq, prost::Message)]
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
