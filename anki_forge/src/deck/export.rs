use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::Context;

use crate::writer::{inspect_apkg, inspect_staging, InspectReport};
use crate::writer_core::{artifact_path_from_ref, BuildArtifactTarget, PackageBuildResult};

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

    pub fn inspect_staging(&self) -> anyhow::Result<InspectReport> {
        inspect_staging(&self.staging_manifest_path)
    }

    pub fn inspect_apkg(&self) -> anyhow::Result<InspectReport> {
        Ok(inspect_apkg(&self.apkg_path)?)
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
    pub fn build(
        &self,
        options: crate::build::BuildOptions,
    ) -> Result<crate::build::BuildReport, crate::build::BuildError> {
        crate::product::Project::from(self.clone()).build(options)
    }

    pub fn to_apkg_bytes(&self) -> anyhow::Result<Vec<u8>> {
        with_temp_artifacts_dir("deck-bytes", |artifacts_dir| {
            let output = artifacts_dir.join("deck.apkg");
            let report = crate::product::Project::from(self.clone())
                .write_apkg(&output)
                .map_err(anyhow::Error::from)?;
            let artifact_path = report
                .artifact
                .as_ref()
                .map(|artifact| artifact.path())
                .unwrap_or(output.as_path());
            fs::read(artifact_path)
                .with_context(|| format!("read apkg bytes: {}", artifact_path.display()))
        })
    }

    pub fn write_to<W: Write>(&self, mut writer: W) -> anyhow::Result<()> {
        writer.write_all(&self.to_apkg_bytes()?)?;
        Ok(())
    }

    pub fn write_apkg(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<crate::build::BuildReport, crate::build::BuildError> {
        crate::product::Project::from(self.clone()).write_apkg(path)
    }
}

fn build_package(
    package: &Package,
    artifacts_dir: impl AsRef<Path>,
) -> anyhow::Result<BuildResult> {
    let artifacts_dir = artifacts_dir.as_ref();
    let stable_ref_prefix = package
        .stable_id
        .as_deref()
        .map(|stable_id| format!("artifacts/{stable_id}"))
        .unwrap_or_else(|| "artifacts".into());
    let artifact_target = BuildArtifactTarget::new(artifacts_dir, stable_ref_prefix.clone());
    let (_report, package_build_result) = crate::product::Project::from(package.root_deck.clone())
        .build_package_artifacts(artifacts_dir, stable_ref_prefix)?;

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
