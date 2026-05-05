use sha1::{Digest, Sha1};
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub enum MediaWriterError {
    CasObjectMissing {
        path: PathBuf,
    },
    CasObjectSizeMismatch {
        path: PathBuf,
        object_id: String,
    },
    CasObjectBlake3Mismatch {
        path: PathBuf,
        object_id: String,
    },
    CasObjectSha1Mismatch {
        path: PathBuf,
        object_id: String,
    },
    CasObjectReadFailed {
        path: PathBuf,
        message: String,
    },
    CasObjectCopyFailed {
        from: PathBuf,
        to: PathBuf,
        message: String,
    },
    ManifestInvariantViolation {
        code: &'static str,
        summary: String,
    },
}

impl MediaWriterError {
    pub fn diagnostic_code(&self) -> &'static str {
        match self {
            Self::CasObjectMissing { .. } => "MEDIA.CAS_OBJECT_MISSING",
            Self::CasObjectSizeMismatch { .. } => "MEDIA.CAS_OBJECT_SIZE_MISMATCH",
            Self::CasObjectBlake3Mismatch { .. } => "MEDIA.CAS_OBJECT_BLAKE3_MISMATCH",
            Self::CasObjectSha1Mismatch { .. } => "MEDIA.CAS_OBJECT_SHA1_MISMATCH",
            Self::CasObjectReadFailed { .. } => "MEDIA.CAS_OBJECT_READ_FAILED",
            Self::CasObjectCopyFailed { .. } => "MEDIA.CAS_OBJECT_COPY_FAILED",
            Self::ManifestInvariantViolation { code, .. } => code,
        }
    }

    pub fn diagnostic_path(&self) -> Option<String> {
        match self {
            Self::CasObjectMissing { path } | Self::CasObjectReadFailed { path, .. } => {
                Some(path.display().to_string())
            }
            Self::CasObjectCopyFailed { to, .. } => Some(to.display().to_string()),
            Self::CasObjectSizeMismatch { path, .. }
            | Self::CasObjectBlake3Mismatch { path, .. }
            | Self::CasObjectSha1Mismatch { path, .. } => Some(path.display().to_string()),
            Self::ManifestInvariantViolation { .. } => None,
        }
    }
}

impl fmt::Display for MediaWriterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CasObjectMissing { path } => {
                write!(formatter, "CAS object missing: {}", path.display())
            }
            Self::CasObjectSizeMismatch { path, object_id } => {
                write!(
                    formatter,
                    "CAS object size mismatch {}: {object_id}",
                    path.display()
                )
            }
            Self::CasObjectBlake3Mismatch { path, object_id } => {
                write!(
                    formatter,
                    "CAS object BLAKE3 mismatch {}: {object_id}",
                    path.display()
                )
            }
            Self::CasObjectSha1Mismatch { path, object_id } => {
                write!(
                    formatter,
                    "CAS object SHA-1 mismatch {}: {object_id}",
                    path.display()
                )
            }
            Self::CasObjectReadFailed { path, message } => {
                write!(
                    formatter,
                    "CAS object read failed {}: {message}",
                    path.display()
                )
            }
            Self::CasObjectCopyFailed { from, to, message } => {
                write!(
                    formatter,
                    "CAS object copy failed {} -> {}: {message}",
                    from.display(),
                    to.display()
                )
            }
            Self::ManifestInvariantViolation { summary, .. } => write!(formatter, "{summary}"),
        }
    }
}

impl std::error::Error for MediaWriterError {}

pub fn copy_verified_cas_object_to_path(
    media_store_dir: &Path,
    object: &authoring_core::MediaObject,
    output_path: &Path,
) -> Result<(), MediaWriterError> {
    let source = cas_object_path(media_store_dir, object)?;
    let mut input = File::open(&source).map_err(|err| classify_open_error(&source, err))?;
    let (temp_path, mut output) = create_temp_output_file(&source, output_path)?;

    let mut blake3_hasher = blake3::Hasher::new();
    let mut sha1_hasher = Sha1::new();
    let mut size_bytes = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];

    loop {
        let read = match input.read(&mut buffer) {
            Ok(read) => read,
            Err(err) => {
                drop(output);
                return Err(read_failed_with_cleanup(
                    &source,
                    &temp_path,
                    err.to_string(),
                ));
            }
        };
        if read == 0 {
            break;
        }
        let chunk = &buffer[..read];
        blake3_hasher.update(chunk);
        sha1_hasher.update(chunk);
        size_bytes += read as u64;
        if let Err(err) = output.write_all(chunk) {
            drop(output);
            return Err(copy_failed_with_cleanup(
                &source,
                output_path,
                &temp_path,
                err.to_string(),
            ));
        }
    }

    if let Err(err) = validate_cas_stream(&source, object, size_bytes, blake3_hasher, sha1_hasher) {
        drop(output);
        let _ = fs::remove_file(&temp_path);
        return Err(err);
    }

    if let Err(err) = output.sync_all() {
        drop(output);
        return Err(copy_failed_with_cleanup(
            &source,
            output_path,
            &temp_path,
            err.to_string(),
        ));
    }
    drop(output);
    if let Err(err) = fs::rename(&temp_path, output_path) {
        return Err(copy_failed_with_cleanup(
            &source,
            output_path,
            &temp_path,
            err.to_string(),
        ));
    }
    Ok(())
}

