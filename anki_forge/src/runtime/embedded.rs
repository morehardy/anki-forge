use std::{
    collections::{BTreeMap, BTreeSet},
    io::{Cursor, Read},
    path::{Component, Path, PathBuf},
    sync::OnceLock,
};

use anyhow::{ensure, Context};
use flate2::read::GzDecoder;
use tempfile::TempDir;

use crate::writer_core::{BuildContext, WriterPolicy};

use super::{load_bundle_from_manifest, RuntimeBundle, RuntimeMode};

const EMBEDDED_BUNDLE_VERSION: &str = env!("ANKI_FORGE_EMBEDDED_BUNDLE_VERSION");
const EMBEDDED_BUNDLE: &[u8] = include_bytes!(env!("ANKI_FORGE_EMBEDDED_BUNDLE_PATH"));

struct EmbeddedRuntime {
    _extraction_dir: TempDir,
    bundle: RuntimeBundle,
}

static EMBEDDED_RUNTIME: OnceLock<Result<EmbeddedRuntime, String>> = OnceLock::new();
static EMBEDDED_WRITER_DEFAULTS: OnceLock<Result<(WriterPolicy, BuildContext), String>> =
    OnceLock::new();

pub(super) fn load_writer_defaults() -> anyhow::Result<(WriterPolicy, BuildContext)> {
    let defaults = EMBEDDED_WRITER_DEFAULTS
        .get_or_init(|| decode_writer_defaults(EMBEDDED_BUNDLE).map_err(|error| error.to_string()));
    match defaults {
        Ok(defaults) => Ok(defaults.clone()),
        Err(message) => anyhow::bail!("failed to load embedded contract bundle: {message}"),
    }
}

