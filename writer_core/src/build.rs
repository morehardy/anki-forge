use anyhow::Result;
use authoring_core::NormalizedIr;

use crate::model::{BuildContext, PackageBuildResult, WriterPolicy};
use crate::staging::{error_result, invalid_result, success_result, StagingPackage};

pub use crate::staging::BuildArtifactTarget;

pub fn build(
    normalized_ir: &NormalizedIr,
    writer_policy: &WriterPolicy,
    build_context: &BuildContext,
    artifact_target: &BuildArtifactTarget,
) -> Result<PackageBuildResult> {
    if !build_context.materialize_staging {
        return Ok(error_result(
            writer_policy,
            build_context,
            "PHASE3.STAGING_DISABLED",
            "build_context.materialize_staging is false",
            "build",
            "materialize_staging",
            Some(format!("build-context={}", build_context.id)),
        ));
    }

    let package = match StagingPackage::from_normalized(normalized_ir, writer_policy, build_context)
    {
        Ok(package) => package,
        Err(diagnostics) => return Ok(invalid_result(writer_policy, build_context, diagnostics)),
    };

    let materialized = match package.materialize(artifact_target) {
        Ok(materialized) => materialized,
        Err(err) => {
            return Ok(error_result(
                writer_policy,
                build_context,
                "PHASE3.STAGING_MATERIALIZATION_FAILED",
                err.to_string(),
                "materialize_staging",
                "write_manifest",
                Some(artifact_target.staging_manifest_path().display().to_string()),
            ))
        }
    };

    Ok(success_result(writer_policy, build_context, materialized))
}
