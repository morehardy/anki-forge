#[cfg(all(test, feature = "internal-tools"))]
use super::assets::{AssetSource, FontBinding};
#[cfg(all(test, feature = "internal-tools"))]
use super::model::{BasicNote, CustomNote, CustomNoteType, ProductNote, ProductNoteType};
use super::{
    helpers::HelperDeclaration,
    lowering::lower_document,
    metadata::{
        FieldMetadataDeclaration, TemplateBrowserAppearanceDeclaration,
        TemplateTargetDeckDeclaration,
    },
    ProductDocument, ProductLoweringError,
};

impl ProductDocument {
    pub fn field_metadata_for(&self, note_type_id: &str) -> Vec<FieldMetadataDeclaration> {
        self.field_metadata
            .iter()
            .filter(|(target_note_type_id, _)| target_note_type_id == note_type_id)
            .map(|(_, field)| field.clone())
            .collect()
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

    #[cfg(all(test, feature = "internal-tools"))]
    pub fn with_custom_notetype(mut self, notetype: CustomNoteType) -> Self {
        self.note_types.push(ProductNoteType::Custom(notetype));
        self
    }

    #[cfg(all(test, feature = "internal-tools"))]
    pub fn add_custom_note(mut self, note: CustomNote) -> Self {
        self.notes.push(ProductNote::Custom(note));
        self
    }

    #[cfg(all(test, feature = "internal-tools"))]
    pub fn with_helper(
        mut self,
        note_type_id: impl Into<String>,
        helper: HelperDeclaration,
    ) -> Self {
        self.helpers.push((note_type_id.into(), helper));
        self
    }

    #[cfg(all(test, feature = "internal-tools"))]
    pub fn add_basic_note(
        self,
        note_type_id: impl Into<String>,
        id: impl Into<String>,
        deck_name: impl Into<String>,
        front: impl Into<String>,
        back: impl Into<String>,
    ) -> Self {
        self.add_basic_note_with_tags(
            note_type_id,
            id,
            deck_name,
            front,
            back,
            std::iter::empty::<String>(),
        )
    }

    #[cfg(all(test, feature = "internal-tools"))]
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

    #[cfg(all(test, feature = "internal-tools"))]
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

    #[cfg(all(test, feature = "internal-tools"))]
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
}
