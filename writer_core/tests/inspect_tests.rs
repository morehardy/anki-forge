use std::collections::BTreeMap;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use authoring_core::stock::resolve_stock_notetype;
use authoring_core::{
    AuthoringNotetype, NormalizedFieldMetadata, NormalizedIr, NormalizedNote, NormalizedNotetype,
};
use sha1::Digest;
use writer_core::{
    build, extract_media_references, inspect_apkg, inspect_build_result, inspect_staging,
    BuildArtifactTarget, BuildContext, WriterPolicy,
};

#[test]
fn inspect_build_result_prefers_staging_when_available() {
    let root = unique_artifact_root("inspect-build-result-staging");
    let target = BuildArtifactTarget::new(
        root.clone(),
        "artifacts/phase3/inspect-build-result-staging",
    );
    let normalized = sample_basic_normalized_ir_with_media(&target.media_store_dir);

    let result = build(
        &normalized,
        &sample_writer_policy(),
        &sample_build_context(true),
        &target,
    )
    .unwrap();

    let staging_report = inspect_build_result(&result, &target).unwrap();
    assert_eq!(staging_report.source_kind, "staging");
    assert_eq!(staging_report.observation_status, "complete");

    let mut apkg_fallback = result.clone();
    apkg_fallback.staging_ref = None;
    let apkg_report = inspect_build_result(&apkg_fallback, &target).unwrap();
    assert_eq!(apkg_report.source_kind, "apkg");
    assert_eq!(apkg_report.source_ref, result.apkg_ref.as_deref().unwrap());
}

#[test]
fn inspect_staging_reports_complete_observations() {
    let root = unique_artifact_root("inspect-staging");
    let target = BuildArtifactTarget::new(root.clone(), "artifacts/phase3/inspect-staging");
    let normalized = sample_basic_normalized_ir_with_media(&target.media_store_dir);

    build(
        &normalized,
        &sample_writer_policy(),
        &sample_build_context(false),
        &target,
    )
    .unwrap();

    let report = inspect_staging(target.staging_manifest_path()).unwrap();
    assert_eq!(report.source_kind, "staging");
    assert!(report.source_ref.ends_with("staging/manifest.json"));
    assert_eq!(report.observation_status, "complete");
    assert!(report.missing_domains.is_empty());
    assert!(report.degradation_reasons.is_empty());
    assert!(!report.observations.notetypes.is_empty());
    assert!(!report.observations.templates.is_empty());
    assert!(!report.observations.fields.is_empty());
    assert!(!report.observations.media.is_empty());
    assert!(!report.observations.metadata.is_empty());
    assert!(!report.observations.references.is_empty());
}

#[test]
fn inspect_staging_reports_manifest_media_object_and_binding_metadata() {
    let root = unique_artifact_root("inspect-staging-cas");
    let media_store = root.join("media-store");
    let normalized = sample_basic_normalized_ir_with_cas_media(&media_store, "hello.txt", b"hello");
    let target = BuildArtifactTarget::new(root.clone(), "artifacts/phase3/inspect-staging-cas")
        .with_media_store_dir(media_store);
    build(
        &normalized,
        &sample_writer_policy(),
        &sample_build_context(false),
        &target,
    )
    .unwrap();

    let report = inspect_staging(root.join("staging/manifest.json")).unwrap();

    let media = &report.observations.media[0];
    assert_eq!(media["filename"], "hello.txt");
    assert_eq!(media["binding_id"], "media:hello");
    assert!(media["object_id"]
        .as_str()
        .unwrap()
        .starts_with("obj:blake3:"));
    assert!(media["object_ref"]
        .as_str()
        .unwrap()
        .starts_with("media://blake3/"));
}

#[test]
fn inspect_staging_marks_manifest_media_mismatch_as_degraded() {
    let root = unique_artifact_root("inspect-staging-cas-mismatch");
    let media_store = root.join("media-store");
    let normalized = sample_basic_normalized_ir_with_cas_media(&media_store, "hello.txt", b"hello");
    let target = BuildArtifactTarget::new(
        root.clone(),
        "artifacts/phase3/inspect-staging-cas-mismatch",
    )
    .with_media_store_dir(media_store);
    build(
        &normalized,
        &sample_writer_policy(),
        &sample_build_context(false),
        &target,
    )
    .unwrap();
    fs::write(root.join("staging/media/hello.txt"), b"hello!").unwrap();

    let report = inspect_staging(root.join("staging/manifest.json")).unwrap();

    assert_eq!(report.observation_status, "degraded");
    assert!(report
        .missing_domains
        .iter()
        .any(|domain| domain == "media"));
    assert!(report
        .degradation_reasons
        .iter()
        .any(|reason| reason.contains("size mismatch")));
    assert!(report
        .degradation_reasons
        .iter()
        .any(|reason| reason.contains("sha1 mismatch")));
}

