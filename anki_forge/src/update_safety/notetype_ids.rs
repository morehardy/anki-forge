use std::collections::{BTreeMap, BTreeSet};

use crate::diagnostics::{
    Diagnostic, DiagnosticCode, DiagnosticDomain, DiagnosticStage, Severity, SourcePath,
};

use super::model::IdentityIndex;

/// Preserve absent note/type history, with artifact truth taking precedence per identity.
pub(crate) fn combined_baseline(
    previous: Option<&IdentityIndex>,
    lockfile: Option<&IdentityIndex>,
) -> Option<IdentityIndex> {
    let mut combined = lockfile.or(previous)?.clone();
    let mut notetypes = BTreeMap::new();
    let mut notes = BTreeMap::new();
    for index in [lockfile, previous].into_iter().flatten() {
        for note in &index.notes {
            notes.insert(note.stable_id.clone(), note.clone());
        }
        for notetype in &index.notetypes {
            notetypes.insert(notetype.note_type_id.clone(), notetype.clone());
        }
    }
    combined.notetypes = notetypes.into_values().collect();
    combined.notes = notes.into_values().collect();
    Some(combined)
}

pub(crate) fn validate_baseline_model_ids(index: &IdentityIndex) -> anyhow::Result<()> {
    let mut keys = BTreeSet::new();
    let mut ids = BTreeSet::new();
    for notetype in &index.notetypes {
        anyhow::ensure!(
            keys.insert(&notetype.note_type_id),
            "UPDATE.NOTETYPE_MODEL_ID_COLLISION: duplicate logical note type {}",
            notetype.note_type_id
        );
        if let Some(id) = notetype.anki_model_id {
            anyhow::ensure!(
                id > 0,
                "UPDATE.NOTETYPE_MODEL_ID_INVALID: {} has non-positive model ID {}",
                notetype.note_type_id,
                id
            );
            anyhow::ensure!(
                ids.insert(id),
                "UPDATE.NOTETYPE_MODEL_ID_COLLISION: duplicate baseline model ID {}",
                id
            );
        }
    }
    Ok(())
}

pub(crate) fn reconcile_notetype_ids(
    current: &mut IdentityIndex,
    previous: Option<&IdentityIndex>,
    lockfile: Option<&IdentityIndex>,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut baseline = BTreeMap::new();
    // Artifact truth overrides lockfile evidence, as it does for note GUIDs.
    for index in [lockfile, previous].into_iter().flatten() {
        for notetype in &index.notetypes {
            if let Some(old) = baseline.insert(notetype.note_type_id.as_str(), notetype) {
                if old.anki_model_id.is_some()
                    && notetype.anki_model_id.is_some()
                    && old.anki_model_id != notetype.anki_model_id
                {
                    diagnostics.push(diagnostic(
                        "UPDATE.NOTETYPE_MODEL_ID_CONFLICT", Severity::Warning, &notetype.note_type_id,
                        format!("previous APKG model ID {:?} overrides lockfile model ID {:?}", notetype.anki_model_id, old.anki_model_id),
                        "the previous APKG is artifact truth; rewrite the lockfile after reviewing the baseline",
                    ));
                }
            }
        }
    }

    for notetype in &mut current.notetypes {
        if let Some(old) = baseline.get(notetype.note_type_id.as_str()) {
            if let Some(id) = old.anki_model_id {
                notetype.anki_model_id = Some(id);
            } else {
                diagnostics.push(diagnostic(
                    "UPDATE.NOTETYPE_MODEL_ID_MISSING", Severity::Error, &notetype.note_type_id,
                    "baseline has no numeric Anki model ID; preservation cannot be verified".into(),
                    "supply compare_to(previous.apkg) to recover legacy IDs and rewrite the identity lockfile",
                ));
            }
        }
    }

    // Reserve absent identities too: a new model must not steal an old model's ID.
    let mut occupied = BTreeMap::new();
    for notetype in baseline.values().copied().chain(current.notetypes.iter()) {
        let Some(id) = notetype.anki_model_id else {
            continue;
        };
        if let Some(other) = occupied.insert(id, &notetype.note_type_id) {
            if other != &notetype.note_type_id {
                diagnostics.push(diagnostic(
                    "UPDATE.NOTETYPE_MODEL_ID_COLLISION", Severity::Error, &notetype.note_type_id,
                    format!("model ID {id} is assigned to both {other} and {}", notetype.note_type_id),
                    "use consistent baselines and distinct logical note type IDs; never reuse a reserved model ID",
                ));
            }
        }
    }
    diagnostics
}

fn diagnostic(code: &str, severity: Severity, id: &str, message: String, help: &str) -> Diagnostic {
    Diagnostic {
        code: DiagnosticCode::new(code),
        severity,
        domain: Some(DiagnosticDomain::new("update_safety")),
        stage: Some(DiagnosticStage::new("update_safety")),
        source: Some(SourcePath::new(format!("notetype[id='{id}']"))),
        message,
        help: Some(help.into()),
    }
}
