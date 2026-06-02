use authoring_core::NormalizedIr;
use std::collections::BTreeMap;

use crate::diagnostics::{Diagnostic, DiagnosticCode, Severity, SourcePath};
use writer_core::WriterPolicy;

use super::model::{
    validate_writer_policy_ref, EffectiveMode, IdentityIndex, ResolvedNoteIdentity,
};

pub struct CurrentIdentityInput<'a> {
    pub project_stable_id: Option<&'a str>,
    pub normalized: &'a NormalizedIr,
    pub writer_policy: &'a WriterPolicy,
    pub mode: EffectiveMode,
    pub resolved_note_identities: &'a BTreeMap<String, ResolvedNoteIdentity>,
}

pub struct CurrentIdentityOutput {
    pub index: IdentityIndex,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn build_current_identity_index(input: CurrentIdentityInput<'_>) -> CurrentIdentityOutput {
    let mut diagnostics = Vec::new();
    let mut index = IdentityIndex::current(input.project_stable_id, input.writer_policy);
    if let Err(err) =
        validate_writer_policy_ref(&input.writer_policy.id, &input.writer_policy.version)
    {
        diagnostics.push(Diagnostic {
            code: err.code,
            severity: err.severity,
            domain: None,
            stage: None,
            message: err.message,
            source: Some(SourcePath::new("writer_policy")),
            help: Some("remove @ and control characters from writer policy id/version".into()),
        });
    }

    for note in &input.normalized.notes {
        let stable_id = note.id.as_str();
        if matches!(input.mode, EffectiveMode::Strict)
            && (stable_id.trim().is_empty() || stable_id.starts_with("generated:"))
        {
            diagnostics.push(Diagnostic {
                code: DiagnosticCode::new("UPDATE.STABLE_ID_MISSING_IN_STRICT_MODE"),
                severity: Severity::Error,
                domain: None,
                stage: None,
                message: "current output note has no resolved stable id in strict mode".into(),
                source: Some(SourcePath::new(format!("note[id='{}']", note.id))),
                help: Some("provide Note::stable_id(value) or an identity recipe".into()),
            });
            continue;
        }
        if is_invalid_anki_guid_candidate(stable_id) {
            diagnostics.push(Diagnostic {
                code: DiagnosticCode::new("UPDATE.ANKI_GUID_INVALID"),
                severity: Severity::Error,
                domain: None,
                stage: None,
                message: format!("stable id {stable_id:?} cannot be used as a Phase 3 Anki GUID candidate"),
                source: Some(SourcePath::new(format!("note[id='{}']", note.id))),
                help: Some("use a non-empty stable id without ASCII control characters and at most 255 bytes".into()),
            });
            continue;
        }
        index.push_current_note(note, input.resolved_note_identities.get(&note.id));
    }

    for notetype in &input.normalized.notetypes {
        index.push_current_notetype(notetype);
    }

    CurrentIdentityOutput { index, diagnostics }
}

fn is_invalid_anki_guid_candidate(value: &str) -> bool {
    value.is_empty() || value.len() > 255 || value.chars().any(char::is_control)
}