#[test]
fn inspect_staging_rejects_escaped_manifest_media_filename() {
    let root = unique_artifact_root("inspect-staging-cas-escaped");
    let media_store = root.join("media-store");
    let outside_path = root.join("staging/escape.txt");
    let normalized = sample_basic_normalized_ir_with_cas_media(&media_store, "hello.txt", b"hello");
    let target =
        BuildArtifactTarget::new(root.clone(), "artifacts/phase3/inspect-staging-cas-escaped")
            .with_media_store_dir(media_store);
    build(
        &normalized,
        &sample_writer_policy(),
        &sample_build_context(false),
        &target,
    )
    .unwrap();
    fs::write(&outside_path, b"hello").unwrap();
    let manifest_path = root.join("staging/manifest.json");
    let mut manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
    manifest["normalized_ir"]["media_bindings"][0]["export_filename"] =
        serde_json::json!("../escape.txt");
    fs::write(&manifest_path, serde_json::to_vec(&manifest).unwrap()).unwrap();

    let report = inspect_staging(manifest_path).unwrap();

    assert_eq!(report.observation_status, "degraded");
    assert!(report
        .missing_domains
        .iter()
        .any(|domain| domain == "media"));
    assert!(report.observations.media.is_empty());
    assert!(report
        .degradation_reasons
        .iter()
        .any(|reason| reason.contains("invalid staged media filename ../escape.txt")));
}

#[test]
fn inspect_emits_browser_template_and_field_label_observations() {
    let root = unique_artifact_root("inspect-browser-metadata");
    let target =
        BuildArtifactTarget::new(root.clone(), "artifacts/phase3/inspect-browser-metadata");

    let mut normalized_ir = sample_basic_normalized_ir();
    normalized_ir.notetypes[0].field_metadata = vec![NormalizedFieldMetadata {
        field_name: "Front".into(),
        label: Some("Prompt".into()),
        role_hint: Some("question".into()),
    }];
    normalized_ir.notetypes[0].templates[0].browser_question_format =
        Some("<span class=\"browser-front\">{{Front}}</span>".into());
    normalized_ir.notetypes[0].templates[0].browser_answer_format =
        Some("<span class=\"browser-back\">{{Back}}</span>".into());
    normalized_ir.notetypes[0].templates[0].browser_font_name = Some("Arial".into());
    normalized_ir.notetypes[0].templates[0].browser_font_size = Some(18);
    normalized_ir.notetypes[0].templates[0].target_deck_name = Some("Custom::Deck".into());

    build(
        &normalized_ir,
        &sample_writer_policy(),
        &sample_build_context(true),
        &target,
    )
    .unwrap();

    let report = inspect_staging(target.staging_manifest_path()).unwrap();
    assert!(report
        .observations
        .field_metadata
        .iter()
        .any(|value| value["field_name"] == "Front" && value["label"] == "Prompt"));
    assert!(report
        .observations
        .browser_templates
        .iter()
        .any(|value| value["template_name"] == "Card 1"
            && value["browser_font_name"] == "Arial"
            && value["browser_font_size"] == 18));
    assert!(report
        .observations
        .template_target_decks
        .iter()
        .any(|value| value["template_name"] == "Card 1"
            && value["target_deck_name"] == "Custom::Deck"
            && value["resolved_target_deck_id"] == 2));

    let apkg_report = inspect_apkg(root.join("package.apkg")).unwrap();
    assert!(apkg_report
        .observations
        .template_target_decks
        .iter()
        .any(|value| value["template_name"] == "Card 1"
            && value["target_deck_name"] == "Custom::Deck"
            && value["resolved_target_deck_id"] == 2));
}

