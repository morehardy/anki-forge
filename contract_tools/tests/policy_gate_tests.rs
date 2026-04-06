use contract_tools::{
    contract_manifest_path,
    manifest::{load_manifest, resolve_asset_path},
    policies::run_policy_gates,
    schema::{load_schema, validate_value},
};
use serde_json::Value;
use std::fs;

#[test]
fn default_policy_assets_validate_against_declared_schemas() {
    run_policy_gates(contract_manifest_path()).expect("policy assets should validate");
}

#[test]
fn phase3_policy_assets_validate_against_declared_schemas() {
    let manifest = load_manifest(contract_manifest_path()).unwrap();

    for (schema_key, asset_key) in [
        ("writer_policy_schema", "writer_policy"),
        ("verification_policy_schema", "verification_policy"),
        ("build_context_schema", "build_context_default"),
    ] {
        let schema = load_schema(resolve_asset_path(&manifest, schema_key).unwrap()).unwrap();
        let value = load_yaml_value(resolve_asset_path(&manifest, asset_key).unwrap());

        validate_value(&schema, &value).expect("policy asset should validate against schema");
    }
}

fn load_yaml_value(path: impl AsRef<std::path::Path>) -> Value {
    let raw = fs::read_to_string(path.as_ref()).unwrap();
    let yaml_value: serde_yaml::Value = serde_yaml::from_str(&raw).unwrap();
    serde_json::to_value(yaml_value).unwrap()
}
