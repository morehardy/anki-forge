use sha1::{Digest, Sha1};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

const SNIFF_SAMPLE_BYTES: usize = 8192;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaSniffConfidence {
    High,
    Low,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SniffedMime {
    pub mime: String,
    pub confidence: MediaSniffConfidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngestedMediaBytes {
    pub blake3: String,
    pub sha1: String,
    pub size_bytes: u64,
    pub sniffed_mime: Option<SniffedMime>,
    pub object_path: PathBuf,
}

pub enum MediaReadSource<'a> {
    File { path: &'a Path },
    InlineBytes { bytes: &'a [u8] },
}

#[derive(Debug)]
pub enum MediaIoError {
    SourceOpen {
        path: PathBuf,
        message: String,
    },
    SourceRead {
        path: Option<PathBuf>,
        message: String,
    },
    InlineBase64Decode {
        message: String,
    },
    InlineBytesTooLarge {
        size: usize,
        limit: usize,
    },
    CasWrite {
        path: PathBuf,
        message: String,
    },
    CasFinalize {
        path: PathBuf,
        message: String,
    },
    CasExistingIntegrity {
        path: PathBuf,
        reason: CasExistingIntegrityReason,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CasExistingIntegrityReason {
    OpenFailed {
        message: String,
    },
    ReadFailed {
        message: String,
    },
    Mismatch {
        expected_blake3: String,
        actual_blake3: String,
        expected_size: u64,
        actual_size: u64,
    },
}

impl MediaIoError {
    pub fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::SourceOpen { .. } => "MEDIA.SOURCE_MISSING",
            Self::SourceRead { .. } => "MEDIA.SOURCE_READ_FAILED",
            Self::InlineBase64Decode { .. } => "MEDIA.INLINE_BASE64_DECODE_FAILED",
            Self::InlineBytesTooLarge { .. } => "MEDIA.INLINE_TOO_LARGE",
            Self::CasWrite { .. } | Self::CasFinalize { .. } => "MEDIA.CAS_WRITE_FAILED",
            Self::CasExistingIntegrity { .. } => "MEDIA.CAS_OBJECT_INTEGRITY_CONFLICT",
        }
    }
}

pub fn decode_inline_bytes(data_base64: &str, limit: usize) -> Result<Vec<u8>, MediaIoError> {
    let decoded_size_upper_bound = decoded_len_upper_bound(data_base64.as_bytes());
    if decoded_size_upper_bound > limit {
        return Err(MediaIoError::InlineBytesTooLarge {
            size: decoded_size_upper_bound,
            limit,
        });
    }

    use base64::Engine as _;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(data_base64.as_bytes())
        .map_err(|err| MediaIoError::InlineBase64Decode {
            message: err.to_string(),
        })?;
    if bytes.len() > limit {
        return Err(MediaIoError::InlineBytesTooLarge {
            size: bytes.len(),
            limit,
        });
    }
    Ok(bytes)
}

fn decoded_len_upper_bound(encoded: &[u8]) -> usize {
    let len = encoded.len();
    let full_groups = len / 4;
    let remainder = len % 4;
    let remainder_decoded = match remainder {
        0 | 1 => 0,
        2 => 1,
        3 => 2,
        _ => unreachable!(),
    };
    let padding = if remainder == 0 {
        encoded
            .iter()
            .rev()
            .take_while(|byte| **byte == b'=')
            .count()
            .min(2)
    } else {
        0
    };
    full_groups
        .saturating_mul(3)
        .saturating_add(remainder_decoded)
        .saturating_sub(padding)
}

pub fn object_store_path(store_dir: &Path, blake3_hex: &str) -> Result<PathBuf, String> {
    let lowercase_hex = blake3_hex
        .bytes()
        .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'));
    if blake3_hex.len() != 64 || !lowercase_hex {
        return Err(format!("invalid lowercase blake3 hex: {blake3_hex}"));
    }
    Ok(store_dir
        .join("objects")
        .join("blake3")
        .join(&blake3_hex[0..2])
        .join(&blake3_hex[2..4])
        .join(blake3_hex))
}

pub fn ingest_media_read_source_to_cas(
    source: MediaReadSource<'_>,
    store_dir: &Path,
) -> Result<IngestedMediaBytes, MediaIoError> {
    let tmp_dir = store_dir.join("tmp");
    fs::create_dir_all(&tmp_dir).map_err(|err| MediaIoError::CasWrite {
        path: tmp_dir.clone(),
        message: err.to_string(),
    })?;

    let (mut reader, source_path): (Box<dyn Read>, Option<PathBuf>) = match source {
        MediaReadSource::File { path } => {
            let path = path.to_path_buf();
            (
                Box::new(File::open(&path).map_err(|err| MediaIoError::SourceOpen {
                    path: path.clone(),
                    message: err.to_string(),
                })?),
                Some(path),
            )
        }
        MediaReadSource::InlineBytes { bytes } => (Box::new(io::Cursor::new(bytes)), None),
    };

    let mut temp =
        tempfile::NamedTempFile::new_in(&tmp_dir).map_err(|err| MediaIoError::CasWrite {
            path: tmp_dir.clone(),
            message: err.to_string(),
        })?;
    let mut blake3_hasher = blake3::Hasher::new();
    let mut sha1_hasher = Sha1::new();
    let mut sample = Vec::with_capacity(SNIFF_SAMPLE_BYTES);
    let mut size_bytes = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];

    loop {
        let read = reader
            .read(&mut buffer)
            .map_err(|err| MediaIoError::SourceRead {
                path: source_path.clone(),
                message: err.to_string(),
            })?;
        if read == 0 {
            break;
        }
        let chunk = &buffer[..read];
        if sample.len() < SNIFF_SAMPLE_BYTES {
            let remaining = SNIFF_SAMPLE_BYTES - sample.len();
            sample.extend_from_slice(&chunk[..chunk.len().min(remaining)]);
        }
        blake3_hasher.update(chunk);
        sha1_hasher.update(chunk);
        size_bytes += read as u64;
        temp.write_all(chunk)
            .map_err(|err| MediaIoError::CasWrite {
                path: temp.path().to_path_buf(),
                message: err.to_string(),
            })?;
    }
    temp.flush().map_err(|err| MediaIoError::CasWrite {
        path: temp.path().to_path_buf(),
        message: err.to_string(),
    })?;
    temp.as_file()
        .sync_all()
        .map_err(|err| MediaIoError::CasWrite {
            path: temp.path().to_path_buf(),
            message: err.to_string(),
        })?;

    let blake3 = blake3_hasher.finalize().to_hex().to_string();
    let sha1 = hex::encode(sha1_hasher.finalize());
    let final_path =
        object_store_path(store_dir, &blake3).map_err(|message| MediaIoError::CasFinalize {
            path: store_dir.to_path_buf(),
            message,
        })?;
    let object_path = finalize_temp_object(temp, &final_path, &blake3, size_bytes)?;

    Ok(IngestedMediaBytes {
        blake3,
        sha1,
        size_bytes,
        sniffed_mime: sniff_mime(&sample),
        object_path,
    })
}

fn finalize_temp_object(
    temp: tempfile::NamedTempFile,
    final_path: &Path,
    blake3_hex: &str,
    size_bytes: u64,
) -> Result<PathBuf, MediaIoError> {
    let parent = final_path
        .parent()
        .ok_or_else(|| MediaIoError::CasFinalize {
            path: final_path.to_path_buf(),
            message: "CAS object path has no parent".into(),
        })?;
    fs::create_dir_all(parent).map_err(|err| MediaIoError::CasWrite {
        path: parent.to_path_buf(),
        message: err.to_string(),
    })?;

    if final_path.exists() {
        verify_existing_object(final_path, blake3_hex, size_bytes)?;
        return Ok(final_path.to_path_buf());
    }

    match temp.persist_noclobber(final_path) {
        Ok(_) => Ok(final_path.to_path_buf()),
        Err(err) if final_path.exists() => {
            err.file
                .close()
                .map_err(|close_err| MediaIoError::CasFinalize {
                    path: final_path.to_path_buf(),
                    message: format!(
                        "failed to remove temporary CAS object after persist race: {close_err}"
                    ),
                })?;
            verify_existing_object(final_path, blake3_hex, size_bytes)?;
            Ok(final_path.to_path_buf())
        }
        Err(err) => Err(MediaIoError::CasFinalize {
            path: final_path.to_path_buf(),
            message: err.error.to_string(),
        }),
    }
}

fn verify_existing_object(
    path: &Path,
    blake3_hex: &str,
    size_bytes: u64,
) -> Result<(), MediaIoError> {
    let mut file = File::open(path).map_err(|err| MediaIoError::CasExistingIntegrity {
        path: path.to_path_buf(),
        reason: CasExistingIntegrityReason::OpenFailed {
            message: err.to_string(),
        },
    })?;
    let mut hasher = blake3::Hasher::new();
    let mut read_size = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|err| MediaIoError::CasExistingIntegrity {
                path: path.to_path_buf(),
                reason: CasExistingIntegrityReason::ReadFailed {
                    message: err.to_string(),
                },
            })?;
        if read == 0 {
            break;
        }
        read_size += read as u64;
        hasher.update(&buffer[..read]);
    }
    let actual_blake3 = hasher.finalize().to_hex().to_string();
    if read_size == size_bytes && actual_blake3 == blake3_hex {
        Ok(())
    } else {
        Err(MediaIoError::CasExistingIntegrity {
            path: path.to_path_buf(),
            reason: CasExistingIntegrityReason::Mismatch {
                expected_blake3: blake3_hex.into(),
                actual_blake3,
                expected_size: size_bytes,
                actual_size: read_size,
            },
        })
    }
}

pub fn sniff_mime(bytes: &[u8]) -> Option<SniffedMime> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some(high("image/png"))
    } else if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        Some(high("image/jpeg"))
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        Some(high("image/gif"))
    } else if bytes.starts_with(b"ID3") {
        Some(high("audio/mpeg"))
    } else if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WAVE") {
        Some(high("audio/wav"))
    } else if !bytes.is_empty()
        && bytes.iter().all(|byte| {
            byte.is_ascii() && (!byte.is_ascii_control() || matches!(*byte, b'\n' | b'\r' | b'\t'))
        })
    {
        Some(SniffedMime {
            mime: "text/plain".into(),
            confidence: MediaSniffConfidence::Low,
        })
    } else {
        None
    }
}

fn high(mime: &str) -> SniffedMime {
    SniffedMime {
        mime: mime.into(),
        confidence: MediaSniffConfidence::High,
    }
}
