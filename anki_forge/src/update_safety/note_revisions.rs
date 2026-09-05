use std::collections::BTreeMap;

use crate::authoring_core::NormalizedIr;
use crate::diagnostics::{Diagnostic, DiagnosticCode, Severity, SourcePath};

use super::model::IdentityIndex;

/// Select content revisions from the same per-note baseline precedence as GUIDs.
pub(crate) fn reconcile(
    current: &mut IdentityIndex,
    normalized: &mut NormalizedIr,
    previous: Option<&IdentityIndex>,
    lockfile: Option<&IdentityIndex>,
) -> Vec<Diagnostic> {
    let previous: BTreeMap<_, _> = previous
        .into_iter()
        .flat_map(|index| &index.notes)
        .map(|note| (note.stable_id.as_str(), note))
        .collect();
    let locked: BTreeMap<_, _> = lockfile
        .into_iter()
        .flat_map(|index| &index.notes)
        .map(|note| (note.stable_id.as_str(), note))
        .collect();
    let mut diagnostics = Vec::new();
    for note in &mut current.notes {
        let prior = previous.get(note.stable_id.as_str());
        let locked = locked.get(note.stable_id.as_str());
        if let (Some(prior), Some(locked)) = (prior, locked) {
            if prior.revision.is_some()
                && locked.revision.is_some()
                && prior.revision != locked.revision
            {
                diagnostics.push(diagnostic("UPDATE.NOTE_REVISION_CONFLICT", Severity::Warning,
                    &note.stable_id, "actual APKG note revision overrides conflicting lockfile evidence",
                    "verify that compare_to is the latest distributed APKG and refresh the lockfile"));
            }
        }
        let Some(baseline) = prior.or(locked) else {
            continue;
        };
        let Some(old) = baseline.revision.as_ref() else {
            diagnostics.push(diagnostic(
                "UPDATE.NOTE_REVISION_MISSING",
                Severity::Error,
                &note.stable_id,
                "baseline lacks full-content revision evidence; content update cannot be verified",
                "supply compare_to(previous.apkg) to recover actual content and modification times",
            ));
            continue;
        };
        let revision = note
            .revision
            .as_mut()
            .expect("current identity records full normalized note content");
        let selected_time = if revision.content_hash == old.content_hash {
            Some(old.mtime_secs)
        } else {
            old.mtime_secs.checked_add(1)
        };
        match selected_time {
            Some(time) => revision.mtime_secs = time,
            None => diagnostics.push(diagnostic(
                "UPDATE.NOTE_MTIME_OVERFLOW",
                Severity::Error,
                &note.stable_id,
                "changed note cannot advance beyond the maximum modification time",
                "repair the invalid timestamp in the baseline; a lower or wrapped time is not safe",
            )),
        }
    }
    let by_id: BTreeMap<_, _> = current
        .notes
        .iter()
        .filter_map(|note| {
            Some((
                note.normalized_note_id.as_deref()?,
                note.revision.as_ref()?.mtime_secs,
            ))
        })
        .collect();
    for note in &mut normalized.notes {
        if let Some(time) = by_id.get(note.id.as_str()) {
            note.mtime_secs = Some(*time);
        }
    }
    diagnostics
}

fn diagnostic(code: &str, severity: Severity, id: &str, message: &str, help: &str) -> Diagnostic {
    Diagnostic {
        code: DiagnosticCode::new(code),
        severity,
        domain: None,
        stage: None,
        source: Some(SourcePath::new(format!("note[id='{id}']"))),
        message: message.into(),
        help: Some(help.into()),
    }
}
