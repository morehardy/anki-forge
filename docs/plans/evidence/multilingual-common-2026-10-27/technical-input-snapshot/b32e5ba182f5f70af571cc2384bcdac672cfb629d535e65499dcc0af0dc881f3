/// The intended role of a typed media content node, not a decoding guarantee.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum MediaUsage {
    /// An image rendered through an HTML image element.
    Image,
    /// An Anki sound reference, which can play audio or video.
    Sound,
}

impl MediaUsage {
    pub(crate) fn accepts(self, media_type: &str) -> bool {
        let category = media_type.split('/').next().unwrap_or("");
        match self {
            Self::Image => category.eq_ignore_ascii_case("image"),
            Self::Sound => {
                category.eq_ignore_ascii_case("audio") || category.eq_ignore_ascii_case("video")
            }
        }
    }
}
