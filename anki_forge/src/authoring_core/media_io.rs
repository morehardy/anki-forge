use crate::authoring_core::mime::APPLICATION_OCTET_STREAM;
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
        cause: io::Error,
    },
    SourceRead {
        path: Option<PathBuf>,
        cause: io::Error,
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
        cause: io::Error,
    },
    CasFinalize {
        path: PathBuf,
        message: String,
        cause: Option<io::Error>,
    },
    CasExistingIntegrity {
        path: PathBuf,
        reason: CasExistingIntegrityReason,
    },
}

#[derive(Debug)]
pub enum CasExistingIntegrityReason {
    OpenFailed {
        cause: io::Error,
    },
    ReadFailed {
        cause: io::Error,
    },
    Mismatch {
        expected_blake3: String,
        actual_blake3: String,
        expected_size: u64,
        actual_size: u64,
    },
}

impl MediaIoError {
    pub(crate) fn into_io_cause(self) -> Option<io::Error> {
        match self {
            Self::SourceOpen { cause, .. }
            | Self::SourceRead { cause, .. }
            | Self::CasWrite { cause, .. } => Some(cause),
            Self::CasFinalize { cause, .. } => cause,
            Self::CasExistingIntegrity {
                reason:
                    CasExistingIntegrityReason::OpenFailed { cause }
                    | CasExistingIntegrityReason::ReadFailed { cause },
                ..
            } => Some(cause),
            _ => None,
        }
    }

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
        cause: err,
    })?;

    let (mut reader, source_path): (Box<dyn Read>, Option<PathBuf>) = match source {
        MediaReadSource::File { path } => {
            let path = path.to_path_buf();
            (
                Box::new(File::open(&path).map_err(|err| MediaIoError::SourceOpen {
                    path: path.clone(),
                    cause: err,
                })?),
                Some(path),
            )
        }
        MediaReadSource::InlineBytes { bytes } => (Box::new(io::Cursor::new(bytes)), None),
    };

    let mut temp =
        tempfile::NamedTempFile::new_in(&tmp_dir).map_err(|err| MediaIoError::CasWrite {
            path: tmp_dir.clone(),
            cause: err,
        })?;
    let mut blake3_hasher = blake3::Hasher::new();
    let mut sha1_hasher = Sha1::new();
    let mut sample = Vec::with_capacity(SNIFF_SAMPLE_BYTES);
    let mut size_bytes = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];

    loop {
        #[cfg(test)]
        io_failure::check(io_failure::Point::Read).map_err(|cause| MediaIoError::SourceRead {
            path: source_path.clone(),
            cause,
        })?;
        let read = reader
            .read(&mut buffer)
            .map_err(|err| MediaIoError::SourceRead {
                path: source_path.clone(),
                cause: err,
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
        #[cfg(test)]
        io_failure::check(io_failure::Point::Write).map_err(|cause| MediaIoError::CasWrite {
            path: temp.path().to_path_buf(),
            cause,
        })?;
        temp.write_all(chunk)
            .map_err(|err| MediaIoError::CasWrite {
                path: temp.path().to_path_buf(),
                cause: err,
            })?;
    }
    temp.flush().map_err(|err| MediaIoError::CasWrite {
        path: temp.path().to_path_buf(),
        cause: err,
    })?;
    #[cfg(test)]
    io_failure::check(io_failure::Point::Sync).map_err(|cause| MediaIoError::CasWrite {
        path: temp.path().to_path_buf(),
        cause,
    })?;
    temp.as_file()
        .sync_all()
        .map_err(|err| MediaIoError::CasWrite {
            path: temp.path().to_path_buf(),
            cause: err,
        })?;

    let blake3 = blake3_hasher.finalize().to_hex().to_string();
    let sha1 = hex::encode(sha1_hasher.finalize());
    let final_path =
        object_store_path(store_dir, &blake3).map_err(|message| MediaIoError::CasFinalize {
            path: store_dir.to_path_buf(),
            message,
            cause: None,
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
            cause: None,
        })?;
    fs::create_dir_all(parent).map_err(|err| MediaIoError::CasWrite {
        path: parent.to_path_buf(),
        cause: err,
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
                    cause: Some(close_err),
                })?;
            verify_existing_object(final_path, blake3_hex, size_bytes)?;
            Ok(final_path.to_path_buf())
        }
        Err(err) => Err(MediaIoError::CasFinalize {
            path: final_path.to_path_buf(),
            message: err.error.to_string(),
            cause: Some(err.error),
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
        reason: CasExistingIntegrityReason::OpenFailed { cause: err },
    })?;
    let mut hasher = blake3::Hasher::new();
    let mut read_size = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|err| MediaIoError::CasExistingIntegrity {
                path: path.to_path_buf(),
                reason: CasExistingIntegrityReason::ReadFailed { cause: err },
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
    } else if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP") {
        Some(high("image/webp"))
    } else if is_bmp(bytes) {
        Some(high("image/bmp"))
    } else if is_svg(bytes) {
        Some(high("image/svg+xml"))
    } else if bytes.starts_with(b"ID3") || is_mp3_frames(bytes) {
        Some(high("audio/mpeg"))
    } else if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WAVE") {
        Some(high("audio/wav"))
    } else if bytes.starts_with(b"OggS") {
        Some(high("audio/ogg"))
    } else if is_aac_adts(bytes) {
        Some(high("audio/aac"))
    } else if is_mp4_family(bytes) {
        Some(high(mp4_family_mime(bytes)))
    } else if is_webm(bytes) {
        Some(high("video/webm"))
    } else if bytes.starts_with(b"%PDF-") {
        Some(high("application/pdf"))
    } else if bytes.starts_with(b"wOFF") {
        Some(high("font/woff"))
    } else if bytes.starts_with(b"wOF2") {
        Some(high("font/woff2"))
    } else if is_sfnt_font(bytes, &[0x00, 0x01, 0x00, 0x00]) || is_sfnt_font(bytes, b"true") {
        Some(high("font/ttf"))
    } else if is_sfnt_font(bytes, b"OTTO") {
        Some(high("font/otf"))
    } else if !bytes.is_empty()
        && bytes.iter().all(|byte| {
            byte.is_ascii() && (!byte.is_ascii_control() || matches!(*byte, b'\n' | b'\r' | b'\t'))
        })
    {
        Some(low(text_mime(bytes)))
    } else {
        None
    }
}

