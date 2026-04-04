#[test]
fn repository_exposes_a_contract_bundle_entrypoint() {
    let manifest_path = contract_tools::contract_manifest_path();

    assert!(manifest_path.is_file());
    assert_eq!(
        manifest_path.file_name().and_then(|name| name.to_str()),
        Some("manifest.yaml")
    );
}

#[test]
fn workspace_exposes_authoring_core_contract_version() {
    assert_eq!(authoring_core::tool_contract_version(), "phase2-v1");
}
