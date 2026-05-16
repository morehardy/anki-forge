use super::MediaRef;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Content {
    Text(String),
    Html(String),
    Media(MediaRef),
    Composite(Vec<Content>),
}

impl Content {
    pub fn text(value: impl Into<String>) -> Self {
        Self::Text(value.into())
    }

    pub fn html(value: impl Into<String>) -> Self {
        Self::Html(value.into())
    }

    pub fn render(&self) -> String {
        match self {
            Self::Text(value) => escape_html(value),
            Self::Html(value) => value.clone(),
            Self::Media(media) => media.filename().to_string(),
            Self::Composite(items) => items.iter().map(Self::render).collect::<String>(),
        }
    }
}

pub fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
