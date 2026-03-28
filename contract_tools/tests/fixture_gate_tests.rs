use contract_tools::{
    contract_manifest_path,
    fixtures::load_fixture_catalog,
    manifest::{load_manifest, resolve_asset_path, resolve_contract_relative_path},
};

#[test]
fn fixture_catalog_contains_normative_cases_and_safe_paths() {
    let manifest = load_manifest(contract_manifest_path()).expect("manifest loads");
    let catalog_path = resolve_asset_path(&manifest, "fixture_catalog")
        .expect("fixture catalog asset should be declared");
    let catalog = load_fixture_catalog(&catalog_path).expect("fixture catalog loads");

    assert!(!catalog.cases.is_empty(), "catalog must not be empty");

    let missing_document_id = catalog
        .cases
        .iter()
        .find(|case| case.id == "missing-document-id")
        .expect("missing-document-id case should exist");
    assert_eq!(
        missing_document_id.expected.as_deref(),
        Some("fixtures/expected/missing-document-id.report.json")
    );

    for case_id in [
        "minimal-authoring-ir",
        "minimal-service-envelope",
        "additive-compatible",
        "incompatible-path-change",
    ] {
        assert!(
            catalog.cases.iter().any(|case| case.id == case_id),
            "catalog should include {case_id}"
        );
    }

    let evolution_cases: Vec<_> = catalog
        .cases
        .iter()
        .filter(|case| case.category == "evolution")
        .collect();
    assert!(
        evolution_cases
            .iter()
            .any(|case| case.compatibility_class.as_deref() == Some("additive_compatible")),
        "catalog must include a compatible evolution case"
    );
    assert!(
        evolution_cases.iter().any(|case| {
            case.compatibility_class.as_deref() == Some("behavior_changing_incompatible")
        }),
        "catalog must include an incompatible evolution case"
    );

    for case in &catalog.cases {
        let input_path = resolve_contract_relative_path(&manifest.contracts_root, &case.input)
            .unwrap_or_else(|error| panic!("case {} input should resolve: {error}", case.id));
        assert!(input_path.is_file(), "case {} input must exist", case.id);

        if let Some(expected) = &case.expected {
            let expected_path = resolve_contract_relative_path(&manifest.contracts_root, expected)
                .unwrap_or_else(|error| panic!("case {} expected should resolve: {error}", case.id));
            assert!(expected_path.is_file(), "case {} expected must exist", case.id);
        } else if case.category == "invalid" {
            panic!("invalid case {} must declare an expected report", case.id);
        }
    }
}
