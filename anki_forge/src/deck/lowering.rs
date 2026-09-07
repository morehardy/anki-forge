use crate::deck::model::{Deck, DeckNote};
use crate::product::{
    render_image_occlusion_cloze, ProductDocument, STOCK_BASIC_ID, STOCK_CLOZE_ID,
    STOCK_IMAGE_OCCLUSION_ID,
};
use std::path::Path;

impl Deck {
    pub fn into_product_document(self) -> anyhow::Result<ProductDocument> {
        product_document_from_notes(self.name, self.stable_id, self.notes)
    }

    fn to_product_document(&self) -> anyhow::Result<ProductDocument> {
        product_document_from_notes(
            self.name.clone(),
            self.stable_id.clone(),
            self.notes.iter().cloned(),
        )
    }

    /// Lowers this deck into a self-contained authoring document.
    ///
    /// File-backed media is embedded as inline base64 in this form. For builds,
    /// prefer `write_apkg()` or `lower_authoring_with_media_source_dir(...)` so
    /// large media stays path-backed through normalization.
    pub fn lower_authoring(&self) -> anyhow::Result<crate::authoring::AuthoringDocument> {
        let product = self.to_product_document()?;
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
    ) -> anyhow::Result<crate::authoring::AuthoringDocument> {
        let product = self.to_product_document()?;
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

fn product_document_from_notes(
    deck_name: String,
    stable_id: Option<String>,
    notes: impl IntoIterator<Item = DeckNote>,
) -> anyhow::Result<ProductDocument> {
    let document_id = stable_id.unwrap_or_else(|| deck_name.clone());
    let mut product = ProductDocument::new(document_id)
        .with_default_deck(deck_name.clone())
        .with_basic(STOCK_BASIC_ID)
        .with_cloze(STOCK_CLOZE_ID)
        .with_image_occlusion(STOCK_IMAGE_OCCLUSION_ID);

    for note in notes {
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