fn is_bmp(bytes: &[u8]) -> bool {
    if bytes.len() < 26 || !bytes.starts_with(b"BM") {
        return false;
    }
    let header_size = u32::from_le_bytes(bytes[14..18].try_into().unwrap());
    let pixel_offset = u32::from_le_bytes(bytes[10..14].try_into().unwrap());
    let planes_offset = match header_size {
        12 => 22,
        40 | 52 | 56 | 64 | 108 | 124 if bytes.len() >= 30 => 26,
        _ => return false,
    };
    let planes = u16::from_le_bytes(bytes[planes_offset..planes_offset + 2].try_into().unwrap());
    let depth = u16::from_le_bytes(
        bytes[planes_offset + 2..planes_offset + 4]
            .try_into()
            .unwrap(),
    );
    pixel_offset >= 14 + header_size && planes == 1 && matches!(depth, 1 | 2 | 4 | 8 | 16 | 24 | 32)
}

fn is_svg(bytes: &[u8]) -> bool {
    let mut remaining = bytes.strip_prefix(b"\xef\xbb\xbf").unwrap_or(bytes);
    loop {
        remaining = remaining.trim_ascii_start();
        let delimiter = if remaining.starts_with(b"<?xml") {
            b"?>".as_slice()
        } else if remaining.starts_with(b"<!--") {
            b"-->".as_slice()
        } else if remaining.starts_with(b"<!DOCTYPE") {
            // Ignore '>' inside a quoted identifier or internal DTD subset.
            let mut quote = None;
            let mut depth = 0usize;
            let end = remaining.iter().enumerate().find_map(|(index, &byte)| {
                if let Some(expected) = quote {
                    if byte == expected {
                        quote = None;
                    }
                } else {
                    match byte {
                        b'\'' | b'"' => quote = Some(byte),
                        b'[' => depth += 1,
                        b']' => depth = depth.saturating_sub(1),
                        b'>' if depth == 0 => return Some(index + 1),
                        _ => {}
                    }
                }
                None
            });
            let Some(end) = end else {
                return false;
            };
            remaining = &remaining[end..];
            continue;
        } else {
            break;
        };
        let Some(end) = remaining
            .windows(delimiter.len())
            .position(|part| part == delimiter)
        else {
            return false;
        };
        remaining = &remaining[end + delimiter.len()..];
    }
    remaining
        .strip_prefix(b"<svg")
        .and_then(|tail| tail.first())
        .is_some_and(|byte| byte.is_ascii_whitespace() || matches!(byte, b'>' | b'/'))
}

