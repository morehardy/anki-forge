use std::path::Path;

use crate::manifest::load_manifest;

pub fn render(manifest_path: impl AsRef<Path>) -> anyhow::Result<String> {
    let manifest = load_manifest(manifest_path)?;
    let writer = crate::policies::load_writer_policy_asset(&manifest, "default")?;
    let context = crate::policies::load_build_context_asset(&manifest, "default")?;
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
        "bundle_version: {}\npublic_axis: {}\nauthoring_protocol: {}\nwriter_protocol: {}\nwriter_policy_ref: {}\nbuild_context_ref: {}\ncomponent_versions:\n{}\nassets:\n{}",
        manifest.data.bundle_version,
        manifest.data.compatibility.public_axis,
        ankiforge::tools::authoring_contract_version(),
        ankiforge::tools::writer_contract_version(),
        ankiforge::tools::policy_ref(&writer.id, &writer.version),
        ankiforge::tools::build_context_ref(&context)?,
        component_versions,
        asset_entries
    ))
}
