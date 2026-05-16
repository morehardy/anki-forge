#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MediaRef {
    filename: String,
}

#[derive(Debug, Clone, Default)]
pub struct MediaRegistry;

impl MediaRef {
    #[allow(dead_code)]
    pub(crate) fn new(filename: impl Into<String>) -> Self {
        Self {
            filename: filename.into(),
        }
    }

    pub fn filename(&self) -> &str {
        &self.filename
    }

    pub fn sound(&self) -> crate::product::Content {
        crate::product::Content::Html(format!("[sound:{}]", self.filename))
    }

    pub fn image(&self) -> crate::product::Content {
        crate::product::Content::Html(format!("<img src=\"{}\">", self.filename))
    }
}
