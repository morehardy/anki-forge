use std::collections::BTreeMap;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use authoring_core::stock::resolve_stock_notetype;
use authoring_core::{
    AuthoringNotetype, NormalizedIr, NormalizedMedia, NormalizedNote, NormalizedNotetype,
};
use writer_core::{
    build, inspect_apkg, inspect_build_result, inspect_staging, BuildArtifactTarget, BuildContext,
    WriterPolicy,
};

#[test]
fn inspect_build_result_prefers_staging_when_available() {
    let root = unique_artifact_root("inspect-build-result-staging");
    let target = BuildArtifactTarget::new(
        root.clone(),
        "artifacts/phase3/inspect-build-result-staging",
    );

    let result = build(
        &sample_basic_normalized_ir_with_media(),
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

    build(
        &sample_basic_normalized_ir_with_media(),
        &sample_writer_policy(),
        &sample_build_context(false),
        &target,
    )
    .unwrap();

    let report = inspect_staging(&target.staging_manifest_path()).unwrap();
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
fn inspect_apkg_reports_complete_observations_and_counts() {
    let root = unique_artifact_root("inspect-apkg");
    let target = BuildArtifactTarget::new(root.clone(), "artifacts/phase3/inspect-apkg");

    build(
        &sample_basic_normalized_ir_with_media(),
        &sample_writer_policy(),
        &sample_build_context(true),
        &target,
    )
    .unwrap();

    let report = inspect_apkg(&root.join("package.apkg")).unwrap();
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
fn inspect_staging_fingerprint_is_independent_of_artifact_root() {
    let left_root = unique_artifact_root("inspect-fingerprint-left");
    let right_root = unique_artifact_root("inspect-fingerprint-right");

    let left_target = BuildArtifactTarget::new(
        left_root.clone(),
        "artifacts/phase3/inspect-fingerprint",
    );
    let right_target = BuildArtifactTarget::new(
        right_root.clone(),
        "artifacts/phase3/inspect-fingerprint",
    );

    build(
        &sample_basic_normalized_ir_with_media(),
        &sample_writer_policy(),
        &sample_build_context(false),
        &left_target,
    )
    .unwrap();
    build(
        &sample_basic_normalized_ir_with_media(),
        &sample_writer_policy(),
        &sample_build_context(false),
        &right_target,
    )
    .unwrap();

    let left_report = inspect_staging(&left_target.staging_manifest_path()).unwrap();
    let right_report = inspect_staging(&right_target.staging_manifest_path()).unwrap();

    assert_eq!(left_report.artifact_fingerprint, right_report.artifact_fingerprint);
}

#[test]
fn inspect_apkg_marks_missing_media_map_as_degraded() {
    let root = unique_artifact_root("inspect-apkg-degraded");
    let target = BuildArtifactTarget::new(
        root.clone(),
        "artifacts/phase3/inspect-apkg-degraded",
    );

    build(
        &sample_basic_normalized_ir_with_media(),
        &sample_writer_policy(),
        &sample_build_context(true),
        &target,
    )
    .unwrap();

    let degraded_apkg = root.join("package-no-media.apkg");
    strip_zip_entry(&root.join("package.apkg"), &degraded_apkg, "media");

    let report = inspect_apkg(&degraded_apkg).unwrap();
    assert_eq!(report.observation_status, "degraded");
    assert!(report.missing_domains.iter().any(|domain| domain == "media"));
    assert!(!report.degradation_reasons.is_empty());
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
        }],
        media: vec![],
    }
}

fn sample_basic_normalized_ir_with_media() -> NormalizedIr {
    let mut normalized = sample_basic_normalized_ir();
    normalized.notes[0]
        .fields
        .insert("Back".into(), r#"<img src="sample.jpg">"#.into());
    normalized.media.push(NormalizedMedia {
        filename: "sample.jpg".into(),
        mime: "image/jpeg".into(),
        data_base64: "aGVsbG8=".into(),
    });
    normalized
}

fn resolved_stock_notetype(id: &str, kind: &str, name: &str) -> NormalizedNotetype {
    let mut notetype = resolve_stock_notetype(&AuthoringNotetype {
        id: id.into(),
        kind: kind.into(),
        name: Some(name.into()),
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
