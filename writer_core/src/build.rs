use anyhow::Result;
use authoring_core::NormalizedIr;

use crate::apkg::emit_apkg;
use crate::model::{BuildContext, PackageBuildResult, WriterPolicy};
use crate::staging::{
    error_result, error_result_with_domain, invalid_result, success_result, ErrorResultDetails,
    StagingPackage,
};

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

    let diagnostics = package.diagnostics().to_vec();
    let materialized = match package.materialize(artifact_target) {
        Ok(materialized) => materialized,
        Err(err) => {
            if let Some(media_err) = err.downcast_ref::<crate::media::MediaWriterError>() {
                return Ok(error_result_with_domain(
                    writer_policy,
                    build_context,
                    ErrorResultDetails {
                        code: media_err.diagnostic_code().into(),
                        summary: err.to_string(),
                        domain: "media".into(),
                        stage: "materialize_staging".into(),
                        operation: "write_media".into(),
                        path: media_err.diagnostic_path(),
                    },
                ));
            }
            return Ok(error_result(
                writer_policy,
                build_context,
                "PHASE3.STAGING_MATERIALIZATION_FAILED",
                err.to_string(),
                "materialize_staging",
                "write_manifest",
                Some(
                    artifact_target
                        .staging_manifest_path()
                        .display()
                        .to_string(),
                ),
            ));
        }
    };

    let apkg = if build_context.emit_apkg {
        Some(emit_apkg(&materialized, artifact_target)?)
    } else {
        None
    };
    let mut result = success_result(writer_policy, build_context, materialized, diagnostics);
    if let Some(apkg) = apkg {
        result.apkg_ref = Some(apkg.apkg_ref);
        result.package_fingerprint = Some(apkg.package_fingerprint);
    }

    Ok(result)
}
