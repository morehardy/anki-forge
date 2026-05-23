use std::collections::{BTreeMap, BTreeSet};

use crate::diagnostics::{Diagnostic, DiagnosticCode, Severity, SourcePath};
use writer_core::WriterGuidAssignment;

use super::model::IdentityIndex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuidSource {
    PreviousApkg,
    Lockfile,
    CurrentDerivation,
}

impl GuidSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PreviousApkg => "previous_apkg",
            Self::Lockfile => "lockfile",
            Self::CurrentDerivation => "current_derivation",
        }
    }
}

#[derive(Debug)]
pub struct ReconcileOutput {
    pub assignments: Vec<WriterGuidAssignment>,
    pub diagnostics: Vec<Diagnostic>,
    pub notes_preserved: usize,
    pub notes_derived: usize,
    pub notes_failed: usize,
    pub baseline_conflicts: usize,
}

pub fn reconcile_guid_plan(
    current: &IdentityIndex,
    previous_apkg: Option<&IdentityIndex>,
    lockfile: Option<&IdentityIndex>,
) -> anyhow::Result<ReconcileOutput> {
    let previous_by_stable = previous_apkg.map(index_by_stable_id).unwrap_or_default();
    let lockfile_by_stable = lockfile.map(index_by_stable_id).unwrap_or_default();
    let mut diagnostics = Vec::new();
    let mut assignments = Vec::new();
    let mut selected = BTreeMap::<String, String>::new();
    let mut notes_preserved = 0;
    let mut notes_derived = 0;
    let mut baseline_conflicts = 0;

    push_writer_policy_mismatch_diagnostics(current, previous_apkg, lockfile, &mut diagnostics);

    for note in &current.notes {
        let normalized_note_id = note
            .normalized_note_id
            .clone()
            .unwrap_or_else(|| note.stable_id.clone());
        let previous = previous_by_stable.get(note.stable_id.as_str());
        let locked = lockfile_by_stable.get(note.stable_id.as_str());
        let (guid, source) = if let Some(previous) = previous {
            if let Some(locked) = locked {
                if locked.anki_guid != previous.anki_guid {
                    baseline_conflicts += 1;
                    diagnostics.push(Diagnostic {
                        code: DiagnosticCode::new("UPDATE.BASELINE_CONFLICT_GUID"),
                        severity: Severity::Warning,
                        message: format!(
                            "previous APKG GUID {} overrides lockfile GUID {} for {}",
                            previous.anki_guid, locked.anki_guid, note.stable_id
                        ),
                        source: Some(SourcePath::new(note.source_path.clone())),
                        help: Some("previous APKG is artifact truth for update safety".into()),
                    });
                }
            }
            notes_preserved += 1;
            (previous.anki_guid.clone(), GuidSource::PreviousApkg)
        } else if let Some(locked) = locked {
            notes_preserved += 1;
            (locked.anki_guid.clone(), GuidSource::Lockfile)
        } else {
            notes_derived += 1;
            (
                note.current_guid_candidate.clone(),
                GuidSource::CurrentDerivation,
            )
        };

        let info_code = match source {
            GuidSource::PreviousApkg => "UPDATE.GUID_PRESERVED_FROM_PREVIOUS",
            GuidSource::Lockfile => "UPDATE.GUID_PRESERVED_FROM_LOCKFILE",
            GuidSource::CurrentDerivation => "UPDATE.GUID_DERIVED_FOR_NEW_NOTE",
        };
        diagnostics.push(Diagnostic {
            code: DiagnosticCode::new(info_code),
            severity: Severity::Info,
            message: format!("selected GUID {guid} for stable id {}", note.stable_id),
            source: Some(SourcePath::new(note.source_path.clone())),
            help: None,
        });

        if let Some(existing) = selected.insert(guid.clone(), note.stable_id.clone()) {
            anyhow::bail!(
                "UPDATE.GUID_DUPLICATE_AT_RECONCILE: {} and {} selected {}",
                existing,
                note.stable_id,
                guid
            );
        }

        if guid != note.current_guid_candidate && source != GuidSource::CurrentDerivation {
            diagnostics.push(Diagnostic {
                code: DiagnosticCode::new("UPDATE.GUID_DERIVATION_DRIFT"),
                severity: Severity::Warning,
                message: format!(
                    "selected GUID {guid} differs from current derivation {}",
                    note.current_guid_candidate
                ),
                source: Some(SourcePath::new(note.source_path.clone())),
                help: Some("update-safe mode preserves existing Anki GUIDs".into()),
            });
        }

        assignments.push(WriterGuidAssignment {
            normalized_note_id,
            stable_id: note.stable_id.clone(),
            selected_anki_guid: guid,
            current_guid_candidate: note.current_guid_candidate.clone(),
            guid_derivation_version: note.guid_derivation_version.clone(),
            recipe_id: note.recipe_id.clone(),
            canonical_payload_hash: note.canonical_payload_hash.clone(),
            provenance: note.provenance.clone(),
            used_override: note.used_override,
            source: source.as_str().into(),
        });
    }

    assignments.sort_by(|left, right| {
        left.normalized_note_id
            .cmp(&right.normalized_note_id)
            .then(left.stable_id.cmp(&right.stable_id))
    });

    Ok(ReconcileOutput {
        assignments,
        diagnostics,
        notes_preserved,
        notes_derived,
        notes_failed: 0,
        baseline_conflicts,
    })
}