#[test]
fn inspect_apkg_reports_complete_observations_and_counts() {
    let root = unique_artifact_root("inspect-apkg");
    let target = BuildArtifactTarget::new(root.clone(), "artifacts/phase3/inspect-apkg");
    let normalized = sample_basic_normalized_ir_with_media(&target.media_store_dir);

    build(
        &normalized,
        &sample_writer_policy(),
        &sample_build_context(true),
        &target,
    )
    .unwrap();

    let report = inspect_apkg(root.join("package.apkg")).unwrap();
    assert_eq!(report.source_kind, "apkg");
    assert!(report.source_ref.ends_with("package.apkg"));
    assert_eq!(report.observation_status, "complete");
    assert!(report.missing_domains.is_empty());
    assert!(report.degradation_reasons.is_empty());

    let meta = report
        .observations
        .metadata
        .iter()
        .find(|value| value["selector"] == "counts")
        .expect("counts metadata observation");
    assert_eq!(meta["notetype_count"], 1);
    assert_eq!(meta["note_count"], 1);
    assert_eq!(meta["card_count"], 1);
    assert_eq!(meta["media_count"], 1);
    assert!(report
        .observations
        .notetypes
        .iter()
        .any(|value| value["selector"] == "notetype[id='basic-main']"));
    assert!(report
        .observations
        .references
        .iter()
        .any(|value| value["selector"] == "note[id='note-1']"));
}

#[test]
fn inspect_apkg_does_not_report_forge_only_media_ids() {
    let root = unique_artifact_root("inspect-apkg-cas");
    let media_store = root.join("media-store");
    let normalized = sample_basic_normalized_ir_with_cas_media(&media_store, "hello.txt", b"hello");
    let target = BuildArtifactTarget::new(root.clone(), "artifacts/phase3/inspect-apkg-cas")
        .with_media_store_dir(media_store);
    build(
        &normalized,
        &sample_writer_policy(),
        &sample_build_context(true),
        &target,
    )
    .unwrap();

    let report = inspect_apkg(root.join("package.apkg")).unwrap();

    let media = &report.observations.media[0];
    assert_eq!(media["filename"], "hello.txt");
    assert!(media.get("binding_id").is_none());
    assert!(media.get("object_id").is_none());
    assert!(media.get("object_ref").is_none());
}

#[test]
fn inspect_apkg_reports_note_and_card_deck_names() {
    let root = unique_artifact_root("inspect-apkg-deck-routing");
    let target =
        BuildArtifactTarget::new(root.clone(), "artifacts/phase3/inspect-apkg-deck-routing");
    let mut normalized_ir = sample_basic_normalized_ir();
    normalized_ir.notes[0].deck_name = "Biology::Cells".into();

    build(
        &normalized_ir,
        &sample_writer_policy(),
        &sample_build_context(true),
        &target,
    )
    .unwrap();

    let report = inspect_apkg(root.join("package.apkg")).unwrap();

    let note = report
        .observations
        .references
        .iter()
        .find(|value| value["selector"] == "note[id='note-1']")
        .expect("note observation");
    assert_eq!(note["deck_name"], "Biology::Cells");

    let card = report
        .observations
        .references
        .iter()
        .find(|value| value["selector"] == "card[note_id='note-1'][ord=0]")
        .expect("card observation");
    assert_eq!(card["deck_name"], "Biology::Cells");
}

#[test]
fn inspect_apkg_reports_actual_card_decks_for_mixed_template_routing() {
    let root = unique_artifact_root("inspect-apkg-mixed-template-routing");
    let target = BuildArtifactTarget::new(
        root.clone(),
        "artifacts/phase3/inspect-apkg-mixed-template-routing",
    );
    let mut normalized_ir = sample_basic_normalized_ir();
    normalized_ir.notes[0].deck_name = "Biology::Cells".into();
    normalized_ir.notetypes[0].kind = "normal".into();
    normalized_ir.notetypes[0].original_stock_kind = None;
    normalized_ir.notetypes[0].templates[0].target_deck_name = Some("Biology::Overrides".into());

    let mut second_template = normalized_ir.notetypes[0].templates[0].clone();
    second_template.name = "Card 2".into();
    second_template.ord = Some(1);
    second_template.target_deck_name = None;
    normalized_ir.notetypes[0].templates.push(second_template);

    let result = build(
        &normalized_ir,
        &sample_writer_policy(),
        &sample_build_context(true),
        &target,
    )
    .unwrap();
    assert!(result.apkg_ref.is_some(), "{result:#?}");

    let report = inspect_apkg(root.join("package.apkg")).unwrap();

    let first_card = report
        .observations
        .references
        .iter()
        .find(|value| value["selector"] == "card[note_id='note-1'][ord=0]")
        .expect("first card observation");
    assert_eq!(first_card["deck_name"], "Biology::Overrides");

    let second_card = report
        .observations
        .references
        .iter()
        .find(|value| value["selector"] == "card[note_id='note-1'][ord=1]")
        .expect("second card observation");
    assert_eq!(second_card["deck_name"], "Biology::Cells");
}

