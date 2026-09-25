use std::borrow::Cow;

use crate::Media;

/// Typed field content. Strings are plain text; HTML requires [`Self::html`].
///
/// Image and sound nodes retain their media snapshots until the project is
/// rendered. Constructing content does not register media or add a note.
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use = "pass this content to a note field or a content sequence"]
pub struct Content(Node);

#[derive(Debug, Clone, PartialEq, Eq)]
enum Node {
    Text(String),
    Html(String),
    Image(Media),
    Sound(Media),
    Sequence(Vec<Content>),
}

impl Content {
    pub(crate) fn has_value(&self) -> bool {
        match &self.0 {
            Node::Text(value) => !value.trim().is_empty(),
            Node::Html(value) => {
                !crate::authoring_core::strip_html_preserving_media_filenames(value)
                    .trim()
                    .is_empty()
            }
            Node::Image(_) | Node::Sound(_) => true,
            Node::Sequence(values) => values.iter().any(Self::has_value),
        }
    }

    /// Creates text that is escaped once when the project is rendered.
    pub fn text(value: impl Into<String>) -> Self {
        Self(Node::Text(value.into()))
    }

    /// Creates explicit HTML, preserved as supplied during rendering.
    pub fn html(value: impl Into<String>) -> Self {
        Self(Node::Html(value.into()))
    }

    /// Combines typed values in order, without rendering or losing dependencies.
    pub fn sequence<I, C>(values: I) -> Self
    where
        I: IntoIterator<Item = C>,
        C: Into<Content>,
    {
        Self(Node::Sequence(values.into_iter().map(Into::into).collect()))
    }

    pub(crate) fn image(media: Media) -> Self {
        Self(Node::Image(media))
    }
    pub(crate) fn sound(media: Media) -> Self {
        Self(Node::Sound(media))
    }

    pub(crate) fn visit_media(&self, visitor: &mut impl FnMut(&Media)) {
        match &self.0 {
            Node::Image(media) | Node::Sound(media) => visitor(media),
            Node::Sequence(values) => {
                for value in values {
                    value.visit_media(visitor);
                }
            }
            Node::Text(_) | Node::Html(_) => {}
        }
    }

    pub(crate) fn render(&self) -> String {
        match &self.0 {
            Node::Text(value) => crate::product::content::escape_html(value),
            Node::Html(value) => value.clone(),
            Node::Image(media) => format!("<img src=\"{}\">", image_url_path(media.filename())),
            Node::Sound(media) => {
                // Sound references decode HTML entities once, including in
                // Anki's player. Preserve literal entity-like filename text.
                format!(
                    "[sound:{}]",
                    crate::product::content::escape_html(media.filename())
                )
            }
            Node::Sequence(values) => values.iter().map(Self::render).collect(),
        }
    }
}

// A filename is not a URL: percent signs and fragment delimiters are literal
// filename bytes. Encode a single path segment before embedding it in HTML.
fn image_url_path(filename: &str) -> String {
    use std::fmt::Write;
    let mut encoded = String::with_capacity(filename.len());
    for byte in filename.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(char::from(byte));
        } else {
            write!(&mut encoded, "%{byte:02X}").expect("write to String");
        }
    }
    encoded
}

impl From<String> for Content {
    fn from(value: String) -> Self {
        Self::text(value)
    }
}

impl From<&str> for Content {
    fn from(value: &str) -> Self {
        Self::text(value)
    }
}

impl From<&String> for Content {
    fn from(value: &String) -> Self {
        Self::text(value.clone())
    }
}

impl From<Cow<'_, str>> for Content {
    fn from(value: Cow<'_, str>) -> Self {
        Self::text(value.into_owned())
    }
}
