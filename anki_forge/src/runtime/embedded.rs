#[cfg(test)]
use super::embedded_bundle::EmbeddedFiles;
#[cfg(test)]
use anyhow::{ensure, Context};
#[cfg(test)]
use flate2::read::GzDecoder;
#[cfg(test)]
use std::io::Cursor;
use std::sync::OnceLock;
#[cfg(test)]
use tempfile::TempDir;

use crate::writer_core::{BuildContext, WriterPolicy};

#[cfg(test)]
use super::{load_bundle_from_manifest, RuntimeBundle, RuntimeMode};

const EMBEDDED_BUNDLE_VERSION: &str = env!("ANKI_FORGE_EMBEDDED_BUNDLE_VERSION");
#[cfg(test)]
const EMBEDDED_BUNDLE: &[u8] = include_bytes!(env!("ANKI_FORGE_EMBEDDED_BUNDLE_PATH"));

#[cfg(test)]
struct EmbeddedRuntime {
    _extraction_dir: TempDir,
    bundle: RuntimeBundle,
}

#[cfg(test)]
static EMBEDDED_RUNTIME: OnceLock<Result<EmbeddedRuntime, String>> = OnceLock::new();
static EMBEDDED_WRITER_DEFAULTS: OnceLock<Result<(WriterPolicy, BuildContext), String>> =
    OnceLock::new();

pub(super) fn load_writer_defaults() -> anyhow::Result<(WriterPolicy, BuildContext)> {
    // build.rs validates the entire bundle with the shared decoder before
    // generating this typed JSON. Runtime startup need not unpack it again.
    let defaults = EMBEDDED_WRITER_DEFAULTS.get_or_init(|| {
        serde_json::from_slice(include_bytes!(concat!(
            env!("OUT_DIR"),
            "/writer-defaults.json"
        )))
        .map_err(|error| error.to_string())
    });
    match defaults {
        Ok(defaults) => Ok(defaults.clone()),
        Err(message) => anyhow::bail!("failed to load embedded writer defaults: {message}"),
    }
}

#[cfg(test)]
fn decode_writer_defaults(bytes: &[u8]) -> anyhow::Result<(WriterPolicy, BuildContext)> {
    super::embedded_bundle::decode_writer_defaults(bytes, EMBEDDED_BUNDLE_VERSION)
}

/// Returns the compatibility version of the contract bundle shipped in this crate.
pub const fn embedded_bundle_version() -> &'static str {
    EMBEDDED_BUNDLE_VERSION
}

/// Loads the self-contained contract bundle shipped in this crate.
#[cfg(test)]
pub fn load_embedded_bundle() -> anyhow::Result<RuntimeBundle> {
    let runtime =
        EMBEDDED_RUNTIME.get_or_init(|| materialize_embedded_runtime().map_err(|e| e.to_string()));
    match runtime {
        Ok(runtime) => Ok(runtime.bundle.clone()),
        Err(message) => anyhow::bail!("failed to load embedded contract bundle: {message}"),
    }
}

#[cfg(test)]
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
    fn generated_defaults_match_complete_path_runtime() {
        let (policy, context) = super::load_writer_defaults().unwrap();
        let bundle = super::load_embedded_bundle().unwrap();
        let path_policy = crate::runtime::load_writer_policy(&bundle, "default").unwrap();
        let path_context = crate::runtime::load_build_context(&bundle, "default").unwrap();
        let runtime = bundle.runtime;
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
    fn generated_defaults_match_shared_bundle_decoder() {
        let generated = super::load_writer_defaults().unwrap();
        let decoded = decode_writer_defaults(EMBEDDED_BUNDLE).unwrap();
        assert_eq!(
            serde_json::to_value(generated).unwrap(),
            serde_json::to_value(decoded).unwrap()
        );
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
