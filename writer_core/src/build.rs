use std::fs;

use anyhow::{Context, Result};
use base64::Engine;
use authoring_core::NormalizedIr;
use sha1::{Digest, Sha1};
use serde::Serialize;

use crate::apkg::emit_apkg;
use crate::canonical_json::to_canonical_json;
use crate::model::{BuildContext, PackageBuildResult, WriterPolicy};
use crate::staging::{
    error_result, invalid_result, success_result, MaterializedStaging, StagingPackage,
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

    let staging_package = match StagingPackage::from_normalized(normalized_ir, writer_policy, build_context) {
        Ok(package) => Some(package),
        Err(diagnostics) if should_fallback_to_image_occlusion_lane(normalized_ir, &diagnostics) => None,
        Err(diagnostics) => return Ok(invalid_result(writer_policy, build_context, diagnostics)),
    };

    let (materialized, diagnostics) = match staging_package {
        Some(package) => {
            let diagnostics = package.diagnostics().to_vec();
            match package.materialize(artifact_target) {
                Ok(materialized) => (materialized, diagnostics),
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
            }
        }
        None => match materialize_image_occlusion_staging(
            normalized_ir,
            writer_policy,
            build_context,
            artifact_target,
        ) {
            Ok(materialized) => (materialized, vec![]),
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
        },
    };

    let mut result = success_result(writer_policy, build_context, materialized, diagnostics);

    if build_context.emit_apkg {
        let apkg = emit_apkg(normalized_ir, artifact_target)?;
        result.apkg_ref = Some(apkg.apkg_ref);
        result.package_fingerprint = Some(apkg.package_fingerprint);
    }

    Ok(result)
}

fn contains_image_occlusion_lane(normalized_ir: &NormalizedIr) -> bool {
    normalized_ir
        .notetypes
        .iter()
        .any(|notetype| notetype.kind == "image_occlusion")
}

fn should_fallback_to_image_occlusion_lane(
    normalized_ir: &NormalizedIr,
    diagnostics: &[crate::model::BuildDiagnosticItem],
) -> bool {
    contains_image_occlusion_lane(normalized_ir)
        && diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code == "PHASE3.UNSUPPORTED_NOTETYPE_KIND")
}

fn materialize_image_occlusion_staging(
    normalized_ir: &NormalizedIr,
    writer_policy: &WriterPolicy,
    build_context: &BuildContext,
    artifact_target: &BuildArtifactTarget,
) -> Result<MaterializedStaging> {
    fs::create_dir_all(artifact_target.staging_dir())
        .with_context(|| format!("create staging directory {}", artifact_target.staging_dir().display()))?;

    if !normalized_ir.media.is_empty() {
        let media_dir = artifact_target.staging_dir().join("media");
        fs::create_dir_all(&media_dir)
            .with_context(|| format!("create staging media directory {}", media_dir.display()))?;
        for media in &normalized_ir.media {
            let payload = base64::engine::general_purpose::STANDARD
                .decode(media.data_base64.as_bytes())
                .with_context(|| format!("decode media payload {}", media.filename))?;
            let media_path = media_dir.join(&media.filename);
            fs::write(&media_path, payload)
                .with_context(|| format!("write staging media {}", media_path.display()))?;
        }
    }

    let manifest_json = to_canonical_json(&ImageOcclusionStagingManifest {
        kind: "staging-package".into(),
        tool_contract_version: crate::tool_contract_version().into(),
        writer_policy_ref: crate::policy::policy_ref(&writer_policy.id, &writer_policy.version),
        build_context_ref: crate::policy::build_context_ref(build_context)?,
        normalized_ir: normalized_ir.clone(),
    })?;
    let manifest_path = artifact_target.staging_manifest_path();
    fs::write(&manifest_path, manifest_json.as_bytes())
        .with_context(|| format!("write staging manifest {}", manifest_path.display()))?;

    Ok(MaterializedStaging {
        manifest_ref: artifact_target.staging_ref(),
        manifest_path,
        artifact_fingerprint: fingerprint(&manifest_json),
    })
}

#[derive(Debug, Serialize)]
struct ImageOcclusionStagingManifest {
    kind: String,
    tool_contract_version: String,
    writer_policy_ref: String,
    build_context_ref: String,
    normalized_ir: NormalizedIr,
}

fn fingerprint(canonical_json: &str) -> String {
    let digest = Sha1::digest(canonical_json.as_bytes());
    format!("artifact:{}", hex::encode(digest))
}
