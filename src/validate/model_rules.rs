use std::collections::HashSet;

use crate::domain::package_spec::PackageSpec;
use crate::validate::diagnostic::Diagnostic;
use crate::validate::mode::ValidationConfig;

pub fn check(spec: &PackageSpec, mode: ValidationConfig) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut seen_model_names = HashSet::new();

    for (model_index, model) in spec.models.iter().enumerate() {
        let model_name_path = format!("models[{model_index}].name");

        if model.name.trim().is_empty() {
            push_mode_diagnostic(
                &mut diagnostics,
                mode,
                model_name_path,
                "model name must not be empty",
            );
        }

        let canonical_name = model.name.trim().to_ascii_lowercase();
        if !canonical_name.is_empty() && !seen_model_names.insert(canonical_name) {
            push_mode_diagnostic(
                &mut diagnostics,
                mode,
                format!("models[{model_index}]"),
                "duplicate model name",
            );
        }

        if model.fields.is_empty() {
            push_mode_diagnostic(
                &mut diagnostics,
                mode,
                format!("models[{model_index}].fields"),
                "model must define at least one field",
            );
        }

        for (field_index, field_name) in model.fields.iter().enumerate() {
            if field_name.trim().is_empty() {
                push_mode_diagnostic(
                    &mut diagnostics,
                    mode,
                    format!("models[{model_index}].fields[{field_index}]"),
                    "field name must not be empty",
                );
            }
        }
    }

    diagnostics
}

fn push_mode_diagnostic(
    diagnostics: &mut Vec<Diagnostic>,
    mode: ValidationConfig,
    path: impl Into<String>,
    reason: impl Into<String>,
) {
    if mode.is_strict() {
        diagnostics.push(Diagnostic::error(path, reason));
    } else {
        diagnostics.push(Diagnostic::warning(path, reason));
    }
}
