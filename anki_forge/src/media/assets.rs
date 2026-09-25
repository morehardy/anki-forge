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

#[derive(Debug)]
pub(crate) struct AssetConflict {
    pub(crate) code: &'static str,
    pub(crate) message: String,
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
                    code: "MEDIA.EXPORT_NAME_COLLISION",
                    message: format!(
                        "media names {:?} and {:?} collide after Unicode normalization and case folding",
                        existing.filename(), media.filename()
                    ),
                });
            }
            if !existing.same_content(media) {
                return Err(AssetConflict {
                    code: "MEDIA.DUPLICATE_FILENAME_CONFLICT",
                    message: format!(
                        "media name {:?} is bound to different content",
                        media.filename()
                    ),
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
