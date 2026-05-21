use std::path::PathBuf;
use std::time::Duration;

use anki_forge::build::{
    ApkgArtifact, BuildCounts, BuildFailureCause, BuildMetrics, BuildReport, MediaSummary,
};
use anki_forge::diagnostics::{Diagnostic, DiagnosticCode, Severity, SourcePath};

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
        media: MediaSummary::default(),
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
        media: MediaSummary::default(),
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
            message: "registered media is not referenced.".into(),
            source: Some(SourcePath::new("project.media[\"unused.png\"]")),
            help: Some("Remove it or reference it from a note, template, or CSS.".into()),
        }],
        metrics: BuildMetrics {
            duration: Duration::from_millis(1),
        },
        inspect: None,
        status: "success".into(),
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
                message: "zulu warning.".into(),
                source: Some(SourcePath::new("project.media[\"b.png\"]")),
                help: None,
            },
            Diagnostic {
                code: DiagnosticCode::new("MEDIA.A_FIRST"),
                severity: Severity::Warning,
                message: "alpha warning.".into(),
                source: Some(SourcePath::new("project.media[\"a.png\"]")),
                help: Some("Alpha help.".into()),
            },
            Diagnostic {
                code: DiagnosticCode::new("MEDIA.B_SECOND"),
                severity: Severity::Warning,
                message: "beta warning.".into(),
                source: Some(SourcePath::new("project.media[\"a.png\"]")),
                help: None,
            },
            Diagnostic {
                code: DiagnosticCode::new("MEDIA.A_FIRST"),
                severity: Severity::Warning,
                message: "aardvark warning.".into(),
                source: Some(SourcePath::new("project.media[\"a.png\"]")),
                help: None,
            },
            Diagnostic {
                code: DiagnosticCode::new("PROJECT.INFO"),
                severity: Severity::Info,
                message: "informational note.".into(),
                source: None,
                help: None,
            },
            Diagnostic {
                code: DiagnosticCode::new("MEDIA.ERROR"),
                severity: Severity::Error,
                message: "fatal media issue.".into(),
                source: Some(SourcePath::new("project.media[\"c.png\"]")),
                help: None,
            },
        ],
        metrics: BuildMetrics {
            duration: Duration::from_millis(5),
        },
        inspect: None,
        status: "invalid".into(),
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
