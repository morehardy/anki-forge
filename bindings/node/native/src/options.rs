use anki_forge::build::{
    ProjectDeclaredMimeMismatchBehavior as Mismatch, ProjectMediaDiagnosticBehavior as Behavior,
    ProjectMediaPolicy,
};
use anki_forge::prelude::InspectLimits;
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct InspectInput {
    max_archive_bytes: Option<u64>,
    max_entries: Option<u64>,
    max_central_directory_bytes: Option<u64>,
    max_zip_entry_bytes: Option<u64>,
    max_zip_total_bytes: Option<u64>,
    max_meta_bytes: Option<u64>,
    max_media_map_bytes: Option<u64>,
    max_collection_bytes: Option<u64>,
    max_media_bytes: Option<u64>,
    max_decoded_total_bytes: Option<u64>,
    max_zstd_window_bytes: Option<u64>,
}
impl InspectInput {
    pub fn limits(self) -> InspectLimits {
        let mut limits = InspectLimits::default();
        macro_rules! apply { ($($field:ident),*) => { $(if let Some(value) = self.$field { limits.$field = value; })* }; }
        apply!(
            max_archive_bytes,
            max_entries,
            max_central_directory_bytes,
            max_zip_entry_bytes,
            max_zip_total_bytes,
            max_meta_bytes,
            max_media_map_bytes,
            max_collection_bytes,
            max_media_bytes,
            max_decoded_total_bytes,
            max_zstd_window_bytes
        );
        limits
    }
}
pub fn default_limits() -> Value {
    let l = InspectLimits::default();
    json!({"maxArchiveBytes":l.max_archive_bytes,"maxEntries":l.max_entries,"maxCentralDirectoryBytes":l.max_central_directory_bytes,
        "maxZipEntryBytes":l.max_zip_entry_bytes,"maxZipTotalBytes":l.max_zip_total_bytes,"maxMetaBytes":l.max_meta_bytes,
        "maxMediaMapBytes":l.max_media_map_bytes,"maxCollectionBytes":l.max_collection_bytes,"maxMediaBytes":l.max_media_bytes,
        "maxDecodedTotalBytes":l.max_decoded_total_bytes,"maxZstdWindowBytes":l.max_zstd_window_bytes})
}
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum DiagnosticBehavior {
    Ignore,
    Info,
    Warning,
    Error,
}
impl From<DiagnosticBehavior> for Behavior {
    fn from(value: DiagnosticBehavior) -> Self {
        match value {
            DiagnosticBehavior::Ignore => Self::Ignore,
            DiagnosticBehavior::Info => Self::Info,
            DiagnosticBehavior::Warning => Self::Warning,
            DiagnosticBehavior::Error => Self::Error,
        }
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum MismatchBehavior {
    Warning,
    Error,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MediaPolicyInput {
    unused_binding: Option<DiagnosticBehavior>,
    unknown_mime: Option<DiagnosticBehavior>,
    declared_mime_mismatch: Option<MismatchBehavior>,
}
impl MediaPolicyInput {
    pub fn policy(self) -> ProjectMediaPolicy {
        let mut policy = ProjectMediaPolicy::strict();
        if let Some(value) = self.unused_binding {
            policy = policy.unused_binding_behavior(value.into());
        }
        if let Some(value) = self.unknown_mime {
            policy = policy.unknown_mime_behavior(value.into());
        }
        if let Some(value) = self.declared_mime_mismatch {
            policy = policy.declared_mime_mismatch_behavior(match value {
                MismatchBehavior::Warning => Mismatch::Warning,
                MismatchBehavior::Error => Mismatch::Error,
            });
        }
        policy
    }
}
