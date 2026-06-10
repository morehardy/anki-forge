use std::collections::{BTreeMap, BTreeSet};

use super::{
    render_image_occlusion_cloze, Content, IdentityRecipe, MediaRef, STOCK_IMAGE_OCCLUSION_ID,
};
use crate::diagnostics::{ErrorCode, ErrorCodeExt};
use crate::{IoMode, IoRect};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProductNoteError {
    ImageOcclusionStableIdMissing,
    ImageOcclusionStableIdBlank,
    ImageOcclusionEmptyMasks,
    ImageOcclusionRectEmpty,
    ImageOcclusionRectDuplicate,
}

impl ProductNoteError {
    pub fn code(&self) -> ErrorCode {
        match self {
            Self::ImageOcclusionStableIdMissing => ErrorCode::DeckMissingStableId,
            Self::ImageOcclusionStableIdBlank => ErrorCode::StableIdBlank,
            Self::ImageOcclusionEmptyMasks => ErrorCode::ImageOcclusionEmptyMasks,
            Self::ImageOcclusionRectEmpty => ErrorCode::ImageOcclusionRectEmpty,
            Self::ImageOcclusionRectDuplicate => ErrorCode::ImageOcclusionRectDuplicate,
        }
    }
}

impl ErrorCodeExt for ProductNoteError {
    fn code(&self) -> ErrorCode {
        ProductNoteError::code(self)
    }
}

impl std::fmt::Display for ProductNoteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.code().as_str())
    }
}

impl std::error::Error for ProductNoteError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    note_type_id: String,
    stable_id: Option<String>,
    deck_name: Option<String>,
    identity: Option<IdentityRecipe>,
    fields: BTreeMap<String, Content>,
    tags: Vec<String>,
}

impl Note {
    pub fn new(note_type_id: impl Into<String>) -> Self {
        Self {
            note_type_id: note_type_id.into(),
            stable_id: None,
            deck_name: None,
            identity: None,
            fields: BTreeMap::new(),
            tags: Vec::new(),
        }
    }

    pub fn basic(front: impl Into<String>, back: impl Into<String>) -> Self {
        Self::new("basic").text("Front", front).text("Back", back)
    }

    pub fn cloze(text: impl Into<String>) -> Self {
        Self::new("cloze").html("Text", text).text("Back Extra", "")
    }

    pub fn image_occlusion(image: MediaRef) -> ImageOcclusionNoteBuilder {
        ImageOcclusionNoteBuilder {
            image,
            mode: IoMode::HideAllGuessOne,
            rects: Vec::new(),
            stable_id: None,
            deck_name: None,
            header: String::new(),
            back_extra: String::new(),
            comments: String::new(),
            tags: Vec::new(),
        }
    }

    pub fn stable_id(mut self, stable_id: impl Into<String>) -> Self {
        self.stable_id = Some(stable_id.into());
        self
    }

    pub fn deck(mut self, deck_name: impl Into<String>) -> Self {
        self.deck_name = Some(deck_name.into());
        self
    }

    pub fn identity<I, S>(mut self, fields: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.identity = Some(IdentityRecipe::fields(fields));
        self
    }

    pub fn text(mut self, field: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.insert(field.into(), Content::text(value));
        self
    }

    pub fn html(mut self, field: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.insert(field.into(), Content::html(value));
        self
    }

    pub fn sound(mut self, field: impl Into<String>, media: MediaRef) -> Self {
        self.fields.insert(field.into(), media.sound());
        self
    }

    pub fn image(mut self, field: impl Into<String>, media: MediaRef) -> Self {
        self.fields.insert(field.into(), media.image());
        self
    }

    pub fn extra(mut self, extra: impl Into<String>) -> Self {
        self.fields
            .insert("Back Extra".into(), Content::text(extra));
        self
    }

    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    pub fn note_type_id(&self) -> &str {
        &self.note_type_id
    }

    pub fn stable_id_ref(&self) -> Option<&str> {
        self.stable_id.as_deref()
    }

    pub fn identity_ref(&self) -> Option<&IdentityRecipe> {
        self.identity.as_ref()
    }

    pub fn deck_name(&self) -> Option<&str> {
        self.deck_name.as_deref()
    }

    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    pub fn rendered_fields(&self) -> BTreeMap<String, String> {
        self.fields
            .iter()
            .map(|(field, content)| (field.clone(), content.render()))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageOcclusionNoteBuilder {
    image: MediaRef,
    mode: IoMode,
    rects: Vec<IoRect>,
    stable_id: Option<String>,
    deck_name: Option<String>,
    header: String,
    back_extra: String,
    comments: String,
    tags: Vec<String>,
}

impl ImageOcclusionNoteBuilder {
    pub fn stable_id(mut self, stable_id: impl Into<String>) -> Self {
        self.stable_id = Some(stable_id.into());
        self
    }

    pub fn deck(mut self, deck_name: impl Into<String>) -> Self {
        self.deck_name = Some(deck_name.into());
        self
    }

    pub fn mode(mut self, mode: IoMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn rect(mut self, x: u32, y: u32, width: u32, height: u32) -> Self {
        self.rects.push(IoRect {
            x,
            y,
            width,
            height,
        });
        self
    }

    pub fn rects<I>(mut self, rects: I) -> Self
    where
        I: IntoIterator<Item = IoRect>,
    {
        self.rects.extend(rects);
        self
    }

    pub fn header(mut self, header: impl Into<String>) -> Self {
        self.header = header.into();
        self
    }

    pub fn back_extra(mut self, back_extra: impl Into<String>) -> Self {
        self.back_extra = back_extra.into();
        self
    }

    pub fn comments(mut self, comments: impl Into<String>) -> Self {
        self.comments = comments.into();
        self
    }

    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    pub fn tags<T, I>(mut self, tags: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        self.tags.extend(tags.into_iter().map(Into::into));
        self
    }

    pub fn build(self) -> Result<Note, ProductNoteError> {
        let stable_id = self
            .stable_id
            .ok_or(ProductNoteError::ImageOcclusionStableIdMissing)?;
        let stable_id = stable_id.trim().to_string();
        if stable_id.is_empty() {
            return Err(ProductNoteError::ImageOcclusionStableIdBlank);
        }
        if self.rects.is_empty() {
            return Err(ProductNoteError::ImageOcclusionEmptyMasks);
        }
        let mut seen = BTreeSet::new();
        for rect in &self.rects {
            if rect.width == 0 || rect.height == 0 {
                return Err(ProductNoteError::ImageOcclusionRectEmpty);
            }
            if !seen.insert((rect.x, rect.y, rect.width, rect.height)) {
                return Err(ProductNoteError::ImageOcclusionRectDuplicate);
            }
        }

        let occlusion = render_image_occlusion_cloze(self.mode, &self.rects)
            .expect("validated image occlusion rects should render");
        let mut note = Note::new(STOCK_IMAGE_OCCLUSION_ID)
            .stable_id(stable_id)
            .html("Occlusion", occlusion)
            .html("Image", self.image.image().render())
            .text("Header", self.header)
            .text("Back Extra", self.back_extra)
            .text("Comments", self.comments);
        if let Some(deck_name) = self.deck_name {
            note = note.deck(deck_name);
        }
        for tag in self.tags {
            note = note.tag(tag);
        }
        Ok(note)
    }
}
