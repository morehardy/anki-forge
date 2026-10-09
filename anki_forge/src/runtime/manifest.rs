//! Manifest parsing shared by installed bundles and build-time embedding.
use anyhow::{bail, ensure, Context};
use jsonschema::JSONSchema;
use serde::Deserialize;
use serde_json::Value as JsonValue;
use std::{collections::BTreeMap, path::Path};

#[derive(Debug, Deserialize)]
struct Compatibility {
    public_axis: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct ManifestData {
    pub(super) bundle_version: String,
    #[serde(rename = "component_versions")]
    _component_versions: BTreeMap<String, String>,
    compatibility: Compatibility,
    pub(super) assets: BTreeMap<String, String>,
}

pub(super) fn parse_manifest_value(raw: &str) -> anyhow::Result<JsonValue> {
    let manifest_yaml: serde_yaml::Value =
        serde_yaml::from_str(raw).context("manifest must be valid YAML")?;
    serde_json::to_value(manifest_yaml)
        .context("manifest YAML must be convertible to JSON for validation")
}

pub(super) fn validate_manifest(
    raw: &str,
    manifest_json: &JsonValue,
    schema: &JsonValue,
) -> anyhow::Result<ManifestData> {
    let schema = JSONSchema::compile(schema)
        .map_err(|error| anyhow::anyhow!(error.to_string()))
        .context("failed to compile manifest schema")?;
    if let Err(errors) = schema.validate(manifest_json) {
        let details = errors
            .map(|error| error.to_string())
            .collect::<Vec<_>>()
            .join("; ");
        bail!("manifest self-validation failed: {}", details);
    }

    let manifest: ManifestData =
        serde_yaml::from_str(raw).context("manifest must deserialize into the manifest model")?;

    ensure!(
        manifest.compatibility.public_axis == "bundle_version",
        "runtime manifest public_axis must be bundle_version"
    );

    Ok(manifest)
}

pub(super) fn validate_relative_asset_path(relative: &Path) -> anyhow::Result<()> {
    ensure!(
        !relative.as_os_str().is_empty(),
        "asset path must not be empty"
    );
    ensure!(
        !relative.is_absolute(),
        "asset path must be relative: {}",
        relative.display()
    );
    Ok(())
}
