use crate::authoring_core::NormalizedIr;
use anyhow::Result;

use crate::writer_core::model::{
    BuildContext, BuildDiagnosticItem, PackageBuildResult, WriterGuidPlan, WriterPolicy,
};
use crate::writer_core::staging::{
    error_result, error_result_with_domain, invalid_result, success_result, BorrowedStagingPackage,
    ErrorResultDetails,
};

pub use crate::writer_core::staging::BuildArtifactTarget;

/// Keep process-local causes beside the legacy serializable result. Native
/// callers must not reconstruct failures from diagnostic text.
pub(crate) struct BuildAttempt {
    pub(crate) result: PackageBuildResult,
    pub(crate) cause: Option<anyhow::Error>,
}

impl From<PackageBuildResult> for BuildAttempt {
    fn from(result: PackageBuildResult) -> Self {
        Self {
            result,
            cause: None,
        }
    }
}

fn failed_attempt(
    mut result: PackageBuildResult,
    mut observations: Vec<BuildDiagnosticItem>,
    cause: anyhow::Error,
) -> BuildAttempt {
    observations.append(&mut result.diagnostics.items);
    result.diagnostics.items = observations;
    BuildAttempt {
        result,
        cause: Some(cause),
    }
}

#[cfg(feature = "internal-tools")]
pub fn build(
    normalized_ir: &NormalizedIr,
    writer_policy: &WriterPolicy,
    build_context: &BuildContext,
    artifact_target: &BuildArtifactTarget,
) -> Result<PackageBuildResult> {
    build_with_guid_plan(
        normalized_ir,
        writer_policy,
        build_context,
        artifact_target,
        None,
    )
}

#[cfg(feature = "internal-tools")]
pub fn build_with_guid_plan(
    normalized_ir: &NormalizedIr,
    writer_policy: &WriterPolicy,
    build_context: &BuildContext,
    artifact_target: &BuildArtifactTarget,
    guid_plan: Option<&WriterGuidPlan>,
) -> Result<PackageBuildResult> {
    build_with_identity_plan(
        normalized_ir,
        writer_policy,
        build_context,
        artifact_target,
        artifact_target,
        guid_plan,
        None,
    )
    .map(|attempt| attempt.result)
}

/// Product builds retain staging artifacts but keep the APKG private until
/// comparison and policy evaluation have succeeded.
pub(crate) fn build_with_identity_plan(
    normalized_ir: &NormalizedIr,
    writer_policy: &WriterPolicy,
    build_context: &BuildContext,
    artifact_target: &BuildArtifactTarget,
    apkg_target: &BuildArtifactTarget,
    guid_plan: Option<&WriterGuidPlan>,
    notetype_ids: Option<&std::collections::BTreeMap<String, i64>>,
) -> Result<BuildAttempt> {
    build_with_prepared_media(
        normalized_ir,
        writer_policy,
        build_context,
        artifact_target,
        apkg_target,
        guid_plan,
        notetype_ids,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn build_with_prepared_media(
    normalized_ir: &NormalizedIr,
    writer_policy: &WriterPolicy,
    build_context: &BuildContext,
    artifact_target: &BuildArtifactTarget,
    apkg_target: &BuildArtifactTarget,
    guid_plan: Option<&WriterGuidPlan>,
    notetype_ids: Option<&std::collections::BTreeMap<String, i64>>,
    prepared_media: Option<&crate::prepared_media::PreparedMedia>,
) -> Result<BuildAttempt> {
    if !build_context.materialize_staging {
        return Ok(error_result(
            writer_policy,
            build_context,
            "PHASE3.STAGING_DISABLED",
            "build_context.materialize_staging is false",
            "build",
            "materialize_staging",
            Some(format!("build-context={}", build_context.id)),
        )
        .into());
    }

    let package = match BorrowedStagingPackage::from_normalized_with_ids(
        normalized_ir,
        writer_policy,
        build_context,
        notetype_ids,
    ) {
        Ok(package) => package,
        Err(diagnostics) => {
            return Ok(invalid_result(writer_policy, build_context, diagnostics).into())
        }
    };

    let diagnostics = package.diagnostics().to_vec();
    let materialized =
        match package.materialize_with_prepared_media(artifact_target, prepared_media) {
            Ok(materialized) => materialized,
            Err(cause) => {
                let result = if let Some(media_err) =
                    cause.downcast_ref::<crate::writer_core::media::MediaWriterError>()
                {
                    error_result_with_domain(
                        writer_policy,
                        build_context,
                        ErrorResultDetails {
                            code: media_err.diagnostic_code().into(),
                            summary: cause.to_string(),
                            domain: "media".into(),
                            stage: "materialize_staging".into(),
                            operation: "write_media".into(),
                            path: media_err.diagnostic_path(),
                        },
                    )
                } else {
                    error_result(
                        writer_policy,
                        build_context,
                        "PHASE3.STAGING_MATERIALIZATION_FAILED",
                        cause.to_string(),
                        "materialize_staging",
                        "write_manifest",
                        Some(
                            artifact_target
                                .staging_manifest_path()
                                .display()
                                .to_string(),
                        ),
                    )
                };
                return Ok(failed_attempt(result, diagnostics, cause));
            }
        };

    let apkg = if build_context.emit_apkg {
        match crate::writer_core::apkg::emit_apkg_with_prepared_media(
            normalized_ir,
            package.notetype_ids(),
            apkg_target,
            guid_plan,
            prepared_media,
        ) {
            Ok(apkg) => Some(apkg),
            Err(cause) => {
                let identity_failure = if cause
                    .to_string()
                    .starts_with("UPDATE.WRITER_GUID_PLAN_MISMATCH")
                {
                    Some(("UPDATE.WRITER_GUID_PLAN_MISMATCH", "validate_guid_plan"))
                } else if cause
                    .to_string()
                    .starts_with("UPDATE.NOTE_DATA_METADATA_UNMERGEABLE")
                {
                    Some(("UPDATE.NOTE_DATA_METADATA_UNMERGEABLE", "merge_note_data"))
                } else {
                    None
                };
                let result = if let Some((code, operation)) = identity_failure {
                    error_result_with_domain(
                        writer_policy,
                        build_context,
                        ErrorResultDetails {
                            code: code.into(),
                            summary: cause.to_string(),
                            domain: "identity".into(),
                            stage: "emit_apkg".into(),
                            operation: operation.into(),
                            path: None,
                        },
                    )
                } else {
                    apkg_error_result(writer_policy, build_context, apkg_target, &cause)
                };
                return Ok(failed_attempt(result, diagnostics, cause));
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

    Ok(result.into())
}

fn apkg_error_result(
    writer_policy: &WriterPolicy,
    build_context: &BuildContext,
    artifact_target: &BuildArtifactTarget,
    err: &anyhow::Error,
) -> PackageBuildResult {
    if let Some(media_err) = err.downcast_ref::<crate::writer_core::media::MediaWriterError>() {
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
            &anyhow::Error::new(
                crate::writer_core::media::MediaWriterError::CasObjectMissing {
                    path: missing_path,
                    cause: std::io::Error::from(std::io::ErrorKind::NotFound),
                },
            ),
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
            &anyhow::anyhow!("zip write failed"),
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
