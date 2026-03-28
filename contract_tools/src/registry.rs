use anyhow::{ensure, Context};
use serde::Deserialize;
use std::{
    collections::HashSet,
    fs,
    path::Path,
};

use crate::manifest::{load_manifest, resolve_asset_path};

#[derive(Debug, Deserialize)]
pub struct ErrorCode {
    pub id: String,
    pub status: String,
    pub summary: String,
}

#[derive(Debug, Deserialize)]
pub struct ErrorRegistry {
    pub codes: Vec<ErrorCode>,
}

pub fn load_registry(path: impl AsRef<Path>) -> anyhow::Result<ErrorRegistry> {
    let path = path.as_ref();
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read error registry: {}", path.display()))?;
    serde_yaml::from_str(&raw)
        .with_context(|| format!("error registry must be valid YAML: {}", path.display()))
}

pub fn run_registry_gates(manifest_path: impl AsRef<Path>) -> anyhow::Result<()> {
    let manifest = load_manifest(manifest_path)?;
    let registry_path = resolve_asset_path(&manifest, "error_registry")?;
    let registry = load_registry(&registry_path)
        .with_context(|| format!("failed registry gate for {}", registry_path.display()))?;

    ensure!(
        !registry.codes.is_empty(),
        "registry must contain at least one error code"
    );

    let mut seen_ids = HashSet::new();
    for code in &registry.codes {
        ensure!(
            seen_ids.insert(code.id.as_str()),
            "duplicate error code id: {}",
            code.id
        );
        ensure!(
            matches!(code.status.as_str(), "active" | "deprecated" | "removed"),
            "unknown error code lifecycle state for {}: {}",
            code.id,
            code.status
        );
        ensure!(
            !code.summary.trim().is_empty(),
            "error code summary must not be empty: {}",
            code.id
        );
    }

    Ok(())
}
