use crate::{
    fixtures::load_fixture_catalog,
    manifest::{load_manifest, resolve_contract_relative_path},
};
use anyhow::{Context, Result};
use flate2::{write::GzEncoder, Compression};
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    fs::{self, File},
    path::{Path, PathBuf},
};
use tar::Builder;

#[derive(Debug, Deserialize)]
struct Phase2FixtureCaseTransitivePaths {
    authoring_input: String,
    #[serde(default)]
    expected_result: Option<String>,
}

fn artifact_name(bundle_version: &str) -> String {
    format!("anki-forge-contract-bundle-{bundle_version}.tar.gz")
}

pub fn build_artifact(
    manifest_path: impl AsRef<Path>,
    out_dir: impl AsRef<Path>,
) -> Result<PathBuf> {
    let manifest = load_manifest(manifest_path)?;
    let out_dir = out_dir.as_ref();
    fs::create_dir_all(out_dir)
        .with_context(|| format!("failed to create output directory: {}", out_dir.display()))?;

    let artifact_path = out_dir.join(artifact_name(&manifest.data.bundle_version));
    let file = File::create(&artifact_path)
        .with_context(|| format!("failed to create artifact: {}", artifact_path.display()))?;
    let encoder = GzEncoder::new(file, Compression::default());
    let mut builder = Builder::new(encoder);

    for (archive_path, source_path) in package_entries(&manifest)? {
        builder
            .append_path_with_name(&source_path, &archive_path)
            .with_context(|| {
                format!(
                    "failed to add package entry to artifact: {} -> {}",
                    source_path.display(),
                    archive_path.display()
                )
            })?;
    }

    builder.finish().context("failed to finalize tar archive")?;
    let encoder = builder
        .into_inner()
        .context("failed to recover gzip encoder from tar builder")?;
    encoder
        .finish()
        .context("failed to finalize gzip archive")?;

    Ok(artifact_path)
}

fn package_entries(
    manifest: &crate::manifest::LoadedManifest,
) -> Result<BTreeMap<PathBuf, PathBuf>> {
    let mut entries = BTreeMap::from([(
        PathBuf::from("contracts/manifest.yaml"),
        manifest.path.clone(),
    )]);

    for asset_rel in manifest.data.assets.values() {
        let source_path = resolve_contract_relative_path(&manifest.contracts_root, asset_rel)
            .with_context(|| {
                format!("failed to resolve manifest asset for packaging: {asset_rel}")
            })?;
        entries.insert(PathBuf::from("contracts").join(asset_rel), source_path);
    }

    if let Some(fixture_catalog_rel) = manifest.data.assets.get("fixture_catalog") {
        let fixture_catalog_path =
            resolve_contract_relative_path(&manifest.contracts_root, fixture_catalog_rel)
                .with_context(|| {
                    format!(
                        "failed to resolve fixture catalog for packaging: {}",
                        fixture_catalog_rel
                    )
                })?;
        let fixture_catalog = load_fixture_catalog(&fixture_catalog_path)?;

        for case in &fixture_catalog.cases {
            add_relative_entry(&mut entries, &manifest.contracts_root, &case.input)?;
            if let Some(expected) = &case.expected {
                add_relative_entry(&mut entries, &manifest.contracts_root, &expected)?;
            }
            if let Some(target_asset) = &case.target_asset {
                add_relative_entry(&mut entries, &manifest.contracts_root, &target_asset)?;
            }
            for affected_path in &case.affected_paths {
                add_relative_entry(&mut entries, &manifest.contracts_root, &affected_path)?;
            }
            add_case_transitive_entries(&mut entries, &manifest.contracts_root, case)?;
        }
    }

    Ok(entries)
}

fn add_relative_entry(
    entries: &mut BTreeMap<PathBuf, PathBuf>,
    contracts_root: &Path,
    relative_path: &str,
) -> Result<()> {
    let source_path = resolve_contract_relative_path(contracts_root, relative_path)
        .with_context(|| format!("failed to resolve transitive package entry: {relative_path}"))?;
    entries.insert(PathBuf::from("contracts").join(relative_path), source_path);
    Ok(())
}

fn add_case_transitive_entries(
    entries: &mut BTreeMap<PathBuf, PathBuf>,
    contracts_root: &Path,
    case: &crate::fixtures::FixtureCase,
) -> Result<()> {
    match case.category.as_str() {
        "phase2-normalization" | "phase2-risk" => {
            let case_path = resolve_contract_relative_path(contracts_root, &case.input)
                .with_context(|| {
                    format!(
                        "failed to resolve phase2 fixture case for packaging: {}",
                        case.input
                    )
                })?;
            let raw = fs::read_to_string(&case_path).with_context(|| {
                format!(
                    "failed to read phase2 fixture case for packaging: {}",
                    case_path.display()
                )
            })?;
            let transitive: Phase2FixtureCaseTransitivePaths =
                serde_yaml::from_str(&raw).with_context(|| {
                    format!(
                        "phase2 fixture case must be valid YAML for packaging: {}",
                        case_path.display()
                    )
                })?;

            add_relative_entry(entries, contracts_root, &transitive.authoring_input)?;
            if let Some(expected_result) = transitive.expected_result {
                add_relative_entry(entries, contracts_root, &expected_result)?;
            }
        }
        _ => {}
    }

    Ok(())
}
