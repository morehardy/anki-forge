use anyhow::{ensure, Context};
use serde_json::Value as JsonValue;
use std::{fs, path::Path};

use crate::{
    manifest::{load_manifest, resolve_asset_path},
    schema::{load_schema, validate_value},
};

pub fn run_policy_gates(manifest_path: impl AsRef<Path>) -> anyhow::Result<()> {
    let manifest = load_manifest(manifest_path)?;

    validate_yaml_asset(
        &manifest,
        "identity_policy_schema",
        "identity_policy",
        "identity policy",
    )?;
    validate_yaml_asset(
        &manifest,
        "risk_policy_schema",
        "risk_policy",
        "risk policy",
    )?;
    validate_yaml_asset(
        &manifest,
        "writer_policy_schema",
        "writer_policy",
        "writer policy",
    )?;
    validate_yaml_asset(
        &manifest,
        "verification_policy_schema",
        "verification_policy",
        "verification policy",
    )?;
    validate_yaml_asset(
        &manifest,
        "build_context_schema",
        "build_context_default",
        "build context",
    )?;

    Ok(())
}

pub fn load_writer_policy_asset(
    manifest: &crate::manifest::LoadedManifest,
    selector: &str,
) -> anyhow::Result<writer_core::WriterPolicy> {
    ensure!(
        selector == "default",
        "only default writer_policy selector is supported initially"
    );
    let policy_path = resolve_asset_path(manifest, "writer_policy")?;
    let raw = fs::read_to_string(&policy_path)
        .with_context(|| format!("failed to read writer policy: {}", policy_path.display()))?;
    let policy = serde_yaml::from_str(&raw)
        .with_context(|| format!("failed to decode writer policy: {}", policy_path.display()))?;
    Ok(policy)
}

pub fn load_build_context_asset(
    manifest: &crate::manifest::LoadedManifest,
    selector: &str,
) -> anyhow::Result<writer_core::BuildContext> {
    ensure!(
        selector == "default",
        "only default build_context selector is supported initially"
    );
    let context_path = resolve_asset_path(manifest, "build_context_default")?;
    let raw = fs::read_to_string(&context_path)
        .with_context(|| format!("failed to read build context: {}", context_path.display()))?;
    let context = serde_yaml::from_str(&raw)
        .with_context(|| format!("failed to decode build context: {}", context_path.display()))?;
    Ok(context)
}

fn validate_yaml_asset(
    manifest: &crate::manifest::LoadedManifest,
    schema_key: &str,
    asset_key: &str,
    asset_label: &str,
) -> anyhow::Result<()> {
    let schema_path = resolve_asset_path(manifest, schema_key)?;
    let asset_path = resolve_asset_path(manifest, asset_key)?;

    let schema = load_schema(&schema_path).with_context(|| {
        format!(
            "failed to load {asset_label} schema: {}",
            schema_path.display()
        )
    })?;
    let asset_value = load_yaml_value(&asset_path)?;

    validate_value(&schema, &asset_value).with_context(|| {
        format!(
            "{asset_label} schema validation failed: {} against {}",
            asset_path.display(),
            schema_path.display()
        )
    })?;

    ensure!(
        non_empty_string(&asset_value, "id"),
        "{asset_label} id must not be empty: {}",
        asset_path.display()
    );
    ensure!(
        non_empty_string(&asset_value, "version"),
        "{asset_label} version must not be empty: {}",
        asset_path.display()
    );

    Ok(())
}

fn load_yaml_value(path: impl AsRef<Path>) -> anyhow::Result<JsonValue> {
    let path = path.as_ref();
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read YAML asset: {}", path.display()))?;
    let yaml_value: serde_yaml::Value = serde_yaml::from_str(&raw)
        .with_context(|| format!("YAML asset must be valid YAML: {}", path.display()))?;
    serde_json::to_value(yaml_value).with_context(|| {
        format!(
            "YAML asset must be convertible to JSON for validation: {}",
            path.display()
        )
    })
}

fn non_empty_string(value: &JsonValue, key: &str) -> bool {
    value
        .get(key)
        .and_then(JsonValue::as_str)
        .map(|text| !text.trim().is_empty())
        .unwrap_or(false)
}
