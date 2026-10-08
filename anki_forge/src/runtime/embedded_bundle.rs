//! Host-side bundle validation and default generation, also compiled in tests.
use super::default_models::{BuildContext, WriterPolicy};
use anyhow::{ensure, Context};
use flate2::read::GzDecoder;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{Cursor, Read},
    path::{Component, Path, PathBuf},
};

pub(super) fn decode_writer_defaults(
    bytes: &[u8],
    expected_version: &str,
) -> anyhow::Result<(WriterPolicy, BuildContext)> {
    // Validate the full inventory and typed defaults before build.rs emits the
    // runtime JSON. The same decoder is exercised by malformed-bundle tests.
    let files = EmbeddedFiles::read(bytes).context("extract embedded contract bundle")?;
    let raw = files.text(Path::new("manifest.yaml"))?;
    let manifest_json = super::manifest::parse_manifest_value(raw)?;
    let schema = serde_json::from_str(files.text(Path::new("schema/manifest.schema.json"))?)
        .context("manifest schema must be valid JSON")?;
    let manifest = super::manifest::validate_manifest(raw, &manifest_json, &schema)?;
    for (key, relative) in &manifest.assets {
        files
            .resolve(Path::new(relative))
            .with_context(|| format!("invalid asset entry: {key}"))?;
    }
    ensure!(
        manifest.bundle_version == expected_version,
        "embedded contract bundle version mismatch: expected {}, got {}",
        expected_version,
        manifest.bundle_version
    );

    let asset_text = |key: &str| -> anyhow::Result<&str> {
        let relative = manifest
            .assets
            .get(key)
            .with_context(|| format!("missing asset key: {key}"))?;
        files.text(Path::new(relative))
    };
    let writer_policy = serde_yaml::from_str(asset_text("writer_policy")?)
        .context("load default writer policy from runtime bundle")?;
    let build_context = serde_yaml::from_str(asset_text("build_context_default")?)
        .context("load default build context from runtime bundle")?;
    Ok((writer_policy, build_context))
}

pub(super) struct EmbeddedFiles {
    pub(super) files: BTreeMap<PathBuf, Vec<u8>>,
    directories: BTreeSet<PathBuf>,
}

impl EmbeddedFiles {
    pub(super) fn read(bytes: &[u8]) -> anyhow::Result<Self> {
        let mut contents = Self {
            files: BTreeMap::new(),
            directories: BTreeSet::from([PathBuf::new()]),
        };
        let mut archive = tar::Archive::new(GzDecoder::new(Cursor::new(bytes)));
        for entry in archive.entries()? {
            let mut entry = entry?;
            let archive_path = entry.path()?.into_owned();
            ensure!(
                archive_path
                    .components()
                    .all(|component| matches!(component, Component::Normal(_) | Component::CurDir)),
                "embedded archive path must stay within contracts/: {}",
                archive_path.display()
            );
            let path: PathBuf = archive_path
                .strip_prefix("contracts")
                .with_context(|| {
                    format!(
                        "embedded archive path is outside contracts/: {}",
                        archive_path.display()
                    )
                })?
                .components()
                .collect();
            let entry_type = entry.header().entry_type();
            ensure!(
                entry_type.is_file() || entry_type.is_dir(),
                "embedded archive entry must be a regular file or directory: {}",
                archive_path.display()
            );
            ensure!(
                !contents.files.contains_key(&path),
                "duplicate embedded archive file: {}",
                archive_path.display()
            );
            if entry_type.is_dir() {
                contents.directories.insert(path.clone());
            } else {
                ensure!(
                    !contents.directories.contains(&path),
                    "embedded archive file conflicts with a directory: {}",
                    archive_path.display()
                );
                let mut data = Vec::new();
                entry.read_to_end(&mut data)?;
                contents.files.insert(path.clone(), data);
            }
            let mut parent = path.parent();
            while let Some(directory) = parent {
                ensure!(
                    !contents.files.contains_key(directory),
                    "embedded archive parent must be a directory: {}",
                    directory.display()
                );
                contents.directories.insert(directory.to_path_buf());
                parent = directory.parent();
            }
        }
        // Read the gzip trailer too, so corrupted embedded bytes cannot evade
        // validation merely because the tar end marker was reached first.
        std::io::copy(&mut archive.into_inner(), &mut std::io::sink())?;
        Ok(contents)
    }

    pub(super) fn resolve(&self, relative: &Path) -> anyhow::Result<&[u8]> {
        super::manifest::validate_relative_asset_path(relative)?;
        let mut path = PathBuf::new();
        for component in relative.components() {
            ensure!(
                self.directories.contains(&path),
                "asset parent must resolve to a directory: {}",
                relative.display()
            );
            match component {
                Component::Normal(part) => path.push(part),
                Component::CurDir => {}
                Component::ParentDir if path.pop() => {}
                _ => anyhow::bail!(
                    "asset path must stay within contracts/: {}",
                    relative.display()
                ),
            }
        }
        ensure!(
            !self.directories.contains(&path),
            "asset path must resolve to a file: {}",
            relative.display()
        );
        self.files
            .get(&path)
            .map(Vec::as_slice)
            .with_context(|| format!("asset path does not exist: {}", relative.display()))
    }

    fn text(&self, relative: &Path) -> anyhow::Result<&str> {
        std::str::from_utf8(self.resolve(relative)?)
            .with_context(|| format!("read embedded contract text: {}", relative.display()))
    }
}
