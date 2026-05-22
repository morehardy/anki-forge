use crate::build::UpdateSafetySummary;
use crate::diagnostics::{Diagnostic, Severity};

use super::model::EffectiveMode;
use super::reconcile::ReconcileOutput;

pub fn summary_from_reconcile(
    mode: EffectiveMode,
    reconcile: &ReconcileOutput,
    diagnostics: &[Diagnostic],
    lockfile_written: bool,
) -> UpdateSafetySummary {
    UpdateSafetySummary {
        mode: match mode {
            EffectiveMode::Disabled => "disabled",
            EffectiveMode::ReportOnly => "report_only",
            EffectiveMode::Strict => "strict",
        }
        .into(),
        baseline_sources: vec![],
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