#[test]
fn inspect_staging_fingerprint_is_independent_of_artifact_root() {
    let left_root = unique_artifact_root("inspect-fingerprint-left");
    let right_root = unique_artifact_root("inspect-fingerprint-right");

    let left_target =
        BuildArtifactTarget::new(left_root.clone(), "artifacts/phase3/inspect-fingerprint");
    let right_target =
        BuildArtifactTarget::new(right_root.clone(), "artifacts/phase3/inspect-fingerprint");
    let left_normalized = sample_basic_normalized_ir_with_media(&left_target.media_store_dir);
    let right_normalized = sample_basic_normalized_ir_with_media(&right_target.media_store_dir);

    build(
        &left_normalized,
        &sample_writer_policy(),
        &sample_build_context(false),
        &left_target,
    )
    .unwrap();
    build(
        &right_normalized,
        &sample_writer_policy(),
        &sample_build_context(false),
        &right_target,
    )
    .unwrap();

    let left_report = inspect_staging(left_target.staging_manifest_path()).unwrap();
    let right_report = inspect_staging(right_target.staging_manifest_path()).unwrap();

    assert_eq!(
        left_report.artifact_fingerprint,
        right_report.artifact_fingerprint
    );
}

#[test]
fn inspect_apkg_marks_missing_media_map_as_degraded() {
    let root = unique_artifact_root("inspect-apkg-degraded");
    let target = BuildArtifactTarget::new(root.clone(), "artifacts/phase3/inspect-apkg-degraded");
    let normalized = sample_basic_normalized_ir_with_media(&target.media_store_dir);

    build(
        &normalized,
        &sample_writer_policy(),
        &sample_build_context(true),
        &target,
    )
    .unwrap();

    let degraded_apkg = root.join("package-no-media.apkg");
    strip_zip_entry(&root.join("package.apkg"), &degraded_apkg, "media");

    let report = inspect_apkg(&degraded_apkg).unwrap();
    assert_eq!(report.observation_status, "degraded");
    assert!(report
        .missing_domains
        .iter()
        .any(|domain| domain == "media"));
    assert!(!report.degradation_reasons.is_empty());
}

#[test]
fn extract_media_references_decodes_html_entities_across_reference_forms() {
    let refs = extract_media_references(
        r#"<img src="sample&#46;jpg"> [sound:voice&#46;mp3] <object data="extra&#46;svg"></object>"#,
    );

    assert_eq!(refs, vec!["voice.mp3", "sample.jpg", "extra.svg"]);
}

fn sample_writer_policy() -> WriterPolicy {
    WriterPolicy {
        id: "writer-policy.default".into(),
        version: "1.0.0".into(),
        compatibility_target: "latest-only".into(),
        stock_notetype_mode: "source-grounded".into(),
        media_entry_mode: "inline".into(),
        apkg_version: "latest".into(),
    }
}

fn sample_build_context(emit_apkg: bool) -> BuildContext {
    BuildContext {
        id: "build-context.default".into(),
        version: "1.0.0".into(),
        emit_apkg,
        materialize_staging: true,
        media_resolution_mode: "inline-only".into(),
        unresolved_asset_behavior: "fail".into(),
        fingerprint_mode: "canonical".into(),
    }
}

