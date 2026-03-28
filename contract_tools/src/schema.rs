use anyhow::{bail, Context};
use jsonschema::JSONSchema;
use serde_json::Value;
use std::{fs, path::Path};

use crate::manifest::{load_manifest, resolve_asset_path};

pub fn load_schema(path: impl AsRef<Path>) -> anyhow::Result<JSONSchema> {
    let path = path.as_ref();
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read schema: {}", path.display()))?;
    let schema: Value = serde_json::from_str(&raw).context("schema file must be valid JSON")?;
    let schema = Box::leak(Box::new(schema));
    JSONSchema::compile(schema).context("failed to compile schema")
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
    for key in [
        "authoring_ir_schema",
        "diagnostic_item_schema",
        "validation_report_schema",
        "service_envelope_schema",
        "error_registry_schema",
    ] {
        let schema_path = resolve_asset_path(&manifest, key)?;
        load_schema(schema_path)?;
    }

    Ok(())
}
