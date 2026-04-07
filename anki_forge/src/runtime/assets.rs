use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

use anyhow::{bail, ensure, Context};
use jsonschema::JSONSchema;
use serde::Deserialize;
use serde_json::Value as JsonValue;
use writer_core::{BuildContext, WriterPolicy};

use super::discovery::{ResolvedRuntime, RuntimeMode};

#[derive(Debug, Deserialize)]
struct Compatibility {
    public_axis: String,
}

#[derive(Debug, Deserialize)]
struct ManifestData {
    bundle_version: String,
    #[serde(rename = "component_versions")]
    _component_versions: BTreeMap<String, String>,
    compatibility: Compatibility,
    assets: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct RuntimeBundle {
    pub runtime: ResolvedRuntime,
    pub assets: BTreeMap<String, String>,
}

pub fn load_bundle_from_manifest(manifest_path: impl AsRef<Path>) -> anyhow::Result<RuntimeBundle> {
    let manifest_path = manifest_path.as_ref();
    let manifest_path = manifest_path
        .canonicalize()
        .with_context(|| format!("resolve manifest path: {}", manifest_path.display()))?;
    let raw = fs::read_to_string(&manifest_path)
        .with_context(|| format!("read manifest: {}", manifest_path.display()))?;
    let manifest_yaml: serde_yaml::Value =
        serde_yaml::from_str(&raw).context("manifest must be valid YAML")?;
    let manifest_json = serde_json::to_value(manifest_yaml)
        .context("manifest YAML must be convertible to JSON for validation")?;
    let schema_path = manifest_path
        .parent()
        .context("runtime manifest must live under contracts/")?
        .join("schema/manifest.schema.json");
    let schema = manifest_schema(&schema_path)?;
    let schema = JSONSchema::compile(schema).context("failed to compile manifest schema")?;
    if let Err(errors) = schema.validate(&manifest_json) {
        let details = errors
            .map(|error| error.to_string())
            .collect::<Vec<_>>()
            .join("; ");
        bail!("manifest self-validation failed: {}", details);
    }

    let manifest: ManifestData =
        serde_yaml::from_str(&raw).context("manifest must deserialize into the manifest model")?;

    ensure!(
        manifest.compatibility.public_axis == "bundle_version",
        "runtime manifest public_axis must be bundle_version"
    );

    let bundle_root = manifest_path
        .parent()
        .context("runtime manifest must live under contracts/")?
        .to_path_buf();

    validate_bundle_assets(&bundle_root, &manifest.assets)?;

    Ok(RuntimeBundle {
        runtime: ResolvedRuntime {
            mode: RuntimeMode::Workspace,
            manifest_path,
            bundle_root,
            bundle_version: manifest.bundle_version,
        },
        assets: manifest.assets,
    })
}

pub fn resolve_asset_path(bundle: &RuntimeBundle, key: &str) -> anyhow::Result<PathBuf> {
    let rel = bundle
        .assets
        .get(key)
        .with_context(|| format!("missing asset key: {key}"))?;
    resolve_contract_relative_path(&bundle.runtime.bundle_root, rel)
        .with_context(|| format!("failed to resolve asset path for key: {key}"))
}

pub fn load_writer_policy(bundle: &RuntimeBundle, selector: &str) -> anyhow::Result<WriterPolicy> {
    if selector != "default" {
        bail!("only default writer_policy selector is supported initially");
    }

    let path = resolve_asset_path(bundle, "writer_policy")?;
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("read writer policy: {}", path.display()))?;
    serde_yaml::from_str(&raw).with_context(|| format!("decode writer policy: {}", path.display()))
}

pub fn load_build_context(bundle: &RuntimeBundle, selector: &str) -> anyhow::Result<BuildContext> {
    if selector != "default" {
        bail!("only default build_context selector is supported initially");
    }

    let path = resolve_asset_path(bundle, "build_context_default")?;
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("read build context: {}", path.display()))?;
    serde_yaml::from_str(&raw).with_context(|| format!("decode build context: {}", path.display()))
}

fn validate_bundle_assets(
    bundle_root: &Path,
    assets: &BTreeMap<String, String>,
) -> anyhow::Result<()> {
    for (key, rel) in assets {
        resolve_contract_relative_path(bundle_root, rel)
            .with_context(|| format!("invalid asset entry: {}", key))?;
    }

    Ok(())
}

fn resolve_contract_relative_path(
    bundle_root: impl AsRef<Path>,
    relative: impl AsRef<Path>,
) -> anyhow::Result<PathBuf> {
    let bundle_root = bundle_root.as_ref();
    let relative = relative.as_ref();

    let bundle_root = bundle_root.canonicalize().with_context(|| {
        format!(
            "failed to resolve contracts root: {}",
            bundle_root.display()
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

    let path = bundle_root.join(relative);
    let path = path
        .canonicalize()
        .with_context(|| format!("asset path does not exist: {}", path.display()))?;
    ensure!(
        path.starts_with(&bundle_root),
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

fn manifest_schema(schema_path: &Path) -> anyhow::Result<&'static JsonValue> {
    static MANIFEST_SCHEMAS: OnceLock<Mutex<BTreeMap<PathBuf, &'static JsonValue>>> =
        OnceLock::new();

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
        .with_context(|| format!("read manifest schema: {}", schema_path.display()))?;
    let schema: JsonValue =
        serde_json::from_str(&schema_raw).context("manifest schema must be valid JSON")?;
    let schema = Box::leak(Box::new(schema));

    let mut cache = cache.lock().expect("schema cache poisoned");
    Ok(*cache.entry(schema_path).or_insert(schema))
}
