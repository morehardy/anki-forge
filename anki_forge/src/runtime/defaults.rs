use std::path::Path;

use crate::writer_core::{BuildContext, WriterPolicy};
use anyhow::Context;

use super::{
    discover_workspace_runtime, load_bundle_from_manifest, load_embedded_bundle, ResolvedRuntime,
    RuntimeBundle,
};

/// Loads the default writer stack from the contract bundle embedded in the crate.
pub fn load_default_writer_stack() -> anyhow::Result<(ResolvedRuntime, WriterPolicy, BuildContext)>
{
    load_writer_stack(load_embedded_bundle()?)
}

/// Loads the default writer stack from a source checkout containing `contracts/manifest.yaml`.
pub fn load_workspace_writer_stack(
    start: impl AsRef<Path>,
) -> anyhow::Result<(ResolvedRuntime, WriterPolicy, BuildContext)> {
    let runtime = discover_workspace_runtime(start)?;
    let bundle = load_bundle_from_manifest(&runtime.manifest_path)?;
    load_writer_stack(bundle)
}

fn load_writer_stack(
    bundle: RuntimeBundle,
) -> anyhow::Result<(ResolvedRuntime, WriterPolicy, BuildContext)> {
    let writer_policy = super::load_writer_policy(&bundle, "default")
        .context("load default writer policy from runtime bundle")?;
    let build_context = super::load_build_context(&bundle, "default")
        .context("load default build context from runtime bundle")?;
    Ok((bundle.runtime, writer_policy, build_context))
}
