use std::path::Path;

use crate::manifest::load_manifest;

pub fn render(manifest_path: impl AsRef<Path>) -> anyhow::Result<String> {
    let manifest = load_manifest(manifest_path)?;
    let component_versions = manifest
        .data
        .component_versions
        .iter()
        .map(|(name, version)| format!("  {name}: {version}"))
        .collect::<Vec<_>>()
        .join("\n");

    let mut asset_entries = Vec::new();
    for key in [
        "manifest_schema",
        "version_policy",
        "compatibility_classes",
        "upgrade_rules",
        "fixture_catalog",
        "validation_semantics",
        "path_semantics",
        "compatibility_semantics",
    ] {
        if let Some(asset) = manifest.data.assets.get(key) {
            asset_entries.push(format!("  {key}: {asset}"));
        }
    }

    Ok(format!(
        "bundle_version: {}\npublic_axis: {}\ncomponent_versions:\n{}\nassets:\n{}",
        manifest.data.bundle_version,
        manifest.data.compatibility.public_axis,
        component_versions,
        asset_entries.join("\n")
    ))
}
