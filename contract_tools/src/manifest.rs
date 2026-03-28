use anyhow::{bail, ensure, Context};
use jsonschema::JSONSchema;
use serde::Deserialize;
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
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
    static MANIFEST_SCHEMAS: OnceLock<Mutex<BTreeMap<PathBuf, &'static Value>>> = OnceLock::new();

    let schema_path = schema_path.canonicalize().with_context(|| {
        format!(
            "failed to resolve manifest schema: {}",
            schema_path.display()
        )
    })?;
    let cache = MANIFEST_SCHEMAS.get_or_init(|| Mutex::new(BTreeMap::new()));

    if let Some(schema) = cache
        .lock()
        .expect("schema cache poisoned")
        .get(&schema_path)
        .copied()
    {
        return Ok(schema);
    }

    let schema_raw = fs::read_to_string(&schema_path)
        .with_context(|| format!("failed to read manifest schema: {}", schema_path.display()))?;
    let schema: Value =
        serde_json::from_str(&schema_raw).context("manifest schema must be valid JSON")?;
    let schema = Box::leak(Box::new(schema));

    let mut cache = cache.lock().expect("schema cache poisoned");
    Ok(*cache.entry(schema_path).or_insert(schema))
}

pub fn resolve_contract_relative_path(
    contracts_root: impl AsRef<Path>,
    relative: impl AsRef<Path>,
) -> anyhow::Result<PathBuf> {
    let contracts_root = contracts_root.as_ref();
    let relative = relative.as_ref();

    let contracts_root = contracts_root.canonicalize().with_context(|| {
        format!(
            "failed to resolve contracts root: {}",
            contracts_root.display()
        )
    })?;

    ensure!(
        !relative.as_os_str().is_empty(),
        "asset path must not be empty"
    );
    ensure!(
        !relative.is_absolute(),
        "asset path must be relative: {}",
        relative.display()
    );

    let path = contracts_root.join(relative);
    let path = path
        .canonicalize()
        .with_context(|| format!("asset path does not exist: {}", path.display()))?;
    ensure!(
        path.starts_with(&contracts_root),
        "asset path must stay within contracts/: {}",
        path.display()
    );
    ensure!(
        path.is_file(),
        "asset path must resolve to a file: {}",
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
