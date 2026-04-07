use std::{fs, path::Path};

use anyhow::{anyhow, bail, Context};
use jsonschema::JSONSchema;
use serde_json::Value;
use url::Url;

use super::{resolve_asset_path, RuntimeBundle};

pub(crate) fn load_schema_asset(bundle: &RuntimeBundle, key: &str) -> anyhow::Result<JSONSchema> {
    let schema_path = resolve_asset_path(bundle, key)?;
    load_schema(&schema_path)
}

pub(crate) fn load_schema(path: impl AsRef<Path>) -> anyhow::Result<JSONSchema> {
    let path = path.as_ref();
    let path = path
        .canonicalize()
        .with_context(|| format!("failed to resolve schema path: {}", path.display()))?;
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("failed to read schema: {}", path.display()))?;
    let mut schema: Value = serde_json::from_str(&raw)
        .with_context(|| format!("schema file must be valid JSON: {}", path.display()))?;
    ensure_schema_id(&mut schema, &path)?;

    let mut options = JSONSchema::options();
    register_sibling_schemas(&mut options, &path)?;

    options
        .compile(&schema)
        .map_err(|error| anyhow!("failed to compile schema: {}: {}", path.display(), error))
}

pub(crate) fn validate_value(schema: &JSONSchema, value: &Value) -> anyhow::Result<()> {
    if let Err(errors) = schema.validate(value) {
        let details = errors
            .map(|error| error.to_string())
            .collect::<Vec<_>>()
            .join("; ");
        bail!(details);
    }

    Ok(())
}

fn ensure_schema_id(schema: &mut Value, path: &Path) -> anyhow::Result<()> {
    if let Value::Object(map) = schema {
        map.entry("$id".to_string())
            .or_insert(Value::String(file_uri(path)?));
    }

    Ok(())
}

fn register_sibling_schemas(
    options: &mut jsonschema::CompilationOptions,
    path: &Path,
) -> anyhow::Result<()> {
    let schema_dir = path
        .parent()
        .with_context(|| format!("schema has no parent directory: {}", path.display()))?;

    for entry in fs::read_dir(schema_dir)
        .with_context(|| format!("failed to read schema directory: {}", schema_dir.display()))?
    {
        let entry = entry.with_context(|| {
            format!(
                "failed to read schema directory entry: {}",
                schema_dir.display()
            )
        })?;
        let sibling_path = entry.path();
        if sibling_path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        let raw = fs::read_to_string(&sibling_path)
            .with_context(|| format!("failed to read schema: {}", sibling_path.display()))?;
        let schema: Value = serde_json::from_str(&raw).with_context(|| {
            format!("schema file must be valid JSON: {}", sibling_path.display())
        })?;
        options.with_document(file_uri(&sibling_path)?, schema);
    }

    Ok(())
}

fn file_uri(path: &Path) -> anyhow::Result<String> {
    let path = path
        .canonicalize()
        .with_context(|| format!("failed to resolve schema path: {}", path.display()))?;
    Url::from_file_path(&path)
        .map(|url| url.into())
        .map_err(|()| {
            anyhow!(
                "failed to convert schema path to file URI: {}",
                path.display()
            )
        })
}
