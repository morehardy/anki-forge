use std::collections::BTreeMap;

use unicase::UniCase;
use unicode_normalization::UnicodeNormalization;

use super::Media;

/// One collision rule shared by model assets and project dependency collection.
#[derive(Debug, Clone, Default)]
pub(crate) struct Assets(BTreeMap<UniCase<String>, Media>);

/// The same portable filename identity governs collection and update history.
pub(crate) fn filename_identity(name: &str) -> UniCase<String> {
    UniCase::unicode(name.nfc().collect::<String>())
}

/// Why two assets cannot occupy the same portable filename space.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum MediaConflictKind {
    /// Different spellings collide after NFC normalization and case folding.
    PortableNameCollision,
    /// The same filename refers to different bytes.
    DifferentContent,
}

#[derive(Debug)]
pub(crate) struct AssetConflict {
    pub(crate) kind: MediaConflictKind,
    pub(crate) existing_name: String,
    pub(crate) incoming_name: String,
}

impl AssetConflict {
    pub(crate) fn code(&self) -> &'static str {
        match self.kind {
            MediaConflictKind::PortableNameCollision => "MEDIA.EXPORT_NAME_COLLISION",
            MediaConflictKind::DifferentContent => "MEDIA.DUPLICATE_FILENAME_CONFLICT",
        }
    }
}

impl std::fmt::Display for AssetConflict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            MediaConflictKind::PortableNameCollision => write!(
                f,
                "media names {:?} and {:?} collide after Unicode normalization and case folding",
                self.existing_name, self.incoming_name
            ),
            MediaConflictKind::DifferentContent => write!(
                f,
                "media name {:?} is bound to different content",
                self.incoming_name
            ),
        }
    }
}

impl Assets {
    pub(crate) fn add(&mut self, media: Media) -> Result<(), AssetConflict> {
        if self.check(&media)? {
            let key = filename_identity(media.filename());
            self.0.insert(key, media);
        }
        Ok(())
    }

    pub(crate) fn check(&self, media: &Media) -> Result<bool, AssetConflict> {
        let key = filename_identity(media.filename());
        if let Some(existing) = self.0.get(&key) {
            if existing.filename() != media.filename() {
                return Err(AssetConflict {
                    kind: MediaConflictKind::PortableNameCollision,
                    existing_name: existing.filename().into(),
                    incoming_name: media.filename().into(),
                });
            }
            if !existing.same_content(media) {
                return Err(AssetConflict {
                    kind: MediaConflictKind::DifferentContent,
                    existing_name: existing.filename().into(),
                    incoming_name: media.filename().into(),
                });
            }
            return Ok(false);
        }
        Ok(true)
    }

    pub(crate) fn merge(&mut self, additions: Self) {
        self.0.extend(additions.0);
    }

    pub(crate) fn values(&self) -> impl Iterator<Item = &Media> {
        self.0.values()
    }

    pub(crate) fn into_values(self) -> Vec<Media> {
        self.0.into_values().collect()
    }
}