fn decode_writer_defaults(bytes: &[u8]) -> anyhow::Result<(WriterPolicy, BuildContext)> {
    // Export only needs the immutable defaults, not paths to the bundle. Validate
    // the same manifest and every asset without materializing the archive. The
    // path-based runtime API below still extracts the complete bundle on demand.
    let files = EmbeddedFiles::read(bytes).context("extract embedded contract bundle")?;
    let raw = files.text(Path::new("manifest.yaml"))?;
    let manifest_json = super::assets::parse_manifest_value(raw)?;
    let schema = serde_json::from_str(files.text(Path::new("schema/manifest.schema.json"))?)
        .context("manifest schema must be valid JSON")?;
    let manifest = super::assets::validate_manifest(raw, &manifest_json, &schema)?;
    for (key, relative) in &manifest.assets {
        files
            .resolve(Path::new(relative))
            .with_context(|| format!("invalid asset entry: {key}"))?;
    }
    ensure!(
        manifest.bundle_version == EMBEDDED_BUNDLE_VERSION,
        "embedded contract bundle version mismatch: expected {}, got {}",
        EMBEDDED_BUNDLE_VERSION,
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

struct EmbeddedFiles {
    files: BTreeMap<PathBuf, Vec<u8>>,
    directories: BTreeSet<PathBuf>,
}

impl EmbeddedFiles {
    fn read(bytes: &[u8]) -> anyhow::Result<Self> {
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

    fn resolve(&self, relative: &Path) -> anyhow::Result<&[u8]> {
        super::assets::validate_relative_asset_path(relative)?;
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

/// Returns the compatibility version of the contract bundle shipped in this crate.
pub const fn embedded_bundle_version() -> &'static str {
    EMBEDDED_BUNDLE_VERSION
}

/// Loads the self-contained contract bundle shipped in this crate.
pub fn load_embedded_bundle() -> anyhow::Result<RuntimeBundle> {
    let runtime =
        EMBEDDED_RUNTIME.get_or_init(|| materialize_embedded_runtime().map_err(|e| e.to_string()));
    match runtime {
        Ok(runtime) => Ok(runtime.bundle.clone()),
        Err(message) => anyhow::bail!("failed to load embedded contract bundle: {message}"),
    }
}

fn materialize_embedded_runtime() -> anyhow::Result<EmbeddedRuntime> {
    let extraction_dir =
        tempfile::tempdir().context("create embedded contract extraction directory")?;
    let decoder = GzDecoder::new(Cursor::new(EMBEDDED_BUNDLE));
    let mut archive = tar::Archive::new(decoder);
    archive
        .unpack(extraction_dir.path())
        .context("extract embedded contract bundle")?;

    let manifest_path = extraction_dir.path().join("contracts/manifest.yaml");
    let mut bundle =
        load_bundle_from_manifest(&manifest_path).context("validate embedded contract bundle")?;
    ensure!(
        bundle.runtime.bundle_version == EMBEDDED_BUNDLE_VERSION,
        "embedded contract bundle version mismatch: expected {}, got {}",
        EMBEDDED_BUNDLE_VERSION,
        bundle.runtime.bundle_version
    );
    bundle.runtime.mode = RuntimeMode::Installed;

    Ok(EmbeddedRuntime {
        _extraction_dir: extraction_dir,
        bundle,
    })
}

#[cfg(test)]
mod tests {
    use std::{io::Write, path::Path};

    use flate2::{write::GzEncoder, Compression};

    use super::{decode_writer_defaults, EmbeddedFiles, EMBEDDED_BUNDLE};

    fn archive(entries: &[(String, Vec<u8>)]) -> Vec<u8> {
        let mut archive = tar::Builder::new(GzEncoder::new(Vec::new(), Compression::fast()));
        for (path, bytes) in entries {
            let mut header = tar::Header::new_gnu();
            header.set_mode(0o644);
            header.set_size(bytes.len() as u64);
            header.set_cksum();
            archive
                .append_data(&mut header, path, bytes.as_slice())
                .unwrap();
        }
        archive.into_inner().unwrap().finish().unwrap()
    }

    fn changed_bundle(
        change: impl FnOnce(&mut std::collections::BTreeMap<std::path::PathBuf, Vec<u8>>),
    ) -> Vec<u8> {
        let mut files = EmbeddedFiles::read(EMBEDDED_BUNDLE).unwrap().files;
        change(&mut files);
        let entries = files
            .into_iter()
            .map(|(path, bytes)| (format!("contracts/{}", path.display()), bytes))
            .collect::<Vec<_>>();
        archive(&entries)
    }

    fn changed_manifest(change: impl FnOnce(&mut serde_yaml::Value)) -> Vec<u8> {
        changed_bundle(|files| {
            let bytes = files.get_mut(Path::new("manifest.yaml")).unwrap();
            let mut manifest = serde_yaml::from_slice(bytes).unwrap();
            change(&mut manifest);
            *bytes = serde_yaml::to_string(&manifest).unwrap().into_bytes();
        })
    }

    #[test]
    fn in_memory_defaults_match_complete_path_runtime() {
        let (policy, context) = super::load_writer_defaults().unwrap();
        let (runtime, path_policy, path_context) =
            crate::runtime::load_default_writer_stack().unwrap();
        assert_eq!(
            serde_json::to_value(policy).unwrap(),
            serde_json::to_value(path_policy).unwrap()
        );
        assert_eq!(
            serde_json::to_value(context).unwrap(),
            serde_json::to_value(path_context).unwrap()
        );
        assert_eq!(runtime.mode, crate::runtime::RuntimeMode::Installed);
        assert_eq!(runtime.bundle_version, super::embedded_bundle_version());
        assert!(runtime.manifest_path.is_file());
        assert!(runtime.bundle_root.is_dir());
        let bundle = crate::runtime::load_embedded_bundle().unwrap();
        let files = EmbeddedFiles::read(EMBEDDED_BUNDLE).unwrap();
        for (key, relative) in &bundle.assets {
            let path = crate::runtime::resolve_asset_path(&bundle, key).unwrap();
            assert_eq!(
                std::fs::read(path).unwrap(),
                files.resolve(Path::new(relative)).unwrap()
            );
        }
    }

    #[test]
    fn in_memory_defaults_validate_the_whole_manifest_and_asset_inventory() {
        let cases = [
            changed_manifest(|manifest| {
                manifest
                    .as_mapping_mut()
                    .unwrap()
                    .remove("component_versions");
            }),
            changed_manifest(|manifest| {
                manifest["compatibility"]["public_axis"] = "other".into();
            }),
            changed_manifest(|manifest| {
                manifest["bundle_version"] = "999.0.0".into();
            }),
            changed_manifest(|manifest| {
                manifest["assets"]["version_policy"] = "missing.md".into();
            }),
            changed_manifest(|manifest| {
                manifest["assets"]["version_policy"] = "schema".into();
            }),
            changed_manifest(|manifest| {
                manifest["assets"]["version_policy"] = "../outside.md".into();
            }),
            changed_manifest(|manifest| {
                manifest["assets"]["version_policy"] = "/outside.md".into();
            }),
        ];
        for bytes in cases {
            assert!(decode_writer_defaults(&bytes).is_err());
        }
    }

    #[test]
    fn in_memory_defaults_preserve_relative_path_resolution_inside_contracts() {
        let bytes = changed_manifest(|manifest| {
            manifest["assets"]["writer_policy"] =
                "schema/../policies/writer-policy.default.yaml".into();
        });
        let (policy, _) = decode_writer_defaults(&bytes).unwrap();
        let (expected, _) = decode_writer_defaults(EMBEDDED_BUNDLE).unwrap();
        assert_eq!(
            serde_json::to_value(policy).unwrap(),
            serde_json::to_value(expected).unwrap()
        );
    }

    #[test]
    fn in_memory_defaults_reject_invalid_policy_and_context() {
        for path in [
            "policies/writer-policy.default.yaml",
            "contexts/build-context.default.yaml",
        ] {
            let bytes = changed_bundle(|files| {
                *files.get_mut(Path::new(path)).unwrap() = b"id: incomplete\n".to_vec();
            });
            assert!(decode_writer_defaults(&bytes).is_err());
        }
    }

    #[test]
    fn embedded_file_inventory_rejects_duplicate_and_conflicting_entries() {
        for entries in [
            vec![
                ("contracts/a".into(), vec![]),
                ("contracts/./a".into(), vec![]),
            ],
            vec![
                ("contracts/a".into(), vec![]),
                ("contracts/a/b".into(), vec![]),
            ],
            vec![
                ("contracts/a/b".into(), vec![]),
                ("contracts/a".into(), vec![]),
            ],
        ] {
            assert!(EmbeddedFiles::read(&archive(&entries)).is_err());
        }
    }

    #[test]
    fn embedded_file_inventory_rejects_links_and_gzip_corruption() {
        let mut tar = tar::Builder::new(Vec::new());
        let mut header = tar::Header::new_gnu();
        header.set_entry_type(tar::EntryType::Symlink);
        header.set_size(0);
        header.set_mode(0o644);
        header.set_link_name("../outside").unwrap();
        header.set_cksum();
        tar.append_data(&mut header, "contracts/link", &[][..])
            .unwrap();
        let mut gzip = GzEncoder::new(Vec::new(), Compression::fast());
        gzip.write_all(&tar.into_inner().unwrap()).unwrap();
        assert!(EmbeddedFiles::read(&gzip.finish().unwrap()).is_err());

        let mut corrupted = EMBEDDED_BUNDLE.to_vec();
        let checksum = corrupted.len() - 8;
        corrupted[checksum] ^= 1;
        assert!(EmbeddedFiles::read(&corrupted).is_err());
    }
}