fn index_by_stable_id(index: &IdentityIndex) -> BTreeMap<&str, &super::model::NoteIdentityEntry> {
    let mut map = BTreeMap::new();
    for note in &index.notes {
        map.insert(note.stable_id.as_str(), note);
    }
    map
}

pub fn current_only_reconcile(current: &IdentityIndex) -> anyhow::Result<ReconcileOutput> {
    reconcile_guid_plan(current, None, None)
}

fn push_writer_policy_mismatch_diagnostics(
    current: &IdentityIndex,
    previous_apkg: Option<&IdentityIndex>,
    lockfile: Option<&IdentityIndex>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for (label, baseline) in [("previous APKG", previous_apkg), ("lockfile", lockfile)] {
        let Some(baseline) = baseline else {
            continue;
        };
        if baseline.writer_policy_ref == "unknown@unknown"
            || baseline.writer_policy_ref == current.writer_policy_ref
        {
            continue;
        }
        diagnostics.push(Diagnostic {
            code: DiagnosticCode::new("UPDATE.WRITER_POLICY_MISMATCH"),
            severity: Severity::Warning,
            message: format!(
                "{label} writer policy {} differs from current {}",
                baseline.writer_policy_ref, current.writer_policy_ref
            ),
            source: Some(SourcePath::new(baseline.source_ref.clone())),
            help: Some("verify the baseline was produced with a compatible writer policy".into()),
        });
    }
}

pub fn selected_identity_index(
    current: &IdentityIndex,
    output: &ReconcileOutput,
    previous_lockfile_index: Option<&IdentityIndex>,
) -> IdentityIndex {
    let by_stable: BTreeMap<_, _> = output
        .assignments
        .iter()
        .map(|assignment| (assignment.stable_id.as_str(), assignment))
        .collect();
    let mut selected = current.clone();
    selected.source_kind = "lockfile".into();
    selected.source_ref = "baseline.identity_lockfile.primary".into();
    for note in &mut selected.notes {
        if let Some(assignment) = by_stable.get(note.stable_id.as_str()) {
            note.anki_guid = assignment.selected_anki_guid.clone();
        }
    }
    let current_stable_ids: BTreeSet<String> = selected
        .notes
        .iter()
        .map(|note| note.stable_id.clone())
        .collect();
    if let Some(previous_lockfile_index) = previous_lockfile_index {
        for old_note in &previous_lockfile_index.notes {
            if current_stable_ids.contains(old_note.stable_id.as_str()) {
                continue;
            }
            let mut absent = old_note.clone();
            absent.normalized_note_id = None;
            absent.entry_lifecycle = "absent_from_current".into();
            absent.source_path = "baseline.identity_lockfile.primary".into();
            selected.notes.push(absent);
        }
    }
    selected
        .notes
        .sort_by(|left, right| left.stable_id.cmp(&right.stable_id));
    selected
}
