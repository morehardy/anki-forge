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
    pub spec: PackageSpec,
    pub diagnostics: Vec<Diagnostic>,
}

impl ValidationReport {
    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(Diagnostic::is_error)
    }
}

#[must_use]
pub fn normalize_and_validate(mut spec: PackageSpec, config: ValidationConfig) -> ValidationReport {
    let mut diagnostics = Vec::new();

    diagnostics.extend(deck_name::normalize(&mut spec, config));
    diagnostics.extend(tags::normalize(&mut spec, config));
    diagnostics.extend(media_rules::normalize(&mut spec, config));

    diagnostics.extend(template_rules::check(&spec, config));
    diagnostics.extend(model_rules::check(&spec, config));

    ValidationReport { spec, diagnostics }
}

#[must_use]
pub fn validate(spec: &PackageSpec, config: ValidationConfig) -> ValidationReport {
    normalize_and_validate(spec.clone(), config)
}

#[cfg(test)]
mod tests {
    use super::mode::ValidationConfig;
    use super::normalize_and_validate;
    use crate::domain::deck::Deck;
    use crate::domain::model::Model;
    use crate::domain::note::Note;
    use crate::domain::package_spec::PackageSpec;
    use crate::domain::template::Template;

    #[test]
    fn strict_mode_emits_errors_for_malformed_values() {
        let mut spec = PackageSpec::default();
        spec.add_deck(Deck::new("Parent::::Child"));
        spec.add_model(Model::new(" "));
        spec.add_note(Note::new(["front", "back"]).with_tag("   "));

        let report = normalize_and_validate(spec, ValidationConfig::Strict);

        assert!(report.has_errors());
        assert!(report.diagnostics.iter().all(|diag| {
            !diag.path.is_empty() && !diag.reason.is_empty()
        }));
    }

    #[test]
    fn permissive_mode_repairs_common_inputs() {
        let mut spec = PackageSpec::default();
        spec.add_deck(Deck::new("Parent::::Child"));
        spec.add_model(
            Model::new("Basic")
                .with_field("Front")
                .with_field("Back")
                .with_template(Template::basic()),
        );
        spec.add_note(Note::new(["front", "back"]).with_tag("TagA TAGB"));

        let report = normalize_and_validate(spec, ValidationConfig::Permissive);

        assert_eq!(report.spec.decks[0].name, "Parent::Child");
        assert_eq!(report.spec.notes[0].tags, vec!["taga", "tagb"]);
    }
}
