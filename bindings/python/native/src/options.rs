use anki_forge::build::{
    ProjectDeclaredMimeMismatchBehavior, ProjectMediaDiagnosticBehavior, ProjectMediaPolicy,
    ProjectNormalizeOptions, RiskLevel, UpdateSafetyMode,
};
use anki_forge::prelude::{BuildOptions, InspectLimits};
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::PathBuf;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildInput {
    output: Option<PathBuf>,
    artifacts_dir: Option<PathBuf>,
    report_json: Option<PathBuf>,
    inspect: Option<bool>,
    inspect_limits: Option<InspectInput>,
    compare_to: Option<PathBuf>,
    fail_on: Option<RiskLevel>,
    identity_lockfile: Option<PathBuf>,
    write_identity_lockfile: Option<bool>,
    update_safety: Option<UpdateSafetyInput>,
    self_contained: Option<bool>,
    media_mode: Option<MediaModeInput>,
    media_policy: Option<MediaPolicyInput>,
    media_store_dir: Option<PathBuf>,
}

impl BuildInput {
    pub fn options(self) -> BuildOptions {
        let mut options = BuildOptions::new();
        if let Some(path) = self.output {
            options = options.output(path);
        }
        if let Some(path) = self.artifacts_dir {
            options = options.artifacts_dir(path);
        }
        if let Some(path) = self.report_json {
            options = options.report_json(path);
        }
        if let Some(inspect) = self.inspect {
            options = options.inspect(inspect);
        }
        if let Some(limits) = self.inspect_limits {
            options = options.inspect_limits(limits.limits());
        }
        if let Some(path) = self.compare_to {
            options = options.compare_to(path);
        }
        if let Some(level) = self.fail_on {
            options = options.fail_on(level);
        }
        if let Some(path) = self.identity_lockfile {
            options = options.identity_lockfile(path);
        }
        if let Some(write) = self.write_identity_lockfile {
            options = options.write_identity_lockfile(write);
        }
        if let Some(mode) = self.update_safety {
            options = options.update_safety(match mode {
                UpdateSafetyInput::Disabled => UpdateSafetyMode::Disabled,
                UpdateSafetyInput::ReportOnly => UpdateSafetyMode::ReportOnly,
                UpdateSafetyInput::Strict => UpdateSafetyMode::Strict,
            });
        }
        if self.self_contained.is_some()
            || self.media_mode.is_some()
            || self.media_policy.is_some()
            || self.media_store_dir.is_some()
        {
            let mut normalize = ProjectNormalizeOptions::strict();
            if let Some(path) = self.media_store_dir {
                normalize = normalize.media_store_dir(path);
            }
            if let Some(policy) = self.media_policy {
                normalize = normalize.media_policy(policy.policy());
            }
            if matches!(self.media_mode, Some(MediaModeInput::SelfContained))
                || self.self_contained == Some(true)
            {
                normalize = normalize.self_contained();
            }
            options = options.normalize_options(normalize);
        }
        options
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum UpdateSafetyInput {
    Disabled,
    ReportOnly,
    Strict,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum MediaModeInput {
    PathBacked,
    SelfContained,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum Behavior {
    Ignore,
    Info,
    Warning,
    Error,
}

impl From<Behavior> for ProjectMediaDiagnosticBehavior {
    fn from(value: Behavior) -> Self {
        match value {
            Behavior::Ignore => Self::Ignore,
            Behavior::Info => Self::Info,
            Behavior::Warning => Self::Warning,
            Behavior::Error => Self::Error,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum Mismatch {
    Warning,
    Error,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MediaPolicyInput {
    unused_binding: Option<Behavior>,
    unknown_mime: Option<Behavior>,
    declared_mime_mismatch: Option<Mismatch>,
}

impl MediaPolicyInput {
    fn policy(self) -> ProjectMediaPolicy {
        let mut policy = ProjectMediaPolicy::strict();
        if let Some(value) = self.unused_binding {
            policy = policy.unused_binding_behavior(value.into());
        }
        if let Some(value) = self.unknown_mime {
            policy = policy.unknown_mime_behavior(value.into());
        }
        if let Some(value) = self.declared_mime_mismatch {
            policy = policy.declared_mime_mismatch_behavior(match value {
                Mismatch::Warning => ProjectDeclaredMimeMismatchBehavior::Warning,
                Mismatch::Error => ProjectDeclaredMimeMismatchBehavior::Error,
            });
        }
        policy
    }
}

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
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
        if let Some(value) = self.max_archive_bytes {
            limits.max_archive_bytes = value;
        }
        if let Some(value) = self.max_entries {
            limits.max_entries = value;
        }
        if let Some(value) = self.max_central_directory_bytes {
            limits.max_central_directory_bytes = value;
        }
        if let Some(value) = self.max_zip_entry_bytes {
            limits.max_zip_entry_bytes = value;
        }
        if let Some(value) = self.max_zip_total_bytes {
            limits.max_zip_total_bytes = value;
        }
        if let Some(value) = self.max_meta_bytes {
            limits.max_meta_bytes = value;
        }
        if let Some(value) = self.max_media_map_bytes {
            limits.max_media_map_bytes = value;
        }
        if let Some(value) = self.max_collection_bytes {
            limits.max_collection_bytes = value;
        }
        if let Some(value) = self.max_media_bytes {
            limits.max_media_bytes = value;
        }
        if let Some(value) = self.max_decoded_total_bytes {
            limits.max_decoded_total_bytes = value;
        }
        if let Some(value) = self.max_zstd_window_bytes {
            limits.max_zstd_window_bytes = value;
        }
        limits
    }
}

pub fn default_limits() -> Value {
    let limits = InspectLimits::default();
    json!({
        "max_archive_bytes": limits.max_archive_bytes,
        "max_entries": limits.max_entries,
        "max_central_directory_bytes": limits.max_central_directory_bytes,
        "max_zip_entry_bytes": limits.max_zip_entry_bytes,
        "max_zip_total_bytes": limits.max_zip_total_bytes,
        "max_meta_bytes": limits.max_meta_bytes,
        "max_media_map_bytes": limits.max_media_map_bytes,
        "max_collection_bytes": limits.max_collection_bytes,
        "max_media_bytes": limits.max_media_bytes,
        "max_decoded_total_bytes": limits.max_decoded_total_bytes,
        "max_zstd_window_bytes": limits.max_zstd_window_bytes,
    })
}
