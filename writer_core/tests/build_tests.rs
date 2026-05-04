use authoring_core::stock::resolve_stock_notetype;
use authoring_core::{AuthoringNotetype, NormalizedIr, NormalizedNote, NormalizedNotetype};
use prost::Message;
use rusqlite::Connection;
use sha1::Digest;
use std::collections::BTreeMap;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use writer_core::{
    build, build_context_ref, policy_ref, to_canonical_json, BuildArtifactTarget, BuildContext,
    BuildDiagnosticItem, BuildDiagnostics, DiffReport, InspectObservations, InspectReport,
    PackageBuildResult, StagingPackage, VerificationGateRule, VerificationPolicy, WriterPolicy,
};

#[test]
fn package_build_result_carries_writer_and_build_context_refs() {
    let result = PackageBuildResult {
        kind: "package-build-result".into(),
        result_status: "success".into(),
        tool_contract_version: writer_core::tool_contract_version().into(),
        writer_policy_ref: "writer-policy.default@1.0.0".into(),
        build_context_ref: "build-context:abc".into(),
        staging_ref: Some("staging:demo".into()),
        artifact_fingerprint: Some("artifact:demo".into()),
        package_fingerprint: None,
        apkg_ref: None,
        diagnostics: BuildDiagnostics {
            kind: "build-diagnostics".into(),
            items: vec![BuildDiagnosticItem {
                level: "warning".into(),
                code: "PHASE3.DEMO".into(),
                summary: "demo".into(),
                domain: Some("writer".into()),
                path: None,
                target_selector: None,
                stage: None,
                operation: None,
            }],
        },
    };

    let json = serde_json::to_value(result).unwrap();
    assert_eq!(json["kind"], "package-build-result");
    assert_eq!(json["writer_policy_ref"], "writer-policy.default@1.0.0");
    assert_eq!(json["build_context_ref"], "build-context:abc");
    assert_eq!(json["staging_ref"], "staging:demo");
    assert_eq!(json["artifact_fingerprint"], "artifact:demo");
}

#[test]
fn canonical_json_orders_phase3_report_keys_stably() {
    let json = to_canonical_json(&serde_json::json!({
        "z": 1,
        "a": { "d": 4, "b": 2 }
    }))
    .unwrap();

    assert_eq!(json, "{\"a\":{\"b\":2,\"d\":4},\"z\":1}");
}

#[test]
fn build_context_ref_is_deterministic_for_equal_contexts() {
    let context = BuildContext {
        id: "build-context.default".into(),
        version: "1.0.0".into(),
        emit_apkg: true,
        materialize_staging: true,
        media_resolution_mode: "inline-only".into(),
        unresolved_asset_behavior: "warn".into(),
        fingerprint_mode: "canonical".into(),
    };

    let left = build_context_ref(&context).unwrap();
    let right = build_context_ref(&context).unwrap();

    assert_eq!(left, right);
    assert!(left.starts_with("build-context:"));
}

#[test]
fn policy_refs_use_id_and_version() {
    assert_eq!(
        policy_ref("writer-policy.default", "1.0.0"),
        "writer-policy.default@1.0.0"
    );
}

#[test]
fn phase3_models_serialize_with_expected_fields() {
    let writer_policy = WriterPolicy {
        id: "writer-policy.default".into(),
        version: "1.0.0".into(),
        compatibility_target: "latest-only".into(),
        stock_notetype_mode: "stock-aware".into(),
        media_entry_mode: "canonical".into(),
        apkg_version: "latest".into(),
    };

    let verification_policy = VerificationPolicy {
        id: "verification-policy.default".into(),
        version: "1.0.0".into(),
        writer_fast_gate: VerificationGateRule {
            minimum_comparison_status: "complete".into(),
            allowed_observation_statuses: vec!["complete".into(), "degraded".into()],
            blocking_severities: vec!["high".into()],
        },
        system_gate: VerificationGateRule {
            minimum_comparison_status: "partial".into(),
            allowed_observation_statuses: vec!["complete".into()],
            blocking_severities: vec!["medium".into(), "high".into()],
        },
        compat_gate: VerificationGateRule {
            minimum_comparison_status: "complete".into(),
            allowed_observation_statuses: vec!["complete".into()],
            blocking_severities: vec!["high".into()],
        },
    };

    let writer_policy_json = serde_json::to_value(writer_policy).unwrap();
    let verification_policy_json = serde_json::to_value(verification_policy).unwrap();

    assert_eq!(writer_policy_json["compatibility_target"], "latest-only");
    assert_eq!(
        verification_policy_json["writer_fast_gate"]["minimum_comparison_status"],
        "complete"
    );
}

#[test]
fn inspect_report_serializes_with_expected_fixed_domains() {
    let report = InspectReport {
        kind: "inspect-report".into(),
        observation_model_version: "phase3-v1".into(),
        source_kind: "staging".into(),
        source_ref: "staging:demo".into(),
        artifact_fingerprint: "artifact:demo".into(),
        observation_status: "complete".into(),
        missing_domains: vec![],
        degradation_reasons: vec![],
        observations: InspectObservations {
            notetypes: vec![],
            templates: vec![],
            fields: vec![],
            media: vec![],
            field_metadata: vec![],
            browser_templates: vec![],
            template_target_decks: vec![],
            metadata: vec![],
            references: vec![],
        },
    };

    let json = serde_json::to_value(report).unwrap();
    assert_eq!(json["kind"], "inspect-report");
    assert_eq!(json["source_kind"], "staging");
    assert_eq!(json["observations"]["notetypes"], serde_json::json!([]));
    assert_eq!(json["observations"]["references"], serde_json::json!([]));
}

#[test]
fn diff_report_keeps_required_empty_arrays_when_no_changes_exist() {
    let report = DiffReport {
        kind: "diff-report".into(),
        comparison_status: "complete".into(),
        left_fingerprint: "artifact:left".into(),
        right_fingerprint: "artifact:right".into(),
        left_observation_model_version: "phase3-v1".into(),
        right_observation_model_version: "phase3-v1".into(),
        summary: "no changes".into(),
        uncompared_domains: vec![],
        comparison_limitations: vec![],
        changes: vec![],
    };

    let json = serde_json::to_value(report).unwrap();
    assert_eq!(json["uncompared_domains"], serde_json::json!([]));
    assert_eq!(json["comparison_limitations"], serde_json::json!([]));
    assert_eq!(json["changes"], serde_json::json!([]));
}

