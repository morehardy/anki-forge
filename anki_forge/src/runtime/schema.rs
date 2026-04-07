use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, OnceLock},
};

use anyhow::{anyhow, bail, Context};
use jsonschema::JSONSchema;
use serde_json::Value;
use url::Url;

use super::{resolve_asset_path, RuntimeBundle};

static SCHEMA_CACHE: OnceLock<Mutex<HashMap<PathBuf, Arc<JSONSchema>>>> = OnceLock::new();

pub(crate) fn load_schema_asset(
    bundle: &RuntimeBundle,
    key: &str,
) -> anyhow::Result<Arc<JSONSchema>> {
    let schema_path = resolve_asset_path(bundle, key)?;
    load_schema(&schema_path)
}

pub(crate) fn load_schema(path: impl AsRef<Path>) -> anyhow::Result<Arc<JSONSchema>> {
    let path = path.as_ref();
    let path = path
        .canonicalize()
        .with_context(|| format!("failed to resolve schema path: {}", path.display()))?;
    if let Some(cached) = schema_cache().lock().unwrap().get(&path).cloned() {
        return Ok(cached);
    }

    let raw = fs::read_to_string(&path)
        .with_context(|| format!("failed to read schema: {}", path.display()))?;
    let mut schema: Value = serde_json::from_str(&raw)
        .with_context(|| format!("schema file must be valid JSON: {}", path.display()))?;
    ensure_schema_id(&mut schema, &path)?;

    let mut options = JSONSchema::options();
    register_sibling_schemas(&mut options, &path)?;

    let compiled = Arc::new(
        options
            .compile(&schema)
            .map_err(|error| anyhow!("failed to compile schema: {}: {}", path.display(), error))?,
    );

    let mut cache = schema_cache().lock().unwrap();
    Ok(cache
        .entry(path)
        .or_insert_with(|| compiled.clone())
        .clone())
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

fn schema_cache() -> &'static Mutex<HashMap<PathBuf, Arc<JSONSchema>>> {
    SCHEMA_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::load_schema;

    #[test]
    fn load_schema_reuses_compiled_schema_for_the_same_path() {
        let root = std::env::temp_dir().join(format!(
            "anki-forge-schema-cache-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();

        let leaf = root.join("leaf.schema.json");
        let root_schema = root.join("root.schema.json");
        fs::write(
            &leaf,
            r#"{
  "$id": "file:///placeholder/leaf.schema.json",
  "type": "string"
}"#,
        )
        .unwrap();
        fs::write(
            &root_schema,
            r#"{
  "$ref": "leaf.schema.json"
}"#,
        )
        .unwrap();

        let first = load_schema(&root_schema).unwrap();
        let second = load_schema(&root_schema).unwrap();

        assert!(std::sync::Arc::ptr_eq(&first, &second));

        let _ = fs::remove_dir_all(&root);
    }
}
