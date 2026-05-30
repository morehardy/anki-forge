use std::path::PathBuf;
use std::time::Duration;

use anki_forge::build::{
    write_report_json_atomic, ApkgArtifact, BuildCounts, BuildFailureCause, BuildMetrics,
    BuildPolicyResult, BuildPolicyStatus, BuildReport, BuildReportJson, BuildStatus,
    ComparisonStatus, MediaSummary, RiskLevel, SerializableBuildReport,
};
use anki_forge::diagnostics::{Diagnostic, DiagnosticCode, ErrorCode, Severity, SourcePath};

#[test]
fn build_report_ensure_success_accepts_successful_artifact() {
    let report = BuildReport {
        artifact: Some(ApkgArtifact {
            path: PathBuf::from("out/spanish.apkg"),
        }),
        counts: BuildCounts {
            notes: 2,
            cards: 2,
            media: 0,
        },
        media: MediaSummary::default(),
        diagnostics: vec![],
        metrics: BuildMetrics {
            duration: Duration::from_millis(25),
        },
        inspect: None,
        previous_inspect: None,
        update_safety: None,
        comparison: ComparisonStatus::NotRequested,
        diff: None,
        risk: None,
        policy: BuildPolicyResult::default(),
        status: BuildStatus::Success,
    };

    report.ensure_success().expect("successful report");
    assert_eq!(report.warning_count(), 0);
    assert_eq!(report.diagnostic_codes(), Vec::<String>::new());
}

#[test]
fn build_report_ensure_success_rejects_error_diagnostic() {
    let report = BuildReport {
        artifact: Some(ApkgArtifact {
            path: PathBuf::from("out/spanish.apkg"),
        }),
        counts: BuildCounts {
            notes: 1,
            cards: 1,
            media: 0,
        },
        media: MediaSummary::default(),
        diagnostics: vec![Diagnostic {
            code: DiagnosticCode::new("MEDIA.MISSING_REFERENCE"),
            severity: Severity::Error,
            domain: None,
            stage: None,
            message: "missing media reference hola.mp3".into(),
            source: None,
            help: Some("register the media before adding the note".into()),
        }],
        metrics: BuildMetrics {
            duration: Duration::from_millis(1),
        },
        inspect: None,
        previous_inspect: None,
        update_safety: None,
        comparison: ComparisonStatus::NotRequested,
        diff: None,
        risk: None,
        policy: BuildPolicyResult::default(),
        status: BuildStatus::Invalid,
    };

    let err = report.ensure_success().expect_err("report should fail");
    assert_eq!(err.cause, BuildFailureCause::Diagnostics);
    assert_eq!(
        err.report.diagnostic_codes(),
        vec!["MEDIA.MISSING_REFERENCE"]
    );
}

#[test]
fn build_report_ensure_success_prefers_diagnostics_over_missing_artifact() {
    let report = BuildReport {
        artifact: None,
        counts: BuildCounts {
            notes: 1,
            cards: 0,
            media: 0,
        },
        media: MediaSummary::default(),
        diagnostics: vec![Diagnostic {
            code: DiagnosticCode::new("PROJECT.NORMALIZE_FAILED"),
            severity: Severity::Error,
            domain: None,
            stage: None,
            message: "normalization failed".into(),
            source: None,
            help: None,
        }],
        metrics: BuildMetrics {
            duration: Duration::from_millis(1),
        },
        inspect: None,
        previous_inspect: None,
        update_safety: None,
        comparison: ComparisonStatus::NotRequested,
        diff: None,
        risk: None,
        policy: BuildPolicyResult::default(),
        status: BuildStatus::Invalid,
    };

    let err = report.ensure_success().expect_err("report should fail");
    assert_eq!(err.cause, BuildFailureCause::Diagnostics);
}

#[test]
fn build_report_ensure_success_uses_status_precedence_over_policy_status() {
    let report = BuildReport {
        artifact: Some(ApkgArtifact {
            path: PathBuf::from("out/internal-error.apkg"),
        }),
        counts: BuildCounts {
            notes: 1,
            cards: 1,
            media: 0,
        },
        media: MediaSummary::default(),
        diagnostics: vec![],
        metrics: BuildMetrics {
            duration: Duration::from_millis(1),
        },
        inspect: None,
        previous_inspect: None,
        update_safety: None,
        comparison: ComparisonStatus::Complete,
        diff: None,
        risk: None,
        policy: BuildPolicyResult {
            status: BuildPolicyStatus::Blocked,
            threshold: Some(RiskLevel::High),
            highest_risk: Some(RiskLevel::Critical),
            blocking_findings: vec!["RISK.TEMPLATE_REMOVED".to_string()],
        },
        status: BuildStatus::Error,
    };

    let err = report.ensure_success().expect_err("report should fail");
    assert_eq!(err.cause, BuildFailureCause::Internal);
    assert_eq!(err.code(), ErrorCode::ProjectBuildInternal);
}

