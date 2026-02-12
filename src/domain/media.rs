#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaRef {
    pub logical_name: String,
    pub source_path: String,
}

impl MediaRef {
    #[must_use]
    pub fn new(logical_name: impl Into<String>, source_path: impl Into<String>) -> Self {
        Self {
            logical_name: logical_name.into(),
            source_path: source_path.into(),
        }
    }
}
