use writer_core::{
    build_context_ref, policy_ref, to_canonical_json, BuildContext, BuildDiagnosticItem,
    BuildDiagnostics, DiffReport, InspectObservations, InspectReport, PackageBuildResult,
    VerificationGateRule, VerificationPolicy, WriterPolicy,
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
