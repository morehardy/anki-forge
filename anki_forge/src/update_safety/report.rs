use crate::build::{BaselineSourceSummary, UpdateSafetySummary};
use crate::diagnostics::{Diagnostic, Severity};

use super::model::{EffectiveMode, IdentityIndex};
use super::reconcile::ReconcileOutput;

pub fn summary_from_reconcile(
    mode: EffectiveMode,
    reconcile: &ReconcileOutput,
    diagnostics: &[Diagnostic],
    baseline_sources: Vec<BaselineSourceSummary>,
    lockfile_written: bool,
) -> UpdateSafetySummary {
    UpdateSafetySummary {
        mode: match mode {
            EffectiveMode::Disabled => "disabled",
            EffectiveMode::ReportOnly => "report_only",
            EffectiveMode::Strict => "strict",
        }
        .into(),
        baseline_sources,
        notes_preserved: reconcile.notes_preserved,
        notes_derived: reconcile.notes_derived,
        notes_failed: reconcile.notes_failed,
        baseline_conflicts: reconcile.baseline_conflicts,
        blocking_diagnostics: diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == Severity::Error)
            .map(|diagnostic| diagnostic.code.as_str().to_string())
            .collect(),
        lockfile_written,
    }
}

pub fn ignored_previous_apkg_source(path: &std::path::Path) -> BaselineSourceSummary {
    BaselineSourceSummary {
        source_kind: "previous_apkg".into(),
        source_ref: "baseline.previous_apkg.primary".into(),
        display_path: Some(path.display().to_string()),
        status: "ignored_disabled".into(),
        used_for_reconcile: false,
        limitations: vec![],
        diagnostic_codes: vec!["UPDATE.BASELINE_IGNORED_DISABLED".into()],
    }
}

pub fn ignored_lockfile_source(path: &std::path::Path) -> BaselineSourceSummary {
    BaselineSourceSummary {
        source_kind: "lockfile".into(),
        source_ref: "baseline.identity_lockfile.primary".into(),
        display_path: Some(path.display().to_string()),
        status: "ignored_disabled".into(),
        used_for_reconcile: false,
        limitations: vec![],
        diagnostic_codes: vec!["UPDATE.BASELINE_IGNORED_DISABLED".into()],
    }
}

pub fn loaded_previous_apkg_source(
    path: &std::path::Path,
    limitations: Vec<String>,
) -> BaselineSourceSummary {
    loaded_source(
        "previous_apkg",
        "baseline.previous_apkg.primary",
        path,
        limitations,
    )
}

pub fn loaded_lockfile_source(
    path: &std::path::Path,
    limitations: Vec<String>,
) -> BaselineSourceSummary {
    loaded_source(
        "lockfile",
        "baseline.identity_lockfile.primary",
        path,
        limitations,
    )
}

pub fn unreadable_previous_apkg_source(
    path: &std::path::Path,
    code: &str,
) -> BaselineSourceSummary {
    unreadable_source(
        "previous_apkg",
        "baseline.previous_apkg.primary",
        path,
        code,
    )
}

pub fn unreadable_lockfile_source(path: &std::path::Path, code: &str) -> BaselineSourceSummary {
    unreadable_source("lockfile", "baseline.identity_lockfile.primary", path, code)
}

fn loaded_source(
    source_kind: &str,
    source_ref: &str,
    path: &std::path::Path,
    limitations: Vec<String>,
) -> BaselineSourceSummary {
    BaselineSourceSummary {
        source_kind: source_kind.into(),
        source_ref: source_ref.into(),
        display_path: Some(path.display().to_string()),
        status: if limitations.is_empty() {
            "loaded".into()
        } else {
            "partial".into()
        },
        used_for_reconcile: true,
        limitations,
        diagnostic_codes: vec![],
    }
}

fn unreadable_source(
    source_kind: &str,
    source_ref: &str,
    path: &std::path::Path,
    code: &str,
) -> BaselineSourceSummary {
    BaselineSourceSummary {
        source_kind: source_kind.into(),
        source_ref: source_ref.into(),
        display_path: Some(path.display().to_string()),
        status: "unreadable".into(),
        used_for_reconcile: false,
        limitations: vec![],
        diagnostic_codes: vec![code.into()],
    }
}

pub fn summary_from_disabled_mode(
    current: &IdentityIndex,
    baseline_sources: Vec<BaselineSourceSummary>,
    blocking_diagnostics: Vec<String>,
) -> UpdateSafetySummary {
    UpdateSafetySummary {
        mode: "disabled".into(),
        baseline_sources,
        notes_preserved: 0,
        notes_derived: current.notes.len(),
        notes_failed: 0,
        baseline_conflicts: 0,
        blocking_diagnostics,
        lockfile_written: false,
    }
}
