use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use authoring_core::stock::resolve_stock_notetype;
use authoring_core::{
    AuthoringNotetype, NormalizedIr, NormalizedMedia, NormalizedNote, NormalizedNotetype,
};
use writer_core::{
    build,
    build_context_ref, policy_ref, to_canonical_json, BuildContext, BuildDiagnosticItem,
    BuildDiagnostics, BuildArtifactTarget, DiffReport, InspectObservations, InspectReport,
    PackageBuildResult, VerificationGateRule, VerificationPolicy, WriterPolicy,
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
    assert_eq!(verification_policy_json["writer_fast_gate"]["minimum_comparison_status"], "complete");
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
    assert!(result.artifact_fingerprint.as_deref().unwrap().starts_with("artifact:"));
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
    assert!(result.artifact_fingerprint.as_deref().unwrap().starts_with("artifact:"));
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
    assert_eq!(fs::read(root.join("staging/media/sample.jpg")).unwrap(), b"hello");
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
    assert_eq!(diag.target_selector.as_deref(), Some("notetype[id='basic-main']"));
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
    assert_eq!(diag.target_selector.as_deref(), Some("notetype[id='cloze-main']"));
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
                ("Text".into(), "The capital of France is {{c1::Paris}}.".into()),
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
