use crate::{
    manifest::{load_manifest, resolve_asset_path, LoadedManifest},
    package::package_entries,
    registry::load_registry,
};
use anyhow::{ensure, Context, Result};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path},
};

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BundleChangeRecord {
    pub baseline_version: String,
    pub bundle_version: String,
    pub compatibility_class: String,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub migration_notes: Option<String>,
    pub changes: Vec<AssetChange>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AssetChange {
    pub path: String,
    pub before: Option<String>,
    pub after: Option<String>,
}

/// Inventory the exact published assets, including fixture dependencies, so
/// packaging and compatibility verification use the same dependency closure.
pub fn change_record_template(current: &Path, baseline: &Path) -> Result<BundleChangeRecord> {
    let current = load_manifest(current)?;
    let baseline = load_manifest(baseline)?;
    Ok(BundleChangeRecord {
        baseline_version: baseline.data.bundle_version.clone(),
        bundle_version: current.data.bundle_version.clone(),
        compatibility_class: "additive_compatible".into(),
        summary: String::new(),
        migration_notes: None,
        changes: actual_changes(&current, &baseline)?,
    })
}

/// Compare two independent bundles. CI supplies a baseline extracted from the
/// PR base / push-before commit; an installed bundle needs no checkout or Git.
pub fn run_change_gates(current: &Path, baseline: &Path) -> Result<()> {
    let current = load_manifest(current)?;
    let baseline = load_manifest(baseline)?;
    let current_version = version(&current.data.bundle_version)?;
    let baseline_version = version(&baseline.data.bundle_version)?;
    ensure!(
        !current_version.cmp_precedence(&baseline_version).is_lt(),
        "bundle_version must not decrease: {baseline_version} -> {current_version}"
    );
    let changes = actual_changes(&current, &baseline)?;
    if changes.is_empty() {
        return Ok(());
    }

    ensure!(current.data.assets.contains_key("bundle_change"),
        "changed contracts require a manifest bundle_change asset; generate one with `contract_tools changes --manifest <current> --baseline-manifest <baseline>`\nactual asset changes:\n{}", serde_yaml::to_string(&changes)?);
    let mut record = read_record(&current)?;
    validate_record(&record, &current)?;
    ensure!(
        record.baseline_version == baseline.data.bundle_version,
        "bundle_change baseline_version must match the actual baseline: expected {}, got {}",
        baseline.data.bundle_version,
        record.baseline_version
    );
    record.changes.sort();
    ensure!(record.changes == changes,
        "bundle_change must describe the exact actual asset changes (including before/after BLAKE3 digests); regenerate the record:\n{}", serde_yaml::to_string(&changes)?);

    let structural_break = baseline
        .data
        .assets
        .iter()
        .filter(|(key, _)| key.as_str() != "bundle_change")
        .any(|(key, path)| current.data.assets.get(key) != Some(path));
    let removed_file = changes.iter().any(|change| change.after.is_none());
    ensure!(!(structural_break || removed_file || removed_registry_code(&current, &baseline)?)
        || record.compatibility_class == "behavior_changing_incompatible",
        "removed/retargeted assets or retired diagnostic codes require compatibility_class=behavior_changing_incompatible");

    let required_bump = match record.compatibility_class.as_str() {
        "behavior_changing_incompatible" => "major",
        "additive_compatible" | "behavior_tightening_compatible" => "minor",
        "fixture_only_non_semantic" => {
            ensure!(
                changes.iter().all(|c| c.path.starts_with("fixtures/")),
                "fixture_only_non_semantic compatibility_class may only change fixtures/"
            );
            "patch"
        }
        "documentation_only_normative_clarification" => {
            ensure!(changes.iter().all(|c| c.path.ends_with(".md") && (c.path.starts_with("semantics/") || c.path.starts_with("versioning/"))),
                "documentation_only_normative_clarification compatibility_class may only change semantics/versioning Markdown");
            "patch"
        }
        _ => unreachable!("record class was validated"),
    };
    let major = current_version.major > baseline_version.major;
    let minor = major
        || (current_version.major == baseline_version.major
            && current_version.minor > baseline_version.minor);
    let patch = minor
        || (current_version.major == baseline_version.major
            && current_version.minor == baseline_version.minor
            && current_version.patch > baseline_version.patch);
    let satisfies = match required_bump {
        "major" => major,
        "minor" => minor,
        _ => patch,
    };
    ensure!(satisfies, "{} requires at least a {required_bump} bundle_version bump: {baseline_version} -> {current_version}", record.compatibility_class);
    Ok(())
}

pub(super) fn validate_current_record(manifest: &LoadedManifest) -> Result<()> {
    version(&manifest.data.bundle_version)?;
    if manifest.data.assets.contains_key("bundle_change") {
        validate_record(&read_record(manifest)?, manifest)?;
    }
    Ok(())
}

fn version(value: &str) -> Result<Version> {
    Version::parse(value).with_context(|| format!("bundle version must be valid SemVer: {value}"))
}

fn read_record(manifest: &LoadedManifest) -> Result<BundleChangeRecord> {
    let path = resolve_asset_path(manifest, "bundle_change")?;
    ensure!(
        !manifest
            .data
            .assets
            .iter()
            .any(|(key, value)| key != "bundle_change"
                && manifest
                    .contracts_root
                    .join(value)
                    .canonicalize()
                    .ok()
                    .as_ref()
                    == Some(&path)),
        "bundle_change must not alias another manifest asset"
    );
    serde_yaml::from_str(&fs::read_to_string(&path)?)
        .with_context(|| format!("invalid bundle_change record: {}", path.display()))
}

fn validate_record(record: &BundleChangeRecord, manifest: &LoadedManifest) -> Result<()> {
    version(&record.baseline_version)?;
    version(&record.bundle_version)?;
    ensure!(
        record.bundle_version == manifest.data.bundle_version,
        "bundle_change bundle_version must match manifest"
    );
    ensure!(
        !record.summary.trim().is_empty(),
        "bundle_change summary must not be empty"
    );
    ensure!(
        matches!(
            record.compatibility_class.as_str(),
            "additive_compatible"
                | "behavior_tightening_compatible"
                | "behavior_changing_incompatible"
                | "fixture_only_non_semantic"
                | "documentation_only_normative_clarification"
        ),
        "unsupported bundle_change compatibility_class: {}",
        record.compatibility_class
    );
    if record.compatibility_class == "behavior_changing_incompatible" {
        ensure!(
            record
                .migration_notes
                .as_ref()
                .is_some_and(|s| !s.trim().is_empty()),
            "incompatible bundle_change requires migration_notes"
        );
    }
    ensure!(
        !record.changes.is_empty(),
        "bundle_change changes must not be empty"
    );
    let mut paths = BTreeSet::new();
    for change in &record.changes {
        ensure!(
            !change.path.is_empty()
                && !change.path.contains('\\')
                && change
                    .path
                    .split('/')
                    .all(|part| !part.is_empty() && part != "." && part != "..")
                && Path::new(&change.path)
                    .components()
                    .all(|part| matches!(part, Component::Normal(_))),
            "bundle_change path must be a canonical contract-relative path: {}",
            change.path
        );
        ensure!(
            paths.insert(&change.path),
            "duplicate bundle_change path: {}",
            change.path
        );
        ensure!(
            change.before != change.after,
            "bundle_change must describe a changed asset: {}",
            change.path
        );
        for digest in change.before.iter().chain(change.after.iter()) {
            ensure!(
                digest.strip_prefix("blake3:").is_some_and(|s| s.len() == 64
                    && s.bytes()
                        .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())),
                "invalid bundle_change BLAKE3 digest for {}",
                change.path
            );
        }
    }
    Ok(())
}