#[test]
fn emit_apkg_materializes_basic_package_from_staging_artifact() {
    let root = unique_artifact_root("basic-apkg");
    let media_store = root.join("media-store");
    let target = BuildArtifactTarget::new(root.clone(), "artifacts/phase3/basic-apkg")
        .with_media_store_dir(media_store.clone());
    let normalized =
        sample_basic_normalized_ir_with_cas_media(&media_store, "sample.jpg", b"hello");
    let package = StagingPackage::from_normalized(
        &normalized,
        &sample_writer_policy(),
        &sample_build_context(true),
    )
    .unwrap();
    let materialized = package.materialize(&target).unwrap();

    let apkg = writer_core::apkg::emit_apkg(&materialized, &target).unwrap();

    assert_eq!(apkg.apkg_ref, "artifacts/phase3/basic-apkg/package.apkg");
    assert!(apkg.apkg_path.exists());
    assert!(apkg.package_fingerprint.starts_with("package:"));

    let mut archive = open_zip(&apkg.apkg_path);
    let names = archive_names(&mut archive);
    for expected in [
        "meta",
        "collection.anki21b",
        "collection.anki2",
        "media",
        "0",
    ] {
        assert!(
            names.contains(expected),
            "missing expected apkg entry {expected}: {names:?}"
        );
    }

    let legacy_collection = read_zip_entry_bytes(&mut archive, "collection.anki2");
    assert_legacy_models_use_schema11_shape(&legacy_collection);
}

#[test]
fn build_materializes_basic_staging_into_caller_owned_root() {
    let root = unique_artifact_root("basic");
    let target = BuildArtifactTarget::new(root.clone(), "artifacts/phase3/basic");

    let result = build(
        &sample_basic_normalized_ir(),
        &sample_writer_policy(),
        &sample_build_context(false),
        &target,
    )
    .unwrap();

    assert_eq!(result.result_status, "success");
    assert_eq!(
        result.staging_ref.as_deref(),
        Some("artifacts/phase3/basic/staging/manifest.json")
    );
    assert!(root.join("staging/manifest.json").exists());
    assert!(result
        .artifact_fingerprint
        .as_deref()
        .unwrap()
        .starts_with("artifact:"));
}

#[test]
fn tracked_rslib_storage_sql_snapshots_exist() {
    for relative in [
        "assets/rslib/storage/schema11.sql",
        "assets/rslib/storage/upgrades/schema14_upgrade.sql",
        "assets/rslib/storage/upgrades/schema15_upgrade.sql",
        "assets/rslib/storage/upgrades/schema17_upgrade.sql",
        "assets/rslib/storage/upgrades/schema18_upgrade.sql",
    ] {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative);
        assert!(
            path.exists(),
            "expected tracked rslib storage snapshot at {}",
            path.display()
        );
    }
}

#[test]
fn build_accepts_numeric_html_entity_media_references() {
    let root = unique_artifact_root("html-entity-media");
    let media_store = root.join("media-store");
    let target = BuildArtifactTarget::new(root, "artifacts/phase3/html-entity-media")
        .with_media_store_dir(media_store.clone());
    let mut normalized =
        sample_basic_normalized_ir_with_cas_media(&media_store, "sample.jpg", b"hello");
    normalized.notes[0]
        .fields
        .insert("Back".into(), "<img src=\"sample&#46;jpg\">".into());

    let result = build(
        &normalized,
        &sample_writer_policy(),
        &sample_build_context(false),
        &target,
    )
    .expect("build should accept numeric html entity media references");

    assert_eq!(result.result_status, "success");
}

#[test]
fn build_materializes_cloze_staging_into_caller_owned_root() {
    let root = unique_artifact_root("cloze");
    let target = BuildArtifactTarget::new(root.clone(), "artifacts/phase3/cloze");

    let result = build(
        &sample_cloze_normalized_ir(),
        &sample_writer_policy(),
        &sample_build_context(false),
        &target,
    )
    .unwrap();

    assert_eq!(result.result_status, "success");
    assert_eq!(
        result.staging_ref.as_deref(),
        Some("artifacts/phase3/cloze/staging/manifest.json")
    );
    assert!(root.join("staging/manifest.json").exists());
    assert!(result
        .artifact_fingerprint
        .as_deref()
        .unwrap()
        .starts_with("artifact:"));
}

#[test]
fn build_materializes_media_payloads_into_staging_tree() {
    let root = unique_artifact_root("basic-media");
    let media_store = root.join("media-store");
    let target = BuildArtifactTarget::new(root.clone(), "artifacts/phase3/basic-media")
        .with_media_store_dir(media_store.clone());

    let result = build(
        &sample_basic_normalized_ir_with_cas_media(&media_store, "sample.jpg", b"hello"),
        &sample_writer_policy(),
        &sample_build_context(false),
        &target,
    )
    .unwrap();

    assert_eq!(result.result_status, "success");
    assert_eq!(
        fs::read(root.join("staging/media/sample.jpg")).unwrap(),
        b"hello"
    );
}

#[test]
fn staging_materializes_media_from_cas_not_inline_payload() {
    let root = unique_artifact_root("cas-staging");
    let media_store = root.join("media-store");
    let normalized = sample_basic_normalized_ir_with_cas_media(&media_store, "hello.txt", b"hello");
    let target = BuildArtifactTarget::new(root.clone(), "artifacts/phase3/cas-staging")
        .with_media_store_dir(media_store.clone());

    let result = build(
        &normalized,
        &sample_writer_policy(),
        &sample_build_context(false),
        &target,
    )
    .unwrap();

    assert_eq!(result.result_status, "success");
    assert_eq!(
        fs::read(root.join("staging/media/hello.txt")).unwrap(),
        b"hello"
    );
}

#[test]
fn writer_reports_missing_cas_object_without_semantic_media_diagnostics() {
    let root = unique_artifact_root("missing-cas");
    let media_store = root.join("media-store");
    let normalized = sample_basic_normalized_ir_with_cas_media(&media_store, "hello.txt", b"hello");
    let target = BuildArtifactTarget::new(root.clone(), "artifacts/phase3/missing-cas")
        .with_media_store_dir(media_store.clone());
    fs::remove_dir_all(&media_store).unwrap();

    let result = build(
        &normalized,
        &sample_writer_policy(),
        &sample_build_context(false),
        &target,
    )
    .unwrap();

    let codes = result
        .diagnostics
        .items
        .iter()
        .map(|item| item.code.as_str())
        .collect::<Vec<_>>();
    assert_eq!(result.result_status, "error");
    assert!(codes.contains(&"MEDIA.CAS_OBJECT_MISSING"));
    assert!(!codes.contains(&"MEDIA.MISSING_REFERENCE"));
    assert!(!codes.contains(&"MEDIA.UNUSED_BINDING"));

    let media_diag = result
        .diagnostics
        .items
        .iter()
        .find(|item| item.code == "MEDIA.CAS_OBJECT_MISSING")
        .expect("missing CAS object diagnostic");
    assert_eq!(media_diag.domain.as_deref(), Some("media"));
    assert!(media_diag.path.as_deref().unwrap().contains("media-store"));
}

#[test]
fn writer_reports_corrupt_cas_size_mismatch_with_media_path() {
    let root = unique_artifact_root("cas-size-mismatch");
    let media_store = root.join("media-store");
    let normalized = sample_basic_normalized_ir_with_cas_media(&media_store, "hello.txt", b"hello");
    let object_path =
        authoring_core::object_store_path(&media_store, &normalized.media_objects[0].blake3)
            .unwrap();
    fs::write(&object_path, b"hello-too-long").unwrap();
    let staged_media_path = seed_staged_media(&root, "hello.txt", b"previous");
    let target = BuildArtifactTarget::new(root.clone(), "artifacts/phase3/cas-size-mismatch")
        .with_media_store_dir(media_store.clone());

    let result = build(
        &normalized,
        &sample_writer_policy(),
        &sample_build_context(false),
        &target,
    )
    .unwrap();

    assert_media_error_path(&result, "MEDIA.CAS_OBJECT_SIZE_MISMATCH", &object_path);
    assert_staged_media_preserved(&staged_media_path, b"previous");
}

