use anyhow::Context;
use base64::Engine as _;
use sha1::{Digest, Sha1};
use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Component, Path};

use crate::deck::model::{
    Deck, MediaRef, RasterImageMetadata, RegisteredMedia, RegisteredMediaSource,
};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MediaSource {
    File { path: std::path::PathBuf },
    Bytes { name: String, bytes: Vec<u8> },
}

pub struct MediaRegistry<'a> {
    deck: &'a mut Deck,
}

impl MediaSource {
    pub fn from_file(path: impl Into<std::path::PathBuf>) -> Self {
        Self::File { path: path.into() }
    }

    pub fn from_bytes(name: impl Into<String>, bytes: Vec<u8>) -> Self {
        Self::Bytes {
            name: name.into(),
            bytes,
        }
    }
}

impl MediaRef {
    pub(crate) fn new(name: String) -> Self {
        Self(name)
    }

    pub fn name(&self) -> &str {
        &self.0
    }
}

impl Deck {
    pub fn media(&mut self) -> MediaRegistry<'_> {
        MediaRegistry { deck: self }
    }
}

impl<'a> MediaRegistry<'a> {
    pub fn add(&mut self, source: MediaSource) -> anyhow::Result<MediaRef> {
        let registered = RegisteredMedia::from_source(source)?;

        if let Some(existing) = self.deck.media.get_mut(&registered.name) {
            if existing.sha1_hex == registered.sha1_hex {
                if existing.raster_image.is_none() {
                    existing.raster_image = registered.raster_image.clone();
                }
                return Ok(MediaRef::new(existing.name.clone()));
            }

            anyhow::bail!("conflicting media payload for {}", registered.name);
        }

        let reference = MediaRef::new(registered.name.clone());
        self.deck.media.insert(registered.name.clone(), registered);
        Ok(reference)
    }

    pub fn get(&self, name: &str) -> Option<MediaRef> {
        self.deck
            .media
            .get(name)
            .map(|registered| MediaRef::new(registered.name.clone()))
    }
}

pub(crate) fn backfill_missing_raster_image_metadata(
    media: &mut BTreeMap<String, RegisteredMedia>,
) {
    for registered in media.values_mut() {
        repair_missing_raster_image_metadata(registered);
    }
}

fn repair_missing_raster_image_metadata(registered: &mut RegisteredMedia) {
    if registered.raster_image.is_some() {
        return;
    }

    registered.raster_image = match &registered.source {
        RegisteredMediaSource::File { path } => {
            raster_image_metadata_from_path(&registered.name, path)
        }
        RegisteredMediaSource::InlineBytes { data_base64 } => {
            let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(data_base64) else {
                return;
            };
            raster_image_metadata_from_bytes(&registered.name, &bytes)
        }
    };
}

impl RegisteredMedia {
    pub fn from_source(source: MediaSource) -> anyhow::Result<Self> {
        let (name, registered_source, sha1_hex, raster_image) = match source {
            MediaSource::File { path } => {
                let name = path
                    .file_name()
                    .and_then(|item| item.to_str())
                    .ok_or_else(|| anyhow::anyhow!("media path must end in a valid filename"))?
                    .to_string();
                validate_source_file(&path)?;
                let sha1_hex = sha1_file_hex(&path)?;
                let raster_image = raster_image_metadata_from_path(&name, &path);
                (
                    name,
                    RegisteredMediaSource::File { path },
                    sha1_hex,
                    raster_image,
                )
            }
            MediaSource::Bytes { name, bytes } => {
                let sha1_hex = hex::encode(Sha1::digest(&bytes));
                let raster_image = raster_image_metadata_from_bytes(&name, &bytes);
                let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
                (
                    name,
                    RegisteredMediaSource::InlineBytes {
                        data_base64: encoded,
                    },
                    sha1_hex,
                    raster_image,
                )
            }
        };
        validate_media_filename(&name)?;

        Ok(Self {
            name: name.clone(),
            source: registered_source,
            declared_mime: Some(mime_from_name(&name)),
            sha1_hex,
            raster_image,
        })
    }

    pub(crate) fn to_authoring_media(
        &self,
        media_source_dir: &Path,
    ) -> anyhow::Result<crate::AuthoringMedia> {
        let source = match &self.source {
            RegisteredMediaSource::File { path } => {
                ensure_safe_media_source_dir(media_source_dir)?;
                let target = media_source_dir.join(&self.name);
                ensure_not_symlink(&target)?;
                std::fs::copy(path, &target).with_context(|| {
                    format!(
                        "copy media source {} to {}",
                        path.display(),
                        target.display()
                    )
                })?;
                crate::AuthoringMediaSource::Path {
                    path: self.name.clone(),
                }
            }
            RegisteredMediaSource::InlineBytes { data_base64 } => {
                crate::AuthoringMediaSource::InlineBytes {
                    data_base64: data_base64.clone(),
                }
            }
        };

        Ok(crate::AuthoringMedia {
            id: format!("media:{}", self.name),
            desired_filename: self.name.clone(),
            source,
            declared_mime: self.declared_mime.clone(),
        })
    }

    pub(crate) fn to_self_contained_authoring_media(
        &self,
    ) -> anyhow::Result<crate::AuthoringMedia> {
        let source = match &self.source {
            RegisteredMediaSource::File { path } => {
                let bytes = std::fs::read(path)
                    .with_context(|| format!("read media source file: {}", path.display()))?;
                crate::AuthoringMediaSource::InlineBytes {
                    data_base64: base64::engine::general_purpose::STANDARD.encode(bytes),
                }
            }
            RegisteredMediaSource::InlineBytes { data_base64 } => {
                crate::AuthoringMediaSource::InlineBytes {
                    data_base64: data_base64.clone(),
                }
            }
        };

        Ok(crate::AuthoringMedia {
            id: format!("media:{}", self.name),
            desired_filename: self.name.clone(),
            source,
            declared_mime: self.declared_mime.clone(),
        })
    }
}

