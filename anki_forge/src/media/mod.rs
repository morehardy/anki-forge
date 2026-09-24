//! Owned media snapshots with explicit per-asset budgets and portable names.
//!
//! A successful import no longer depends on the original source file. Clones
//! share storage; large assets spill to owned temporary files, removed with the
//! final owner. Construct an export name before placing media in content.

pub(crate) mod assets;
mod error;
mod snapshot;

pub(crate) use assets::Assets;

pub use error::{MediaError, MediaErrorKind, MediaLimitExceeded};

use crate::authoring_core::media_io::{sniff_mime, MediaSniffConfidence};
use snapshot::Snapshot;
use std::{path::Path, sync::Arc};

/// The maximum size of one imported asset. Zero permits only empty content.
///
/// This budget applies before snapshot creation. It does not govern memory the
/// caller already owns, other assets, or later APKG inspection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MediaLimits {
    /// Maximum bytes in one asset; defaults to 256 MiB.
    pub max_bytes: u64,
}

impl Default for MediaLimits {
    fn default() -> Self {
        Self {
            max_bytes: 256 << 20,
        }
    }
}

/// A shareable content snapshot with a deterministic or explicit export name.
///
/// Equality compares filename, media type and content digest/length, never the
/// source path or storage address. Naming a clone does not rename prior clones.
#[derive(Debug, Clone)]
#[must_use = "use this media in content or pass it to a model or project asset entry"]
pub struct Media {
    pub(crate) snapshot: Arc<Snapshot>,
    filename: String,
    media_type: String,
}

impl Media {
    /// Creates an image content node retaining this media and its current name.
    pub fn image(&self) -> crate::Content {
        crate::Content::image(self.clone())
    }

    /// Creates a sound content node retaining this media and its current name.
    pub fn sound(&self) -> crate::Content {
        crate::Content::sound(self.clone())
    }

    /// Streams a regular file into owned storage using the default byte budget.
    pub fn file(path: impl AsRef<Path>) -> Result<Self, MediaError> {
        Self::file_with_limits(path, MediaLimits::default())
    }

    /// Takes a snapshot, checking both initial length and actual streamed bytes.
    pub fn file_with_limits(
        path: impl AsRef<Path>,
        limits: MediaLimits,
    ) -> Result<Self, MediaError> {
        let path = path.as_ref();
        let (snapshot, sample) =
            Snapshot::file(path, limits).map_err(|error| error.at_path(path))?;
        let media_type = sniff_mime(&sample)
            .map_or_else(|| "application/octet-stream".into(), |sniffed| sniffed.mime);
        Ok(Self::new(snapshot, media_type))
    }

    /// Takes ownership of bytes with an explicit MIME type and default budget.
    pub fn bytes(bytes: Vec<u8>, media_type: impl Into<String>) -> Result<Self, MediaError> {
        Self::bytes_with_limits(bytes, media_type, MediaLimits::default())
    }

    /// Takes ownership of bytes after checking the declared MIME type and budget.
    pub fn bytes_with_limits(
        bytes: Vec<u8>,
        media_type: impl Into<String>,
        limits: MediaLimits,
    ) -> Result<Self, MediaError> {
        if bytes.len() as u64 > limits.max_bytes {
            return Err(MediaError::exceeded(limits.max_bytes, bytes.len() as u64));
        }
        let value = media_type.into();
        let parsed = value.parse::<mime::Mime>().map_err(|cause| {
            MediaError::new(
                MediaErrorKind::InvalidMediaType,
                "MEDIA.TYPE_INVALID",
                "media_type must be a valid MIME type",
            )
            .caused_by(cause)
        })?;
        if parsed.type_() == mime::STAR || parsed.subtype() == mime::STAR {
            return Err(MediaError::new(
                MediaErrorKind::InvalidMediaType,
                "MEDIA.TYPE_INVALID",
                "media_type must identify a concrete type",
            ));
        }
        if let Some(sniffed) = sniff_mime(&bytes[..bytes.len().min(snapshot::SAMPLE_BYTES)]) {
            if sniffed.confidence == MediaSniffConfidence::High
                && sniffed.mime != parsed.essence_str()
            {
                return Err(MediaError::new(
                    MediaErrorKind::MediaTypeMismatch,
                    "MEDIA.TYPE_MISMATCH",
                    format!("declared {parsed}, but content is {}", sniffed.mime),
                ));
            }
        }
        let snapshot = Snapshot::bytes(bytes, limits)?;
        Ok(Self::new(snapshot, parsed.to_string()))
    }