#[test]
fn build_error_without_diagnostics_uses_stable_cause_code() {
    let report = BuildReport {
        artifact: None,
        counts: BuildCounts::default(),
        media: MediaSummary::default(),
        diagnostics: vec![],
        metrics: BuildMetrics {
            duration: Duration::from_millis(1),
        },
        inspect: None,
        previous_inspect: None,
        update_safety: None,
        comparison: ComparisonStatus::NotRequested,
        diff: None,
        risk: None,
        policy: BuildPolicyResult::default(),
        status: BuildStatus::Success,
    };

    let err = report
        .ensure_success()
        .expect_err("missing artifact should fail");
    assert_eq!(err.cause, BuildFailureCause::MissingArtifact);
    assert_eq!(err.code(), ErrorCode::ProjectBuildMissingArtifact);
    assert_eq!(err.code().as_str(), "PROJECT.BUILD_MISSING_ARTIFACT");
}

#[test]
fn build_report_ensure_success_accepts_warning_diagnostics() {
    let report = BuildReport {
        artifact: Some(ApkgArtifact {
            path: PathBuf::from("out/spanish.apkg"),
        }),
        counts: BuildCounts {
            notes: 1,
            cards: 1,
            media: 1,
        },
        media: MediaSummary {
            objects: 1,
            bindings: 1,
            references: 0,
            missing_references: 0,
            unsafe_references: 0,
            unused_bindings: 1,
            unique_bytes: 31,
        },
        diagnostics: vec![Diagnostic {
            code: DiagnosticCode::new("MEDIA.UNUSED_BINDING"),
            severity: Severity::Warning,
            domain: None,
            stage: None,
            message: "registered media is not referenced.".into(),
            source: Some(SourcePath::new("project.media[\"unused.png\"]")),
            help: Some("Remove it or reference it from a note, template, or CSS.".into()),
        }],
        metrics: BuildMetrics {
            duration: Duration::from_millis(1),
        },
        inspect: None,
        previous_inspect: None,
        update_safety: None,
        comparison: ComparisonStatus::NotRequested,
        diff: None,
        risk: None,
        policy: BuildPolicyResult::default(),
        status: BuildStatus::Success,
    };

    report
        .ensure_success()
        .expect("warnings should not fail a successful report");
}

#[test]
fn build_report_pretty_report_prints_media_rows_and_sorted_diagnostics() {
    let report = BuildReport {
        artifact: Some(ApkgArtifact {
            path: PathBuf::from("out/media.apkg"),
        }),
        counts: BuildCounts {
            notes: 1,
            cards: 1,
            media: 3,
        },
        media: MediaSummary {
            objects: 2,
            bindings: 3,
            references: 4,
            missing_references: 1,
            unsafe_references: 0,
            unused_bindings: 1,
            unique_bytes: 48213,
        },
        diagnostics: vec![
            Diagnostic {
                code: DiagnosticCode::new("MEDIA.Z_LAST"),
                severity: Severity::Warning,
                domain: None,
                stage: None,
                message: "zulu warning.".into(),
                source: Some(SourcePath::new("project.media[\"b.png\"]")),
                help: None,
            },
            Diagnostic {
                code: DiagnosticCode::new("MEDIA.A_FIRST"),
                severity: Severity::Warning,
                domain: None,
                stage: None,
                message: "alpha warning.".into(),
                source: Some(SourcePath::new("project.media[\"a.png\"]")),
                help: Some("Alpha help.".into()),
            },
            Diagnostic {
                code: DiagnosticCode::new("MEDIA.B_SECOND"),
                severity: Severity::Warning,
                domain: None,
                stage: None,
                message: "beta warning.".into(),
                source: Some(SourcePath::new("project.media[\"a.png\"]")),
                help: None,
            },
            Diagnostic {
                code: DiagnosticCode::new("MEDIA.A_FIRST"),
                severity: Severity::Warning,
                domain: None,
                stage: None,
                message: "aardvark warning.".into(),
                source: Some(SourcePath::new("project.media[\"a.png\"]")),
                help: None,
            },
            Diagnostic {
                code: DiagnosticCode::new("PROJECT.INFO"),
                severity: Severity::Info,
                domain: None,
                stage: None,
                message: "informational note.".into(),
                source: None,
                help: None,
            },
            Diagnostic {
                code: DiagnosticCode::new("MEDIA.ERROR"),
                severity: Severity::Error,
                domain: None,
                stage: None,
                message: "fatal media issue.".into(),
                source: Some(SourcePath::new("project.media[\"c.png\"]")),
                help: None,
            },
        ],
        metrics: BuildMetrics {
            duration: Duration::from_millis(5),
        },
        inspect: None,
        previous_inspect: None,
        update_safety: None,
        comparison: ComparisonStatus::NotRequested,
        diff: None,
        risk: None,
        policy: BuildPolicyResult::default(),
        status: BuildStatus::Invalid,
    };

    assert_eq!(
        report.pretty_report(),
        concat!(
            "Media:\n",
            "  objects: 2\n",
            "  bindings: 3\n",
            "  references: 4\n",
            "  missing_references: 1\n",
            "  unsafe_references: 0\n",
            "  unused_bindings: 1\n",
            "  unique_bytes: 48213\n",
            "[error MEDIA.ERROR] project.media[\"c.png\"]: fatal media issue.\n",
            "[warning MEDIA.A_FIRST] project.media[\"a.png\"]: aardvark warning.\n",
            "[warning MEDIA.A_FIRST] project.media[\"a.png\"]: alpha warning. Alpha help.\n",
            "[warning MEDIA.B_SECOND] project.media[\"a.png\"]: beta warning.\n",
            "[warning MEDIA.Z_LAST] project.media[\"b.png\"]: zulu warning.\n",
            "[info PROJECT.INFO] informational note."
        )
    );
}

