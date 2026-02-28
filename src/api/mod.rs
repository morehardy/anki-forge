use crate::domain::deck::Deck;
use crate::domain::model::Model;
use crate::domain::note::Note;
use crate::domain::package_spec::PackageSpec;
use crate::domain::template::Template;
use crate::package_builder::PackageBuilder;

#[derive(Debug, Default, Clone, Copy)]
pub struct Facade;

impl Facade {
    #[must_use]
    pub fn builder() -> PackageBuilder {
        PackageBuilder::new()
    }

    #[must_use]
    pub fn from_spec(spec: PackageSpec) -> PackageBuilder {
        PackageBuilder::new().with_spec(spec)
    }

    #[must_use]
    pub fn minimal_basic(front: impl Into<String>, back: impl Into<String>) -> PackageBuilder {
        let model = Model::new("Basic")
            .with_field("Front")
            .with_field("Back")
            .with_template(Template::basic());

        PackageBuilder::new()
            .add_deck(Deck::default())
            .add_model(model)
            .add_note(Note::new([front.into(), back.into()]))
    }
}
