use anki_forge::build::{
    ProjectDeclaredMimeMismatchBehavior as Mismatch, ProjectMediaDiagnosticBehavior as Behavior,
    ProjectMediaPolicy,
};
use anki_forge::prelude::InspectLimits;
use serde::Deserialize;
use serde_json::{json, Value};

/// Safe JSON numbers and exact decimal strings are the only binding representations.
struct InspectBudget(u64);
impl<'de> Deserialize<'de> for InspectBudget {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Input {
            Number(u64),
            Decimal(String),
        }
        match Input::deserialize(deserializer)? {
            Input::Number(value) if value <= 9_007_199_254_740_991 => Ok(Self(value)),
            Input::Decimal(value)
                if !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()) =>
            {
                value
                    .parse::<u64>()
                    .map(Self)
                    .map_err(serde::de::Error::custom)
            }
            _ => Err(serde::de::Error::custom(
                "expected a safe unsigned integer or u64 decimal string",
            )),
        }
    }
}

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct InspectInput {
    max_archive_bytes: Option<InspectBudget>,
    max_entries: Option<InspectBudget>,
    max_central_directory_bytes: Option<InspectBudget>,
    max_zip_entry_bytes: Option<InspectBudget>,
    max_zip_total_bytes: Option<InspectBudget>,
    max_meta_bytes: Option<InspectBudget>,
    max_media_map_bytes: Option<InspectBudget>,
    max_collection_bytes: Option<InspectBudget>,
    max_media_bytes: Option<InspectBudget>,
    max_decoded_total_bytes: Option<InspectBudget>,
    max_zstd_window_bytes: Option<InspectBudget>,
}
impl InspectInput {
    pub fn limits(self) -> InspectLimits {
        let mut limits = InspectLimits::default();
        macro_rules! apply { ($($field:ident),*) => { $(if let Some(value) = self.$field { limits.$field = value.0; })* }; }
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
