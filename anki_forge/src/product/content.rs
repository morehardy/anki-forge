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

    pub(crate) fn into_rendered(self) -> String {
        match self {
            Self::Text(value) => escape_html(&value),
            Self::Html(value) => value,
            Self::Media(media) => media.filename().to_string(),
            Self::Composite(items) => items.into_iter().map(Self::into_rendered).collect(),
        }
    }
}

pub fn escape_html(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_html_escapes_text_once_in_source_order() {
        assert_eq!(
            escape_html("AT&T <b>\"phone\"</b> 'ok'"),
            "AT&amp;T &lt;b&gt;&quot;phone&quot;&lt;/b&gt; &#39;ok&#39;"
        );
    }

    #[test]
    fn consuming_nested_content_preserves_rendering() {
        let mut media = crate::product::MediaRegistry::default();
        let image = media
            .add_bytes("image", b"image".to_vec())
            .unwrap()
            .export_as("image.bin")
            .unwrap();
        let content = Content::Composite(vec![
            Content::html("<b>原始 &amp; HTML</b>"),
            Content::text("<&>\"' 中文"),
            Content::Composite(vec![Content::Media(image), Content::text("&amp;")]),
        ]);
        assert_eq!(content.clone().into_rendered(), content.render());
    }
}
