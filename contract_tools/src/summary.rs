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
    let asset_entries = manifest
        .data
        .assets
        .iter()
        .map(|(name, asset)| format!("  {name}: {asset}"))
        .collect::<Vec<_>>()
        .join("\n");

    Ok(format!(
        "bundle_version: {}\npublic_axis: {}\ncomponent_versions:\n{}\nassets:\n{}",
        manifest.data.bundle_version,
        manifest.data.compatibility.public_axis,
        component_versions,
        asset_entries
    ))
}
