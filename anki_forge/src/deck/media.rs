use base64::Engine as _;
use sha1::{Digest, Sha1};
use std::collections::BTreeMap;
use std::path::{Component, Path};

use crate::deck::model::{Deck, MediaRef, RasterImageMetadata, RegisteredMedia};

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

    let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(&registered.data_base64)
    else {
        return;
    };
    registered.raster_image = raster_image_metadata(&registered.name, &bytes);
}

impl RegisteredMedia {
    pub fn from_source(source: MediaSource) -> anyhow::Result<Self> {
        let (name, bytes) = match source {
            MediaSource::File { path } => {
                let name = path
                    .file_name()
                    .and_then(|item| item.to_str())
                    .ok_or_else(|| anyhow::anyhow!("media path must end in a valid filename"))?
                    .to_string();
                (name, std::fs::read(path)?)
            }
            MediaSource::Bytes { name, bytes } => (name, bytes),
        };
        validate_media_filename(&name)?;

        let sha1_hex = hex::encode(Sha1::digest(&bytes));
        let raster_image = raster_image_metadata(&name, &bytes);
        Ok(Self {
            name: name.clone(),
            mime: mime_from_name(&name),
            data_base64: base64::engine::general_purpose::STANDARD.encode(bytes),
            sha1_hex,
            raster_image,
        })
    }
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

fn raster_image_metadata(name: &str, bytes: &[u8]) -> Option<RasterImageMetadata> {
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
