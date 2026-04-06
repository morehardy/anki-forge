use anyhow::{bail, Context};
use authoring_core::NormalizedIr;
use serde_json::Value;
use std::{fs, path::PathBuf};

use crate::{
    manifest::{load_manifest, resolve_asset_path},
    policies::{load_build_context_asset, load_writer_policy_asset},
    schema::{load_schema, validate_value},
};

pub fn run(
    manifest: &str,
    input: &str,
    writer_policy: &str,
    build_context: &str,
    artifacts_dir: &str,
    output: &str,
) -> anyhow::Result<String> {
    let manifest = load_manifest(manifest)?;
    let input_raw =
        fs::read_to_string(input).with_context(|| format!("failed to read input: {input}"))?;
    let input_value: Value = serde_json::from_str(&input_raw)
        .with_context(|| format!("input must be valid JSON: {input}"))?;

    let normalized_schema_path = resolve_asset_path(&manifest, "normalized_ir_schema")?;
    let normalized_schema = load_schema(&normalized_schema_path)?;
    validate_value(&normalized_schema, &input_value).with_context(|| {
        format!(
            "build input must satisfy normalized_ir_schema: {}",
            normalized_schema_path.display()
        )
    })?;

    let normalized_ir: NormalizedIr = serde_json::from_value(input_value)
        .context("input must map into normalized IR execution model")?;
    let writer_policy = load_writer_policy_asset(&manifest, writer_policy)?;
    let build_context = load_build_context_asset(&manifest, build_context)?;
    let artifact_target = writer_core::BuildArtifactTarget::new(
        PathBuf::from(artifacts_dir),
        "artifacts".to_string(),
    );

    let result = writer_core::build(
        &normalized_ir,
        &writer_policy,
        &build_context,
        &artifact_target,
    )?;

    match output {
        "contract-json" => writer_core::to_canonical_json(&result),
        "human" => Ok(format!("status: {}", result.result_status)),
        other => bail!("unsupported build output mode: {other}"),
    }
}
