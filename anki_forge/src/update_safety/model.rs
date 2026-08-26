use crate::build::{BuildOptions, UpdateSafetyMode};
use crate::diagnostics::{DiagnosticCode, Severity};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectiveMode {
    Disabled,
    ReportOnly,
    Strict,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModeSelectionError {
    pub code: DiagnosticCode,
    pub severity: Severity,
    pub message: String,
}

pub fn effective_mode(options: &BuildOptions) -> Result<EffectiveMode, ModeSelectionError> {
    if options.write_identity_lockfile && options.identity_lockfile.is_none() {
        return Err(ModeSelectionError {
            code: DiagnosticCode::new("UPDATE.LOCKFILE_PATH_REQUIRED"),
            severity: Severity::Error,
            message: "write_identity_lockfile(true) requires identity_lockfile(path)".into(),
        });
    }

    if let Some(mode) = options.update_safety {
        return Ok(match mode {
            UpdateSafetyMode::Disabled => EffectiveMode::Disabled,
            UpdateSafetyMode::ReportOnly => EffectiveMode::ReportOnly,
            UpdateSafetyMode::Strict => EffectiveMode::Strict,
        });
    }

    if options.identity_lockfile.is_some() || options.compare_to.is_some() {
        return Ok(EffectiveMode::Strict);
    }

    Ok(EffectiveMode::Disabled)
}

pub fn validate_writer_policy_ref(id: &str, version: &str) -> Result<String, ModeSelectionError> {
    let invalid = id.is_empty()
        || version.is_empty()
        || id.contains('@')
        || version.contains('@')
        || id.chars().any(char::is_control)
        || version.chars().any(char::is_control);
    if invalid {
        return Err(ModeSelectionError {
            code: DiagnosticCode::new("UPDATE.WRITER_POLICY_REF_INVALID"),
            severity: Severity::Error,
            message: "writer policy id and version must be non-empty and must not contain @ or control characters".into(),
        });
    }
    Ok(crate::writer_core::policy_ref(id, version))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdentityIndex {
    pub schema_version: String,
    pub source_kind: String,
    pub source_ref: String,
    pub writer_policy_ref: String,
    pub project_stable_id: Option<String>,
    pub notes: Vec<NoteIdentityEntry>,
    pub notetypes: Vec<NotetypeIdentityEntry>,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NoteIdentityEntry {
    pub stable_id: String,
    pub normalized_note_id: Option<String>,
    pub anki_guid: String,
    pub current_guid_candidate: String,
    pub guid_derivation_version: String,
    pub note_type_id: String,
    pub recipe_id: String,
    pub canonical_payload_hash: Option<String>,
    pub provenance: String,
    pub used_override: bool,
    pub entry_lifecycle: String,
    pub source_path: String,
    pub recovery_method: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedNoteIdentity {
    pub stable_id: String,
    pub current_guid_candidate: String,
    pub recipe_id: String,
    pub canonical_payload_hash: Option<String>,
    pub provenance: String,
    pub used_override: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NotetypeIdentityEntry {
    pub note_type_id: String,
    pub anki_model_id: Option<i64>,
    pub name: String,
    pub fields: Vec<FieldMergeEntry>,
    pub templates: Vec<TemplateMergeEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FieldMergeEntry {
    pub field_key: String,
    pub field_name: String,
    pub ord: u32,
    pub config_id: i64,
    pub tag: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TemplateMergeEntry {
    pub template_key: String,
    pub template_name: String,
    pub ord: u32,
    pub config_id: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct IdentityLockfile {
    pub schema_version: String,
    pub project_stable_id: String,
    pub writer_policy_ref: String,
    pub identity_index: IdentityIndex,
    pub generated_by: GeneratedBy,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct GeneratedBy {
    pub tool: String,
    pub tool_version: String,
    pub writer_policy_ref: String,
}

impl IdentityIndex {
    pub fn empty_lockfile(project_stable_id: &str, writer_policy_ref: &str) -> Self {
        Self {
            schema_version: "identity-index-v1".into(),
            source_kind: "lockfile".into(),
            source_ref: "baseline.identity_lockfile.primary".into(),
            writer_policy_ref: writer_policy_ref.into(),
            project_stable_id: Some(project_stable_id.into()),
            notes: vec![],
            notetypes: vec![],
            limitations: vec![],
        }
    }

    pub fn current(
        project_stable_id: Option<&str>,
        writer_policy: &crate::writer_core::WriterPolicy,
    ) -> Self {
        Self {
            schema_version: "identity-index-v1".into(),
            source_kind: "current".into(),
            source_ref: "current".into(),
            writer_policy_ref: crate::writer_core::policy_ref(
                &writer_policy.id,
                &writer_policy.version,
            ),
            project_stable_id: project_stable_id.map(str::to_string),
            notes: vec![],
            notetypes: vec![],
            limitations: vec![],
        }
    }

    pub fn push_current_note(
        &mut self,
        note: &crate::authoring_core::NormalizedNote,
        resolved: Option<&ResolvedNoteIdentity>,
    ) {
        let stable_id = resolved
            .map(|identity| identity.stable_id.clone())
            .unwrap_or_else(|| note.id.clone());
        let current_guid_candidate = resolved
            .map(|identity| identity.current_guid_candidate.clone())
            .unwrap_or_else(|| note.id.clone());
        let recipe_id = resolved
            .map(|identity| identity.recipe_id.clone())
            .unwrap_or_else(|| "product.explicit-or-normalized.v1".into());
        let canonical_payload_hash =
            resolved.and_then(|identity| identity.canonical_payload_hash.clone());
        let provenance = resolved
            .map(|identity| identity.provenance.clone())
            .unwrap_or_else(|| "ExplicitStableId".into());
        let used_override = resolved
            .map(|identity| identity.used_override)
            .unwrap_or(false);

        self.notes.push(NoteIdentityEntry {
            stable_id,
            normalized_note_id: Some(note.id.clone()),
            anki_guid: current_guid_candidate.clone(),
            current_guid_candidate,
            guid_derivation_version: "guid.raw-stable-id.v1".into(),
            note_type_id: note.notetype_id.clone(),
            recipe_id,
            canonical_payload_hash,
            provenance,
            used_override,
            entry_lifecycle: "active".into(),
            source_path: format!("note[id='{}']", note.id),
            recovery_method: "current_resolution".into(),
        });
    }

    pub fn push_current_notetype(&mut self, notetype: &crate::authoring_core::NormalizedNotetype) {
        self.notetypes.push(NotetypeIdentityEntry {
            note_type_id: notetype.id.clone(),
            anki_model_id: None,
            name: notetype.name.clone(),
            fields: notetype
                .fields
                .iter()
                .enumerate()
                .map(|(ord, field)| FieldMergeEntry {
                    field_key: field_merge_key(&field.name, field.config_id),
                    field_name: field.name.clone(),
                    ord: ord as u32,
                    config_id: field.config_id.unwrap_or(0),
                    tag: field.tag.map(|t| t as i32).unwrap_or(0),
                })
                .collect(),
            templates: notetype
                .templates
                .iter()
                .enumerate()
                .map(|(ord, template)| TemplateMergeEntry {
                    template_key: template_merge_key(&template.name, template.config_id),
                    template_name: template.name.clone(),
                    ord: template.ord.unwrap_or(ord as u32),
                    config_id: template.config_id.unwrap_or(0),
                })
                .collect(),
        });
    }
}

pub fn field_merge_key(name: &str, config_id: Option<i64>) -> String {
    stable_merge_key("field", name, config_id)
}

pub fn template_merge_key(name: &str, config_id: Option<i64>) -> String {
    stable_merge_key("template", name, config_id)
}

fn stable_merge_key(kind: &str, name: &str, config_id: Option<i64>) -> String {
    match config_id {
        Some(id) if id != 0 => format!("{kind}:config:{id}"),
        _ => name.to_string(),
    }
}
