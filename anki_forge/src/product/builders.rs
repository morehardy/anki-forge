use super::{
    assets::{AssetSource, FontBinding},
    helpers::HelperDeclaration,
    lowering::lower_document,
    metadata::{
        FieldMetadataDeclaration, TemplateBrowserAppearanceDeclaration,
        TemplateTargetDeckDeclaration,
    },
    model::{
        BasicNote, ClozeNote, ClozeNoteType, CustomNote, CustomNoteType, ImageOcclusionNote,
        ImageOcclusionNoteType, ProductNote, ProductNoteType,
    },
    ProductDocument, ProductLoweringError,
};

impl ProductDocument {
    pub fn with_default_deck(mut self, deck_name: impl Into<String>) -> Self {
        self.default_deck_name = Some(deck_name.into());
        self
    }

    pub fn default_deck_name(&self) -> Option<&str> {
        self.default_deck_name.as_deref()
    }

    pub fn with_cloze(mut self, id: impl Into<String>) -> Self {
        self.note_types.push(ProductNoteType::Cloze(ClozeNoteType {
            id: id.into(),
            name: None,
        }));
        self
    }

    pub fn with_image_occlusion(mut self, id: impl Into<String>) -> Self {
        self.note_types
            .push(ProductNoteType::ImageOcclusion(ImageOcclusionNoteType {
                id: id.into(),
                name: None,
            }));
        self
    }

    pub fn with_custom_notetype(mut self, notetype: CustomNoteType) -> Self {
        self.note_types.push(ProductNoteType::Custom(notetype));
        self
    }

    pub fn add_basic_note(
        self,
        note_type_id: impl Into<String>,
        id: impl Into<String>,
        deck_name: impl Into<String>,
        front: impl Into<String>,
        back: impl Into<String>,
    ) -> Self {
        self.add_basic_note_with_tags(note_type_id, id, deck_name, front, back, std::iter::empty::<String>())
    }

    pub fn add_basic_note_with_tags(
        mut self,
        note_type_id: impl Into<String>,
        id: impl Into<String>,
        deck_name: impl Into<String>,
        front: impl Into<String>,
        back: impl Into<String>,
        tags: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.notes.push(ProductNote::Basic(BasicNote {
            id: id.into(),
            note_type_id: note_type_id.into(),
            deck_name: deck_name.into(),
            front: front.into(),
            back: back.into(),
            tags: tags.into_iter().map(Into::into).collect(),
        }));
        self
    }

    pub fn add_cloze_note(
        self,
        note_type_id: impl Into<String>,
        id: impl Into<String>,
        deck_name: impl Into<String>,
        text: impl Into<String>,
        back_extra: impl Into<String>,
    ) -> Self {
        self.add_cloze_note_with_tags(
            note_type_id,
            id,
            deck_name,
            text,
            back_extra,
            std::iter::empty::<String>(),
        )
    }

