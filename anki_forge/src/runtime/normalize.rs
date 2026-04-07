use std::{fs, path::Path};

use anyhow::Context;
use serde::Deserialize;
use serde_json::Value;

use crate::{
    normalize, AuthoringDocument, AuthoringMedia, AuthoringNote, AuthoringNotetype,
    NormalizationRequest, NormalizationResult,
};

use super::{
    load_bundle_from_manifest, schema::load_schema_asset, schema::validate_value, ResolvedRuntime,
};

#[derive(Debug, Deserialize)]
struct InputDocument {
    kind: String,
    schema_version: String,
    metadata: InputMetadata,
    #[serde(default)]
    notetypes: Vec<AuthoringNotetype>,
    #[serde(default)]
    notes: Vec<AuthoringNote>,
    #[serde(default)]
    media: Vec<AuthoringMedia>,
}

#[derive(Debug, Deserialize)]
struct InputMetadata {
    document_id: String,
}

pub fn normalize_from_path(
    runtime: &ResolvedRuntime,
    input_path: impl AsRef<Path>,
) -> anyhow::Result<NormalizationResult> {
    let bundle = load_bundle_from_manifest(&runtime.manifest_path)?;
    let input_path = input_path.as_ref();
    let input_raw = fs::read_to_string(input_path)
        .with_context(|| format!("failed to read input: {}", input_path.display()))?;
    let input_value: Value = serde_json::from_str(&input_raw)
        .with_context(|| format!("input must be valid JSON: {}", input_path.display()))?;
    let schema = load_schema_asset(&bundle, "authoring_ir_schema")?;
    validate_value(&schema, &input_value)
        .context("normalize input must satisfy authoring_ir_schema")?;

    let input_document: InputDocument = serde_json::from_value(input_value)
        .context("input must map into normalize execution model")?;

    let document = AuthoringDocument {
        kind: input_document.kind,
        schema_version: input_document.schema_version,
        metadata_document_id: input_document.metadata.document_id,
        notetypes: input_document.notetypes,
        notes: input_document.notes,
        media: input_document.media,
    };

    Ok(normalize(NormalizationRequest::new(document)))
}
