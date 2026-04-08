use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductDocument {
    document_id: String,
    note_types: Vec<ProductNoteType>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProductNoteType {
    Basic(BasicNoteType),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BasicNoteType {
    pub id: String,
    pub name: Option<String>,
}

impl ProductDocument {
    pub fn new(document_id: impl Into<String>) -> Self {
        Self {
            document_id: document_id.into(),
            note_types: Vec::new(),
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
}
