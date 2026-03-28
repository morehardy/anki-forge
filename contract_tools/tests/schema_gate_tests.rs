use contract_tools::{
    contract_manifest_path,
    manifest::{load_manifest, resolve_asset_path},
    schema::{load_schema, run_schema_gates, validate_value},
};
use serde_json::json;
use serde_json::Value;
use std::fs;

#[test]
fn authoring_ir_schema_accepts_the_minimal_valid_shape() {
    let manifest = load_manifest(contract_manifest_path()).unwrap();
    let schema =
        load_schema(resolve_asset_path(&manifest, "authoring_ir_schema").unwrap()).unwrap();
    let value = json!({
        "kind": "authoring-ir",
        "schema_version": "0.1.0",
        "metadata": { "document_id": "demo-doc" },
        "notetypes": [],
        "notes": []
    });

    assert!(validate_value(&schema, &value).is_ok());
}

#[test]
fn validation_report_schema_requires_a_diagnostics_array() {
    let manifest = load_manifest(contract_manifest_path()).unwrap();
    let schema =
        load_schema(resolve_asset_path(&manifest, "validation_report_schema").unwrap()).unwrap();
    let value = json!({
        "kind": "validation-report",
        "status": "invalid"
    });

    assert!(validate_value(&schema, &value).is_err());
}

#[test]
fn schema_gates_run_against_the_bundled_contract_manifest() {
    run_schema_gates(contract_manifest_path().to_str().unwrap()).unwrap();
}

#[test]
fn diagnostic_item_schema_matches_the_validation_report_local_definition() {
    let manifest = load_manifest(contract_manifest_path()).unwrap();
    let diagnostic_item_path = resolve_asset_path(&manifest, "diagnostic_item_schema").unwrap();
    let validation_report_path = resolve_asset_path(&manifest, "validation_report_schema").unwrap();

    let standalone = normalized_schema_value(&diagnostic_item_path);
    let validation_report = schema_value(&validation_report_path);
    let local_definition = validation_report
        .get("$defs")
        .and_then(|defs| defs.get("diagnostic_item"))
        .cloned()
        .expect("validation report includes a local diagnostic_item definition");

    assert_eq!(standalone, local_definition);
}

fn schema_value(path: &std::path::Path) -> Value {
    let raw = fs::read_to_string(path).unwrap();
    serde_json::from_str(&raw).unwrap()
}

fn normalized_schema_value(path: &std::path::Path) -> Value {
    let mut value = schema_value(path);
    if let Value::Object(map) = &mut value {
        map.remove("$schema");
    }
    value
}