#[test]
fn writer_reports_corrupt_cas_blake3_mismatch_with_media_path() {
    let root = unique_artifact_root("cas-blake3-mismatch");
    let media_store = root.join("media-store");
    let normalized = sample_basic_normalized_ir_with_cas_media(&media_store, "hello.txt", b"hello");
    let object_path =
        authoring_core::object_store_path(&media_store, &normalized.media_objects[0].blake3)
            .unwrap();
    fs::write(&object_path, b"hullo").unwrap();
    let staged_media_path = seed_staged_media(&root, "hello.txt", b"previous");
    let target = BuildArtifactTarget::new(root.clone(), "artifacts/phase3/cas-blake3-mismatch")
        .with_media_store_dir(media_store.clone());

    let result = build(
        &normalized,
        &sample_writer_policy(),
        &sample_build_context(false),
        &target,
    )
    .unwrap();

    assert_media_error_path(&result, "MEDIA.CAS_OBJECT_BLAKE3_MISMATCH", &object_path);
    assert_staged_media_preserved(&staged_media_path, b"previous");
}

#[test]
fn writer_reports_corrupt_cas_sha1_mismatch_with_media_path() {
    let root = unique_artifact_root("cas-sha1-mismatch");
    let media_store = root.join("media-store");
    let mut normalized =
        sample_basic_normalized_ir_with_cas_media(&media_store, "hello.txt", b"hello");
    let object_path =
        authoring_core::object_store_path(&media_store, &normalized.media_objects[0].blake3)
            .unwrap();
    normalized.media_objects[0].sha1 = "0".repeat(40);
    let staged_media_path = seed_staged_media(&root, "hello.txt", b"previous");
    let target = BuildArtifactTarget::new(root.clone(), "artifacts/phase3/cas-sha1-mismatch")
        .with_media_store_dir(media_store.clone());

    let result = build(
        &normalized,
        &sample_writer_policy(),
        &sample_build_context(false),
        &target,
    )
    .unwrap();

    assert_media_error_path(&result, "MEDIA.CAS_OBJECT_SHA1_MISMATCH", &object_path);
    assert_staged_media_preserved(&staged_media_path, b"previous");
}

#[cfg(unix)]
#[test]
fn cas_copy_verifies_exact_bytes_written_to_staging_in_single_pass() {
    let root = unique_artifact_root("cas-copy-single-pass");
    let media_store = root.join("media-store");
    let normalized = sample_basic_normalized_ir_with_cas_media(&media_store, "hello.txt", b"hello");
    let object = normalized.media_objects[0].clone();
    let object_path = authoring_core::object_store_path(&media_store, &object.blake3).unwrap();
    fs::remove_file(&object_path).unwrap();
    let status = std::process::Command::new("mkfifo")
        .arg(&object_path)
        .status()
        .unwrap();
    assert!(status.success());

    let output_dir = root.join("staging/media");
    fs::create_dir_all(&output_dir).unwrap();
    let output_path = output_dir.join("hello.txt");

    let (good_done_tx, good_done_rx) = std::sync::mpsc::channel();
    let good_object_path = object_path.clone();
    let good_writer = std::thread::spawn(move || {
        let mut fifo = fs::OpenOptions::new()
            .write(true)
            .open(&good_object_path)
            .unwrap();
        fifo.write_all(b"hello").unwrap();
        drop(fifo);
        good_done_tx.send(()).unwrap();
    });

    let copy_media_store = media_store.clone();
    let copy_object = object.clone();
    let copy_output_path = output_path.clone();
    let (copy_done_tx, copy_done_rx) = std::sync::mpsc::channel();
    let copy_thread = std::thread::spawn(move || {
        let result = writer_core::media::copy_verified_cas_object_to_path(
            &copy_media_store,
            &copy_object,
            &copy_output_path,
        );
        copy_done_tx.send(result).unwrap();
    });

    good_done_rx
        .recv_timeout(std::time::Duration::from_secs(5))
        .unwrap();

    let mut bad_writer = None;
    let result = match copy_done_rx.recv_timeout(std::time::Duration::from_secs(1)) {
        Ok(result) => result,
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
            let bad_object_path = object_path.clone();
            bad_writer = Some(std::thread::spawn(move || {
                let mut fifo = fs::OpenOptions::new()
                    .write(true)
                    .open(&bad_object_path)
                    .unwrap();
                fifo.write_all(b"hullo").unwrap();
                drop(fifo);
            }));
            copy_done_rx
                .recv_timeout(std::time::Duration::from_secs(5))
                .unwrap()
        }
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
            panic!("copy thread disconnected before returning result")
        }
    };
    assert!(result.is_ok(), "{result:?}");

    copy_thread.join().unwrap();
    good_writer.join().unwrap();
    if let Some(bad_writer) = bad_writer {
        bad_writer.join().unwrap();
    }
    assert_eq!(fs::read(&output_path).unwrap(), b"hello");
}

#[test]
fn writer_rejects_cross_field_media_invariant_violations() {
    let root = unique_artifact_root("media-invariant");
    let media_store = root.join("media-store");
    let mut normalized =
        sample_basic_normalized_ir_with_cas_media(&media_store, "hello.txt", b"hello");
    normalized.media_objects[0].object_ref = "media://blake3/not-the-object-hash".into();
    normalized
        .media_bindings
        .push(authoring_core::MediaBinding {
            id: "media:other".into(),
            export_filename: "other.txt".into(),
            object_id: "obj:blake3:missing".into(),
        });
    let target = BuildArtifactTarget::new(root.clone(), "artifacts/phase3/media-invariant")
        .with_media_store_dir(media_store);

    let result = build(
        &normalized,
        &sample_writer_policy(),
        &sample_build_context(false),
        &target,
    )
    .unwrap();

    let codes = result
        .diagnostics
        .items
        .iter()
        .map(|item| item.code.as_str())
        .collect::<Vec<_>>();
    assert_eq!(result.result_status, "invalid");
    assert!(codes.contains(&"MEDIA.INVALID_MEDIA_OBJECT_INVARIANT"));
    assert!(codes.contains(&"MEDIA.MEDIA_OBJECT_MISSING"));
}

