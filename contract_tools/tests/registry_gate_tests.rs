use contract_tools::{
    contract_manifest_path,
    manifest::{load_manifest, resolve_asset_path},
    registry::load_registry,
    semantics::load_semantics_doc,
};

#[test]
fn error_registry_codes_are_unique_and_lifecycle_states_are_known() {
    let manifest = load_manifest(contract_manifest_path()).unwrap();
    let registry = load_registry(resolve_asset_path(&manifest, "error_registry").unwrap()).unwrap();

    assert!(registry.codes.iter().any(|code| code.id == "AF0001"));
    assert!(registry
        .codes
        .iter()
        .all(|code| matches!(code.status.as_str(), "active" | "deprecated" | "removed")));

    let mut ids = registry
        .codes
        .iter()
        .map(|code| code.id.as_str())
        .collect::<Vec<_>>();
    ids.sort_unstable();
    ids.dedup();

    assert_eq!(ids.len(), registry.codes.len());
}

#[test]
fn semantics_docs_declare_asset_references() {
    let manifest = load_manifest(contract_manifest_path()).unwrap();
    let doc = load_semantics_doc(resolve_asset_path(&manifest, "path_semantics").unwrap()).unwrap();

    assert!(doc
        .asset_refs
        .iter()
        .any(|item| item == "schema/diagnostic-item.schema.json"));
}
