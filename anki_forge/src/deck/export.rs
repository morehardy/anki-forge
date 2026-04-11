use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{ensure, Context};
use authoring_core::{normalize, NormalizationRequest};

use crate::{inspect_apkg, inspect_staging};
use writer_core::{artifact_path_from_ref, BuildArtifactTarget, PackageBuildResult};

use super::model::{Deck, Package};

pub struct BuildResult {
    package_build_result: PackageBuildResult,
    apkg_path: PathBuf,
    staging_manifest_path: PathBuf,
}

impl BuildResult {
    pub fn package_build_result(&self) -> &PackageBuildResult {
        &self.package_build_result
    }

    pub fn apkg_path(&self) -> &Path {
        &self.apkg_path
    }

    pub fn staging_manifest_path(&self) -> &Path {
        &self.staging_manifest_path
    }

    pub fn inspect_staging(&self) -> anyhow::Result<crate::InspectReport> {
        inspect_staging(&self.staging_manifest_path)
    }

    pub fn inspect_apkg(&self) -> anyhow::Result<crate::InspectReport> {
        inspect_apkg(&self.apkg_path)
    }
}

impl Package {
    pub fn build(&self, artifacts_dir: impl AsRef<Path>) -> anyhow::Result<BuildResult> {
        build_package(self, artifacts_dir)
    }

    pub fn to_apkg_bytes(&self) -> anyhow::Result<Vec<u8>> {
        with_temp_artifacts_dir("package-bytes", |artifacts_dir| {
            let build = self.build(artifacts_dir)?;
            fs::read(build.apkg_path())
                .with_context(|| format!("read apkg bytes: {}", build.apkg_path().display()))
        })
    }

    pub fn write_to<W: Write>(&self, mut writer: W) -> anyhow::Result<()> {
        writer.write_all(&self.to_apkg_bytes()?)?;
        Ok(())
    }

    pub fn write_apkg(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let path = path.as_ref();
        fs::write(path, self.to_apkg_bytes()?)
            .with_context(|| format!("write apkg: {}", path.display()))
    }
}

impl Deck {
    pub fn build(&self, artifacts_dir: impl AsRef<Path>) -> anyhow::Result<BuildResult> {
        Package::single(self.clone()).build(artifacts_dir)
    }

    pub fn to_apkg_bytes(&self) -> anyhow::Result<Vec<u8>> {
        Package::single(self.clone()).to_apkg_bytes()
    }

    pub fn write_to<W: Write>(&self, writer: W) -> anyhow::Result<()> {
        Package::single(self.clone()).write_to(writer)
    }

    pub fn write_apkg(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        Package::single(self.clone()).write_apkg(path)
    }
}

fn build_package(
    package: &Package,
    artifacts_dir: impl AsRef<Path>,
) -> anyhow::Result<BuildResult> {
    let root_deck = package.root_deck.clone();
    root_deck.validate()?;
    let lowered = root_deck.lower_authoring()?;
    let normalized = normalize(NormalizationRequest::new(lowered));
    ensure!(
        normalized.result_status == "success",
        "normalization failed with status {}",
        normalized.result_status
    );
    let normalized_ir = normalized
        .normalized_ir
        .context("normalization did not produce a normalized_ir")?;

    let current_dir = std::env::current_dir().context("resolve current directory")?;
    let (_runtime, writer_policy, build_context) =
        crate::runtime::load_default_writer_stack(current_dir)?;
    let stable_ref_prefix = package
        .stable_id
        .as_deref()
        .map(|stable_id| format!("artifacts/{stable_id}"))
        .unwrap_or_else(|| "artifacts".into());
    let artifact_target =
        BuildArtifactTarget::new(artifacts_dir.as_ref().to_path_buf(), stable_ref_prefix);
    let package_build_result = crate::build(
        &normalized_ir,
        &writer_policy,
        &build_context,
        &artifact_target,
    )?;
    ensure!(
        package_build_result.result_status == "success",
        "build failed with status {}",
        package_build_result.result_status
    );

    let apkg_ref = package_build_result
        .apkg_ref
        .as_deref()
        .context("successful build must include apkg_ref")?;
    let staging_ref = package_build_result
        .staging_ref
        .as_deref()
        .context("successful build must include staging_ref")?;

    Ok(BuildResult {
        apkg_path: artifact_path_from_ref(&artifact_target, apkg_ref)?,
        staging_manifest_path: artifact_path_from_ref(&artifact_target, staging_ref)?,
        package_build_result,
    })
}

fn with_temp_artifacts_dir<T>(
    label: &str,
    f: impl FnOnce(&Path) -> anyhow::Result<T>,
) -> anyhow::Result<T> {
    let temp_dir = tempfile::Builder::new()
        .prefix(&format!("anki-forge-{label}-"))
        .tempdir()
        .context("create temp artifacts dir")?;
    f(temp_dir.path())
}
