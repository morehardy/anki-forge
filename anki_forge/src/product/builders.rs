use super::{lowering::lower_document, model::BasicNote, ProductDocument, ProductLoweringError};

impl ProductDocument {
    pub fn add_basic_note(
        mut self,
        id: impl Into<String>,
        note_type_id: impl Into<String>,
        deck_name: impl Into<String>,
        front: impl Into<String>,
        back: impl Into<String>,
    ) -> Self {
        self.notes.push(super::model::ProductNote::Basic(BasicNote {
            id: id.into(),
            note_type_id: note_type_id.into(),
            deck_name: deck_name.into(),
            front: front.into(),
            back: back.into(),
        }));
        self
    }

    pub fn lower(&self) -> Result<super::lowering::LoweringPlan, ProductLoweringError> {
        lower_document(self)
    }
}

