use crate::deck::model::{Deck, DeckNote};
use crate::product::{
    render_image_occlusion_cloze, ProductDocument, STOCK_BASIC_ID, STOCK_CLOZE_ID,
    STOCK_IMAGE_OCCLUSION_ID,
};
use std::path::Path;

impl Deck {
    pub fn into_product_document(self) -> anyhow::Result<ProductDocument> {
        let document_id = self.stable_id.clone().unwrap_or_else(|| self.name.clone());
        let deck_name = self.name.clone();
        let mut product = ProductDocument::new(document_id)
            .with_default_deck(deck_name.clone())
            .with_basic(STOCK_BASIC_ID)
            .with_cloze(STOCK_CLOZE_ID)
            .with_image_occlusion(STOCK_IMAGE_OCCLUSION_ID);

        for note in self.notes {
            product = match note {
                DeckNote::Basic(note) => product.add_basic_note_with_tags(
                    STOCK_BASIC_ID,
                    note.id,
                    deck_name.clone(),
                    note.front,
                    note.back,
                    note.tags,
                ),
                DeckNote::Cloze(note) => product.add_cloze_note_with_tags(
                    STOCK_CLOZE_ID,
                    note.id,
                    deck_name.clone(),
                    note.text,
                    note.extra,
                    note.tags,
                ),
                DeckNote::ImageOcclusion(note) => product.add_image_occlusion_note_with_tags(
                    STOCK_IMAGE_OCCLUSION_ID,
                    note.id,
                    deck_name.clone(),
                    render_image_occlusion_cloze(note.mode, &note.rects)?,
                    format!("<img src=\"{}\">", note.image.name()),
                    note.header,
                    note.back_extra,
                    note.comments,
                    note.tags,
                ),
            };
        }

        Ok(product)
    }

    pub fn lower_authoring(&self) -> anyhow::Result<crate::AuthoringDocument> {
        let product = self.clone().into_product_document()?;
        let mut lowered = product
            .lower()
            .map_err(|err| anyhow::anyhow!("lower product document: {:?}", err))?
            .authoring_document;
        let media = self
            .media
            .values()
            .map(|media| media.to_self_contained_authoring_media())
            .collect::<anyhow::Result<Vec<_>>>()?;
        lowered.media.extend(media);
        Ok(lowered)
    }

    pub fn lower_authoring_with_media_source_dir(
        &self,
        media_source_dir: &Path,
    ) -> anyhow::Result<crate::AuthoringDocument> {
        let product = self.clone().into_product_document()?;
        let mut lowered = product
            .lower()
            .map_err(|err| anyhow::anyhow!("lower product document: {:?}", err))?
            .authoring_document;
        let media = self
            .media
            .values()
            .map(|media| media.to_authoring_media(media_source_dir))
            .collect::<anyhow::Result<Vec<_>>>()?;
        lowered.media.extend(media);
        Ok(lowered)
    }
}