fn is_mp3_frames(bytes: &[u8]) -> bool {
    // Two consecutive Layer III frame headers avoid confusing arbitrary 0xff
    // bytes or ADTS AAC with MPEG audio. A file extension can still identify
    // an unrecognized short/free-format stream without a high-confidence claim.
    let Some((length, version, frequency)) = mp3_frame(bytes) else {
        return false;
    };
    bytes
        .get(length..)
        .and_then(mp3_frame)
        .is_some_and(|(_, next_version, next_frequency)| {
            next_version == version && next_frequency == frequency
        })
}

fn mp3_frame(bytes: &[u8]) -> Option<(usize, u8, u8)> {
    let header = bytes.get(..4)?;
    let version = (header[1] >> 3) & 3;
    let frequency = (header[2] >> 2) & 3;
    let bitrate = header[2] >> 4;
    if header[0] != 0xff || header[1] & 0xe0 != 0xe0
        || version == 1 || header[1] & 6 != 2 // Reserved version or non-Layer-III audio.
        || frequency == 3 || bitrate == 0 || bitrate == 15
        || header[3] & 3 == 2
    // Reserved emphasis.
    {
        return None;
    }
    const MPEG1: [usize; 14] = [
        32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320,
    ];
    const MPEG2: [usize; 14] = [8, 16, 24, 32, 40, 48, 56, 64, 80, 96, 112, 128, 144, 160];
    let base_rate = [44100, 48000, 32000][usize::from(frequency)];
    let divisor = match version {
        3 => 1,
        2 => 2,
        _ => 4,
    };
    let sample_rate = base_rate / divisor;
    let bitrate = if version == 3 { MPEG1 } else { MPEG2 }[usize::from(bitrate - 1)];
    let coefficient = if version == 3 { 144000 } else { 72000 };
    let padding = usize::from((header[2] >> 1) & 1);
    Some((
        coefficient * bitrate / sample_rate + padding,
        version,
        frequency,
    ))
}

fn is_aac_adts(bytes: &[u8]) -> bool {
    bytes.len() >= 2 && bytes[0] == 0xff && matches!(bytes[1] & 0xf6, 0xf0)
}

fn is_mp4_family(bytes: &[u8]) -> bool {
    bytes.len() >= 12 && bytes.get(4..8) == Some(b"ftyp")
}

fn mp4_family_mime(bytes: &[u8]) -> &'static str {
    match bytes.get(8..12) {
        Some(b"M4A ") | Some(b"M4B ") | Some(b"M4P ") => "audio/mp4",
        _ => "video/mp4",
    }
}

fn is_webm(bytes: &[u8]) -> bool {
    bytes.starts_with(&[0x1a, 0x45, 0xdf, 0xa3])
        && bytes
            .windows(4)
            .take(2048)
            .any(|window| window.eq_ignore_ascii_case(b"webm"))
}

fn is_sfnt_font(bytes: &[u8], version: &[u8]) -> bool {
    if bytes.len() < 12 || !bytes.starts_with(version) {
        return false;
    }

    let num_tables = u16::from_be_bytes([bytes[4], bytes[5]]);
    if num_tables == 0 || num_tables > 512 {
        return false;
    }

    let search_range = u16::from_be_bytes([bytes[6], bytes[7]]) as u32;
    let entry_selector = u16::from_be_bytes([bytes[8], bytes[9]]) as u32;
    let range_shift = u16::from_be_bytes([bytes[10], bytes[11]]) as u32;
    let (expected_search_range, expected_entry_selector, expected_range_shift) =
        sfnt_table_directory_fields(num_tables as u32);

    search_range == expected_search_range
        && entry_selector == expected_entry_selector
        && range_shift == expected_range_shift
}

