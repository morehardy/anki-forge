use anyhow::{anyhow, bail, Context};
use jsonschema::JSONSchema;
use serde_json::Value;
use std::{fs, path::Path};

use crate::manifest::{load_manifest, resolve_asset_path};

pub fn load_schema(path: impl AsRef<Path>) -> anyhow::Result<JSONSchema> {
    let path = path.as_ref();
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read schema: {}", path.display()))?;
    let schema: Value = serde_json::from_str(&raw).context("schema file must be valid JSON")?;
    JSONSchema::compile(&schema)
        .map_err(|error| anyhow!("failed to compile schema: {}: {}", path.display(), error))
}

pub fn validate_value(schema: &JSONSchema, value: &Value) -> anyhow::Result<()> {
    if let Err(errors) = schema.validate(value) {
        let details = errors
            .map(|error| error.to_string())
            .collect::<Vec<_>>()
            .join("; ");
        bail!(details);
    }

    Ok(())
}

pub fn run_schema_gates(manifest_path: impl AsRef<Path>) -> anyhow::Result<()> {
    let manifest = load_manifest(manifest_path)?;
    for (key, _) in manifest
        .data
        .assets
        .iter()
        .filter(|(key, _)| key.as_str() != "manifest_schema" && key.ends_with("_schema"))
    {
        let schema_path = resolve_asset_path(&manifest, key)?;
        load_schema(&schema_path).with_context(|| {
            format!(
                "failed schema gate for asset `{}` at {}",
                key,
                schema_path.display()
            )
        })?;
    }

    Ok(())
}
