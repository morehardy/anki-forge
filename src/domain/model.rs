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

    #[must_use]
    pub fn with_id(mut self, id: ModelId) -> Self {
        self.id = id;
        self
    }

    #[must_use]
    pub fn with_field(mut self, field_name: impl Into<String>) -> Self {
        self.fields.push(field_name.into());
        self
    }

    #[must_use]
    pub fn with_template(mut self, template: Template) -> Self {
        self.templates.push(template);
        self
    }
}

impl Default for Model {
    fn default() -> Self {
        Self::new("Basic")
    }
}
