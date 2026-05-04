use std::{fs, path::Path};

use anyhow::Context;
use serde_json::Value;

use crate::{build, BuildArtifactTarget, NormalizedIr, PackageBuildResult};

use super::{
    load_build_context, load_bundle_from_manifest, load_writer_policy, schema::load_schema_asset,
    schema::validate_value, ResolvedRuntime,
};

pub fn build_from_path(
    runtime: &ResolvedRuntime,
    input_path: impl AsRef<Path>,
    writer_policy_selector: &str,
    build_context_selector: &str,
    artifacts_dir: impl AsRef<Path>,
) -> anyhow::Result<PackageBuildResult> {
    let bundle = load_bundle_from_manifest(&runtime.manifest_path)?;
    let input_path = input_path.as_ref();
    let input_raw = fs::read_to_string(input_path)
        .with_context(|| format!("failed to read input: {}", input_path.display()))?;
    let input_value: Value = serde_json::from_str(&input_raw)
        .with_context(|| format!("input must be valid JSON: {}", input_path.display()))?;

    let schema = load_schema_asset(&bundle, "normalized_ir_schema")?;
    validate_value(&schema, &input_value)
        .context("build input must satisfy normalized_ir_schema")?;

    let normalized_ir: NormalizedIr = serde_json::from_value(input_value)
        .context("input must map into normalized IR execution model")?;
    let writer_policy = load_writer_policy(&bundle, writer_policy_selector)?;
    let build_context = load_build_context(&bundle, build_context_selector)?;
    let media_store_dir = input_path
        .parent()
        .map(|parent| parent.join(".anki-forge-media"))
        .unwrap_or_else(|| Path::new(".anki-forge-media").to_path_buf());
    let artifact_target =
        BuildArtifactTarget::new(artifacts_dir.as_ref().to_path_buf(), "artifacts")
            .with_media_store_dir(media_store_dir);

    build(
        &normalized_ir,
        &writer_policy,
        &build_context,
        &artifact_target,
    )
}
