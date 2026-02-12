use crate::domain::ids::{CardId, DeckId, NoteId};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CardMeta {
    pub id: CardId,
    pub note_id: NoteId,
    pub deck_id: DeckId,
    pub template_ord: u16,
}
