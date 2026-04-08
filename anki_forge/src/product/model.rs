use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductDocument {
    document_id: String,
    note_types: Vec<ProductNoteType>,
    pub(super) notes: Vec<ProductNote>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProductNoteType {
    Basic(BasicNoteType),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProductNote {
    Basic(BasicNote),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BasicNoteType {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BasicNote {
    pub id: String,
    pub note_type_id: String,
    pub deck_name: String,
    pub front: String,
    pub back: String,
}

impl ProductDocument {
    pub fn new(document_id: impl Into<String>) -> Self {
        Self {
            document_id: document_id.into(),
            note_types: Vec::new(),
            notes: Vec::new(),
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