    fn new(snapshot: Arc<Snapshot>, media_type: String) -> Self {
        let essence = media_type.split(';').next().unwrap_or(&media_type);
        let extension = extension(essence);
        let mut name_hash = blake3::Hasher::new_derive_key("ankiforge:media-name:v1");
        name_hash.update(snapshot.digest.as_bytes());
        name_hash.update(essence.as_bytes());
        let filename = format!("{}.{}", name_hash.finalize().to_hex(), extension);
        Self {
            snapshot,
            filename,
            media_type,
        }
    }

    /// Returns a value sharing this snapshot with an explicit export filename.
    /// Existing clones and references keep their prior name.
    pub fn with_export_name(mut self, name: impl Into<String>) -> Result<Self, MediaError> {
        let name = name.into();
        validate_name(&name)?;
        self.filename = name;
        Ok(self)
    }

    /// Returns the immutable filename used by this value's content references.
    pub fn filename(&self) -> &str {
        &self.filename
    }

    /// Returns the MIME type recognized or declared when importing this asset.
    pub fn media_type(&self) -> &str {
        &self.media_type
    }

    /// Returns the snapshot's byte count without reading the source file.
    pub fn len(&self) -> u64 {
        self.snapshot.len
    }

    /// Whether the snapshot contains zero bytes.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub(crate) fn same_content(&self, other: &Self) -> bool {
        self.snapshot.digest == other.snapshot.digest && self.len() == other.len()
    }
}

impl PartialEq for Media {
    fn eq(&self, other: &Self) -> bool {
        self.filename == other.filename
            && self.media_type == other.media_type
            && self.same_content(other)
    }
}
impl Eq for Media {}

pub(crate) fn validate_name(name: &str) -> Result<(), MediaError> {
    let stem = name.split('.').next().unwrap_or(name).to_ascii_uppercase();
    let reserved = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL" | "CLOCK$")
        || ((stem.starts_with("COM") || stem.starts_with("LPT"))
            && matches!(
                &stem[3..],
                "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
            ));
    if name.is_empty()
        || name.len() > 255
        || name.trim() != name
        || name.ends_with('.')
        || name == "."
        || name == ".."
        || reserved
        || name
            .chars()
            .any(|c| c.is_control() || "/\\<>:\"|?*[]".contains(c))
    {
        Err(MediaError::new(
            MediaErrorKind::InvalidName,
            "MEDIA.EXPORT_NAME_INVALID",
            format!("unsafe or nonportable media filename: {name:?}"),
        ))
    } else {
        Ok(())
    }
}

fn extension(media_type: &str) -> &'static str {
    match media_type {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/svg+xml" => "svg",
        "audio/mpeg" => "mp3",
        "audio/wav" => "wav",
        "audio/ogg" => "ogg",
        "audio/opus" => "opus",
        "audio/mp4" => "m4a",
        "audio/aac" => "aac",
        "video/mp4" => "mp4",
        "video/webm" => "webm",
        "font/woff" => "woff",
        "font/woff2" => "woff2",
        "font/ttf" => "ttf",
        "font/otf" => "otf",
        "application/pdf" => "pdf",
        "text/css" => "css",
        "text/javascript" | "application/javascript" => "js",
        "text/html" => "html",
        "text/plain" => "txt",
        _ => "bin",
    }
}
