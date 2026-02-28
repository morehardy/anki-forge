use std::collections::HashSet;

use crate::domain::package_spec::PackageSpec;
use crate::validate::diagnostic::Diagnostic;
use crate::validate::mode::ValidationConfig;

pub fn check(spec: &PackageSpec, mode: ValidationConfig) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for (model_index, model) in spec.models.iter().enumerate() {
        let mut seen_template_names = HashSet::new();

        if model.templates.is_empty() {
            push_mode_diagnostic(
                &mut diagnostics,
                mode,
                format!("models[{model_index}].templates"),
                "model must define at least one template",
            );
            continue;
        }

        for (template_index, template) in model.templates.iter().enumerate() {
            if template.name.trim().is_empty() {
                push_mode_diagnostic(
                    &mut diagnostics,
                    mode,
                    format!("models[{model_index}].templates[{template_index}].name"),
                    "template name must not be empty",
                );
            }
            if template.front.trim().is_empty() {
                push_mode_diagnostic(
                    &mut diagnostics,
                    mode,
                    format!("models[{model_index}].templates[{template_index}].front"),
                    "template front must not be empty",
                );
            }
            if template.back.trim().is_empty() {
                push_mode_diagnostic(
                    &mut diagnostics,
                    mode,
                    format!("models[{model_index}].templates[{template_index}].back"),
                    "template back must not be empty",
                );
            }

            let canonical_name = template.name.trim().to_ascii_lowercase();
            if !canonical_name.is_empty() && !seen_template_names.insert(canonical_name) {
                push_mode_diagnostic(
                    &mut diagnostics,
                    mode,
                    format!("models[{model_index}].templates[{template_index}]"),
                    "duplicate template name within model",
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