#[test]
fn build_report_can_carry_update_safety_summary() {
    use anki_forge::build::{
        BaselineSourceSummary, BuildCounts, BuildMetrics, BuildPolicyResult, BuildReport,
        BuildStatus, ComparisonStatus, MediaSummary, UpdateSafetySummary,
    };

    let report = BuildReport {
        artifact: None,
        counts: BuildCounts::default(),
        media: MediaSummary::default(),
        diagnostics: vec![],
        metrics: BuildMetrics::default(),
        inspect: None,
        previous_inspect: None,
        update_safety: Some(UpdateSafetySummary {
            mode: "strict".into(),
            baseline_sources: vec![BaselineSourceSummary {
                source_kind: "previous_apkg".into(),
                source_ref: "baseline.previous_apkg.primary".into(),
                display_path: Some("previous.apkg".into()),
                status: "loaded".into(),
                used_for_reconcile: true,
                limitations: vec![],
                diagnostic_codes: vec![],
            }],
            notes_preserved: 1,
            notes_derived: 0,
            notes_failed: 0,
            baseline_conflicts: 0,
            blocking_diagnostics: vec![],
            lockfile_written: false,
        }),
        comparison: ComparisonStatus::NotRequested,
        diff: None,
        risk: None,
        policy: BuildPolicyResult::default(),
        status: BuildStatus::Success,
    };

    let summary = report.update_safety.as_ref().expect("summary");
    assert_eq!(summary.mode, "strict");
    assert_eq!(
        summary.baseline_sources[0].source_ref,
        "baseline.previous_apkg.primary"
    );
}

#[test]
fn risk_level_order_matches_fail_on_thresholds() {
    assert!(RiskLevel::Critical >= RiskLevel::High);
    assert!(RiskLevel::High >= RiskLevel::Medium);
    assert!(RiskLevel::Medium >= RiskLevel::Low);
    assert!(RiskLevel::Low >= RiskLevel::Info);
}

#[test]
fn policy_blocks_at_or_above_threshold() {
    let result = BuildPolicyResult::evaluate(
        Some(RiskLevel::High),
        Some(RiskLevel::High),
        vec!["RISK.BASELINE_UNAVAILABLE".to_string()],
    );

    assert_eq!(result.status, BuildPolicyStatus::Blocked);
    assert_eq!(result.threshold, Some(RiskLevel::High));
    assert_eq!(result.highest_risk, Some(RiskLevel::High));
    assert_eq!(
        result.blocking_findings,
        vec!["RISK.BASELINE_UNAVAILABLE".to_string()]
    );
}

#[test]
fn policy_passes_below_threshold() {
    let result = BuildPolicyResult::evaluate(
        Some(RiskLevel::High),
        Some(RiskLevel::Medium),
        vec!["RISK.FIELD_REMOVED_OR_RENAMED".to_string()],
    );

    assert_eq!(result.status, BuildPolicyStatus::Passed);
    assert_eq!(result.blocking_findings, Vec::<String>::new());
}

