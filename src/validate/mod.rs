pub mod deck_name;
pub mod diagnostic;
pub mod media_rules;
pub mod mode;
pub mod model_rules;
pub mod tags;
pub mod template_rules;

use crate::domain::package_spec::PackageSpec;
use crate::validate::diagnostic::Diagnostic;
use crate::validate::mode::ValidationConfig;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ValidationReport {
    pub diagnostics: Vec<Diagnostic>,
}

#[must_use]
pub fn validate(spec: &PackageSpec, _config: ValidationConfig) -> ValidationReport {
    let mut diagnostics = Vec::new();
    diagnostics.extend(deck_name::check(spec));
    diagnostics.extend(tags::check(spec));
    diagnostics.extend(template_rules::check(spec));
    diagnostics.extend(model_rules::check(spec));
    diagnostics.extend(media_rules::check(spec));

    ValidationReport { diagnostics }
}
