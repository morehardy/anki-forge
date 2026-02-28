use crate::domain::deck::Deck;
use crate::domain::media::MediaRef;
use crate::domain::model::Model;
use crate::domain::note::Note;
use crate::domain::package_spec::PackageSpec;
use crate::options::BuildOptions;
use crate::validate::mode::ValidationConfig;

#[derive(Debug, Clone)]
pub struct PackageBuilder {
    options: BuildOptions,
    spec: PackageSpec,
}

impl PackageBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            options: BuildOptions::default(),
            spec: PackageSpec::default(),
        }
    }

    #[must_use]
    pub const fn options(&self) -> BuildOptions {
        self.options
    }

    #[must_use]
    pub fn with_options(mut self, options: BuildOptions) -> Self {
        self.options = options;
        self
    }

    #[must_use]
    pub fn with_spec(mut self, spec: PackageSpec) -> Self {
        self.spec = spec;
        self
    }

    #[must_use]
    pub fn add_deck(mut self, deck: Deck) -> Self {
        self.spec.add_deck(deck);
        self
    }

    #[must_use]
    pub fn add_model(mut self, model: Model) -> Self {
        self.spec.add_model(model);
        self
    }

    #[must_use]
    pub fn add_note(mut self, note: Note) -> Self {
        self.spec.add_note(note);
        self
    }

    #[must_use]
    pub fn add_media(mut self, media: MediaRef) -> Self {
        self.spec.add_media(media);
        self
    }

    #[must_use]
    pub const fn spec(&self) -> &PackageSpec {
        &self.spec
    }

    pub fn build(self) -> crate::Result<PackageSpec> {
        let report =
            crate::validate::normalize_and_validate(self.spec, ValidationConfig::from(self.options.validation_mode));

        report.spec.validate_for_build()?;
        Ok(report.spec)
    }
}

impl Default for PackageBuilder {
    fn default() -> Self {
        Self::new()
    }
}
