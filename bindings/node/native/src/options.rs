use ankiforge::{
    build::InspectLimits,
    media::MediaLimits,
    update::{CompareOptions, RiskCode, RiskLevel, UpdatePolicy},
    BuildOptions,
};
use napi::Result;
use serde::Deserialize;
#[derive(Deserialize)]
#[serde(untagged)]
enum Budget {
    Number(u64),
    Text(String),
}
impl Budget {
    fn value(self) -> Result<u64> {
        match self {
            Self::Number(v) => Ok(v),
            Self::Text(s) => s
                .parse()
                .map_err(|_| crate::configuration_error("budget exceeds u64")),
        }
    }
}

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct MediaInput {
    max_bytes: Option<Budget>,
}
pub fn media_limits(input: &str) -> Result<MediaLimits> {
    let i: MediaInput = crate::parse(input)?;
    let mut l = MediaLimits::default();
    if let Some(v) = i.max_bytes {
        l.max_bytes = v.value()?
    }
    Ok(l)
}
#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct InspectInput {
    max_archive_bytes: Option<Budget>,
    max_entries: Option<Budget>,
    max_central_directory_bytes: Option<Budget>,
    max_zip_entry_bytes: Option<Budget>,
    max_zip_total_bytes: Option<Budget>,
    max_meta_bytes: Option<Budget>,
    max_media_map_bytes: Option<Budget>,
    max_identity_bytes: Option<Budget>,
    max_collection_bytes: Option<Budget>,
    max_media_bytes: Option<Budget>,
    max_decoded_total_bytes: Option<Budget>,
    max_zstd_window_bytes: Option<Budget>,
}
impl InspectInput {
    fn limits(self) -> Result<InspectLimits> {
        let mut l = InspectLimits::default();
        macro_rules! set{($($f:ident),*)=>{$(if let Some(v)=self.$f{l.$f=v.value()?})*}}
        set!(
            max_archive_bytes,
            max_entries,
            max_central_directory_bytes,
            max_zip_entry_bytes,
            max_zip_total_bytes,
            max_meta_bytes,
            max_media_map_bytes,
            max_identity_bytes,
            max_collection_bytes,
            max_media_bytes,
            max_decoded_total_bytes,
            max_zstd_window_bytes
        );
        Ok(l)
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum Level {
    Info,
    Low,
    Medium,
    High,
    Critical,
}
#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
struct PolicyInput {
    fail_on: Option<Level>,
    allow: Vec<String>,
}
impl PolicyInput {
    fn policy(self) -> Result<UpdatePolicy> {
        let mut p = UpdatePolicy::default();
        if let Some(l) = self.fail_on {
            p = p.fail_on(match l {
                Level::Info => RiskLevel::Info,
                Level::Low => RiskLevel::Low,
                Level::Medium => RiskLevel::Medium,
                Level::High => RiskLevel::High,
                Level::Critical => RiskLevel::Critical,
            })
        }
        for s in self.allow {
            p = p.allow(s.parse::<RiskCode>().map_err(|e| {
                crate::domain("policy", e.kind(), e.code(), &e, serde_json::Value::Null)
            })?)
        }
        Ok(p)
    }
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct BuildInput {
    output: Option<String>,
    temporary: bool,
    update_from: Option<String>,
    inspect_limits: Option<InspectInput>,
    update_policy: Option<PolicyInput>,
}
pub fn build(input: &str) -> Result<BuildOptions> {
    let i: BuildInput = crate::parse(input)?;
    let mut b = match (i.output, i.temporary) {
        (Some(p), false) => BuildOptions::to(p),
        (None, true) => BuildOptions::temporary(),
        _ => {
            return Err(crate::configuration_error(
                "exactly one build destination is required",
            ))
        }
    };
    if let Some(p) = i.update_from {
        b = b.update_from(p)
    }
    if let Some(l) = i.inspect_limits {
        b = b.inspect_limits(l.limits()?)
    }
    if let Some(p) = i.update_policy {
        b = b.update_policy(p.policy()?)
    }
    Ok(b)
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CompareInput {
    baseline: String,
    inspect_limits: Option<InspectInput>,
    update_policy: Option<PolicyInput>,
}
pub fn compare(input: &str) -> Result<CompareOptions> {
    let i: CompareInput = crate::parse(input)?;
    let mut b = CompareOptions::against(i.baseline);
    if let Some(l) = i.inspect_limits {
        b = b.inspect_limits(l.limits()?)
    }
    if let Some(p) = i.update_policy {
        b = b.update_policy(p.policy()?)
    }
    Ok(b)
}
