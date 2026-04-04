use contract_tools::{contract_manifest_path, policies::run_policy_gates};

#[test]
fn default_policy_assets_validate_against_declared_schemas() {
    run_policy_gates(contract_manifest_path()).expect("policy assets should validate");
}
