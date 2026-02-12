use crate::domain::deck::Deck;
use crate::domain::media::MediaRef;
use crate::domain::model::Model;
use crate::domain::note::Note;

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
}
