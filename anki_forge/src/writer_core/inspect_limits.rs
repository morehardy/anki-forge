use std::fmt;

/// Finite resource budgets for one APKG inspection. Byte limits are independent:
/// ZIP output includes encoded zstd frames; decoded output counts final content.
/// Raising a limit is an explicit caller decision, never an automatic retry.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct InspectLimits {
    /// Compressed input file size (default 2 GiB).
    pub max_archive_bytes: u64,
    /// ZIP entry count and media-map entry count, independently (default 100,000).
    pub max_entries: u64,
    /// Central directory and ZIP64 extended footer, independently (default 16 MiB).
    pub max_central_directory_bytes: u64,
    /// Actual output of one ZIP entry, before nested zstd (default 1 GiB).
    pub max_zip_entry_bytes: u64,
    /// Actual output of all ZIP entries read by this inspection (default 4 GiB).
    pub max_zip_total_bytes: u64,
    /// Decoded package metadata (default 64 KiB).
    pub max_meta_bytes: u64,
    /// Decoded media index, before JSON/protobuf parsing (default 16 MiB).
    pub max_media_map_bytes: u64,
    /// Decoded collection written to a temporary file (default 512 MiB).
    pub max_collection_bytes: u64,
    /// Decoded individual media payload (default 256 MiB).
    pub max_media_bytes: u64,
    /// All decoded metadata, collection, and media bytes combined (default 4 GiB).
    pub max_decoded_total_bytes: u64,
    /// Window declared by each zstd frame (default 64 MiB).
    pub max_zstd_window_bytes: u64,
}

impl Default for InspectLimits {
    fn default() -> Self {
        Self {
            max_archive_bytes: 2 << 30,
            max_entries: 100_000,
            max_central_directory_bytes: 16 << 20,
            max_zip_entry_bytes: 1 << 30,
            max_zip_total_bytes: 4 << 30,
            max_meta_bytes: 64 << 10,
            max_media_map_bytes: 16 << 20,
            max_collection_bytes: 512 << 20,
            max_media_bytes: 256 << 20,
            max_decoded_total_bytes: 4 << 30,
            max_zstd_window_bytes: 64 << 20,
        }
    }
}

/// A resource limit encountered while inspecting an APKG.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectLimitExceeded {
    pub resource: &'static str,
    pub entry: Option<String>,
    pub limit: u64,
    /// First observed excess, not the unknown full size of a rejected stream.
    pub observed: u64,
}

impl fmt::Display for InspectLimitExceeded {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "INSPECT.RESOURCE_LIMIT_EXCEEDED: {}", self.resource)?;
        if let Some(entry) = &self.entry {
            write!(f, " (entry {entry:?})")?;
        }
        write!(f, " exceeds {} (observed {})", self.limit, self.observed)
    }
}

impl std::error::Error for InspectLimitExceeded {}

/// Inspection failure. Resource exhaustion is distinct from malformed input or
/// ordinary I/O failures, and must never be swallowed as missing media.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum InspectError {
    LimitExceeded(InspectLimitExceeded),
    Read(String),
}

impl InspectError {
    pub fn limit_exceeded(&self) -> Option<&InspectLimitExceeded> {
        match self {
            Self::LimitExceeded(limit) => Some(limit),
            Self::Read(_) => None,
        }
    }

    pub(crate) fn from_anyhow(error: anyhow::Error) -> Self {
        for cause in error.chain() {
            if let Some(limit) = cause.downcast_ref::<InspectLimitExceeded>() {
                return Self::LimitExceeded(limit.clone());
            }
            if let Some(limit) = cause
                .downcast_ref::<std::io::Error>()
                .and_then(std::io::Error::get_ref)
                .and_then(|inner| inner.downcast_ref::<InspectLimitExceeded>())
            {
                return Self::LimitExceeded(limit.clone());
            }
        }
        Self::Read(format!("{error:#}"))
    }
}

impl fmt::Display for InspectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LimitExceeded(limit) => limit.fmt(f),
            Self::Read(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for InspectError {}

pub(crate) fn check(
    resource: &'static str,
    entry: Option<&str>,
    limit: u64,
    observed: u64,
) -> Result<(), InspectLimitExceeded> {
    if observed > limit {
        Err(InspectLimitExceeded {
            resource,
            entry: entry.map(str::to_owned),
            limit,
            observed,
        })
    } else {
        Ok(())
    }
}
