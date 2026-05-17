use super::{IdentityRecipe, Template};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FieldKey(String);

impl FieldKey {
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    key: FieldKey,
    name: String,
    identity: bool,
    sort: bool,
    required: bool,
    optional: bool,
    key_auto_derived: bool,
}

impl Field {
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            key: FieldKey::new(slug_key(&name)),
            name,
            identity: false,
            sort: false,
            required: false,
            optional: false,
            key_auto_derived: true,
        }
    }

    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = FieldKey::new(key);
        self.key_auto_derived = false;
        self
    }

    pub fn identity(mut self) -> Self {
        self.identity = true;
        self
    }

    pub fn sort(mut self) -> Self {
        self.sort = true;
        self
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self.optional = false;
        self
    }

    pub fn optional(mut self) -> Self {
        self.optional = true;
        self.required = false;
        self
    }

    pub fn key_ref(&self) -> &FieldKey {
        &self.key
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn is_identity(&self) -> bool {
        self.identity
    }

    pub fn is_sort(&self) -> bool {
        self.sort
    }

    pub fn is_required(&self) -> bool {
        self.required
    }

    pub fn is_optional(&self) -> bool {
        self.optional
    }

    pub fn key_auto_derived(&self) -> bool {
        self.key_auto_derived
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteType {
    id: String,
    name: Option<String>,
    fields: Vec<Field>,
    templates: Vec<Template>,
    identity: Option<IdentityRecipe>,
}

impl NoteType {
    pub fn custom(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: None,
            fields: Vec::new(),
            templates: Vec::new(),
            identity: None,
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn field(mut self, field: Field) -> Self {
        self.fields.push(field);
        self
    }

    pub fn template(mut self, template: Template) -> Self {
        self.templates.push(template);
        self
    }

    pub fn identity(mut self, identity: IdentityRecipe) -> Self {
        self.identity = Some(identity);
        self
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name_ref(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn fields(&self) -> &[Field] {
        &self.fields
    }

    pub fn templates(&self) -> &[Template] {
        &self.templates
    }

    pub fn identity_ref(&self) -> Option<&IdentityRecipe> {
        self.identity.as_ref()
    }
}

fn slug_key(name: &str) -> String {
    name.trim()
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}
