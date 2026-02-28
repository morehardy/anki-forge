use crate::domain::deck::Deck;
use crate::domain::media::MediaRef;
use crate::domain::model::Model;
use crate::domain::note::Note;
use crate::error::SpecError;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PackageSpec {
    pub decks: Vec<Deck>,
    pub models: Vec<Model>,
    pub notes: Vec<Note>,
    pub media: Vec<MediaRef>,
}

impl PackageSpec {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.decks.is_empty()
            && self.models.is_empty()
            && self.notes.is_empty()
            && self.media.is_empty()
    }

    pub fn add_deck(&mut self, deck: Deck) {
        self.decks.push(deck);
    }

    pub fn add_model(&mut self, model: Model) {
        self.models.push(model);
    }

    pub fn add_note(&mut self, note: Note) {
        self.notes.push(note);
    }

    pub fn add_media(&mut self, media: MediaRef) {
        self.media.push(media);
    }

    pub fn validate_for_build(&self) -> Result<(), SpecError> {
        for (deck_index, deck) in self.decks.iter().enumerate() {
            if deck.name.trim().is_empty() {
                return Err(SpecError::EmptyDeckName { deck_index });
            }
        }

        for (model_index, model) in self.models.iter().enumerate() {
            if model.name.trim().is_empty() {
                return Err(SpecError::EmptyModelName { model_index });
            }

            if model.fields.is_empty() {
                return Err(SpecError::ModelHasNoFields { model_index });
            }

            for (field_index, field_name) in model.fields.iter().enumerate() {
                if field_name.trim().is_empty() {
                    return Err(SpecError::EmptyModelFieldName {
                        model_index,
                        field_index,
                    });
                }
            }

            if model.templates.is_empty() {
                return Err(SpecError::ModelHasNoTemplates { model_index });
            }

            for (template_index, template) in model.templates.iter().enumerate() {
                if template.name.trim().is_empty() {
                    return Err(SpecError::EmptyTemplateName {
                        model_index,
                        template_index,
                    });
                }
                if template.front.trim().is_empty() {
                    return Err(SpecError::EmptyTemplateFront {
                        model_index,
                        template_index,
                    });
                }
                if template.back.trim().is_empty() {
                    return Err(SpecError::EmptyTemplateBack {
                        model_index,
                        template_index,
                    });
                }
            }
        }

        if !self.notes.is_empty() && self.models.is_empty() {
            return Err(SpecError::NotesRequireModel);
        }

        if let Some(model) = self.models.first() {
            let expected = model.fields.len();
            for (note_index, note) in self.notes.iter().enumerate() {
                if note.fields.len() != expected {
                    return Err(SpecError::NoteFieldCountMismatch {
                        note_index,
                        expected,
                        actual: note.fields.len(),
                    });
                }
            }
        }

        for (media_index, media) in self.media.iter().enumerate() {
            if media.logical_name.trim().is_empty() {
                return Err(SpecError::EmptyMediaLogicalName { media_index });
            }
            if media.source_path.trim().is_empty() {
                return Err(SpecError::EmptyMediaSourcePath { media_index });
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::PackageSpec;
    use crate::domain::deck::Deck;
    use crate::domain::model::Model;
    use crate::domain::note::Note;
    use crate::domain::template::Template;
    use crate::error::SpecError;

    fn basic_model() -> Model {
        Model::new("Basic")
            .with_field("Front")
            .with_field("Back")
            .with_template(Template::basic())
    }

    #[test]
    fn domain_validate_accepts_minimal_valid_spec() {
        let mut spec = PackageSpec::default();
        spec.add_deck(Deck::default());
        spec.add_model(basic_model());
        spec.add_note(Note::new(["Question", "Answer"]));

        assert!(spec.validate_for_build().is_ok());
    }

    #[test]
    fn domain_validate_rejects_empty_deck_name() {
        let mut spec = PackageSpec::default();
        spec.add_deck(Deck::new("  "));

        let error = spec.validate_for_build().expect_err("spec must fail");
        assert_eq!(error, SpecError::EmptyDeckName { deck_index: 0 });
    }

    #[test]
    fn domain_validate_rejects_note_field_count_mismatch() {
        let mut spec = PackageSpec::default();
        spec.add_model(basic_model());
        spec.add_note(Note::new(["only one field"]));

        let error = spec.validate_for_build().expect_err("spec must fail");
        assert_eq!(
            error,
            SpecError::NoteFieldCountMismatch {
                note_index: 0,
                expected: 2,
                actual: 1
            }
        );
    }
}
