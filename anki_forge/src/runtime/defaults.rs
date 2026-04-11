use std::path::Path;

use anyhow::Context;
use writer_core::{BuildContext, WriterPolicy};

use super::{discover_workspace_runtime, load_bundle_from_manifest, ResolvedRuntime};

pub fn load_default_writer_stack(
    start: impl AsRef<Path>,
) -> anyhow::Result<(ResolvedRuntime, WriterPolicy, BuildContext)> {
    let runtime = discover_workspace_runtime(start)?;
    let bundle = load_bundle_from_manifest(&runtime.manifest_path)?;
    let writer_policy = super::load_writer_policy(&bundle, "default")
        .context("load default writer policy from runtime bundle")?;
    let build_context = super::load_build_context(&bundle, "default")
        .context("load default build context from runtime bundle")?;
    Ok((runtime, writer_policy, build_context))
}
