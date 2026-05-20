use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::{Component, Path, PathBuf};

use base64::Engine as _;
use sha1::{Digest, Sha1};

const INLINE_MEDIA_LIMIT_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MediaRef {
    filename: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProductMediaSource {
    File {
        path: PathBuf,
    },
    InlineBytes {
        source_label: String,
        data_base64: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MediaFingerprint {
    blake3_hex: String,
    size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductMedia {
    pub id: String,
    pub export_filename: String,
    pub source: ProductMediaSource,
    pub declared_mime: Option<String>,
    pub sha1_hex: String,
    pub(crate) observed_fingerprint: MediaFingerprint,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProductMediaSourceDiagnostic {
    pub code: &'static str,
    pub message: String,
    pub source_path: String,
}

#[derive(Debug, Clone, Default)]
pub struct MediaRegistry {
    media: BTreeMap<String, ProductMedia>,
}

#[derive(Debug)]
pub struct PendingMedia<'a> {
    registry: &'a mut MediaRegistry,
    source: ProductMediaSource,
    fingerprint: MediaFingerprint,
    sha1_hex: String,
}

impl MediaRef {
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

impl MediaRegistry {
    pub fn add_file(&mut self, path: impl AsRef<Path>) -> anyhow::Result<PendingMedia<'_>> {
        let path = path.as_ref().to_path_buf();
        let observed = observe_file_source(&path, EmptySourceBehavior::Reject)?;

        Ok(PendingMedia {
            registry: self,
            source: ProductMediaSource::File { path },
            fingerprint: observed.fingerprint,
            sha1_hex: observed.sha1_hex,
        })
    }

    pub fn add_bytes(
        &mut self,
        source_label: impl Into<String>,
        bytes: Vec<u8>,
    ) -> anyhow::Result<PendingMedia<'_>> {
        let source_label = source_label.into();
        validate_source_label(&source_label)?;
        anyhow::ensure!(!bytes.is_empty(), "MEDIA.EMPTY_SOURCE: {source_label}");
        anyhow::ensure!(
            bytes.len() <= INLINE_MEDIA_LIMIT_BYTES,
            "MEDIA.INLINE_TOO_LARGE: {source_label} has {} bytes, above inline limit {INLINE_MEDIA_LIMIT_BYTES}",
            bytes.len()
        );

        let fingerprint = fingerprint_bytes(&bytes);
        let sha1_hex = hex::encode(Sha1::digest(&bytes));
        Ok(PendingMedia {
            registry: self,
            source: ProductMediaSource::InlineBytes {
                source_label,
                data_base64: base64::engine::general_purpose::STANDARD.encode(bytes),
            },
            fingerprint,
            sha1_hex,
        })
    }

    pub(crate) fn media(&self) -> impl Iterator<Item = &ProductMedia> {
        self.media.values()
    }
}

impl ProductMedia {
    pub(crate) fn verify_registered_source(&self) -> Result<(), ProductMediaSourceDiagnostic> {
        match &self.source {
            ProductMediaSource::File { path } => {
                let observed = observe_file_source(path, EmptySourceBehavior::Allow)
                    .map_err(|err| registration_error_to_diagnostic(err, &self.export_filename))?;
                if observed.fingerprint != self.observed_fingerprint {
                    return Err(ProductMediaSourceDiagnostic {
                        code: "MEDIA.SOURCE_CHANGED",
                        message: format!(
                            "media source {} changed after registration",
                            path.display()
                        ),
                        source_path: media_source_path(&self.export_filename),
                    });
                }
                Ok(())
            }
            ProductMediaSource::InlineBytes { .. } => Ok(()),
        }
    }
}

impl PendingMedia<'_> {
    pub fn export_as(self, filename: impl Into<String>) -> anyhow::Result<MediaRef> {
        let filename = filename.into();
        validate_media_filename(&filename)?;

        if let Some(existing) = self.registry.media.get(&filename) {
            anyhow::ensure!(
                existing.observed_fingerprint == self.fingerprint,
                "MEDIA.DUPLICATE_FILENAME_CONFLICT: {filename}"
            );
            return Ok(MediaRef::new(filename));
        }

        let media = ProductMedia {
            id: format!("media:{filename}"),
            export_filename: filename.clone(),
            source: self.source,
            declared_mime: Some(mime_from_name(&filename)),
            sha1_hex: self.sha1_hex,
            observed_fingerprint: self.fingerprint,
        };
        self.registry.media.insert(filename.clone(), media);
        Ok(MediaRef::new(filename))
    }
}

struct ObservedMedia {
    fingerprint: MediaFingerprint,
    sha1_hex: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EmptySourceBehavior {
    Allow,
    Reject,
}

fn observe_file_source(
    path: &Path,
    empty_source_behavior: EmptySourceBehavior,
) -> anyhow::Result<ObservedMedia> {
    let metadata = std::fs::metadata(path).map_err(|err| match err.kind() {
        std::io::ErrorKind::NotFound => {
            anyhow::anyhow!("MEDIA.SOURCE_MISSING: {}: {err}", path.display())
        }
        _ => anyhow::anyhow!("MEDIA.SOURCE_READ_FAILED: {}: {err}", path.display()),
    })?;
    anyhow::ensure!(
        metadata.is_file(),
        "MEDIA.SOURCE_NOT_REGULAR_FILE: {}",
        path.display()
    );
    if empty_source_behavior == EmptySourceBehavior::Reject {
        anyhow::ensure!(metadata.len() > 0, "MEDIA.EMPTY_SOURCE: {}", path.display());
    }

    let mut file = File::open(path).map_err(|err| match err.kind() {
        std::io::ErrorKind::NotFound => {
            anyhow::anyhow!("MEDIA.SOURCE_MISSING: {}: {err}", path.display())
        }
        _ => anyhow::anyhow!("MEDIA.SOURCE_READ_FAILED: {}: {err}", path.display()),
    })?;
    let mut blake3_hasher = blake3::Hasher::new();
    let mut sha1_hasher = Sha1::new();
    let mut size_bytes = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];

    loop {
        let read = file.read(&mut buffer).map_err(|err| {
            anyhow::anyhow!("MEDIA.SOURCE_READ_FAILED: {}: {err}", path.display())
        })?;
        if read == 0 {
            break;
        }
        let chunk = &buffer[..read];
        blake3_hasher.update(chunk);
        sha1_hasher.update(chunk);
        size_bytes += read as u64;
    }

    if empty_source_behavior == EmptySourceBehavior::Reject {
        anyhow::ensure!(size_bytes > 0, "MEDIA.EMPTY_SOURCE: {}", path.display());
    }
    Ok(ObservedMedia {
        fingerprint: MediaFingerprint {
            blake3_hex: blake3_hasher.finalize().to_hex().to_string(),
            size_bytes,
        },
        sha1_hex: hex::encode(sha1_hasher.finalize()),
    })
}

fn fingerprint_bytes(bytes: &[u8]) -> MediaFingerprint {
    MediaFingerprint {
        blake3_hex: blake3::hash(bytes).to_hex().to_string(),
        size_bytes: bytes.len() as u64,
    }
}

fn validate_source_label(source_label: &str) -> anyhow::Result<()> {
    anyhow::ensure!(
        !source_label.trim().is_empty(),
        "MEDIA.INVALID_SOURCE_LABEL: source label must not be empty"
    );
    anyhow::ensure!(
        !source_label.chars().any(char::is_control),
        "MEDIA.INVALID_SOURCE_LABEL: source label contains a control character"
    );
    Ok(())
}

fn registration_error_to_diagnostic(
    error: anyhow::Error,
    export_filename: &str,
) -> ProductMediaSourceDiagnostic {
    let message = error.to_string();
    let code = if message.contains("MEDIA.SOURCE_MISSING") {
        "MEDIA.SOURCE_MISSING"
    } else if message.contains("MEDIA.SOURCE_NOT_REGULAR_FILE") {
        "MEDIA.SOURCE_NOT_REGULAR_FILE"
    } else if message.contains("MEDIA.EMPTY_SOURCE") {
        "MEDIA.EMPTY_SOURCE"
    } else {
        "MEDIA.SOURCE_READ_FAILED"
    };
    ProductMediaSourceDiagnostic {
        code,
        message,
        source_path: media_source_path(export_filename),
    }
}

fn media_source_path(value: &str) -> String {
    format!("project.media[{value:?}]")
}

fn validate_media_filename(filename: &str) -> anyhow::Result<()> {
    anyhow::ensure!(!filename.trim().is_empty(), "MEDIA.EXPORT_NAME_EMPTY");
    anyhow::ensure!(
        !filename.contains(['/', '\\']),
        "MEDIA.EXPORT_NAME_CONTAINS_SEPARATOR"
    );

    let mut components = Path::new(filename).components();
    let only_component = matches!(components.next(), Some(Component::Normal(_)))
        && components.next().is_none()
        && !Path::new(filename).is_absolute();
    anyhow::ensure!(only_component, "MEDIA.EXPORT_NAME_NOT_BARE_FILENAME");
    anyhow::ensure!(
        filename
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_')),
        "MEDIA.EXPORT_NAME_UNSAFE_CHARACTER"
    );

    Ok(())
}

fn mime_from_name(name: &str) -> String {
    match name.rsplit('.').next().map(str::to_ascii_lowercase) {
        Some(ext) if ext == "png" => "image/png".into(),
        Some(ext) if ext == "jpg" || ext == "jpeg" => "image/jpeg".into(),
        Some(ext) if ext == "mp3" => "audio/mpeg".into(),
        Some(ext) if ext == "wav" => "audio/wav".into(),
        _ => "application/octet-stream".into(),
    }
}
