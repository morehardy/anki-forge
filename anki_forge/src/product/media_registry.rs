use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use anyhow::Context;
use base64::Engine as _;
use sha1::{Digest, Sha1};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MediaRef {
    filename: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProductMediaSource {
    File { path: PathBuf },
    InlineBytes { data_base64: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductMedia {
    pub id: String,
    pub export_filename: String,
    pub source: ProductMediaSource,
    pub declared_mime: Option<String>,
    pub sha1_hex: String,
}

#[derive(Debug, Clone, Default)]
pub struct MediaRegistry {
    media: BTreeMap<String, ProductMedia>,
}

#[derive(Debug)]
pub struct PendingMedia<'a> {
    registry: &'a mut MediaRegistry,
    media: ProductMedia,
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
        let bytes = std::fs::read(&path)
            .with_context(|| format!("read media source file: {}", path.display()))?;
        let filename = path
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| anyhow::anyhow!("media path must end in a valid filename"))?
            .to_string();

        Ok(PendingMedia {
            registry: self,
            media: ProductMedia {
                id: format!("media:{filename}"),
                export_filename: filename.clone(),
                source: ProductMediaSource::File { path },
                declared_mime: Some(mime_from_name(&filename)),
                sha1_hex: hex::encode(Sha1::digest(bytes)),
            },
        })
    }

    pub fn add_bytes(&mut self, filename: impl Into<String>, bytes: Vec<u8>) -> PendingMedia<'_> {
        let filename = filename.into();
        PendingMedia {
            registry: self,
            media: ProductMedia {
                id: format!("media:{filename}"),
                export_filename: filename.clone(),
                source: ProductMediaSource::InlineBytes {
                    data_base64: base64::engine::general_purpose::STANDARD.encode(&bytes),
                },
                declared_mime: Some(mime_from_name(&filename)),
                sha1_hex: hex::encode(Sha1::digest(bytes)),
            },
        }
    }

    pub(crate) fn media(&self) -> impl Iterator<Item = &ProductMedia> {
        self.media.values()
    }
}

impl PendingMedia<'_> {
    pub fn export_as(mut self, filename: impl Into<String>) -> anyhow::Result<MediaRef> {
        let filename = filename.into();
        validate_media_filename(&filename)?;
        self.media.id = format!("media:{filename}");
        self.media.export_filename = filename.clone();
        self.media.declared_mime = Some(mime_from_name(&filename));

        if let Some(existing) = self.registry.media.get(&filename) {
            anyhow::ensure!(
                existing.sha1_hex == self.media.sha1_hex,
                "MEDIA.FILENAME_COLLISION: {filename}"
            );
            return Ok(MediaRef::new(filename));
        }

        self.registry.media.insert(filename.clone(), self.media);
        Ok(MediaRef::new(filename))
    }
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
