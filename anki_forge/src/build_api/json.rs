//! Serializable observations and full outcome snapshots. Paths do not own files.

use super::{BuildCounts, BuildErrorKind};
use crate::diagnostics::Diagnostic;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// A lossless JSON representation of a native path, with no file ownership.
///
/// Unicode paths serialize as strings. Other Unix paths serialize as
/// `{"encoding":"unix_bytes","bytes":[...]}`; Windows paths containing
/// unpaired UTF-16 surrogates use `{"encoding":"windows_wide","units":[...]}`.
/// Encoded paths deserialize only on their corresponding platform. The numeric
/// arrays preserve native code units; they are not display text or UTF-8 bytes
/// obtained by replacing invalid characters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathSnapshot(PathBuf);

impl PathSnapshot {
    /// Copies or takes a native path without accessing the filesystem.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self(path.into())
    }

    /// Borrows the exact native path represented by this snapshot.
    pub fn as_path(&self) -> &Path {
        &self.0
    }

    /// Returns the exact native path without accessing or retaining a file.
    pub fn into_path_buf(self) -> PathBuf {
        self.0
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "encoding", rename_all = "snake_case", deny_unknown_fields)]
enum EncodedPath {
    UnixBytes { bytes: Vec<u8> },
    WindowsWide { units: Vec<u16> },
}

impl Serialize for PathSnapshot {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serialize_path(&self.0, serializer)
    }
}

fn serialize_path<S: serde::Serializer>(path: &Path, serializer: S) -> Result<S::Ok, S::Error> {
    if let Some(path) = path.to_str() {
        return serializer.serialize_str(path);
    }
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        EncodedPath::UnixBytes {
            bytes: path.as_os_str().as_bytes().to_vec(),
        }
        .serialize(serializer)
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        EncodedPath::WindowsWide {
            units: path.as_os_str().encode_wide().collect(),
        }
        .serialize(serializer)
    }
    #[cfg(not(any(unix, windows)))]
    Err(serde::ser::Error::custom(
        "non-Unicode paths require Unix or Windows native path encoding",
    ))
}

impl<'de> Deserialize<'de> for PathSnapshot {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Representation {
            Text(String),
            Native(EncodedPath),
        }
        match Representation::deserialize(deserializer)? {
            Representation::Text(path) => Ok(Self(path.into())),
            Representation::Native(EncodedPath::UnixBytes { bytes }) => {
                #[cfg(unix)]
                {
                    use std::os::unix::ffi::OsStringExt;
                    Ok(Self(std::ffi::OsString::from_vec(bytes).into()))
                }
                #[cfg(not(unix))]
                {
                    let _ = bytes;
                    Err(serde::de::Error::custom(
                        "unix_bytes paths require a Unix platform",
                    ))
                }
            }
            Representation::Native(EncodedPath::WindowsWide { units }) => {
                #[cfg(windows)]
                {
                    use std::os::windows::ffi::OsStringExt;
                    Ok(Self(std::ffi::OsString::from_wide(&units).into()))
                }
                #[cfg(not(windows))]
                {
                    let _ = units;
                    Err(serde::de::Error::custom(
                        "windows_wide paths require a Windows platform",
                    ))
                }
            }
        }
    }
}

/// Observation-only snapshot. It contains no success flag or artifact ownership.
#[derive(Debug, Clone, Serialize)]
pub struct ReportSnapshot {
    /// Version of this observation schema.
    pub schema_version: String,
    /// Counts collected during the operation.
    pub counts: BuildCounts,
    /// Counts from a completely verified baseline, retained after later failure.
    pub baseline_counts: Option<BuildCounts>,
    /// Diagnostics collected in deterministic stage order.
    pub diagnostics: Vec<Diagnostic>,
    /// Elapsed operation time in milliseconds.
    pub duration_ms: u64,
    /// Completed update analysis, including any policy blocking findings.
    pub comparison: Option<crate::update::json::ComparisonSnapshot>,
}

/// Actual file publication stage at the point an operation returned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PublicationStage {
    /// The operation has not replaced or created the destination.
    NotPublished,
    /// The destination was replaced or created.
    Published,
}

/// Whether the operation confirmed file and directory durability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Durability {
    /// File contents and the directory entry were synchronized successfully.
    Confirmed,
    /// Durability was not confirmed, including on platforms without that facility.
    Unconfirmed,
}

