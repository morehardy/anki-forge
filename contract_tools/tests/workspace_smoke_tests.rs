#[test]
fn repository_exposes_a_contract_bundle_entrypoint() {
    let manifest_path = contract_tools::contract_manifest_path();

    assert!(manifest_path.is_file());
    assert_eq!(manifest_path.file_name().and_then(|name| name.to_str()), Some("manifest.yaml"));
}
