use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use authoring_core::{NormalizedIr, NormalizedNote, NormalizedNotetype, NormalizedTemplate};
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
    assert_eq!(diag.target_selector.as_deref(), Some("notetype_id=missing-main"));
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
        notetypes: vec![NormalizedNotetype {
            id: "basic-main".into(),
            kind: "basic".into(),
            name: "Basic".into(),
            fields: vec!["Front".into(), "Back".into()],
            templates: vec![NormalizedTemplate {
                name: "Card 1".into(),
                question_format: "{{Front}}".into(),
                answer_format: "{{Back}}".into(),
            }],
            css: "".into(),
        }],
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
        notetypes: vec![NormalizedNotetype {
            id: "cloze-main".into(),
            kind: "cloze".into(),
            name: "Cloze".into(),
            fields: vec!["Text".into(), "Back Extra".into()],
            templates: vec![NormalizedTemplate {
                name: "Cloze".into(),
                question_format: "{{cloze:Text}}".into(),
                answer_format: "{{cloze:Text}}<br>\n{{Back Extra}}".into(),
            }],
            css: "".into(),
        }],
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
