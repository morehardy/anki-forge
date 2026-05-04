use crate::{
    fixtures::load_fixture_catalog,
    manifest::{load_manifest, resolve_contract_relative_path},
};
use anyhow::{ensure, Context, Result};
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

#[derive(Debug, Deserialize)]
struct Phase3WriterCaseTransitivePaths {
    normalized_input: String,
    expected_build: String,
    expected_inspect: String,
    #[serde(default)]
    expected_diff: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Phase3E2ECaseTransitivePaths {
    authoring_input: String,
    expected_build: String,
    expected_inspect: String,
    #[serde(default)]
    expected_diff: Option<String>,
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
                add_relative_entry(&mut entries, &manifest.contracts_root, expected)?;
            }
            if let Some(target_asset) = &case.target_asset {
                add_relative_entry(&mut entries, &manifest.contracts_root, target_asset)?;
            }
            for affected_path in &case.affected_paths {
                add_relative_entry(&mut entries, &manifest.contracts_root, affected_path)?;
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

fn add_sibling_tree_if_exists(
    entries: &mut BTreeMap<PathBuf, PathBuf>,
    contracts_root: &Path,
    relative_file: &str,
    sibling_name: &str,
) -> Result<()> {
    let parent = Path::new(relative_file)
        .parent()
        .with_context(|| format!("fixture path must have parent: {relative_file}"))?;
    add_relative_dir_tree_if_exists(entries, contracts_root, &parent.join(sibling_name))
}

fn add_sibling_media_store_objects_if_exists(
    entries: &mut BTreeMap<PathBuf, PathBuf>,
    contracts_root: &Path,
    relative_file: &str,
) -> Result<()> {
    let parent = Path::new(relative_file)
        .parent()
        .with_context(|| format!("fixture path must have parent: {relative_file}"))?;
    add_relative_dir_tree_if_exists(
        entries,
        contracts_root,
        &parent.join(".anki-forge-media/objects/blake3"),
    )
}

fn add_relative_dir_tree_if_exists(
    entries: &mut BTreeMap<PathBuf, PathBuf>,
    contracts_root: &Path,
    relative_dir: &Path,
) -> Result<()> {
    let contracts_root = contracts_root.canonicalize().with_context(|| {
        format!(
            "failed to resolve contracts root for packaging: {}",
            contracts_root.display()
        )
    })?;
    let source_dir = contracts_root.join(relative_dir);
    if !source_dir.exists() {
        return Ok(());
    }
    let source_dir = source_dir.canonicalize().with_context(|| {
        format!(
            "failed to resolve transitive package directory: {}",
            relative_dir.display()
        )
    })?;
    ensure!(
        source_dir.starts_with(&contracts_root),
        "transitive package directory must stay within contracts/: {}",
        source_dir.display()
    );
    ensure!(
        source_dir.is_dir(),
        "transitive package path must resolve to a directory: {}",
        source_dir.display()
    );

    add_dir_entries(entries, &contracts_root, &source_dir)
}

fn add_dir_entries(
    entries: &mut BTreeMap<PathBuf, PathBuf>,
    contracts_root: &Path,
    source_dir: &Path,
) -> Result<()> {
    for entry in fs::read_dir(source_dir)
        .with_context(|| format!("failed to read package directory: {}", source_dir.display()))?
    {
        let entry = entry.with_context(|| {
            format!(
                "failed to read package directory entry: {}",
                source_dir.display()
            )
        })?;
        let file_type = entry.file_type().with_context(|| {
            format!(
                "failed to inspect package entry: {}",
                entry.path().display()
            )
        })?;
        let path = entry.path();
        if file_type.is_dir() {
            add_dir_entries(entries, contracts_root, &path)?;
        } else if file_type.is_file() {
            let relative = path.strip_prefix(contracts_root).with_context(|| {
                format!(
                    "package directory entry must stay within contracts/: {}",
                    path.display()
                )
            })?;
            entries.insert(PathBuf::from("contracts").join(relative), path);
        }
    }

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
            let transitive: Phase2FixtureCaseTransitivePaths = serde_yaml::from_str(&raw)
                .with_context(|| {
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
        "phase3-writer" => {
            let case_path = resolve_contract_relative_path(contracts_root, &case.input)
                .with_context(|| {
                    format!(
                        "failed to resolve phase3 writer fixture case for packaging: {}",
                        case.input
                    )
                })?;
            let raw = fs::read_to_string(&case_path).with_context(|| {
                format!(
                    "failed to read phase3 writer fixture case for packaging: {}",
                    case_path.display()
                )
            })?;
            let transitive: Phase3WriterCaseTransitivePaths = serde_yaml::from_str(&raw)
                .with_context(|| {
                    format!(
                        "phase3 writer fixture case must be valid YAML for packaging: {}",
                        case_path.display()
                    )
                })?;

            add_relative_entry(entries, contracts_root, &transitive.normalized_input)?;
            add_sibling_media_store_objects_if_exists(
                entries,
                contracts_root,
                &transitive.normalized_input,
            )?;
            add_relative_entry(entries, contracts_root, &transitive.expected_build)?;
            add_relative_entry(entries, contracts_root, &transitive.expected_inspect)?;
            if let Some(expected_diff) = transitive.expected_diff {
                add_relative_entry(entries, contracts_root, &expected_diff)?;
            }
        }
        "phase3-e2e" => {
            let case_path = resolve_contract_relative_path(contracts_root, &case.input)
                .with_context(|| {
                    format!(
                        "failed to resolve phase3 e2e fixture case for packaging: {}",
                        case.input
                    )
                })?;
            let raw = fs::read_to_string(&case_path).with_context(|| {
                format!(
                    "failed to read phase3 e2e fixture case for packaging: {}",
                    case_path.display()
                )
            })?;
            let transitive: Phase3E2ECaseTransitivePaths = serde_yaml::from_str(&raw)
                .with_context(|| {
                    format!(
                        "phase3 e2e fixture case must be valid YAML for packaging: {}",
                        case_path.display()
                    )
                })?;

            add_relative_entry(entries, contracts_root, &transitive.authoring_input)?;
            add_sibling_tree_if_exists(
                entries,
                contracts_root,
                &transitive.authoring_input,
                "assets",
            )?;
            add_relative_entry(entries, contracts_root, &transitive.expected_build)?;
            add_relative_entry(entries, contracts_root, &transitive.expected_inspect)?;
            if let Some(expected_diff) = transitive.expected_diff {
                add_relative_entry(entries, contracts_root, &expected_diff)?;
            }
        }
        _ => {}
    }

    Ok(())
}
