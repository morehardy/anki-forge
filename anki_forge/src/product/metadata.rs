use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldMetadataDeclaration {
    pub field_name: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub role_hint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateBrowserAppearanceDeclaration {
    pub template_name: String,
    #[serde(default)]
    pub question_format: Option<String>,
    #[serde(default)]
    pub answer_format: Option<String>,
    #[serde(default)]
    pub font_name: Option<String>,
    #[serde(default)]
    pub font_size: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateTargetDeckDeclaration {
    pub template_name: String,
    pub deck_name: String,
}
