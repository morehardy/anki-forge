use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductDocument {
    document_id: String,
    pub(super) note_types: Vec<ProductNoteType>,
    pub(super) notes: Vec<ProductNote>,
    pub(super) helpers: Vec<(String, super::helpers::HelperDeclaration)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProductNoteType {
    Basic(BasicNoteType),
    Cloze(ClozeNoteType),
    ImageOcclusion(ImageOcclusionNoteType),
    Custom(CustomNoteType),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProductNote {
    Basic(BasicNote),
    Cloze(ClozeNote),
    ImageOcclusion(ImageOcclusionNote),
    Custom(CustomNote),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BasicNoteType {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClozeNoteType {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageOcclusionNoteType {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomNoteType {
    pub id: String,
    pub name: Option<String>,
    pub fields: Vec<CustomField>,
    pub templates: Vec<CustomTemplate>,
    pub css: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BasicNote {
    pub id: String,
    pub note_type_id: String,
    pub deck_name: String,
    pub front: String,
    pub back: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClozeNote {
    pub id: String,
    pub note_type_id: String,
    pub deck_name: String,
    pub text: String,
    pub back_extra: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageOcclusionNote {
    pub id: String,
    pub note_type_id: String,
    pub deck_name: String,
    pub occlusion: String,
    pub image: String,
    pub header: String,
    pub back_extra: String,
    pub comments: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomField {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomTemplate {
    pub name: String,
    pub question_format: String,
    pub answer_format: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomNote {
    pub id: String,
    pub note_type_id: String,
    pub deck_name: String,
    pub fields: BTreeMap<String, String>,
    pub tags: Vec<String>,
}

impl ProductDocument {
    pub fn new(document_id: impl Into<String>) -> Self {
        Self {
            document_id: document_id.into(),
            note_types: Vec::new(),
            notes: Vec::new(),
            helpers: Vec::new(),
        }
    }

    pub fn with_basic(mut self, id: impl Into<String>) -> Self {
        self.note_types.push(ProductNoteType::Basic(BasicNoteType {
            id: id.into(),
            name: None,
        }));
        self
    }

    pub fn document_id(&self) -> &str {
        &self.document_id
    }

    pub fn note_types(&self) -> &[ProductNoteType] {
        &self.note_types
    }

    pub fn notes(&self) -> &[ProductNote] {
        &self.notes
    }
}
