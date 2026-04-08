use super::{
    lowering::lower_document,
    model::{
        BasicNote, ClozeNote, ClozeNoteType, CustomNote, CustomNoteType, ImageOcclusionNote,
        ImageOcclusionNoteType, ProductNote, ProductNoteType,
    },
    ProductDocument, ProductLoweringError,
};

impl ProductDocument {
    pub fn with_cloze(mut self, id: impl Into<String>) -> Self {
        self.note_types
            .push(ProductNoteType::Cloze(ClozeNoteType { id: id.into(), name: None }));
        self
    }

    pub fn with_image_occlusion(mut self, id: impl Into<String>) -> Self {
        self.note_types.push(ProductNoteType::ImageOcclusion(
            ImageOcclusionNoteType {
                id: id.into(),
                name: None,
            },
        ));
        self
    }

    pub fn with_custom_notetype(mut self, notetype: CustomNoteType) -> Self {
        self.note_types.push(ProductNoteType::Custom(notetype));
        self
    }

    pub fn add_basic_note(
        mut self,
        note_type_id: impl Into<String>,
        id: impl Into<String>,
        deck_name: impl Into<String>,
        front: impl Into<String>,
        back: impl Into<String>,
    ) -> Self {
        self.notes.push(ProductNote::Basic(BasicNote {
            id: id.into(),
            note_type_id: note_type_id.into(),
            deck_name: deck_name.into(),
            front: front.into(),
            back: back.into(),
        }));
        self
    }

    pub fn add_cloze_note(
        mut self,
        note_type_id: impl Into<String>,
        id: impl Into<String>,
        deck_name: impl Into<String>,
        text: impl Into<String>,
        back_extra: impl Into<String>,
    ) -> Self {
        self.notes.push(ProductNote::Cloze(ClozeNote {
            id: id.into(),
            note_type_id: note_type_id.into(),
            deck_name: deck_name.into(),
            text: text.into(),
            back_extra: back_extra.into(),
        }));
        self
    }

    pub fn add_image_occlusion_note(
        mut self,
        note_type_id: impl Into<String>,
        id: impl Into<String>,
        deck_name: impl Into<String>,
        occlusion: impl Into<String>,
        image: impl Into<String>,
        header: impl Into<String>,
        back_extra: impl Into<String>,
        comments: impl Into<String>,
    ) -> Self {
        self.notes.push(ProductNote::ImageOcclusion(ImageOcclusionNote {
            id: id.into(),
            note_type_id: note_type_id.into(),
            deck_name: deck_name.into(),
            occlusion: occlusion.into(),
            image: image.into(),
            header: header.into(),
            back_extra: back_extra.into(),
            comments: comments.into(),
        }));
        self
    }

    pub fn add_custom_note(mut self, note: CustomNote) -> Self {
        self.notes.push(ProductNote::Custom(note));
        self
    }

    pub fn lower(&self) -> Result<super::lowering::LoweringPlan, ProductLoweringError> {
        lower_document(self)
    }
}
