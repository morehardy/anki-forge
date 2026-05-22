use crate::build::{BuildOptions, UpdateSafetyMode};
use crate::diagnostics::{DiagnosticCode, Severity};

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
    Ok(writer_core::policy_ref(id, version))
}
