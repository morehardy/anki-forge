use crate::{core_error, mapped};
use ankiforge::{
    build::{json::PathSnapshot, InspectLimits},
    update::{CompareOptions, RiskLevel, UpdatePolicy},
    BuildOptions,
};
use pyo3::{exceptions::PyValueError, prelude::*};
use serde::Deserialize;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildInput {
    output: Option<PathSnapshot>,
    baseline: Option<PathSnapshot>,
    limits: Option<InspectInput>,
    policy: Option<PolicyInput>,
}
impl BuildInput {
    pub fn options(self) -> PyResult<BuildOptions> {
        let mut b = self
            .output
            .map(PathSnapshot::into_path_buf)
            .map_or_else(BuildOptions::temporary, BuildOptions::to);
        if let Some(p) = self.baseline {
            b = b.update_from(p.into_path_buf());
        }
        if let Some(l) = self.limits {
            b = b.inspect_limits(l.limits());
        }
        if let Some(p) = self.policy {
            b = b.update_policy(p.policy()?);
        }
        Ok(b)
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompareInput {
    baseline: PathSnapshot,
    limits: Option<InspectInput>,
    policy: Option<PolicyInput>,
}
impl CompareInput {
    pub fn options(self) -> PyResult<CompareOptions> {
        let mut b = CompareOptions::against(self.baseline.into_path_buf());
        if let Some(l) = self.limits {
            b = b.inspect_limits(l.limits());
        }
        if let Some(p) = self.policy {
            b = b.update_policy(p.policy()?);
        }
        Ok(b)
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PolicyInput {
    threshold: String,
    allowances: Vec<String>,
}
impl PolicyInput {
    fn policy(self) -> PyResult<UpdatePolicy> {
        let level = match self.threshold.as_str() {
            "info" => RiskLevel::Info,
            "low" => RiskLevel::Low,
            "medium" => RiskLevel::Medium,
            "high" => RiskLevel::High,
            "critical" => RiskLevel::Critical,
            _ => return Err(PyValueError::new_err("invalid risk level")),
        };
        let mut p = UpdatePolicy::default().fail_on(level);
        for c in self.allowances {
            p = p.allow(
                c.parse::<ankiforge::update::RiskCode>()
                    .map_err(mapped!("policy"))?,
            );
        }
        Ok(p)
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
    max_identity_bytes: Option<u64>,
    max_collection_bytes: Option<u64>,
    max_media_bytes: Option<u64>,
    max_decoded_total_bytes: Option<u64>,
    max_zstd_window_bytes: Option<u64>,
}

impl InspectInput {
    pub fn limits(self) -> InspectLimits {
        let mut limits = InspectLimits::default();
        if let Some(value) = self.max_identity_bytes {
            limits.max_identity_bytes = value;
        }
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
