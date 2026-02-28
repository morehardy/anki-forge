use crate::domain::ids::DeckId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deck {
    pub id: DeckId,
    pub name: String,
}

impl Deck {
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: DeckId::default(),
            name: name.into(),
        }
    }

    #[must_use]
    pub fn with_id(mut self, id: DeckId) -> Self {
        self.id = id;
        self
    }
}

impl Default for Deck {
    fn default() -> Self {
        Self::new("Default")
    }
}