#[test]
fn writer_rejects_malformed_media_invariant_shapes() {
    let root = unique_artifact_root("media-invariant-shape");
    let media_store = root.join("media-store");
    let mut normalized =
        sample_basic_normalized_ir_with_cas_media(&media_store, "hello.txt", b"hello");
    normalized.media_objects[0].blake3 = normalized.media_objects[0].blake3.to_uppercase();
    normalized.media_objects[0].id = format!("obj:blake3:{}", normalized.media_objects[0].blake3);
    normalized.media_objects[0].object_ref =
        format!("media://blake3/{}", normalized.media_objects[0].blake3);
    normalized.media_objects[0].sha1 = normalized.media_objects[0].sha1.to_uppercase();
    normalized.media_objects[0].mime.clear();
    normalized.media_bindings[0].object_id = "obj:blake3:not-lowercase-hex".into();
    let target = BuildArtifactTarget::new(root.clone(), "artifacts/phase3/media-invariant-shape")
        .with_media_store_dir(media_store);

    let result = build(
        &normalized,
        &sample_writer_policy(),
        &sample_build_context(false),
        &target,
    )
    .unwrap();

    let codes = result
        .diagnostics
        .items
        .iter()
        .map(|item| item.code.as_str())
        .collect::<Vec<_>>();
    assert_eq!(result.result_status, "invalid");
    assert!(codes.contains(&"MEDIA.INVALID_MEDIA_OBJECT_INVARIANT"));
    assert!(codes.contains(&"MEDIA.INVALID_MEDIA_BINDING_INVARIANT"));
}

#[cfg(unix)]
#[test]
fn staging_media_copy_replaces_existing_symlink_without_following_it() {
    let root = unique_artifact_root("media-symlink-replace");
    let media_store = root.join("media-store");
    let normalized = sample_basic_normalized_ir_with_cas_media(&media_store, "hello.txt", b"hello");
    let target = BuildArtifactTarget::new(root.clone(), "artifacts/phase3/media-symlink-replace")
        .with_media_store_dir(media_store);
    let media_dir = root.join("staging/media");
    fs::create_dir_all(&media_dir).unwrap();
    let outside = root.join("outside.txt");
    fs::write(&outside, b"outside").unwrap();
    std::os::unix::fs::symlink(&outside, media_dir.join("hello.txt")).unwrap();

    let result = build(
        &normalized,
        &sample_writer_policy(),
        &sample_build_context(false),
        &target,
    )
    .unwrap();

    assert_eq!(result.result_status, "success");
    assert_eq!(fs::read(&outside).unwrap(), b"outside");
    assert_eq!(fs::read(media_dir.join("hello.txt")).unwrap(), b"hello");
    assert!(!fs::symlink_metadata(media_dir.join("hello.txt"))
        .unwrap()
        .file_type()
        .is_symlink());
}

