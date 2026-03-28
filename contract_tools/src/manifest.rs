use anyhow::{bail, ensure, Context};
use jsonschema::JSONSchema;
use serde::Deserialize;
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path, PathBuf},
    sync::OnceLock,
};

#[derive(Debug, Deserialize)]
pub struct Compatibility {
    pub public_axis: String,
}

#[derive(Debug, Deserialize)]
pub struct Manifest {
    pub bundle_version: String,
    pub component_versions: BTreeMap<String, String>,
    pub compatibility: Compatibility,
    pub assets: BTreeMap<String, String>,
}

#[derive(Debug)]
pub struct LoadedManifest {
    pub path: PathBuf,
    pub contracts_root: PathBuf,
    pub data: Manifest,
}

pub fn load_manifest(path: impl AsRef<Path>) -> anyhow::Result<LoadedManifest> {
    let path = path.as_ref();
    let path = path
        .canonicalize()
        .with_context(|| format!("failed to resolve manifest path: {}", path.display()))?;
    let contracts_root = path
        .parent()
        .context("manifest must live under contracts/")?
        .to_path_buf();
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("failed to read manifest: {}", path.display()))?;

    let schema_path = contracts_root.join("schema/manifest.schema.json");
    let yaml_value: serde_yaml::Value =
        serde_yaml::from_str(&raw).context("manifest must be valid YAML")?;
    let json_value = serde_json::to_value(yaml_value)
        .context("manifest YAML must be convertible to JSON for validation")?;
    let schema = manifest_schema(&schema_path)?;
    let validator = JSONSchema::compile(schema).context("failed to compile manifest schema")?;
    let validation_result = validator.validate(&json_value);
    if let Err(errors) = validation_result {
        let details = errors
            .map(|error| error.to_string())
            .collect::<Vec<_>>()
            .join("; ");
        bail!("manifest self-validation failed: {}", details);
    }

    let data: Manifest =
        serde_yaml::from_str(&raw).context("manifest must deserialize into the manifest model")?;
    let loaded = LoadedManifest {
        path,
        contracts_root,
        data,
    };

    ensure!(
        loaded.data.compatibility.public_axis == "bundle_version",
        "manifest compatibility.public_axis must be bundle_version"
    );

    for key in loaded.data.assets.keys() {
        resolve_asset_path(&loaded, key)
            .with_context(|| format!("invalid asset entry: {}", key))?;
    }

    Ok(loaded)
}

fn manifest_schema(schema_path: &Path) -> anyhow::Result<&'static Value> {
    static MANIFEST_SCHEMA: OnceLock<&'static Value> = OnceLock::new();

    if let Some(schema) = MANIFEST_SCHEMA.get() {
        return Ok(*schema);
    }

    let schema_raw = fs::read_to_string(schema_path)
        .with_context(|| format!("failed to read manifest schema: {}", schema_path.display()))?;
    let schema: Value =
        serde_json::from_str(&schema_raw).context("manifest schema must be valid JSON")?;
    Ok(*MANIFEST_SCHEMA.get_or_init(|| Box::leak(Box::new(schema))))
}

pub fn resolve_contract_relative_path(
    contracts_root: impl AsRef<Path>,
    relative: impl AsRef<Path>,
) -> anyhow::Result<PathBuf> {
    let contracts_root = contracts_root.as_ref();
    let relative = relative.as_ref();

    ensure!(
        !relative.as_os_str().is_empty(),
        "asset path must not be empty"
    );
    ensure!(
        !relative.is_absolute(),
        "asset path must be relative: {}",
        relative.display()
    );
    ensure!(
        relative
            .components()
            .all(|component| matches!(component, Component::Normal(_))),
        "asset path must not escape contracts/: {}",
        relative.display()
    );

    let path = contracts_root.join(relative);
    ensure!(
        path.exists(),
        "asset path does not exist: {}",
        path.display()
    );
    Ok(path)
}

pub fn resolve_asset_path(manifest: &LoadedManifest, key: &str) -> anyhow::Result<PathBuf> {
    let rel = manifest
        .data
        .assets
        .get(key)
        .with_context(|| format!("missing asset key: {}", key))?;
    resolve_contract_relative_path(&manifest.contracts_root, rel)
        .with_context(|| format!("failed to resolve asset path for key: {}", key))
}
