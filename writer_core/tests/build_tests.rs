use authoring_core::stock::resolve_stock_notetype;
use authoring_core::{
    AuthoringNotetype, NormalizedIr, NormalizedMedia, NormalizedNote, NormalizedNotetype,
};
use prost::Message;
use rusqlite::Connection;
use std::collections::BTreeMap;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
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
    let target = BuildArtifactTarget::new(root.clone(), "artifacts/phase3/basic-apkg");
    let package = StagingPackage::from_normalized(
        &sample_basic_normalized_ir_with_media(),
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
    let target = BuildArtifactTarget {
        root_dir: root.clone(),
        stable_ref_prefix: "artifacts/phase3/basic".into(),
    };

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
    let target = BuildArtifactTarget {
        root_dir: root,
        stable_ref_prefix: "artifacts/phase3/html-entity-media".into(),
    };
    let mut normalized = sample_basic_normalized_ir_with_media();
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
    let target = BuildArtifactTarget {
        root_dir: root.clone(),
        stable_ref_prefix: "artifacts/phase3/cloze".into(),
    };

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
    let target = BuildArtifactTarget::new(root.clone(), "artifacts/phase3/basic-media");

    let result = build(
        &sample_basic_normalized_ir_with_media(),
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
fn build_materializes_image_occlusion_apkg_into_caller_owned_root() {
    let root = unique_artifact_root("image-occlusion");
    let target = BuildArtifactTarget::new(root.clone(), "artifacts/phase3/image-occlusion");

    let result = build(
        &sample_image_occlusion_normalized_ir(),
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
fn build_rejects_image_occlusion_notetype_that_drifts_from_source_grounded_shape() {
    let root = unique_artifact_root("image-occlusion-shape-drift");
    let target = BuildArtifactTarget::new(root, "artifacts/phase3/image-occlusion-shape-drift");

    let mut normalized = sample_image_occlusion_normalized_ir();
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
    let target_prefix = "artifacts/phase3/image-occlusion";

    let left = build(
        &sample_image_occlusion_normalized_ir(),
        &sample_writer_policy(),
        &sample_build_context(true),
        &BuildArtifactTarget::new(left_root, target_prefix),
    )
    .unwrap();
    let right = build(
        &sample_image_occlusion_normalized_ir(),
        &sample_writer_policy(),
        &sample_build_context(true),
        &BuildArtifactTarget::new(right_root, target_prefix),
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
        &BuildArtifactTarget {
            root_dir: left_root,
            stable_ref_prefix: "artifacts/phase3/basic".into(),
        },
    )
    .unwrap();
    let right = build(
        &sample_basic_normalized_ir(),
        &sample_writer_policy(),
        &sample_build_context(false),
        &BuildArtifactTarget {
            root_dir: right_root,
            stable_ref_prefix: "artifacts/phase3/basic".into(),
        },
    )
    .unwrap();

    assert_eq!(left.artifact_fingerprint, right.artifact_fingerprint);
}

#[test]
fn build_rejects_unknown_notetype_with_selector_and_path_diagnostics() {
    let root = unique_artifact_root("invalid");
    let target = BuildArtifactTarget {
        root_dir: root,
        stable_ref_prefix: "artifacts/phase3/invalid".into(),
    };

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
    let target = BuildArtifactTarget::new(root, "artifacts/phase3/media-encoded");

    let result = build(
        &sample_basic_normalized_ir_with_encoded_media_ref(),
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
        }],
        media: vec![],
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

fn sample_basic_normalized_ir_with_encoded_media_ref() -> NormalizedIr {
    let mut normalized = sample_basic_normalized_ir();
    normalized.notes[0]
        .fields
        .insert("Back".into(), r#"<img src="a&amp;b.jpg">"#.into());
    normalized.media.push(NormalizedMedia {
        filename: "a&b.jpg".into(),
        mime: "image/jpeg".into(),
        data_base64: "aGVsbG8=".into(),
    });
    normalized
}

fn sample_image_occlusion_normalized_ir() -> NormalizedIr {
    NormalizedIr {
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
        }],
        media: vec![NormalizedMedia {
            filename: "occlusion.png".into(),
            mime: "image/png".into(),
            data_base64: "aGVsbG8=".into(),
        }],
    }
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

fn open_zip(path: &PathBuf) -> zip::ZipArchive<File> {
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
struct TestMediaEntry {
    #[prost(string, tag = "1")]
    name: String,
    #[prost(uint32, tag = "2")]
    size: u32,
    #[prost(bytes, tag = "3")]
    sha1: Vec<u8>,
}
