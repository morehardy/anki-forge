use base64::Engine as _;
use sha1::{Digest, Sha1};

use crate::deck::model::{Deck, MediaRef, RegisteredMedia};

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

        if let Some(existing) = self.deck.media.get(&registered.name) {
            if existing.sha1_hex == registered.sha1_hex {
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

        let sha1_hex = hex::encode(Sha1::digest(&bytes));
        Ok(Self {
            name: name.clone(),
            mime: mime_from_name(&name),
            data_base64: base64::engine::general_purpose::STANDARD.encode(bytes),
            sha1_hex,
        })
    }
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