#[test]
fn build_rejects_media_filenames_that_escape_staging_media_root() {
    let root = unique_artifact_root("media-traversal");
    let media_store = root.join("media-store");
    let target = BuildArtifactTarget::new(root.clone(), "artifacts/phase3/media-traversal")
        .with_media_store_dir(media_store.clone());
    let mut normalized =
        sample_basic_normalized_ir_with_cas_media(&media_store, "sample.jpg", b"hello");
    normalized.media_bindings[0].export_filename = "../escape.jpg".into();
    normalized.notes[0]
        .fields
        .insert("Back".into(), r#"<img src="../escape.jpg">"#.into());

    let result = build(
        &normalized,
        &sample_writer_policy(),
        &sample_build_context(false),
        &target,
    )
    .expect("build should surface staging failure as error result");

    assert_eq!(result.result_status, "invalid");
    assert_eq!(result.diagnostics.items[0].code, "MEDIA.UNSAFE_FILENAME");
    assert!(result.diagnostics.items[0]
        .summary
        .contains("media filename"));
    assert!(!root.join("staging/escape.jpg").exists());
    assert!(!root.join("escape.jpg").exists());
}

#[test]
fn build_preserves_bundled_media_entries() {
    let root = unique_artifact_root("bundled-media");
    let media_store = root.join("media-store");
    let target = BuildArtifactTarget::new(root.clone(), "artifacts/phase3/bundled-media")
        .with_media_store_dir(media_store.clone());

    let result = build(
        &sample_basic_normalized_ir_with_cas_media(&media_store, "sample.jpg", b"hello"),
        &sample_writer_policy(),
        &sample_build_context(false),
        &target,
    )
    .unwrap();

    assert_eq!(result.result_status, "success");

    let manifest_json = fs::read_to_string(root.join("staging/manifest.json")).unwrap();
    let manifest: serde_json::Value = serde_json::from_str(&manifest_json).unwrap();
    assert_eq!(
        manifest["normalized_ir"]["media_objects"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        manifest["normalized_ir"]["media_bindings"][0]["export_filename"],
        serde_json::json!("sample.jpg")
    );
    assert_eq!(
        manifest["normalized_ir"]["media_objects"][0]["mime"],
        serde_json::json!("text/plain")
    );
}

#[test]
fn build_materializes_image_occlusion_apkg_into_caller_owned_root() {
    let root = unique_artifact_root("image-occlusion");
    let media_store = root.join("media-store");
    let target = BuildArtifactTarget::new(root.clone(), "artifacts/phase3/image-occlusion")
        .with_media_store_dir(media_store.clone());

    let result = build(
        &sample_image_occlusion_normalized_ir(&media_store),
        &sample_writer_policy(),
        &sample_build_context(true),
        &target,
    )
    .unwrap();

    assert_eq!(result.result_status, "success");
    assert_eq!(
        result.apkg_ref.as_deref(),
        Some("artifacts/phase3/image-occlusion/package.apkg")
    );
    assert!(result
        .package_fingerprint
        .as_deref()
        .unwrap()
        .starts_with("package:"));

    let apkg_path = root.join("package.apkg");
    assert!(apkg_path.exists(), "expected caller-owned apkg artifact");

    let mut archive = open_zip(&apkg_path);
    let names = archive_names(&mut archive);
    for expected in [
        "meta",
        "collection.anki21b",
        "collection.anki2",
        "media",
        "0",
    ] {
        assert!(
            names.contains(expected),
            "missing expected apkg entry {expected}: {names:?}"
        );
    }

    assert_eq!(
        zstd::stream::decode_all(read_zip_entry_bytes(&mut archive, "0").as_slice()).unwrap(),
        b"hello"
    );

    let meta = decode_package_metadata(read_zip_entry_bytes(&mut archive, "meta"));
    assert!(meta.version > 0);

    let latest_collection = zstd::stream::decode_all(
        read_zip_entry_bytes(&mut archive, "collection.anki21b").as_slice(),
    )
    .expect("decode latest collection");
    assert!(
        latest_collection.starts_with(b"SQLite format 3"),
        "latest collection should be a SQLite database"
    );
    assert_latest_collection_has_required_system_tables(&latest_collection);
    assert!(
        read_zip_entry_bytes(&mut archive, "collection.anki2").starts_with(b"SQLite format 3"),
        "legacy dummy collection should be a SQLite database"
    );

    let media_entries = decode_media_entries(
        zstd::stream::decode_all(read_zip_entry_bytes(&mut archive, "media").as_slice()).unwrap(),
    );
    assert_eq!(media_entries.entries.len(), 1);
    assert_eq!(media_entries.entries[0].name, "occlusion.png");
    assert_eq!(media_entries.entries[0].size, 5);
}

#[test]
fn exported_apkg_media_entries_do_not_emit_removed_tag4_legacy_filename() {
    let root = unique_artifact_root("media-map-wire-shape");
    let media_store = root.join("media-store");
    let target = BuildArtifactTarget::new(root.clone(), "artifacts/phase3/media-map-wire-shape")
        .with_media_store_dir(media_store.clone());

    build(
        &sample_basic_normalized_ir_with_cas_media(&media_store, "sample.jpg", b"hello"),
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

#[test]
fn apkg_media_entries_follow_export_filename_then_media_id_order() {
    let root = unique_artifact_root("apkg-order");
    let media_store = root.join("media-store");
    let mut normalized = sample_basic_normalized_ir_with_cas_media(&media_store, "b.txt", b"b");
    let second = sample_basic_normalized_ir_with_cas_media(&media_store, "a.txt", b"a");
    normalized.media_objects.extend(second.media_objects);
    normalized.media_bindings.extend(second.media_bindings);
    normalized.media_bindings[0].id = "media:b".into();
    normalized.media_bindings[1].id = "media:a".into();
    normalized.media_bindings.sort_by(|left, right| {
        left.export_filename
            .as_bytes()
            .cmp(right.export_filename.as_bytes())
            .then_with(|| left.id.as_bytes().cmp(right.id.as_bytes()))
    });
    let target = BuildArtifactTarget::new(root.clone(), "artifacts/phase3/apkg-order")
        .with_media_store_dir(media_store);

    build(
        &normalized,
        &sample_writer_policy(),
        &sample_build_context(true),
        &target,
    )
    .unwrap();

    let mut archive = open_zip(&root.join("package.apkg"));
    let media_entries = decode_media_entries(
        zstd::stream::decode_all(read_zip_entry_bytes(&mut archive, "media").as_slice()).unwrap(),
    );
    assert_eq!(media_entries.entries[0].name, "a.txt");
    assert_eq!(media_entries.entries[1].name, "b.txt");
}

#[test]
fn apkg_writes_duplicate_object_once_per_export_filename() {
    let root = unique_artifact_root("apkg-deduped-object");
    let media_store = root.join("media-store");
    let mut normalized = sample_basic_normalized_ir_with_cas_media(&media_store, "a.txt", b"same");
    normalized
        .media_bindings
        .push(authoring_core::MediaBinding {
            id: "media:b".into(),
            export_filename: "b.txt".into(),
            object_id: normalized.media_objects[0].id.clone(),
        });
    normalized.media_bindings.sort_by(|left, right| {
        left.export_filename
            .as_bytes()
            .cmp(right.export_filename.as_bytes())
            .then_with(|| left.id.as_bytes().cmp(right.id.as_bytes()))
    });
    let target = BuildArtifactTarget::new(root.clone(), "artifacts/phase3/apkg-deduped-object")
        .with_media_store_dir(media_store);

    build(
        &normalized,
        &sample_writer_policy(),
        &sample_build_context(true),
        &target,
    )
    .unwrap();

    let mut archive = open_zip(&root.join("package.apkg"));
    let names = archive_names(&mut archive);
    assert!(names.contains("0"));
    assert!(names.contains("1"));
    assert_eq!(
        zstd::stream::decode_all(read_zip_entry_bytes(&mut archive, "0").as_slice()).unwrap(),
        b"same"
    );
    assert_eq!(
        zstd::stream::decode_all(read_zip_entry_bytes(&mut archive, "1").as_slice()).unwrap(),
        b"same"
    );
    let media_entries = decode_media_entries(
        zstd::stream::decode_all(read_zip_entry_bytes(&mut archive, "media").as_slice()).unwrap(),
    );
    assert_eq!(media_entries.entries[0].name, "a.txt");
    assert_eq!(media_entries.entries[1].name, "b.txt");
}

#[test]
fn emit_apkg_reads_media_from_cas_without_staging_media_dir() {
    let root = unique_artifact_root("apkg-cas-source");
    let media_store = root.join("media-store");
    let target = BuildArtifactTarget::new(root.clone(), "artifacts/phase3/apkg-cas-source")
        .with_media_store_dir(media_store.clone());
    let normalized =
        sample_basic_normalized_ir_with_cas_media(&media_store, "sample.jpg", b"hello");
    let package = StagingPackage::from_normalized(
        &normalized,
        &sample_writer_policy(),
        &sample_build_context(true),
    )
    .unwrap();
    let materialized = package.materialize(&target).unwrap();
    let staging_media_dir = materialized.manifest_path.parent().unwrap().join("media");
    fs::remove_dir_all(&staging_media_dir).unwrap();

    let apkg = writer_core::apkg::emit_apkg(&materialized, &target).unwrap();

    let mut archive = open_zip(&apkg.apkg_path);
    assert_eq!(
        zstd::stream::decode_all(read_zip_entry_bytes(&mut archive, "0").as_slice()).unwrap(),
        b"hello"
    );
}

#[test]
fn latest_collection_derives_sfld_and_csum_from_first_notetype_field() {
    let root = unique_artifact_root("note-storage-first-field");
    let target =
        BuildArtifactTarget::new(root.clone(), "artifacts/phase3/note-storage-first-field");

    build(
        &sample_basic_normalized_ir(),
        &sample_writer_policy(),
        &sample_build_context(true),
        &target,
    )
    .unwrap();

    let conn = latest_collection_from_built_apkg(&root);
    let row: (String, String, u32) = conn
        .query_row(
            "select flds, cast(sfld as text), csum from notes where guid = 'note-1'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
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
        .query_row(
            "select cast(sfld as text), csum from notes where guid = 'note-1'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();

    assert_eq!(row.0, "front");
    assert_eq!(row.1, 460_909_371);
}

#[test]
fn latest_collection_ignores_script_style_and_comment_bodies_for_sfld_and_csum() {
    let root = unique_artifact_root("note-storage-html-blocks");
    let target =
        BuildArtifactTarget::new(root.clone(), "artifacts/phase3/note-storage-html-blocks");
    let mut normalized = sample_basic_normalized_ir();
    normalized.notes[0].fields.insert(
        "Front".into(),
        "<script>ignored()</script><style>.ignored{}</style><!-- hidden <b>noise</b> --><b>front</b>"
            .into(),
    );

    build(
        &normalized,
        &sample_writer_policy(),
        &sample_build_context(true),
        &target,
    )
    .unwrap();

    let conn = latest_collection_from_built_apkg(&root);
    let row: (String, u32) = conn
        .query_row(
            "select cast(sfld as text), csum from notes where guid = 'note-1'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();

    assert_eq!(row.0, "front");
    assert_eq!(row.1, 460_909_371);
}

#[test]
fn latest_collection_preserves_media_filename_with_case_insensitive_spaced_attr() {
    let root = unique_artifact_root("note-storage-media-attr");
    let media_store = root.join("media-store");
    let target = BuildArtifactTarget::new(root.clone(), "artifacts/phase3/note-storage-media-attr")
        .with_media_store_dir(media_store.clone());
    let mut normalized =
        sample_basic_normalized_ir_with_cas_media(&media_store, "sample.jpg", b"hello");
    normalized.notes[0]
        .fields
        .insert("Front".into(), r#"<IMG SRC = "sample.jpg">"#.into());

    build(
        &normalized,
        &sample_writer_policy(),
        &sample_build_context(true),
        &target,
    )
    .unwrap();

    let conn = latest_collection_from_built_apkg(&root);
    let row: (String, u32) = conn
        .query_row(
            "select cast(sfld as text), csum from notes where guid = 'note-1'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();

    assert_eq!(row.0, " sample.jpg ");
    assert_eq!(row.1, 1_786_670_956);
}

#[test]
fn latest_collection_preserves_media_filename_when_quoted_attr_contains_gt() {
    let root = unique_artifact_root("note-storage-media-quoted-gt");
    let media_store = root.join("media-store");
    let target = BuildArtifactTarget::new(
        root.clone(),
        "artifacts/phase3/note-storage-media-quoted-gt",
    )
    .with_media_store_dir(media_store.clone());
    let mut normalized =
        sample_basic_normalized_ir_with_cas_media(&media_store, "sample.jpg", b"hello");
    normalized.notes[0].fields.insert(
        "Front".into(),
        r#"<img data-note="1 > 0" src="sample.jpg">"#.into(),
    );

    build(
        &normalized,
        &sample_writer_policy(),
        &sample_build_context(true),
        &target,
    )
    .unwrap();

    let conn = latest_collection_from_built_apkg(&root);
    let row: (String, u32) = conn
        .query_row(
            "select cast(sfld as text), csum from notes where guid = 'note-1'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();

    assert_eq!(row.0, " sample.jpg ");
    assert_eq!(row.1, 1_786_670_956);
}

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
        .query_row("select mod from notes where guid = 'note-1'", [], |row| {
            row.get(0)
        })
        .unwrap();

    assert_eq!(mtime_secs, 1_777_777_777);
}

#[test]
fn build_rejects_non_positive_explicit_normalized_note_mtime() {
    let root = unique_artifact_root("note-storage-invalid-mtime");
    let target =
        BuildArtifactTarget::new(root.clone(), "artifacts/phase3/note-storage-invalid-mtime");
    let mut normalized = sample_basic_normalized_ir();
    normalized.notes[0].mtime_secs = Some(0);

    let result = build(
        &normalized,
        &sample_writer_policy(),
        &sample_build_context(false),
        &target,
    )
    .unwrap();

    assert_eq!(result.result_status, "invalid");
    let diag = result
        .diagnostics
        .items
        .iter()
        .find(|item| item.code == "PHASE3.INVALID_NOTE_MTIME")
        .expect("invalid note mtime diagnostic");
    assert_eq!(diag.level, "error");
    assert_eq!(diag.domain.as_deref(), Some("notes"));
    assert_eq!(diag.path.as_deref(), Some("notes[0].mtime_secs"));
    assert_eq!(diag.target_selector.as_deref(), Some("note[id='note-1']"));
    assert!(
        diag.summary.contains("mtime_secs") && diag.summary.contains("positive"),
        "unexpected summary: {}",
        diag.summary
    );
}

#[test]
fn build_rejects_image_occlusion_notetype_that_drifts_from_source_grounded_shape() {
    let root = unique_artifact_root("image-occlusion-shape-drift");
    let media_store = root.join("media-store");
    let target = BuildArtifactTarget::new(root, "artifacts/phase3/image-occlusion-shape-drift");

    let mut normalized = sample_image_occlusion_normalized_ir(&media_store);
    normalized.notetypes[0].templates[0].answer_format = "{{Image}}".into();

    let result = build(
        &normalized,
        &sample_writer_policy(),
        &sample_build_context(false),
        &target,
    )
    .unwrap();

    assert_eq!(result.result_status, "invalid");
    let diag = result
        .diagnostics
        .items
        .iter()
        .find(|item| item.code == "PHASE3.STOCK_NOTETYPE_SHAPE_MISMATCH")
        .expect("stock mismatch diagnostic");
    assert_eq!(diag.domain.as_deref(), Some("notetypes"));
    assert_eq!(
        diag.path.as_deref(),
        Some("notetypes[0].templates[0].answer_format")
    );
    assert_eq!(
        diag.target_selector.as_deref(),
        Some("notetype[id='io-main']")
    );
}

#[test]
fn build_apkg_package_fingerprint_is_stable_across_roots() {
    let left_root = unique_artifact_root("image-occlusion-left");
    let right_root = unique_artifact_root("image-occlusion-right");
    let left_media_store = left_root.join("media-store");
    let right_media_store = right_root.join("media-store");
    let target_prefix = "artifacts/phase3/image-occlusion";

    let left = build(
        &sample_image_occlusion_normalized_ir(&left_media_store),
        &sample_writer_policy(),
        &sample_build_context(true),
        &BuildArtifactTarget::new(left_root, target_prefix).with_media_store_dir(left_media_store),
    )
    .unwrap();
    let right = build(
        &sample_image_occlusion_normalized_ir(&right_media_store),
        &sample_writer_policy(),
        &sample_build_context(true),
        &BuildArtifactTarget::new(right_root, target_prefix)
            .with_media_store_dir(right_media_store),
    )
    .unwrap();

    assert_eq!(left.apkg_ref, right.apkg_ref);
    assert_eq!(left.package_fingerprint, right.package_fingerprint);
}

#[test]
fn build_artifact_fingerprint_is_stable_across_roots() {
    let left_root = unique_artifact_root("fingerprint-left");
    let right_root = unique_artifact_root("fingerprint-right");

    let left = build(
        &sample_basic_normalized_ir(),
        &sample_writer_policy(),
        &sample_build_context(false),
        &BuildArtifactTarget::new(left_root, "artifacts/phase3/basic"),
    )
    .unwrap();
    let right = build(
        &sample_basic_normalized_ir(),
        &sample_writer_policy(),
        &sample_build_context(false),
        &BuildArtifactTarget::new(right_root, "artifacts/phase3/basic"),
    )
    .unwrap();

    assert_eq!(left.artifact_fingerprint, right.artifact_fingerprint);
}

#[test]
fn build_rejects_unknown_notetype_with_selector_and_path_diagnostics() {
    let root = unique_artifact_root("invalid");
    let target = BuildArtifactTarget::new(root, "artifacts/phase3/invalid");

    let mut normalized = sample_basic_normalized_ir();
    normalized.notes[0].notetype_id = "missing-main".into();

    let result = build(
        &normalized,
        &sample_writer_policy(),
        &sample_build_context(false),
        &target,
    )
    .unwrap();

    assert_eq!(result.result_status, "invalid");
    let diag = &result.diagnostics.items[0];
    assert_eq!(diag.code, "PHASE3.UNKNOWN_NOTETYPE_ID");
    assert_eq!(diag.domain.as_deref(), Some("notes"));
    assert_eq!(diag.path.as_deref(), Some("notes[0].notetype_id"));
    assert_eq!(diag.target_selector.as_deref(), Some("note[id='note-1']"));
}

#[test]
fn build_rejects_basic_notetype_that_drifts_from_source_grounded_shape() {
    let root = unique_artifact_root("basic-shape-drift");
    let target = BuildArtifactTarget::new(root, "artifacts/phase3/basic-shape-drift");

    let mut normalized = sample_basic_normalized_ir();
    normalized.notetypes[0].templates[0].answer_format = "{{Back}}".into();

    let result = build(
        &normalized,
        &sample_writer_policy(),
        &sample_build_context(false),
        &target,
    )
    .unwrap();

    assert_eq!(result.result_status, "invalid");
    let diag = result
        .diagnostics
        .items
        .iter()
        .find(|item| item.code == "PHASE3.STOCK_NOTETYPE_SHAPE_MISMATCH")
        .expect("stock mismatch diagnostic");
    assert_eq!(diag.domain.as_deref(), Some("notetypes"));
    assert_eq!(
        diag.path.as_deref(),
        Some("notetypes[0].templates[0].answer_format")
    );
    assert_eq!(
        diag.target_selector.as_deref(),
        Some("notetype[id='basic-main']")
    );
}

#[test]
fn build_rejects_cloze_notetype_that_drifts_from_source_grounded_css() {
    let root = unique_artifact_root("cloze-shape-drift");
    let target = BuildArtifactTarget::new(root, "artifacts/phase3/cloze-shape-drift");

    let mut normalized = sample_cloze_normalized_ir();
    normalized.notetypes[0].css.clear();

    let result = build(
        &normalized,
        &sample_writer_policy(),
        &sample_build_context(false),
        &target,
    )
    .unwrap();

    assert_eq!(result.result_status, "invalid");
    let diag = result
        .diagnostics
        .items
        .iter()
        .find(|item| item.code == "PHASE3.STOCK_NOTETYPE_SHAPE_MISMATCH")
        .expect("stock mismatch diagnostic");
    assert_eq!(diag.domain.as_deref(), Some("notetypes"));
    assert_eq!(diag.path.as_deref(), Some("notetypes[0].css"));
    assert_eq!(
        diag.target_selector.as_deref(),
        Some("notetype[id='cloze-main']")
    );
}

#[test]
fn build_rejects_unresolved_media_refs_when_behavior_is_fail() {
    let root = unique_artifact_root("media-fail");
    let target = BuildArtifactTarget::new(root, "artifacts/phase3/media-fail");

    let mut normalized = sample_basic_normalized_ir();
    normalized.notes[0]
        .fields
        .insert("Back".into(), r#"<img src="missing.png">"#.into());

    let result = build(
        &normalized,
        &sample_writer_policy(),
        &sample_build_context(false),
        &target,
    )
    .unwrap();

    assert_eq!(result.result_status, "invalid");
    let diag = result
        .diagnostics
        .items
        .iter()
        .find(|item| item.code == "PHASE3.UNRESOLVED_MEDIA_REFERENCE")
        .expect("unresolved media diagnostic");
    assert_eq!(diag.level, "error");
    assert_eq!(diag.domain.as_deref(), Some("notes"));
    assert_eq!(diag.path.as_deref(), Some(r#"notes[0].fields["Back"]"#));
    assert_eq!(diag.target_selector.as_deref(), Some("note[id='note-1']"));
}

#[test]
fn build_rejects_unquoted_src_media_refs_when_behavior_is_fail() {
    let root = unique_artifact_root("media-fail-unquoted");
    let target = BuildArtifactTarget::new(root, "artifacts/phase3/media-fail-unquoted");

    let mut normalized = sample_basic_normalized_ir();
    normalized.notes[0]
        .fields
        .insert("Back".into(), "<img src=missing.png>".into());

    let result = build(
        &normalized,
        &sample_writer_policy(),
        &sample_build_context(false),
        &target,
    )
    .unwrap();

    assert_eq!(result.result_status, "invalid");
    let diag = result
        .diagnostics
        .items
        .iter()
        .find(|item| item.code == "PHASE3.UNRESOLVED_MEDIA_REFERENCE")
        .expect("unresolved media diagnostic");
    assert_eq!(diag.path.as_deref(), Some(r#"notes[0].fields["Back"]"#));
}

#[test]
fn build_warns_on_unresolved_media_refs_when_behavior_is_warn() {
    let root = unique_artifact_root("media-warn");
    let target = BuildArtifactTarget::new(root.clone(), "artifacts/phase3/media-warn");

    let mut normalized = sample_basic_normalized_ir();
    normalized.notes[0]
        .fields
        .insert("Back".into(), "[sound:missing.mp3]".into());

    let mut build_context = sample_build_context(false);
    build_context.unresolved_asset_behavior = "warn".into();

    let result = build(
        &normalized,
        &sample_writer_policy(),
        &build_context,
        &target,
    )
    .unwrap();

    assert_eq!(result.result_status, "success");
    assert_eq!(
        result.staging_ref.as_deref(),
        Some("artifacts/phase3/media-warn/staging/manifest.json")
    );
    let diag = result
        .diagnostics
        .items
        .iter()
        .find(|item| item.code == "PHASE3.UNRESOLVED_MEDIA_REFERENCE")
        .expect("warning diagnostic");
    assert_eq!(diag.level, "warning");
    assert!(root.join("staging/manifest.json").exists());
}

#[test]
fn build_accepts_html_entity_encoded_media_refs_when_payload_exists() {
    let root = unique_artifact_root("media-encoded");
    let media_store = root.join("media-store");
    let target = BuildArtifactTarget::new(root, "artifacts/phase3/media-encoded")
        .with_media_store_dir(media_store.clone());

    let result = build(
        &sample_basic_normalized_ir_with_encoded_media_ref(&media_store),
        &sample_writer_policy(),
        &sample_build_context(false),
        &target,
    )
    .unwrap();

    assert_eq!(result.result_status, "success");
    assert!(!result
        .diagnostics
        .items
        .iter()
        .any(|item| item.code == "PHASE3.UNRESOLVED_MEDIA_REFERENCE"));
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

fn sample_cloze_normalized_ir() -> NormalizedIr {
    NormalizedIr {
        kind: "normalized-ir".into(),
        schema_version: "0.1.0".into(),
        document_id: "demo-doc".into(),
        resolved_identity: "document:demo-doc".into(),
        notetypes: vec![resolved_stock_notetype("cloze-main", "cloze", "Cloze")],
        notes: vec![NormalizedNote {
            id: "note-1".into(),
            notetype_id: "cloze-main".into(),
            deck_name: "Default".into(),
            fields: BTreeMap::from([
                (
                    "Text".into(),
                    "The capital of France is {{c1::Paris}}.".into(),
                ),
                ("Back Extra".into(), "".into()),
            ]),
            tags: vec!["demo".into()],
            mtime_secs: None,
        }],
        media_objects: vec![],
        media_bindings: vec![],
        media_references: vec![],
    }
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

fn assert_media_error_path(result: &PackageBuildResult, code: &str, expected_path: &Path) {
    assert_eq!(result.result_status, "error");
    let diagnostic = result
        .diagnostics
        .items
        .iter()
        .find(|item| item.code == code)
        .unwrap_or_else(|| {
            panic!(
                "expected diagnostic {code:?}: {:?}",
                result.diagnostics.items
            )
        });
    assert_eq!(diagnostic.domain.as_deref(), Some("media"));
    assert_eq!(
        diagnostic.path.as_deref(),
        Some(expected_path.to_string_lossy().as_ref())
    );
}

fn seed_staged_media(root: &Path, filename: &str, bytes: &[u8]) -> PathBuf {
    let media_path = root.join("staging/media").join(filename);
    fs::create_dir_all(media_path.parent().unwrap()).unwrap();
    fs::write(&media_path, bytes).unwrap();
    media_path
}

fn assert_staged_media_preserved(media_path: &Path, expected: &[u8]) {
    assert_eq!(fs::read(media_path).unwrap(), expected);
    let filenames = fs::read_dir(media_path.parent().unwrap())
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect::<Vec<_>>();
    assert_eq!(
        filenames,
        vec![media_path.file_name().unwrap().to_os_string()]
    );
}

fn sample_basic_normalized_ir_with_encoded_media_ref(media_store: &Path) -> NormalizedIr {
    let mut normalized =
        sample_basic_normalized_ir_with_cas_media(media_store, "a&b.jpg", b"hello");
    normalized.notes[0]
        .fields
        .insert("Back".into(), r#"<img src="a&amp;b.jpg">"#.into());
    normalized
}

fn sample_image_occlusion_normalized_ir(media_store: &Path) -> NormalizedIr {
    let mut normalized = NormalizedIr {
        kind: "normalized-ir".into(),
        schema_version: "0.1.0".into(),
        document_id: "demo-doc".into(),
        resolved_identity: "document:demo-doc".into(),
        notetypes: vec![resolved_stock_notetype(
            "io-main",
            "image_occlusion",
            "Image Occlusion",
        )],
        notes: vec![NormalizedNote {
            id: "note-io-1".into(),
            notetype_id: "io-main".into(),
            deck_name: "Default".into(),
            fields: BTreeMap::from([
                ("Occlusion".into(), "{{c1::Mask 1}}".into()),
                ("Image".into(), r#"<img src="occlusion.png">"#.into()),
                ("Header".into(), "Header".into()),
                ("Back Extra".into(), "Extra".into()),
                ("Comments".into(), "Comments".into()),
            ]),
            tags: vec!["demo".into()],
            mtime_secs: None,
        }],
        media_objects: vec![],
        media_bindings: vec![],
        media_references: vec![],
    };
    let media = sample_basic_normalized_ir_with_cas_media(media_store, "occlusion.png", b"hello");
    normalized.media_objects = media.media_objects;
    normalized.media_bindings = media.media_bindings;
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

fn open_zip(path: &Path) -> zip::ZipArchive<File> {
    let file = File::open(path).unwrap();
    zip::ZipArchive::new(file).unwrap()
}

fn archive_names(archive: &mut zip::ZipArchive<File>) -> std::collections::BTreeSet<String> {
    (0..archive.len())
        .map(|index| archive.by_index(index).unwrap().name().to_string())
        .collect()
}

fn read_zip_entry_bytes(archive: &mut zip::ZipArchive<File>, name: &str) -> Vec<u8> {
    let mut file = archive.by_name(name).unwrap();
    let mut buf = vec![];
    file.read_to_end(&mut buf).unwrap();
    buf
}

fn latest_collection_from_built_apkg(root: &Path) -> Connection {
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

fn decode_package_metadata(bytes: Vec<u8>) -> TestPackageMetadata {
    TestPackageMetadata::decode(bytes.as_slice()).unwrap()
}

fn decode_media_entries(bytes: Vec<u8>) -> TestMediaEntries {
    TestMediaEntries::decode(bytes.as_slice()).unwrap()
}

fn assert_legacy_models_use_schema11_shape(bytes: &[u8]) {
    let root = unique_artifact_root("legacy-models");
    let db_path = root.join("collection.anki2");
    fs::write(&db_path, bytes).unwrap();

    let conn = Connection::open(&db_path).unwrap();
    let models_json: String = conn
        .query_row("select models from col where id = 1", [], |row| row.get(0))
        .unwrap();
    let models: serde_json::Value = serde_json::from_str(&models_json).unwrap();
    let first_notetype = models
        .as_object()
        .and_then(|items| items.values().next())
        .expect("legacy models should contain one stock notetype");

    assert!(
        first_notetype
            .get("flds")
            .is_some_and(serde_json::Value::is_array),
        "legacy models should use schema11 field entries"
    );
    assert!(
        first_notetype
            .get("tmpls")
            .is_some_and(serde_json::Value::is_array),
        "legacy models should use schema11 template entries"
    );
}

fn assert_latest_collection_has_required_system_tables(bytes: &[u8]) {
    let root = unique_artifact_root("latest-system-tables");
    let db_path = root.join("collection.anki21b");
    fs::write(&db_path, bytes).unwrap();

    let conn = Connection::open(&db_path).unwrap();
    let table_names: std::collections::BTreeSet<String> = conn
        .prepare("select name from sqlite_master where type = 'table' order by name")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .map(|row| row.unwrap())
        .collect();

    for expected in ["config", "deck_config", "tags"] {
        assert!(
            table_names.contains(expected),
            "latest collection should include `{expected}` table: {table_names:?}"
        );
    }

    let schema_version: i64 = conn
        .query_row("select ver from col where id = 1", [], |row| row.get(0))
        .unwrap();
    assert_eq!(
        schema_version, 18,
        "latest collection should advertise schema V18"
    );

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

    let deck_blob_types: (String, String) = conn
        .query_row(
            "select typeof(common), typeof(kind) from decks where id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(
        deck_blob_types,
        ("blob".to_string(), "blob".to_string()),
        "latest decks rows should persist protobuf payloads as blob columns"
    );

    let field_config_types: Vec<String> = conn
        .prepare("select typeof(config) from fields order by ntid, ord")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .map(|row| row.unwrap())
        .collect();
    assert!(
        field_config_types.iter().all(|kind| kind == "blob"),
        "latest field config rows should persist protobuf payloads as blobs: {field_config_types:?}"
    );

    let template_config_types: Vec<String> = conn
        .prepare("select typeof(config) from templates order by ntid, ord")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .map(|row| row.unwrap())
        .collect();
    assert!(
        template_config_types.iter().all(|kind| kind == "blob"),
        "latest template config rows should persist protobuf payloads as blobs: {template_config_types:?}"
    );

    let deck_config_count: i64 = conn
        .query_row("select count(*) from deck_config", [], |row| row.get(0))
        .unwrap();
    assert!(
        deck_config_count >= 1,
        "latest collection should include at least one deck_config row"
    );

    let tags: std::collections::BTreeSet<String> = conn
        .prepare("select tag from tags order by tag")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .map(|row| row.unwrap())
        .collect();
    assert_eq!(tags, std::collections::BTreeSet::from(["demo".to_string()]));
}

#[derive(Clone, PartialEq, Message)]
struct TestPackageMetadata {
    #[prost(int32, tag = "1")]
    version: i32,
}

#[derive(Clone, PartialEq, Message)]
struct TestMediaEntries {
    #[prost(message, repeated, tag = "1")]
    entries: Vec<TestMediaEntry>,
}

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
