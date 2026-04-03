#[test]
fn workspace_exposes_authoring_core_contract_version() {
    assert_eq!(authoring_core::tool_contract_version(), "phase2-v1");
}