/// File publication facts, with no file ownership or lifetime extension.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PublicationSnapshot {
    /// Destination involved in the operation, serialized using [`PathSnapshot`].
    #[serde(serialize_with = "serialize_path")]
    pub path: PathBuf,
    /// The actual publication stage reached.
    pub stage: PublicationStage,
    /// Whether a runtime handle owns deletion of this path.
    pub temporary: bool,
    /// Whether durability was confirmed before returning.
    pub durability: Durability,
}

/// An outcome obtained from the owner of the real build result.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum BuildResultSnapshot {
    /// A completed build with a guaranteed published APKG.
    Success {
        /// Observable artifact path, serialized using [`PathSnapshot`]. Saving
        /// this value does not retain the file.
        #[serde(serialize_with = "serialize_path")]
        artifact: PathBuf,
        /// Whether the artifact's runtime handle owns deletion.
        temporary: bool,
    },
    /// A failed request, potentially after a file was published.
    Failure {
        /// Structured operation classification.
        kind: BuildErrorKind,
        /// Primary machine code assigned at failure.
        code: String,
        /// Human-readable error explanation.
        message: String,
        /// Text snapshots of the actual source error chain.
        causes: Vec<String>,
        /// Actual publication facts, rather than a bare list of paths.
        publications: Vec<PublicationSnapshot>,
    },
}

/// A full build outcome plus observations. This DTO owns no files.
#[derive(Debug, Clone, Serialize)]
pub struct BuildSnapshot {
    /// Version of this outcome schema, independent of the crate version.
    pub schema_version: String,
    /// Version of the library that produced the snapshot.
    pub tool_version: String,
    /// Actual success or failure supplied by the runtime result owner.
    pub result: BuildResultSnapshot,
    /// Observations, which do not independently decide the outcome.
    pub report: ReportSnapshot,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unicode_paths_keep_the_json_string_shape() {
        let path = PathSnapshot::new("folder/中文.apkg");
        let json = serde_json::to_string(&path).unwrap();
        assert_eq!(json, "\"folder/中文.apkg\"");
        assert_eq!(serde_json::from_str::<PathSnapshot>(&json).unwrap(), path);
    }

    #[cfg(unix)]
    #[test]
    fn unix_bytes_round_trip_without_loss_in_success_and_publication_snapshots() {
        use std::os::unix::ffi::OsStringExt;
        let path = PathBuf::from(std::ffi::OsString::from_vec(b"folder/\xff.apkg".to_vec()));
        let expected = serde_json::json!({"encoding":"unix_bytes", "bytes":b"folder/\xff.apkg"});
        let success = BuildResultSnapshot::Success {
            artifact: path.clone(),
            temporary: false,
        };
        assert_eq!(serde_json::to_value(success).unwrap()["artifact"], expected);
        let publication = PublicationSnapshot {
            path: path.clone(),
            stage: PublicationStage::Published,
            temporary: false,
            durability: Durability::Unconfirmed,
        };
        assert_eq!(serde_json::to_value(publication).unwrap()["path"], expected);
        let decoded: PathSnapshot = serde_json::from_value(expected).unwrap();
        assert_eq!(decoded.as_path(), path);
        assert_eq!(decoded.into_path_buf(), path);
        assert!(serde_json::from_value::<PathSnapshot>(
            serde_json::json!({"encoding":"unix_bytes", "bytes":[256]})
        )
        .is_err());
        assert!(serde_json::from_value::<PathSnapshot>(
            serde_json::json!({"encoding":"windows_wide", "units":[0xd800]})
        )
        .is_err());
    }

    #[cfg(windows)]
    #[test]
    fn windows_unpaired_surrogates_round_trip_without_loss() {
        use std::os::windows::ffi::OsStringExt;
        let units = [
            b'C' as u16,
            b':' as u16,
            b'\\' as u16,
            0xd800,
            b'.' as u16,
            b'a' as u16,
        ];
        let path = PathBuf::from(std::ffi::OsString::from_wide(&units));
        let expected = serde_json::json!({"encoding":"windows_wide", "units":units});
        assert_eq!(
            serde_json::to_value(PathSnapshot::new(&path)).unwrap(),
            expected
        );
        let decoded: PathSnapshot = serde_json::from_value(expected).unwrap();
        assert_eq!(decoded.into_path_buf(), path);
        assert!(serde_json::from_value::<PathSnapshot>(
            serde_json::json!({"encoding":"unix_bytes", "bytes":[255]})
        )
        .is_err());
    }
}
