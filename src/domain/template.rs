#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Template {
    pub name: String,
    pub front: String,
    pub back: String,
}

impl Template {
    #[must_use]
    pub fn new(name: impl Into<String>, front: impl Into<String>, back: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            front: front.into(),
            back: back.into(),
        }
    }
}