fn create_temp_output_file(
    source: &Path,
    output_path: &Path,
) -> Result<(PathBuf, File), MediaWriterError> {
    let parent = output_path
        .parent()
        .ok_or_else(|| MediaWriterError::CasObjectCopyFailed {
            from: source.to_path_buf(),
            to: output_path.to_path_buf(),
            message: "output path has no parent directory".into(),
        })?;
    let filename = output_path
        .file_name()
        .map(|name| name.to_string_lossy())
        .unwrap_or_else(|| "media".into());
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);

    for attempt in 0..100 {
        let temp_path = parent.join(format!(
            ".{filename}.tmp-{}-{nonce}-{attempt}",
            std::process::id()
        ));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
        {
            Ok(file) => return Ok((temp_path, file)),
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(err) => {
                return Err(MediaWriterError::CasObjectCopyFailed {
                    from: source.to_path_buf(),
                    to: output_path.to_path_buf(),
                    message: format!("create temp output {}: {err}", temp_path.display()),
                });
            }
        }
    }

    Err(MediaWriterError::CasObjectCopyFailed {
        from: source.to_path_buf(),
        to: output_path.to_path_buf(),
        message: "create unique temp output: too many collisions".into(),
    })
}

fn copy_failed_with_cleanup(
    source: &Path,
    output_path: &Path,
    temp_path: &Path,
    message: String,
) -> MediaWriterError {
    let message = match fs::remove_file(temp_path) {
        Ok(()) => message,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => message,
        Err(err) => format!(
            "{message}; cleanup temp output {} failed: {err}",
            temp_path.display()
        ),
    };
    MediaWriterError::CasObjectCopyFailed {
        from: source.to_path_buf(),
        to: output_path.to_path_buf(),
        message,
    }
}

fn read_failed_with_cleanup(source: &Path, temp_path: &Path, message: String) -> MediaWriterError {
    let message = match fs::remove_file(temp_path) {
        Ok(()) => message,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => message,
        Err(err) => format!(
            "{message}; cleanup temp output {} failed: {err}",
            temp_path.display()
        ),
    };
    MediaWriterError::CasObjectReadFailed {
        path: source.to_path_buf(),
        message,
    }
}

pub fn verify_cas_object_streaming(
    media_store_dir: &Path,
    object: &authoring_core::MediaObject,
) -> Result<PathBuf, MediaWriterError> {
    let path = cas_object_path(media_store_dir, object)?;
    let mut file = File::open(&path).map_err(|err| classify_open_error(&path, err))?;
    let mut blake3_hasher = blake3::Hasher::new();
    let mut sha1_hasher = Sha1::new();
    let mut size_bytes = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];

    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|err| MediaWriterError::CasObjectReadFailed {
                path: path.clone(),
                message: err.to_string(),
            })?;
        if read == 0 {
            break;
        }
        let chunk = &buffer[..read];
        blake3_hasher.update(chunk);
        sha1_hasher.update(chunk);
        size_bytes += read as u64;
    }

    validate_cas_stream(&path, object, size_bytes, blake3_hasher, sha1_hasher)?;
    Ok(path)
}

fn cas_object_path(
    media_store_dir: &Path,
    object: &authoring_core::MediaObject,
) -> Result<PathBuf, MediaWriterError> {
    authoring_core::object_store_path(media_store_dir, &object.blake3).map_err(|message| {
        MediaWriterError::ManifestInvariantViolation {
            code: "MEDIA.INVALID_MEDIA_OBJECT_INVARIANT",
            summary: message,
        }
    })
}

fn validate_cas_stream(
    path: &Path,
    object: &authoring_core::MediaObject,
    size_bytes: u64,
    blake3_hasher: blake3::Hasher,
    sha1_hasher: Sha1,
) -> Result<(), MediaWriterError> {
    if size_bytes != object.size_bytes {
        return Err(MediaWriterError::CasObjectSizeMismatch {
            path: path.to_path_buf(),
            object_id: object.id.clone(),
        });
    }
    let blake3_hex = blake3_hasher.finalize().to_hex();
    if blake3_hex.as_str() != object.blake3 {
        return Err(MediaWriterError::CasObjectBlake3Mismatch {
            path: path.to_path_buf(),
            object_id: object.id.clone(),
        });
    }
    if hex::encode(sha1_hasher.finalize()) != object.sha1 {
        return Err(MediaWriterError::CasObjectSha1Mismatch {
            path: path.to_path_buf(),
            object_id: object.id.clone(),
        });
    }
    Ok(())
}

fn classify_open_error(path: &Path, err: std::io::Error) -> MediaWriterError {
    if err.kind() == std::io::ErrorKind::NotFound {
        MediaWriterError::CasObjectMissing {
            path: path.to_path_buf(),
        }
    } else {
        MediaWriterError::CasObjectReadFailed {
            path: path.to_path_buf(),
            message: err.to_string(),
        }
    }
}