fn sfnt_table_directory_fields(num_tables: u32) -> (u32, u32, u32) {
    let mut max_power_of_two = 1;
    let mut entry_selector = 0;
    while max_power_of_two * 2 <= num_tables {
        max_power_of_two *= 2;
        entry_selector += 1;
    }

    let search_range = max_power_of_two * 16;
    let range_shift = num_tables * 16 - search_range;
    (search_range, entry_selector, range_shift)
}

fn text_mime(bytes: &[u8]) -> &'static str {
    let text = std::str::from_utf8(bytes).unwrap_or("");
    let trimmed = text.trim_start();
    let lowered = trimmed
        .chars()
        .take(128)
        .collect::<String>()
        .to_ascii_lowercase();
    if lowered.starts_with("<!doctype html")
        || lowered.starts_with("<html")
        || lowered.starts_with("<head")
        || lowered.starts_with("<body")
    {
        "text/html"
    } else if lowered.starts_with("import ")
        || lowered.starts_with("export ")
        || lowered.starts_with("function ")
        || lowered.starts_with("const ")
        || lowered.starts_with("let ")
        || lowered.starts_with("var ")
    {
        "text/javascript"
    } else if looks_like_css(trimmed, &lowered) {
        "text/css"
    } else {
        "text/plain"
    }
}

fn looks_like_css(trimmed: &str, lowered_prefix: &str) -> bool {
    const CSS_SHAPE_SCAN_CHARS: usize = 1024;

    if lowered_prefix.starts_with("@charset")
        || lowered_prefix.starts_with("@import")
        || lowered_prefix.starts_with("@font-face")
        || lowered_prefix.starts_with("@media")
    {
        return true;
    }

    if !matches!(trimmed.chars().next(), Some('.' | '#' | ':' | '*' | '[')) {
        return false;
    }

    let mut has_open_brace = false;
    let mut has_close_brace = false;
    for ch in trimmed.chars().take(CSS_SHAPE_SCAN_CHARS) {
        match ch {
            '{' => has_open_brace = true,
            '}' => has_close_brace = true,
            _ => {}
        }
        if has_open_brace && has_close_brace {
            return true;
        }
    }

    false
}

fn high(mime: &str) -> SniffedMime {
    SniffedMime {
        mime: mime.into(),
        confidence: MediaSniffConfidence::High,
    }
}

fn low(mime: &str) -> SniffedMime {
    SniffedMime {
        mime: if mime.is_empty() {
            APPLICATION_OCTET_STREAM.into()
        } else {
            mime.into()
        },
        confidence: MediaSniffConfidence::Low,
    }
}

/// A test may fail one operation on its own thread. The original error is moved
/// through the same conversion as the corresponding operating-system failure.
#[cfg(test)]
pub(crate) mod io_failure {
    use std::{cell::RefCell, io};

    #[derive(Clone, Copy, PartialEq, Eq)]
    pub(crate) enum Point {
        Read,
        Write,
        Sync,
    }

    thread_local! {
        static FAILURE: RefCell<Option<(Point, io::Error)>> = const { RefCell::new(None) };
    }

    pub(super) fn check(point: Point) -> io::Result<()> {
        FAILURE.with_borrow_mut(|failure| {
            if failure
                .as_ref()
                .is_some_and(|(expected, _)| *expected == point)
            {
                Err(failure.take().expect("matching failure").1)
            } else {
                Ok(())
            }
        })
    }

    pub(crate) fn during<T>(point: Point, cause: io::Error, operation: impl FnOnce() -> T) -> T {
        struct Restore(Option<(Point, io::Error)>);
        impl Drop for Restore {
            fn drop(&mut self) {
                FAILURE.set(self.0.take());
            }
        }
        let _restore = Restore(FAILURE.replace(Some((point, cause))));
        operation()
    }
}