#[test]
fn build_status_precedence_prefers_error_over_invalid_blocked_success() {
    let status = BuildStatus::highest([
        BuildStatus::Success,
        BuildStatus::Blocked,
        BuildStatus::Invalid,
        BuildStatus::Error,
    ]);

    assert_eq!(status, BuildStatus::Error);
}

#[test]
fn comparison_status_serializes_as_snake_case() {
    assert_eq!(
        serde_json::to_string(&ComparisonStatus::NotRequested).unwrap(),
        "\"not_requested\""
    );
    assert_eq!(
        serde_json::to_string(&ComparisonStatus::Unavailable).unwrap(),
        "\"unavailable\""
    );
}

fn successful_report_fixture() -> BuildReport {
    BuildReport {
        artifact: Some(ApkgArtifact {
            path: PathBuf::from("out/success.apkg"),
        }),
        counts: BuildCounts {
            notes: 1,
            cards: 1,
            media: 0,
        },
        media: MediaSummary::default(),
        diagnostics: Vec::new(),
        metrics: BuildMetrics {
            duration: std::time::Duration::from_millis(1),
        },
        inspect: None,
        previous_inspect: None,
        update_safety: None,
        comparison: ComparisonStatus::NotRequested,
        diff: None,
        risk: None,
        policy: BuildPolicyResult::default(),
        status: BuildStatus::Success,
    }
}

#[test]
fn build_report_projection_has_phase4_contract_header() {
    let report = successful_report_fixture();
    let projected = BuildReportJson::from_report(&report);
    let projected_from_trait = report.to_report_json();

    assert_eq!(projected.kind, "anki-forge-build-report");
    assert_eq!(projected_from_trait.kind, "anki-forge-build-report");
    assert_eq!(projected.schema_version, "phase4-build-report-v2");
    assert!(!projected.tool_version.is_empty());
    assert_eq!(projected.status, BuildStatus::Success);
    assert_eq!(projected.comparison, ComparisonStatus::NotRequested);
}

#[test]
fn build_report_projection_serializes_duration_as_milliseconds() {
    let mut report = successful_report_fixture();
    report.metrics = BuildMetrics {
        duration: std::time::Duration::from_millis(42),
    };

    let json = serde_json::to_value(BuildReportJson::from_report(&report)).unwrap();
    assert_eq!(json["metrics"]["duration_ms"], 42);
}

#[test]
fn build_report_projection_serializes_diagnostics_as_strings() {
    let mut report = successful_report_fixture();
    report.diagnostics.push(Diagnostic {
        code: DiagnosticCode::new("PROJECT.NORMALIZE_FAILED"),
        severity: Severity::Error,
        domain: Some(anki_forge::diagnostics::DiagnosticDomain::new("project")),
        stage: Some(anki_forge::diagnostics::DiagnosticStage::new("normalize")),
        message: "normalization failed".to_string(),
        source: Some(SourcePath::new("project")),
        help: Some("inspect the Product input".to_string()),
    });

    let json = serde_json::to_value(BuildReportJson::from_report(&report)).unwrap();
    assert_eq!(json["diagnostics"][0]["code"], "PROJECT.NORMALIZE_FAILED");
    assert_eq!(json["diagnostics"][0]["severity"], "error");
    assert_eq!(json["diagnostics"][0]["domain"], "project");
    assert_eq!(json["diagnostics"][0]["stage"], "normalize");
    assert_eq!(json["diagnostics"][0]["path"], "project");
    assert_eq!(
        json["diagnostics"][0]["suggested_fix"],
        "inspect the Product input"
    );
    assert!(json["diagnostics"][0].get("source").is_none());
    assert!(json["diagnostics"][0].get("help").is_none());
}

#[test]
fn write_report_json_atomic_preserves_unrelated_tmp_sibling() {
    let temp = tempfile::tempdir().expect("tempdir");
    let report_path = temp.path().join("build-report.json");
    let unrelated_tmp = temp.path().join("build-report.tmp");
    std::fs::write(&unrelated_tmp, "keep me").expect("write tmp sibling");

    write_report_json_atomic(&report_path, &successful_report_fixture()).expect("write report");

    assert_eq!(std::fs::read_to_string(&unrelated_tmp).unwrap(), "keep me");
    let written: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(report_path).unwrap()).unwrap();
    assert_eq!(written["kind"], "anki-forge-build-report");
}
