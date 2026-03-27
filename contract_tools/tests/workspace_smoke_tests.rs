#[test]
fn repository_exposes_a_contract_bundle_entrypoint() {
    assert!(contract_tools::contract_manifest_path().exists());
}
