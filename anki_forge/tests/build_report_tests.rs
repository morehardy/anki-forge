use std::path::PathBuf;
use std::time::Duration;

use anki_forge::build::{ApkgArtifact, BuildCounts, BuildFailureCause, BuildMetrics, BuildReport};
use anki_forge::diagnostics::{Diagnostic, DiagnosticCode, Severity};

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
        diagnostics: vec![],
        metrics: BuildMetrics {
            duration: Duration::from_millis(25),
        },
        inspect: None,
        status: "success".into(),
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
        diagnostics: vec![Diagnostic {
            code: DiagnosticCode::new("MEDIA.MISSING_REFERENCE"),
            severity: Severity::Error,
            message: "missing media reference hola.mp3".into(),
            source: None,
            help: Some("register the media before adding the note".into()),
        }],
        metrics: BuildMetrics {
            duration: Duration::from_millis(1),
        },
        inspect: None,
        status: "invalid".into(),
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
        diagnostics: vec![Diagnostic {
            code: DiagnosticCode::new("PROJECT.NORMALIZE_FAILED"),
            severity: Severity::Error,
            message: "normalization failed".into(),
            source: None,
            help: None,
        }],
        metrics: BuildMetrics {
            duration: Duration::from_millis(1),
        },
        inspect: None,
        status: "invalid".into(),
    };

    let err = report.ensure_success().expect_err("report should fail");
    assert_eq!(err.cause, BuildFailureCause::Diagnostics);
}