fn actual_changes(current: &LoadedManifest, baseline: &LoadedManifest) -> Result<Vec<AssetChange>> {
    let current = snapshot(current)?;
    let baseline = snapshot(baseline)?;
    let paths: BTreeSet<_> = current.keys().chain(baseline.keys()).collect();
    Ok(paths
        .into_iter()
        .filter_map(|path| {
            let before = baseline.get(path);
            let after = current.get(path);
            (before != after).then(|| AssetChange {
                path: path.clone(),
                before: before.cloned(),
                after: after.cloned(),
            })
        })
        .collect())
}

fn snapshot(manifest: &LoadedManifest) -> Result<BTreeMap<String, String>> {
    let record_path = manifest
        .data
        .assets
        .get("bundle_change")
        .map(|_| resolve_asset_path(manifest, "bundle_change"))
        .transpose()?;
    let mut result = BTreeMap::new();
    for (archive_path, source_path) in package_entries(manifest)? {
        if record_path.as_ref() == Some(&source_path) {
            continue;
        }
        let path = archive_path
            .strip_prefix("contracts")?
            .to_string_lossy()
            .replace('\\', "/");
        let bytes = if source_path == manifest.path {
            // Version bookkeeping and the record itself must not make their
            // own content digests recursive. Asset key/path changes do count.
            let assets: BTreeMap<_, _> = manifest
                .data
                .assets
                .iter()
                .filter(|(key, _)| key.as_str() != "bundle_change")
                .collect();
            serde_json::to_vec(
                &serde_json::json!({"assets": assets, "compatibility": {"public_axis": manifest.data.compatibility.public_axis}}),
            )?
        } else {
            fs::read(&source_path)
                .with_context(|| format!("read contract asset {}", source_path.display()))?
        };
        result.insert(path, format!("blake3:{}", blake3::hash(&bytes)));
    }
    Ok(result)
}

fn removed_registry_code(current: &LoadedManifest, baseline: &LoadedManifest) -> Result<bool> {
    if !baseline.data.assets.contains_key("error_registry")
        || !current.data.assets.contains_key("error_registry")
    {
        return Ok(false);
    }
    let baseline = load_registry(resolve_asset_path(baseline, "error_registry")?)?;
    let current = load_registry(resolve_asset_path(current, "error_registry")?)?;
    let current: BTreeMap<_, _> = current
        .codes
        .iter()
        .map(|code| (code.id.as_str(), code.status.as_str()))
        .collect();
    Ok(baseline
        .codes
        .iter()
        .any(|code| match current.get(code.id.as_str()) {
            None => true,
            Some(&"removed") => code.status != "removed",
            _ => false,
        }))
}
