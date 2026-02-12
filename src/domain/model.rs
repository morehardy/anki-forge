use crate::domain::ids::ModelId;
use crate::domain::template::Template;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Model {
    pub id: ModelId,
    pub name: String,
    pub fields: Vec<String>,
    pub templates: Vec<Template>,
}

impl Model {
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: ModelId::default(),
            name: name.into(),
            fields: Vec::new(),
            templates: Vec::new(),
        }
    }
}

impl Default for Model {
    fn default() -> Self {
        Self::new("Basic")
    }
}