    pub fn add_cloze_note_with_tags(
        mut self,
        note_type_id: impl Into<String>,
        id: impl Into<String>,
        deck_name: impl Into<String>,
        text: impl Into<String>,
        back_extra: impl Into<String>,
        tags: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.notes.push(ProductNote::Cloze(ClozeNote {
            id: id.into(),
            note_type_id: note_type_id.into(),
            deck_name: deck_name.into(),
            text: text.into(),
            back_extra: back_extra.into(),
            tags: tags.into_iter().map(Into::into).collect(),
        }));
        self
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_image_occlusion_note(
        self,
        note_type_id: impl Into<String>,
        id: impl Into<String>,
        deck_name: impl Into<String>,
        occlusion: impl Into<String>,
        image: impl Into<String>,
        header: impl Into<String>,
        back_extra: impl Into<String>,
        comments: impl Into<String>,
    ) -> Self {
        self.add_image_occlusion_note_with_tags(
            note_type_id,
            id,
            deck_name,
            occlusion,
            image,
            header,
            back_extra,
            comments,
            std::iter::empty::<String>(),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_image_occlusion_note_with_tags(
        mut self,
        note_type_id: impl Into<String>,
        id: impl Into<String>,
        deck_name: impl Into<String>,
        occlusion: impl Into<String>,
        image: impl Into<String>,
        header: impl Into<String>,
        back_extra: impl Into<String>,
        comments: impl Into<String>,
        tags: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.notes
            .push(ProductNote::ImageOcclusion(ImageOcclusionNote {
                id: id.into(),
                note_type_id: note_type_id.into(),
                deck_name: deck_name.into(),
                occlusion: occlusion.into(),
                image: image.into(),
                header: header.into(),
                back_extra: back_extra.into(),
                comments: comments.into(),
                tags: tags.into_iter().map(Into::into).collect(),
            }));
        self
    }

    pub fn add_custom_note(mut self, note: CustomNote) -> Self {
        self.notes.push(ProductNote::Custom(note));
        self
    }

    pub fn with_helper(
        mut self,
        note_type_id: impl Into<String>,
        helper: HelperDeclaration,
    ) -> Self {
        self.helpers.push((note_type_id.into(), helper));
        self
    }

    pub fn bundle_inline_template_asset(
        mut self,
        namespace: impl Into<String>,
        filename: impl Into<String>,
        mime: impl Into<String>,
        data_base64: impl Into<String>,
    ) -> Self {
        self.assets.push(AssetSource::InlineTemplateStatic {
            namespace: namespace.into(),
            filename: filename.into(),
            mime: mime.into(),
            data_base64: data_base64.into(),
        });
        self
    }

    pub fn bind_font(
        mut self,
        note_type_id: impl Into<String>,
        family: impl Into<String>,
        filename: impl Into<String>,
    ) -> Self {
        self.font_bindings.push(FontBinding {
            note_type_id: note_type_id.into(),
            family: family.into(),
            filename: filename.into(),
        });
        self
    }

    pub fn with_field_metadata(
        mut self,
        note_type_id: impl Into<String>,
        field: FieldMetadataDeclaration,
    ) -> Self {
        self.field_metadata.push((note_type_id.into(), field));
        self
    }

    pub fn field_metadata_for(&self, note_type_id: &str) -> Vec<FieldMetadataDeclaration> {
        self.field_metadata
            .iter()
            .filter(|(target_note_type_id, _)| target_note_type_id == note_type_id)
            .map(|(_, field)| field.clone())
            .collect()
    }

    pub fn with_browser_appearance(
        mut self,
        note_type_id: impl Into<String>,
        declaration: TemplateBrowserAppearanceDeclaration,
    ) -> Self {
        self.browser_appearance
            .push((note_type_id.into(), declaration));
        self
    }

    pub fn browser_appearance_for(
        &self,
        note_type_id: &str,
        template_name: &str,
    ) -> Option<TemplateBrowserAppearanceDeclaration> {
        self.browser_appearance
            .iter()
            .find(|(target_note_type_id, declaration)| {
                target_note_type_id == note_type_id && declaration.template_name == template_name
            })
            .map(|(_, declaration)| declaration.clone())
    }

    pub fn with_template_target_deck(
        mut self,
        note_type_id: impl Into<String>,
        declaration: TemplateTargetDeckDeclaration,
    ) -> Self {
        self.template_target_decks
            .push((note_type_id.into(), declaration));
        self
    }

    pub fn template_target_deck_for(
        &self,
        note_type_id: &str,
        template_name: &str,
    ) -> Option<TemplateTargetDeckDeclaration> {
        self.template_target_decks
            .iter()
            .find(|(target_note_type_id, declaration)| {
                target_note_type_id == note_type_id && declaration.template_name == template_name
            })
            .map(|(_, declaration)| declaration.clone())
    }

    pub fn helpers_for(&self, note_type_id: &str) -> Vec<HelperDeclaration> {
        self.helpers
            .iter()
            .filter(|(target_note_type_id, _)| target_note_type_id == note_type_id)
            .map(|(_, helper)| helper.clone())
            .collect()
    }

    pub fn lower(&self) -> Result<super::lowering::LoweringPlan, ProductLoweringError> {
        lower_document(self)
    }
}
