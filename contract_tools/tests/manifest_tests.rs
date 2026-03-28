use contract_tools::{
    contract_manifest_path,
    manifest::{load_manifest, resolve_asset_path, resolve_contract_relative_path},
};

#[test]
fn manifest_uses_bundle_version_as_the_only_public_axis() {
    let manifest = load_manifest(contract_manifest_path()).expect("manifest loads");
    assert_eq!(manifest.data.bundle_version, "0.1.0");
    assert_eq!(manifest.data.compatibility.public_axis, "bundle_version");
    assert!(manifest.data.component_versions.contains_key("schema"));
}

#[test]
fn manifest_resolves_asset_paths_relative_to_manifest_directory() {
    let manifest = load_manifest(contract_manifest_path()).expect("manifest loads");
    let schema_path = resolve_asset_path(&manifest, "manifest_schema").unwrap();
    assert_eq!(
        schema_path,
        manifest.contracts_root.join("schema/manifest.schema.json")
    );
}

#[test]
fn relative_contract_paths_reject_absolute_and_escape_attempts() {
    let manifest = load_manifest(contract_manifest_path()).expect("manifest loads");

    let absolute_err =
        resolve_contract_relative_path(&manifest.contracts_root, "/tmp/evil").unwrap_err();
    assert!(absolute_err
        .to_string()
        .contains("asset path must be relative"));

    let escape_err =
        resolve_contract_relative_path(&manifest.contracts_root, "../evil").unwrap_err();
    assert!(escape_err
        .to_string()
        .contains("asset path must not escape contracts/"));
}