fn validate_source_file(path: &Path) -> anyhow::Result<()> {
    let metadata = std::fs::metadata(path)
        .with_context(|| format!("stat media source file: {}", path.display()))?;
    anyhow::ensure!(
        metadata.is_file(),
        "media source path must be a regular file: {}",
        path.display()
    );
    Ok(())
}

fn ensure_safe_media_source_dir(media_source_dir: &Path) -> anyhow::Result<()> {
    match std::fs::symlink_metadata(media_source_dir) {
        Ok(metadata) => {
            anyhow::ensure!(
                !metadata.file_type().is_symlink(),
                "media source directory must not be a symlink: {}",
                media_source_dir.display()
            );
            anyhow::ensure!(
                metadata.is_dir(),
                "media source path must be a directory: {}",
                media_source_dir.display()
            );
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            std::fs::create_dir_all(media_source_dir).with_context(|| {
                format!(
                    "create media source directory: {}",
                    media_source_dir.display()
                )
            })?;
            let metadata = std::fs::symlink_metadata(media_source_dir).with_context(|| {
                format!(
                    "stat media source directory: {}",
                    media_source_dir.display()
                )
            })?;
            anyhow::ensure!(
                !metadata.file_type().is_symlink(),
                "media source directory must not be a symlink: {}",
                media_source_dir.display()
            );
        }
        Err(err) => {
            return Err(err).with_context(|| {
                format!(
                    "stat media source directory: {}",
                    media_source_dir.display()
                )
            });
        }
    }

    Ok(())
}

fn ensure_not_symlink(path: &Path) -> anyhow::Result<()> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) => {
            anyhow::ensure!(
                !metadata.file_type().is_symlink(),
                "media input target must not be a symlink: {}",
                path.display()
            );
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => {
            return Err(err)
                .with_context(|| format!("stat media input target: {}", path.display()));
        }
    }
    Ok(())
}

fn sha1_file_hex(path: &Path) -> anyhow::Result<String> {
    let mut file = std::fs::File::open(path)
        .with_context(|| format!("open media source file: {}", path.display()))?;
    let mut hasher = Sha1::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let read = file
            .read(&mut buffer)
            .with_context(|| format!("read media source file: {}", path.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hex::encode(hasher.finalize()))
}

fn validate_media_filename(name: &str) -> anyhow::Result<()> {
    anyhow::ensure!(!name.is_empty(), "media filename must not be empty");
    anyhow::ensure!(
        !name.contains(['/', '\\']),
        "media filename must be a bare filename without path separators: {}",
        name
    );

    let mut components = Path::new(name).components();
    let only_component = matches!(components.next(), Some(Component::Normal(_)))
        && components.next().is_none()
        && !Path::new(name).is_absolute();

    anyhow::ensure!(
        only_component,
        "media filename must be a bare filename without path traversal: {}",
        name
    );

    Ok(())
}

fn mime_from_name(name: &str) -> String {
    match name.rsplit('.').next().map(|ext| ext.to_ascii_lowercase()) {
        Some(ext) if ext == "png" => "image/png".into(),
        Some(ext) if ext == "jpg" || ext == "jpeg" => "image/jpeg".into(),
        Some(ext) if ext == "svg" => "image/svg+xml".into(),
        Some(ext) if ext == "mp3" => "audio/mpeg".into(),
        Some(ext) if ext == "wav" => "audio/wav".into(),
        _ => "application/octet-stream".into(),
    }
}

fn raster_image_metadata_from_path(name: &str, path: &Path) -> Option<RasterImageMetadata> {
    match mime_from_name(name).as_str() {
        "image/png" | "image/jpeg" => imagesize::size(path).ok().map(|size| RasterImageMetadata {
            width_px: size.width as u32,
            height_px: size.height as u32,
        }),
        _ => None,
    }
}

fn raster_image_metadata_from_bytes(name: &str, bytes: &[u8]) -> Option<RasterImageMetadata> {
    match mime_from_name(name).as_str() {
        "image/png" | "image/jpeg" => {
            imagesize::blob_size(bytes)
                .ok()
                .map(|size| RasterImageMetadata {
                    width_px: size.width as u32,
                    height_px: size.height as u32,
                })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deck::model::IoMode;

    const HEART_PNG: &[u8] = include_bytes!(
        "../../../contracts/fixtures/phase3/manual-desktop-v1/S03_io_minimal/assets/occlusion-heart.png"
    );

    #[test]
    fn same_sha_registration_repairs_missing_raster_metadata() {
        let mut deck = Deck::new("Anatomy");
        let mut legacy_media = RegisteredMedia::from_source(MediaSource::from_bytes(
            "occlusion-heart.png",
            HEART_PNG.to_vec(),
        ))
        .expect("legacy media");
        legacy_media.raster_image = None;
        deck.media
            .insert("occlusion-heart.png".to_string(), legacy_media);

        let image = deck
            .media()
            .add(MediaSource::from_bytes(
                "occlusion-heart.png",
                HEART_PNG.to_vec(),
            ))
            .expect("same media registration");

        let raster_image = deck
            .media
            .get("occlusion-heart.png")
            .and_then(|media| media.raster_image.as_ref())
            .expect("raster metadata repaired");
        assert_eq!(raster_image.width_px, 228);
        assert_eq!(raster_image.height_px, 86);

        deck.image_occlusion()
            .note(image)
            .mode(IoMode::HideAllGuessOne)
            .rect(10, 20, 30, 40)
            .add()
            .expect("io identity uses repaired dimensions");
    }
}
