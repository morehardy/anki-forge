use anyhow::{bail, Context};
use serde::Deserialize;
use serde_json::Value;
use std::fs;

use crate::{
    manifest::{load_manifest, resolve_asset_path},
    schema::{load_schema, validate_value},
};

#[derive(Debug, Deserialize)]
struct InputDocument {
    kind: String,
    schema_version: String,
    metadata: InputMetadata,
}

#[derive(Debug, Deserialize)]
struct InputMetadata {
    document_id: String,
}

pub fn run(manifest: &str, input: &str, output: &str) -> anyhow::Result<String> {
    let manifest = load_manifest(manifest)?;
    let input_raw =
        fs::read_to_string(input).with_context(|| format!("failed to read input: {input}"))?;
    let input_value: Value = serde_json::from_str(&input_raw)
        .with_context(|| format!("input must be valid JSON: {input}"))?;
    let authoring_schema_path = resolve_asset_path(&manifest, "authoring_ir_schema")?;
    let authoring_schema = load_schema(&authoring_schema_path)?;
    validate_value(&authoring_schema, &input_value).with_context(|| {
        format!(
            "normalize input must satisfy authoring_ir_schema: {}",
            authoring_schema_path.display()
        )
    })?;

    let input_document: InputDocument = serde_json::from_value(input_value)
        .context("input must map into normalize execution model")?;

    let document = authoring_core::AuthoringDocument {
        kind: input_document.kind,
        schema_version: input_document.schema_version,
        metadata_document_id: input_document.metadata.document_id,
    };
    let result = authoring_core::normalize(authoring_core::NormalizationRequest::new(document));

    match output {
        "contract-json" => authoring_core::to_canonical_json(&result),
        "human" => Ok(format!("status: {}", result.result_status)),
        other => bail!("unsupported normalize output mode: {other}"),
    }
}
