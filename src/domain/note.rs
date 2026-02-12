#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Note {
    pub fields: Vec<String>,
    pub tags: Vec<String>,
    pub guid_override: Option<String>,
}
