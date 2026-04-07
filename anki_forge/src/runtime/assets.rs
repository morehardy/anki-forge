use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{bail, ensure, Context};
use serde::Deserialize;
use writer_core::{BuildContext, WriterPolicy};

use super::discovery::{ResolvedRuntime, RuntimeMode};

#[derive(Debug, Deserialize)]
struct Compatibility {
    public_axis: String,
}

#[derive(Debug, Deserialize)]
struct ManifestData {
    bundle_version: String,
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
    let manifest: ManifestData =
        serde_yaml::from_str(&raw).context("decode runtime manifest YAML")?;

    ensure!(
        manifest.compatibility.public_axis == "bundle_version",
        "runtime manifest public_axis must be bundle_version"
    );

    let bundle_root = manifest_path
        .parent()
        .context("runtime manifest must live under contracts/")?
        .to_path_buf();

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
    let path = bundle.runtime.bundle_root.join(rel);
    let path = path
        .canonicalize()
        .with_context(|| format!("resolve asset path: {}", path.display()))?;

    ensure!(
        path.starts_with(&bundle.runtime.bundle_root),
        "asset path must stay within contracts/: {}",
        path.display()
    );
    ensure!(path.is_file(), "asset path must resolve to a file: {}", path.display());
    Ok(path)
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