fn sample_basic_normalized_ir() -> NormalizedIr {
    NormalizedIr {
        kind: "normalized-ir".into(),
        schema_version: "0.1.0".into(),
        document_id: "demo-doc".into(),
        resolved_identity: "document:demo-doc".into(),
        notetypes: vec![resolved_stock_notetype("basic-main", "basic", "Basic")],
        notes: vec![NormalizedNote {
            id: "note-1".into(),
            notetype_id: "basic-main".into(),
            deck_name: "Default".into(),
            fields: BTreeMap::from([
                ("Front".into(), "front".into()),
                ("Back".into(), "back".into()),
            ]),
            tags: vec!["demo".into()],
            mtime_secs: None,
        }],
        media_objects: vec![],
        media_bindings: vec![],
        media_references: vec![],
    }
}

fn sample_basic_normalized_ir_with_media(media_store: &Path) -> NormalizedIr {
    let mut normalized = sample_basic_normalized_ir();
    let bytes = b"hello";
    let blake3_hex = blake3::hash(bytes).to_hex().to_string();
    let sha1_hex = hex::encode(sha1::Sha1::digest(bytes));
    let object_id = format!("obj:blake3:{blake3_hex}");
    let object_path = authoring_core::object_store_path(media_store, &blake3_hex).unwrap();
    fs::create_dir_all(object_path.parent().unwrap()).unwrap();
    fs::write(&object_path, bytes).unwrap();
    normalized.notes[0]
        .fields
        .insert("Back".into(), r#"<img src="sample.jpg">"#.into());
    normalized.media_objects = vec![authoring_core::MediaObject {
        id: object_id.clone(),
        object_ref: format!("media://blake3/{blake3_hex}"),
        blake3: blake3_hex,
        sha1: sha1_hex,
        size_bytes: bytes.len() as u64,
        mime: "image/jpeg".into(),
    }];
    normalized.media_bindings = vec![authoring_core::MediaBinding {
        id: "media:sample".into(),
        export_filename: "sample.jpg".into(),
        object_id,
    }];
    normalized.media_references = vec![];
    normalized
}

fn sample_basic_normalized_ir_with_cas_media(
    media_store: &Path,
    filename: &str,
    bytes: &[u8],
) -> NormalizedIr {
    let mut normalized = sample_basic_normalized_ir();
    let blake3_hex = blake3::hash(bytes).to_hex().to_string();
    let sha1_hex = hex::encode(sha1::Sha1::digest(bytes));
    let object_id = format!("obj:blake3:{blake3_hex}");
    let object_path = authoring_core::object_store_path(media_store, &blake3_hex).unwrap();
    fs::create_dir_all(object_path.parent().unwrap()).unwrap();
    fs::write(&object_path, bytes).unwrap();
    normalized.media_objects = vec![authoring_core::MediaObject {
        id: object_id.clone(),
        object_ref: format!("media://blake3/{blake3_hex}"),
        blake3: blake3_hex,
        sha1: sha1_hex,
        size_bytes: bytes.len() as u64,
        mime: "text/plain".into(),
    }];
    normalized.media_bindings = vec![authoring_core::MediaBinding {
        id: "media:hello".into(),
        export_filename: filename.into(),
        object_id,
    }];
    normalized.media_references = vec![];
    normalized
}

fn resolved_stock_notetype(id: &str, kind: &str, name: &str) -> NormalizedNotetype {
    let mut notetype = resolve_stock_notetype(&AuthoringNotetype {
        id: id.into(),
        kind: kind.into(),
        name: Some(name.into()),
        original_stock_kind: None,
        original_id: None,
        fields: None,
        templates: None,
        css: None,
        field_metadata: vec![],
    })
    .expect("resolve stock notetype");
    notetype.id = id.into();
    notetype
}

fn unique_artifact_root(case: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "anki-forge-phase3-{case}-{}-{nanos}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

fn strip_zip_entry(source: &Path, target: &Path, missing_entry: &str) {
    let mut archive = zip::ZipArchive::new(File::open(source).unwrap()).unwrap();
    let mut writer = zip::ZipWriter::new(File::create(target).unwrap());

    for index in 0..archive.len() {
        let mut file = archive.by_index(index).unwrap();
        if file.name() == missing_entry {
            continue;
        }

        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        writer
            .start_file(
                file.name(),
                zip::write::FileOptions::<'static, ()>::default()
                    .compression_method(zip::CompressionMethod::Stored),
            )
            .unwrap();
        writer.write_all(&bytes).unwrap();
    }

    writer.finish().unwrap();
}
