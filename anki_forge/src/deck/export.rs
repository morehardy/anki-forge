use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use anyhow::Context;

use crate::writer::{inspect_apkg_with_limits, inspect_staging, InspectLimits, InspectReport};
use crate::writer_core::{artifact_path_from_ref, BuildArtifactTarget, PackageBuildResult};

use super::model::{Deck, Package};

pub struct BuildResult {
    package_build_result: PackageBuildResult,
    apkg_path: PathBuf,
    staging_manifest_path: PathBuf,
    inspect_limits: InspectLimits,
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
        Ok(inspect_apkg_with_limits(
            &self.apkg_path,
            &self.inspect_limits,
        )?)
    }
}

impl Package {
    pub fn build(&self, artifacts_dir: impl AsRef<Path>) -> anyhow::Result<BuildResult> {
        self.build_with_limits(artifacts_dir, InspectLimits::default())
    }

    /// Build with explicit, finite APKG inspection budgets.
    ///
    /// A limit violation still prevents publication; larger legitimate packages
    /// require the caller to raise the relevant limit. The returned result uses
    /// these same limits when `inspect_apkg()` is called. Byte/write convenience
    /// methods retain the defaults; use this build's artifact for custom budgets.
    pub fn build_with_limits(
        &self,
        artifacts_dir: impl AsRef<Path>,
        limits: InspectLimits,
    ) -> anyhow::Result<BuildResult> {
        build_package(self, artifacts_dir, limits)
    }

    pub fn to_apkg_bytes(&self) -> anyhow::Result<Vec<u8>> {
        self.with_apkg("package-bytes", read_apkg_bytes)
    }

    pub fn write_to<W: Write>(&self, mut writer: W) -> anyhow::Result<()> {
        self.with_apkg("package-write", |path| copy_apkg(path, &mut writer))
    }

    pub fn write_apkg(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let path = path.as_ref();
        self.with_apkg("package-file", |artifact| {
            // Complete and validate the build before opening the destination.
            fs::File::create(path)
                .map_err(anyhow::Error::from)
                .and_then(|mut output| copy_apkg(artifact, &mut output))
                .with_context(|| format!("write apkg: {}", path.display()))
        })
    }

    fn with_apkg<T>(
        &self,
        label: &str,
        use_artifact: impl FnOnce(&Path) -> anyhow::Result<T>,
    ) -> anyhow::Result<T> {
        with_temp_artifacts_dir(label, |artifacts_dir| {
            let build = self.build(artifacts_dir)?;
            use_artifact(build.apkg_path())
        })
    }
}

impl Deck {
    pub fn build(
        &self,
        options: crate::build::BuildOptions,
    ) -> Result<crate::build::BuildReport, crate::build::BuildError> {
        crate::product::Project::from_deck(self).into_build(options)
    }

    pub fn to_apkg_bytes(&self) -> anyhow::Result<Vec<u8>> {
        self.with_apkg("deck-bytes", read_apkg_bytes)
    }

    pub fn write_to<W: Write>(&self, mut writer: W) -> anyhow::Result<()> {
        self.with_apkg("deck-write", |path| copy_apkg(path, &mut writer))
    }

    fn with_apkg<T>(
        &self,
        label: &str,
        use_artifact: impl FnOnce(&Path) -> anyhow::Result<T>,
    ) -> anyhow::Result<T> {
        with_temp_artifacts_dir(label, |artifacts_dir| {
            let output = artifacts_dir.join("deck.apkg");
            let report = self.write_apkg(&output).map_err(anyhow::Error::from)?;
            let artifact_path = report
                .artifact
                .as_ref()
                .map(|artifact| artifact.path())
                .unwrap_or(output.as_path());
            use_artifact(artifact_path)
        })
    }

    pub fn write_apkg(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<crate::build::BuildReport, crate::build::BuildError> {
        self.build(crate::build::BuildOptions::new().output(path.as_ref()))
    }
}

fn read_apkg_bytes(path: &Path) -> anyhow::Result<Vec<u8>> {
    fs::read(path).with_context(|| format!("read apkg bytes: {}", path.display()))
}

fn copy_apkg(path: &Path, writer: &mut impl Write) -> anyhow::Result<()> {
    let mut file = fs::File::open(path)
        .with_context(|| format!("open apkg for copying: {}", path.display()))?;
    copy_bounded(&mut file, writer)?;
    Ok(())
}

fn copy_bounded(reader: &mut impl Read, writer: &mut impl Write) -> io::Result<()> {
    let mut buffer = [0; 64 * 1024];
    loop {
        let count = match reader.read(&mut buffer) {
            Ok(0) => return Ok(()),
            Ok(count) => count,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        };
        writer.write_all(&buffer[..count])?;
    }
}

fn build_package(
    package: &Package,
    artifacts_dir: impl AsRef<Path>,
    inspect_limits: InspectLimits,
) -> anyhow::Result<BuildResult> {
    let artifacts_dir = artifacts_dir.as_ref();
    let stable_ref_prefix = package
        .stable_id
        .as_deref()
        .map(|stable_id| format!("artifacts/{stable_id}"))
        .unwrap_or_else(|| "artifacts".into());
    let artifact_target = BuildArtifactTarget::new(artifacts_dir, stable_ref_prefix.clone());
    let (_report, package_build_result) = crate::product::Project::from_deck(&package.root_deck)
        .build_package_artifacts(artifacts_dir, stable_ref_prefix, inspect_limits.clone())?;

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
        inspect_limits,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_copy_retries_interrupted_reads_and_preserves_read_errors() {
        struct Reader {
            state: u8,
        }
        impl Read for Reader {
            fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
                self.state += 1;
                match self.state {
                    1 => Err(io::ErrorKind::Interrupted.into()),
                    2 => {
                        buffer[..3].copy_from_slice(b"abc");
                        Ok(3)
                    }
                    _ => Err(io::Error::other("read failed")),
                }
            }
        }
        let mut output = Vec::new();
        let error = copy_bounded(&mut Reader { state: 0 }, &mut output).unwrap_err();
        assert_eq!(output, b"abc");
        assert_eq!(error.to_string(), "read failed");
    }
}
