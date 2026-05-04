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
        match emit_apkg(&materialized, artifact_target) {
            Ok(apkg) => Some(apkg),
            Err(err) => {
                return Ok(apkg_error_result(
                    writer_policy,
                    build_context,
                    artifact_target,
                    err,
                ));
            }
        }
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

fn apkg_error_result(
    writer_policy: &WriterPolicy,
    build_context: &BuildContext,
    artifact_target: &BuildArtifactTarget,
    err: anyhow::Error,
) -> PackageBuildResult {
    if let Some(media_err) = err.downcast_ref::<crate::media::MediaWriterError>() {
        return error_result_with_domain(
            writer_policy,
            build_context,
            ErrorResultDetails {
                code: media_err.diagnostic_code().into(),
                summary: err.to_string(),
                domain: "media".into(),
                stage: "emit_apkg".into(),
                operation: "write_media".into(),
                path: media_err.diagnostic_path(),
            },
        );
    }

    error_result_with_domain(
        writer_policy,
        build_context,
        ErrorResultDetails {
            code: "PHASE3.APKG_EMISSION_FAILED".into(),
            summary: err.to_string(),
            domain: "apkg".into(),
            stage: "emit_apkg".into(),
            operation: "write_package".into(),
            path: Some(
                artifact_target
                    .root_dir
                    .join("package.apkg")
                    .display()
                    .to_string(),
            ),
        },
    )
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn apkg_media_errors_are_returned_as_build_diagnostics() {
        let missing_path = PathBuf::from("/tmp/missing-cas-object");
        let result = apkg_error_result(
            &sample_writer_policy(),
            &sample_build_context(),
            &BuildArtifactTarget::new("/tmp/anki-forge-apkg-error", "artifacts/error"),
            anyhow::Error::new(crate::media::MediaWriterError::CasObjectMissing {
                path: missing_path,
            }),
        );

        assert_eq!(result.result_status, "error");
        let diagnostic = result.diagnostics.items.first().expect("diagnostic");
        assert_eq!(diagnostic.code, "MEDIA.CAS_OBJECT_MISSING");
        assert_eq!(diagnostic.domain.as_deref(), Some("media"));
        assert_eq!(diagnostic.stage.as_deref(), Some("emit_apkg"));
        assert_eq!(diagnostic.operation.as_deref(), Some("write_media"));
        assert_eq!(diagnostic.path.as_deref(), Some("/tmp/missing-cas-object"));
    }

    #[test]
    fn apkg_non_media_errors_are_returned_as_build_diagnostics() {
        let result = apkg_error_result(
            &sample_writer_policy(),
            &sample_build_context(),
            &BuildArtifactTarget::new("/tmp/anki-forge-apkg-error", "artifacts/error"),
            anyhow::anyhow!("zip write failed"),
        );

        assert_eq!(result.result_status, "error");
        let diagnostic = result.diagnostics.items.first().expect("diagnostic");
        assert_eq!(diagnostic.code, "PHASE3.APKG_EMISSION_FAILED");
        assert_eq!(diagnostic.domain.as_deref(), Some("apkg"));
        assert_eq!(diagnostic.stage.as_deref(), Some("emit_apkg"));
        assert_eq!(diagnostic.operation.as_deref(), Some("write_package"));
        assert_eq!(
            diagnostic.path.as_deref(),
            Some("/tmp/anki-forge-apkg-error/package.apkg")
        );
    }

    fn sample_writer_policy() -> WriterPolicy {
        WriterPolicy {
            id: "writer-policy.test".into(),
            version: "1.0.0".into(),
            compatibility_target: "anki-2.1".into(),
            stock_notetype_mode: "source-grounded".into(),
            media_entry_mode: "manifest".into(),
            apkg_version: "latest".into(),
        }
    }

    fn sample_build_context() -> BuildContext {
        BuildContext {
            id: "build-context.test".into(),
            version: "1.0.0".into(),
            emit_apkg: true,
            materialize_staging: true,
            media_resolution_mode: "fail".into(),
            unresolved_asset_behavior: "fail".into(),
            fingerprint_mode: "stable".into(),
        }
    }
}
