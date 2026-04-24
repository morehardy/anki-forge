use contract_tools::{
    contract_manifest_path,
    fixtures::{load_fixture_catalog, run_fixture_gates},
    manifest::{load_manifest, resolve_asset_path},
};
use serde_json::json;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

#[test]
fn fixture_gates_accept_the_bundled_catalog_and_fixtures() {
    run_fixture_gates(copied_bundled_manifest_path("bundled-fixture-gates"))
        .expect("bundled fixture gate should pass");
}

#[test]
fn bundled_catalog_declares_phase3_writer_cases() {
    let manifest = load_manifest(contract_manifest_path()).expect("load bundled manifest");
    let catalog_path = resolve_asset_path(&manifest, "fixture_catalog").expect("fixture catalog");
    let catalog = load_fixture_catalog(&catalog_path).expect("load bundled fixture catalog");

    for case_id in [
        "phase3-writer-basic-minimal",
        "phase3-writer-cloze-minimal",
        "phase3-writer-image-occlusion-minimal",
    ] {
        assert!(
            catalog
                .cases
                .iter()
                .any(|case| case.id == case_id && case.category == "phase3-writer"),
            "expected bundled catalog to declare phase3 writer case {case_id}"
        );
    }
}

#[test]
fn bundled_catalog_declares_phase3_e2e_cases() {
    let manifest = load_manifest(contract_manifest_path()).expect("load bundled manifest");
    let catalog_path = resolve_asset_path(&manifest, "fixture_catalog").expect("fixture catalog");
    let catalog = load_fixture_catalog(&catalog_path).expect("load bundled fixture catalog");

    for case_id in [
        "phase3-e2e-basic-minimal",
        "phase3-e2e-cloze-minimal",
        "phase3-e2e-image-occlusion-minimal",
    ] {
        assert!(
            catalog
                .cases
                .iter()
                .any(|case| case.id == case_id && case.category == "phase3-e2e"),
            "expected bundled catalog to declare phase3 e2e case {case_id}"
        );
    }
}

#[test]
fn fixture_gates_reject_unknown_error_codes_in_expected_reports() {
    let manifest = load_manifest(write_bundle(
        &registry_yaml(),
        &catalog_yaml(),
        &expected_report_json("AF9999"),
        &additive_evolution_yaml("additive_compatible", "minor"),
        &incompatible_evolution_yaml("behavior_changing_incompatible", "major"),
    ))
    .expect("temp manifest loads");

    let err = run_fixture_gates(&manifest.path).expect_err("unknown diagnostic codes should fail");
    assert!(err
        .to_string()
        .contains("diagnostic code must exist in registry"));
}

#[test]
fn fixture_gates_reject_evolution_metadata_drift() {
    let manifest = load_manifest(write_bundle(
        &registry_yaml(),
        &catalog_yaml_with_drift(),
        &expected_report_json("AF0001"),
        &additive_evolution_yaml("behavior_changing_incompatible", "major"),
        &incompatible_evolution_yaml("behavior_changing_incompatible", "major"),
    ))
    .expect("temp manifest loads");

    let err = run_fixture_gates(&manifest.path).expect_err("evolution drift should fail");
    assert!(err
        .to_string()
        .contains("evolution fixture metadata must match catalog"));
}

#[test]
fn fixture_gates_execute_phase2_cases_and_compare_expected_output() {
    let manifest = load_manifest(write_bundle(
        &registry_yaml(),
        &catalog_yaml_with_phase2(),
        &expected_report_json("AF0001"),
        &additive_evolution_yaml("additive_compatible", "minor"),
        &incompatible_evolution_yaml("behavior_changing_incompatible", "major"),
    ))
    .expect("temp manifest loads");

    run_fixture_gates(&manifest.path).expect("phase2 executable fixtures should pass");
}

#[test]
fn fixture_gates_reject_phase2_expected_output_mismatch() {
    let manifest = load_manifest(write_bundle(
        &registry_yaml(),
        &catalog_yaml_with_phase2(),
        &expected_report_json("AF0001"),
        &additive_evolution_yaml("additive_compatible", "minor"),
        &incompatible_evolution_yaml("behavior_changing_incompatible", "major"),
    ))
    .expect("temp manifest loads");

    let bundle_root = manifest
        .path
        .parent()
        .expect("manifest parent")
        .to_path_buf();
    fs::write(
        bundle_root.join("fixtures/phase2/expected/minimal-success.result.json"),
        phase2_minimal_success_result_mismatch_json(),
    )
    .expect("overwrite phase2 expected result");

    let err = run_fixture_gates(&manifest.path).expect_err("phase2 mismatches should fail");
    assert!(err
        .to_string()
        .contains("phase2 normalization output mismatch"));
}

#[test]
fn fixture_gates_execute_phase3_writer_and_e2e_cases() {
    let manifest_path = copied_bundled_manifest_path("phase3-fixture-gates");

    run_fixture_gates(&manifest_path).expect("phase3 executable fixtures should pass");
}

#[test]
fn fixture_gates_reject_phase3_inspect_golden_mismatch() {
    let manifest_path = copied_bundled_manifest_path("phase3-fixture-mismatch");
    let bundle_root = manifest_path
        .parent()
        .expect("manifest parent")
        .to_path_buf();
    let expected_path = bundle_root.join("fixtures/phase3/expected/basic.inspect.json");
    let mut expected: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&expected_path).expect("read phase3 inspect golden"),
    )
    .expect("decode phase3 inspect golden");
    expected["observations"]["metadata"][0]["card_count"] = serde_json::json!(999);

    fs::write(
        &expected_path,
        serde_json::to_string_pretty(&expected).expect("encode corrupted inspect golden"),
    )
    .expect("overwrite phase3 inspect golden");

    let err = run_fixture_gates(&manifest_path).expect_err("phase3 mismatches should fail");
    assert!(err.to_string().contains("phase3 inspect output mismatch"));
}

#[test]
fn fixture_gates_reject_note_identity_stable_id_hash_mismatch() {
    let manifest_path = copied_bundled_manifest_path("note-identity-hash-mismatch");
    let bundle_root = manifest_path
        .parent()
        .expect("manifest parent")
        .to_path_buf();
    let fixture_path = bundle_root.join("fixtures/note-identity/basic-front-only.case.json");
    let mut fixture: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&fixture_path).expect("read note fixture"))
            .expect("decode note fixture");
    fixture["expected"]["stable_id"] = serde_json::json!(
        "afid:v1:0000000000000000000000000000000000000000000000000000000000000000"
    );

    fs::write(
        &fixture_path,
        serde_json::to_string_pretty(&fixture).expect("encode corrupted note fixture"),
    )
    .expect("overwrite note fixture");

    let err = run_fixture_gates(&manifest_path).expect_err("hash mismatches should fail");
    assert!(err
        .to_string()
        .contains("note-identity stable_id must match canonical_payload hash"));
}

#[test]
fn fixture_gates_reject_note_identity_recipe_input_schema_mismatch() {
    let manifest_path = copied_bundled_manifest_path("note-identity-schema-mismatch");
    let bundle_root = manifest_path
        .parent()
        .expect("manifest parent")
        .to_path_buf();
    let fixture_path = bundle_root.join("fixtures/note-identity/basic-front-only.case.json");
    let mut fixture: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&fixture_path).expect("read note fixture"))
            .expect("decode note fixture");
    fixture["note_kind"] = serde_json::json!("cloze");
    fixture["input"] = serde_json::json!({
        "frnot": "hola",
        "back": "hello"
    });

    fs::write(
        &fixture_path,
        serde_json::to_string_pretty(&fixture).expect("encode corrupted note fixture"),
    )
    .expect("overwrite note fixture");

    let err = run_fixture_gates(&manifest_path).expect_err("schema mismatches should fail");
    assert!(err
        .to_string()
        .contains("note-identity fixture must satisfy note_identity_fixture_schema"));
}

#[test]
fn fixture_gates_reject_note_identity_noncanonical_payload_string() {
    let manifest_path = copied_bundled_manifest_path("note-identity-noncanonical-payload");
    let bundle_root = manifest_path
        .parent()
        .expect("manifest parent")
        .to_path_buf();
    let fixture_path = bundle_root.join("fixtures/note-identity/basic-front-only.case.json");
    let mut fixture: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&fixture_path).expect("read note fixture"))
            .expect("decode note fixture");
    fixture["expected"]["canonical_payload"] = serde_json::json!(
        "{\"recipe_id\":\"basic.core.v1\",\"algo_version\":1,\"notetype_family\":\"stock\",\"notetype_key\":\"basic\",\"components\":{\"selected_fields\":[{\"value\":\"hola\",\"name\":\"front\"}]}}"
    );

    fs::write(
        &fixture_path,
        serde_json::to_string_pretty(&fixture).expect("encode corrupted note fixture"),
    )
    .expect("overwrite note fixture");

    let err = run_fixture_gates(&manifest_path).expect_err("noncanonical payloads should fail");
    assert!(err
        .to_string()
        .contains("note-identity canonical_payload must be canonical JSON"));
}

fn temp_contract_root(label: &str) -> PathBuf {
    static NEXT_TEMP_ROOT_ID: AtomicU64 = AtomicU64::new(0);
    let unique = NEXT_TEMP_ROOT_ID.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "anki-forge-contract-tools-{}-{}-{}",
        label,
        std::process::id(),
        unique
    ))
}

fn copy_tree(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).expect("create destination tree");
    for entry in fs::read_dir(src).expect("read source tree") {
        let entry = entry.expect("read source entry");
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_tree(&src_path, &dst_path);
        } else {
            fs::copy(&src_path, &dst_path).expect("copy source file");
        }
    }
}

fn copied_bundled_manifest_path(label: &str) -> PathBuf {
    let root = temp_contract_root(label);
    copy_tree(
        contract_manifest_path()
            .parent()
            .expect("contracts root for bundled manifest"),
        &root,
    );

    let artifacts_root = root.join("artifacts");
    if artifacts_root.exists() {
        fs::remove_dir_all(&artifacts_root).expect("remove generated artifact tree");
    }

    root.join("manifest.yaml")
}

fn write_bundle(
    registry_yaml: &str,
    catalog_yaml: &str,
    expected_report_json: &str,
    additive_evolution_yaml: &str,
    incompatible_evolution_yaml: &str,
) -> PathBuf {
    let root = temp_contract_root("fixture-gates");
    fs::create_dir_all(root.join("schema")).expect("create schema dir");
    fs::create_dir_all(root.join("versioning")).expect("create versioning dir");
    fs::create_dir_all(root.join("errors")).expect("create errors dir");
    fs::create_dir_all(root.join("semantics")).expect("create semantics dir");
    fs::create_dir_all(root.join("fixtures/valid")).expect("create valid fixtures dir");
    fs::create_dir_all(root.join("fixtures/invalid")).expect("create invalid fixtures dir");
    fs::create_dir_all(root.join("fixtures/expected")).expect("create expected fixtures dir");
    fs::create_dir_all(root.join("fixtures/service-envelope"))
        .expect("create envelope fixtures dir");
    fs::create_dir_all(root.join("fixtures/evolution")).expect("create evolution fixtures dir");
    fs::create_dir_all(root.join("fixtures/phase2/normalization"))
        .expect("create phase2 normalization fixtures dir");
    fs::create_dir_all(root.join("fixtures/phase2/risk")).expect("create phase2 risk fixtures dir");
    fs::create_dir_all(root.join("fixtures/phase2/inputs"))
        .expect("create phase2 input fixtures dir");
    fs::create_dir_all(root.join("fixtures/phase2/expected"))
        .expect("create phase2 expected fixtures dir");

    fs::write(root.join("manifest.yaml"), manifest_yaml()).expect("write manifest");
    fs::write(root.join("schema/manifest.schema.json"), manifest_schema())
        .expect("write manifest schema");
    fs::write(
        root.join("schema/diagnostic-item.schema.json"),
        diagnostic_item_schema(),
    )
    .expect("write diagnostic item schema");
    fs::write(
        root.join("schema/authoring-ir.schema.json"),
        authoring_ir_schema(),
    )
    .expect("write authoring schema");
    fs::write(
        root.join("schema/service-envelope.schema.json"),
        service_envelope_schema(),
    )
    .expect("write envelope schema");
    fs::write(
        root.join("schema/validation-report.schema.json"),
        validation_report_schema(),
    )
    .expect("write validation report schema");
    fs::write(
        root.join("schema/error-registry.schema.json"),
        error_registry_schema(),
    )
    .expect("write error registry schema");
    fs::write(root.join("versioning/policy.md"), "# policy\n").expect("write policy");
    fs::write(
        root.join("versioning/compatibility-classes.yaml"),
        compatibility_classes_yaml(),
    )
    .expect("write compatibility classes");
    fs::write(
        root.join("versioning/upgrade-rules.yaml"),
        upgrade_rules_yaml(),
    )
    .expect("write upgrade rules");
    fs::write(root.join("errors/error-registry.yaml"), registry_yaml).expect("write registry");
    fs::write(
        root.join("semantics/validation.md"),
        "---\nasset_refs:\n  - schema/diagnostic-item.schema.json\n---\n# Validation\n",
    )
    .expect("write validation semantics");
    fs::write(
        root.join("semantics/path-conventions.md"),
        "---\nasset_refs:\n  - schema/diagnostic-item.schema.json\n---\n# Path Conventions\n",
    )
    .expect("write path semantics");
    fs::write(
        root.join("semantics/compatibility.md"),
        "---\nasset_refs:\n  - versioning/compatibility-classes.yaml\n---\n# Compatibility\n",
    )
    .expect("write compatibility semantics");
    fs::write(root.join("fixtures/index.yaml"), catalog_yaml).expect("write catalog");
    fs::write(
        root.join("fixtures/valid/minimal-authoring-ir.json"),
        minimal_authoring_ir_json(),
    )
    .expect("write authoring fixture");
    fs::write(
        root.join("fixtures/invalid/missing-document-id.json"),
        missing_document_id_json(),
    )
    .expect("write invalid fixture");
    fs::write(
        root.join("fixtures/expected/missing-document-id.report.json"),
        expected_report_json,
    )
    .expect("write expected report");
    fs::write(
        root.join("fixtures/service-envelope/minimal-success.json"),
        minimal_service_envelope_json(),
    )
    .expect("write envelope fixture");
    fs::write(
        root.join("fixtures/evolution/additive-compatible.yaml"),
        additive_evolution_yaml,
    )
    .expect("write additive evolution fixture");
    fs::write(
        root.join("fixtures/evolution/incompatible-path-change.yaml"),
        incompatible_evolution_yaml,
    )
    .expect("write incompatible evolution fixture");
    fs::write(
        root.join("fixtures/phase2/normalization/minimal-success.case.yaml"),
        phase2_minimal_success_case_yaml(),
    )
    .expect("write phase2 minimal success case");
    fs::write(
        root.join("fixtures/phase2/risk/complete-low.case.yaml"),
        phase2_complete_low_case_yaml(),
    )
    .expect("write phase2 complete low case");
    fs::write(
        root.join("fixtures/phase2/normalization/identity-random-warning.case.yaml"),
        phase2_identity_random_warning_case_yaml(),
    )
    .expect("write phase2 random warning case");
    fs::write(
        root.join("fixtures/phase2/risk/partial-high.case.yaml"),
        phase2_partial_high_case_yaml(),
    )
    .expect("write phase2 partial risk case");
    fs::write(
        root.join("fixtures/phase2/inputs/minimal-authoring-ir.json"),
        phase2_minimal_authoring_ir_json(),
    )
    .expect("write phase2 input fixture");
    fs::write(
        root.join("fixtures/phase2/expected/minimal-success.result.json"),
        phase2_minimal_success_result_json(),
    )
    .expect("write phase2 expected normalization result");
    fs::write(
        root.join("fixtures/phase2/expected/complete-low.risk.json"),
        phase2_complete_low_risk_json(),
    )
    .expect("write phase2 expected risk report");

    root.join("manifest.yaml")
}

fn manifest_yaml() -> String {
    r#"bundle_version: "0.1.0"
component_versions:
  schema: "0.1.0"
  fixtures: "0.1.0"
  service_envelope: "0.1.0"
  error_registry: "0.1.0"
compatibility:
  public_axis: bundle_version
assets:
  manifest_schema: schema/manifest.schema.json
  version_policy: versioning/policy.md
  compatibility_classes: versioning/compatibility-classes.yaml
  upgrade_rules: versioning/upgrade-rules.yaml
  authoring_ir_schema: schema/authoring-ir.schema.json
  diagnostic_item_schema: schema/diagnostic-item.schema.json
  validation_report_schema: schema/validation-report.schema.json
  service_envelope_schema: schema/service-envelope.schema.json
  error_registry_schema: schema/error-registry.schema.json
  error_registry: errors/error-registry.yaml
  fixture_catalog: fixtures/index.yaml
  validation_semantics: semantics/validation.md
  path_semantics: semantics/path-conventions.md
  compatibility_semantics: semantics/compatibility.md
"#
    .to_string()
}

fn manifest_schema() -> String {
    let schema = json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "required": ["bundle_version", "component_versions", "compatibility", "assets"],
        "additionalProperties": false,
        "properties": {
            "bundle_version": { "type": "string", "minLength": 1 },
            "component_versions": {
                "type": "object",
                "required": ["schema", "fixtures", "service_envelope", "error_registry"],
                "additionalProperties": false,
                "properties": {
                    "schema": { "type": "string" },
                    "fixtures": { "type": "string" },
                    "service_envelope": { "type": "string" },
                    "error_registry": { "type": "string" }
                }
            },
            "compatibility": {
                "type": "object",
                "required": ["public_axis"],
                "additionalProperties": false,
                "properties": {
                    "public_axis": { "const": "bundle_version" }
                }
            },
            "assets": {
                "type": "object",
                "required": [
                    "manifest_schema",
                    "version_policy",
                    "compatibility_classes",
                    "upgrade_rules",
                    "authoring_ir_schema",
                    "diagnostic_item_schema",
                    "validation_report_schema",
                    "service_envelope_schema",
                    "error_registry_schema",
                    "error_registry",
                    "fixture_catalog",
                    "validation_semantics",
                    "path_semantics",
                    "compatibility_semantics"
                ],
                "additionalProperties": false,
                "properties": {
                    "manifest_schema": { "type": "string" },
                    "version_policy": { "type": "string" },
                    "compatibility_classes": { "type": "string" },
                    "upgrade_rules": { "type": "string" },
                    "authoring_ir_schema": { "type": "string" },
                    "diagnostic_item_schema": { "type": "string" },
                    "validation_report_schema": { "type": "string" },
                    "service_envelope_schema": { "type": "string" },
                    "error_registry_schema": { "type": "string" },
                    "error_registry": { "type": "string" },
                    "fixture_catalog": { "type": "string" },
                    "validation_semantics": { "type": "string" },
                    "path_semantics": { "type": "string" },
                    "compatibility_semantics": { "type": "string" }
                }
            }
        }
    });

    serde_json::to_string_pretty(&schema).expect("serialize manifest schema")
}

fn authoring_ir_schema() -> String {
    r#"{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["kind", "schema_version", "metadata", "notetypes", "notes"],
  "additionalProperties": false,
  "properties": {
    "kind": { "const": "authoring-ir" },
    "schema_version": { "type": "string", "minLength": 1 },
    "metadata": {
      "type": "object",
      "required": ["document_id"],
      "additionalProperties": false,
      "properties": {
        "document_id": { "type": "string", "minLength": 1 }
      }
    },
    "notetypes": { "type": "array" },
    "notes": { "type": "array" }
  }
}
"#
    .to_string()
}

fn diagnostic_item_schema() -> String {
    r#"{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["level", "code", "path", "message"],
  "additionalProperties": false,
  "properties": {
    "level": { "enum": ["warning", "error"] },
    "code": { "type": "string", "minLength": 1 },
    "path": { "type": "string", "minLength": 1 },
    "message": { "type": "string", "minLength": 1 }
  }
}
"#
    .to_string()
}

fn service_envelope_schema() -> String {
    r#"{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["kind", "request_id", "status"],
  "additionalProperties": false,
  "properties": {
    "kind": { "const": "service-envelope" },
    "request_id": { "type": "string", "minLength": 1 },
    "status": { "enum": ["ok", "error"] }
  }
}
"#
    .to_string()
}

fn validation_report_schema() -> String {
    let schema = json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "$defs": {
            "diagnostic_item": {
                "type": "object",
                "required": ["level", "code", "path", "message"],
                "additionalProperties": false,
                "properties": {
                    "level": { "enum": ["warning", "error"] },
                    "code": { "type": "string", "minLength": 1 },
                    "path": { "type": "string", "minLength": 1 },
                    "message": { "type": "string", "minLength": 1 }
                }
            }
        },
        "required": ["kind", "status", "diagnostics"],
        "additionalProperties": false,
        "properties": {
            "kind": { "const": "validation-report" },
            "status": { "enum": ["valid", "invalid"] },
            "diagnostics": {
                "type": "array",
                "items": { "$ref": "#/$defs/diagnostic_item" }
            }
        }
    });

    serde_json::to_string_pretty(&schema).expect("serialize validation report schema")
}

fn error_registry_schema() -> String {
    let schema = json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "required": ["codes"],
        "additionalProperties": false,
        "properties": {
            "codes": {
                "type": "array",
                "items": {
                    "type": "object",
                    "required": ["id", "status", "summary"],
                    "additionalProperties": false,
                    "properties": {
                        "id": { "type": "string", "minLength": 1 },
                        "status": { "enum": ["active", "deprecated", "removed"] },
                        "summary": { "type": "string", "minLength": 1 }
                    }
                }
            }
        }
    });

    serde_json::to_string_pretty(&schema).expect("serialize registry schema")
}

fn registry_yaml() -> String {
    r#"codes:
  - id: AF0001
    status: active
    summary: document_id is required
  - id: AF0002
    status: active
    summary: diagnostics array is required
  - id: AF0003
    status: active
    summary: manifest self-validation failed
  - id: AF0004
    status: active
    summary: contract-relative path escaped the bundle
  - id: AF0005
    status: active
    summary: semantics frontmatter is missing asset references
"#
    .to_string()
}

fn catalog_yaml() -> String {
    r#"cases:
  - id: minimal-authoring-ir
    category: valid
    input: fixtures/valid/minimal-authoring-ir.json
  - id: missing-document-id
    category: invalid
    input: fixtures/invalid/missing-document-id.json
    expected: fixtures/expected/missing-document-id.report.json
  - id: minimal-service-envelope
    category: service-envelope
    input: fixtures/service-envelope/minimal-success.json
  - id: additive-compatible
    category: evolution
    compatibility_class: additive_compatible
    upgrade_rules:
      - fixture_updates_required
    target_asset: fixtures/index.yaml
    affected_paths:
      - fixtures/valid/minimal-authoring-ir.json
    expected_bundle_bump: minor
    input: fixtures/evolution/additive-compatible.yaml
  - id: incompatible-path-change
    category: evolution
    compatibility_class: behavior_changing_incompatible
    upgrade_rules:
      - migration_notes_required
      - executable_checks_required
    target_asset: fixtures/index.yaml
    affected_paths:
      - fixtures/invalid/missing-document-id.json
      - fixtures/expected/missing-document-id.report.json
    expected_bundle_bump: major
    input: fixtures/evolution/incompatible-path-change.yaml
  - id: phase2-normalization-minimal-success
    category: phase2-normalization
    compatibility_class: additive_compatible
    upgrade_rules:
      - fixture_updates_required
    input: fixtures/phase2/normalization/minimal-success.case.yaml
  - id: phase2-normalization-identity-random-warning
    category: phase2-normalization
    compatibility_class: additive_compatible
    upgrade_rules:
      - fixture_updates_required
    input: fixtures/phase2/normalization/identity-random-warning.case.yaml
  - id: phase2-risk-complete-low
    category: phase2-risk
    compatibility_class: additive_compatible
    upgrade_rules:
      - fixture_updates_required
    input: fixtures/phase2/risk/complete-low.case.yaml
  - id: phase2-risk-partial-high
    category: phase2-risk
    compatibility_class: additive_compatible
    upgrade_rules:
      - fixture_updates_required
    input: fixtures/phase2/risk/partial-high.case.yaml
"#
    .to_string()
}

fn catalog_yaml_with_drift() -> String {
    r#"cases:
  - id: minimal-authoring-ir
    category: valid
    input: fixtures/valid/minimal-authoring-ir.json
  - id: missing-document-id
    category: invalid
    input: fixtures/invalid/missing-document-id.json
    expected: fixtures/expected/missing-document-id.report.json
  - id: minimal-service-envelope
    category: service-envelope
    input: fixtures/service-envelope/minimal-success.json
  - id: additive-compatible
    category: evolution
    compatibility_class: additive_compatible
    upgrade_rules:
      - fixture_updates_required
    target_asset: fixtures/index.yaml
    affected_paths:
      - fixtures/valid/minimal-authoring-ir.json
    expected_bundle_bump: minor
    input: fixtures/evolution/additive-compatible.yaml
  - id: incompatible-path-change
    category: evolution
    compatibility_class: behavior_changing_incompatible
    upgrade_rules:
      - migration_notes_required
      - executable_checks_required
    target_asset: fixtures/index.yaml
    affected_paths:
      - fixtures/invalid/missing-document-id.json
      - fixtures/expected/missing-document-id.report.json
    expected_bundle_bump: major
    input: fixtures/evolution/incompatible-path-change.yaml
  - id: phase2-normalization-minimal-success
    category: phase2-normalization
    compatibility_class: additive_compatible
    upgrade_rules:
      - fixture_updates_required
    input: fixtures/phase2/normalization/minimal-success.case.yaml
  - id: phase2-normalization-identity-random-warning
    category: phase2-normalization
    compatibility_class: additive_compatible
    upgrade_rules:
      - fixture_updates_required
    input: fixtures/phase2/normalization/identity-random-warning.case.yaml
  - id: phase2-risk-complete-low
    category: phase2-risk
    compatibility_class: additive_compatible
    upgrade_rules:
      - fixture_updates_required
    input: fixtures/phase2/risk/complete-low.case.yaml
  - id: phase2-risk-partial-high
    category: phase2-risk
    compatibility_class: additive_compatible
    upgrade_rules:
      - fixture_updates_required
    input: fixtures/phase2/risk/partial-high.case.yaml
"#
    .to_string()
}

fn catalog_yaml_with_phase2() -> String {
    r#"cases:
  - id: minimal-authoring-ir
    category: valid
    input: fixtures/valid/minimal-authoring-ir.json
  - id: missing-document-id
    category: invalid
    input: fixtures/invalid/missing-document-id.json
    expected: fixtures/expected/missing-document-id.report.json
  - id: minimal-service-envelope
    category: service-envelope
    input: fixtures/service-envelope/minimal-success.json
  - id: additive-compatible
    category: evolution
    compatibility_class: additive_compatible
    upgrade_rules:
      - fixture_updates_required
    target_asset: fixtures/index.yaml
    affected_paths:
      - fixtures/valid/minimal-authoring-ir.json
    expected_bundle_bump: minor
    input: fixtures/evolution/additive-compatible.yaml
  - id: incompatible-path-change
    category: evolution
    compatibility_class: behavior_changing_incompatible
    upgrade_rules:
      - migration_notes_required
      - executable_checks_required
    target_asset: fixtures/index.yaml
    affected_paths:
      - fixtures/invalid/missing-document-id.json
      - fixtures/expected/missing-document-id.report.json
    expected_bundle_bump: major
    input: fixtures/evolution/incompatible-path-change.yaml
  - id: phase2-normalization-minimal-success
    category: phase2-normalization
    compatibility_class: additive_compatible
    upgrade_rules:
      - fixture_updates_required
    input: fixtures/phase2/normalization/minimal-success.case.yaml
  - id: phase2-risk-complete-low
    category: phase2-risk
    compatibility_class: additive_compatible
    upgrade_rules:
      - fixture_updates_required
    input: fixtures/phase2/risk/complete-low.case.yaml
  - id: phase2-normalization-identity-random-warning
    category: phase2-normalization
    compatibility_class: additive_compatible
    upgrade_rules:
      - fixture_updates_required
    input: fixtures/phase2/normalization/identity-random-warning.case.yaml
  - id: phase2-risk-partial-high
    category: phase2-risk
    compatibility_class: additive_compatible
    upgrade_rules:
      - fixture_updates_required
    input: fixtures/phase2/risk/partial-high.case.yaml
"#
    .to_string()
}

fn expected_report_json(code: &str) -> String {
    let report = json!({
        "kind": "validation-report",
        "status": "invalid",
        "diagnostics": [
            {
                "level": "error",
                "code": code,
                "path": "/metadata/document_id",
                "message": "document_id is required"
            }
        ]
    });

    serde_json::to_string_pretty(&report).expect("serialize validation report")
}

fn minimal_authoring_ir_json() -> String {
    r#"{
  "kind": "authoring-ir",
  "schema_version": "0.1.0",
  "metadata": {
    "document_id": "demo-doc"
  },
  "notetypes": [],
  "notes": []
}
"#
    .to_string()
}

fn missing_document_id_json() -> String {
    r#"{
  "kind": "authoring-ir",
  "schema_version": "0.1.0",
  "metadata": {},
  "notetypes": [],
  "notes": []
}
"#
    .to_string()
}

fn minimal_service_envelope_json() -> String {
    r#"{
  "kind": "service-envelope",
  "request_id": "req-minimal-001",
  "status": "ok"
}
"#
    .to_string()
}

fn phase2_minimal_success_case_yaml() -> String {
    r#"kind: phase2-normalization-case
authoring_input: fixtures/phase2/inputs/minimal-authoring-ir.json
comparison_context: null
expected_result: fixtures/phase2/expected/minimal-success.result.json
"#
    .to_string()
}

fn phase2_complete_low_case_yaml() -> String {
    r#"kind: phase2-risk-case
authoring_input: fixtures/phase2/inputs/minimal-authoring-ir.json
comparison_context:
  kind: comparison-context
  baseline_kind: normalized_ir
  baseline_artifact_fingerprint: baseline-minimal-success
  risk_policy_ref: risk-policy.default@1.0.0
  comparison_mode: strict
expected_result: fixtures/phase2/expected/complete-low.risk.json
"#
    .to_string()
}

fn phase2_identity_random_warning_case_yaml() -> String {
    r#"kind: phase2-normalization-case
authoring_input: fixtures/phase2/inputs/minimal-authoring-ir.json
comparison_context: null
identity_override_mode: random
reason_code: manual_randomization
expected_result_status: success
expected_diagnostic_codes:
  - PHASE2.IDENTITY_RANDOM_OVERRIDE
resolved_identity_prefix: "rnd:"
"#
    .to_string()
}

fn phase2_partial_high_case_yaml() -> String {
    r#"kind: phase2-risk-case
authoring_input: fixtures/phase2/inputs/minimal-authoring-ir.json
comparison_context:
  kind: comparison-context
  baseline_kind: identity_index
  baseline_artifact_fingerprint: baseline-identity-only
  risk_policy_ref: risk-policy.default@1.0.0
  comparison_mode: best_effort
expected_result_status: success
expected_comparison_status: partial
expected_overall_level: medium
expected_comparison_reasons:
  - BASELINE_IDENTITY_INDEX_ONLY
"#
    .to_string()
}

fn phase2_minimal_authoring_ir_json() -> String {
    r#"{
  "kind": "authoring-ir",
  "schema_version": "0.1.0",
  "metadata": {
    "document_id": "demo-doc"
  },
  "notetypes": [],
  "notes": []
}
"#
    .to_string()
}

fn phase2_minimal_success_result_json() -> String {
    let result = json!({
        "kind": "normalization-result",
        "result_status": "success",
        "tool_contract_version": "phase2-v1",
        "policy_refs": {
            "identity_policy_ref": "identity-policy.default@1.0.0",
            "risk_policy_ref": null
        },
        "comparison_context": null,
        "diagnostics": {
            "kind": "normalization-diagnostics",
            "status": "valid",
            "items": []
        },
        "normalized_ir": {
            "kind": "normalized-ir",
            "schema_version": "0.1.0",
            "document_id": "demo-doc",
            "resolved_identity": "det:demo-doc",
            "notetypes": [],
            "notes": [],
            "media": []
        },
        "merge_risk_report": null
    });

    serde_json::to_string_pretty(&result).expect("serialize phase2 normalization result")
}

fn phase2_minimal_success_result_mismatch_json() -> String {
    let result = json!({
        "kind": "normalization-result",
        "result_status": "success",
        "tool_contract_version": "phase2-v1",
        "policy_refs": {
            "identity_policy_ref": "identity-policy.default@1.0.0",
            "risk_policy_ref": null
        },
        "comparison_context": null,
        "diagnostics": {
            "kind": "normalization-diagnostics",
            "status": "valid",
            "items": []
        },
        "normalized_ir": {
            "kind": "normalized-ir",
            "schema_version": "0.1.0",
            "document_id": "demo-doc",
            "resolved_identity": "det:not-the-same",
            "notetypes": [],
            "notes": [],
            "media": []
        },
        "merge_risk_report": null
    });

    serde_json::to_string_pretty(&result).expect("serialize mismatched phase2 result")
}

fn phase2_complete_low_risk_json() -> String {
    let report = json!({
        "kind": "merge-risk-report",
        "comparison_status": "complete",
        "overall_level": "low",
        "policy_version": "risk-policy.default@1.0.0",
        "baseline_artifact_fingerprint": "baseline-minimal-success",
        "current_artifact_fingerprint": "det:demo-doc",
        "comparison_reasons": []
    });

    serde_json::to_string_pretty(&report).expect("serialize phase2 risk report")
}

fn compatibility_classes_yaml() -> String {
    r#"classes:
  - additive_compatible
  - behavior_tightening_compatible
  - behavior_changing_incompatible
  - fixture_only_non_semantic
  - documentation_only_normative_clarification
"#
    .to_string()
}

fn upgrade_rules_yaml() -> String {
    r#"rules:
  - id: migration_notes_required
  - id: fixture_updates_required
  - id: executable_checks_required
  - id: legacy_fixture_overlap_allowed
"#
    .to_string()
}

fn additive_evolution_yaml(compatibility_class: &str, expected_bundle_bump: &str) -> String {
    format!(
        r#"kind: evolution-case
compatibility_class: {compatibility_class}
upgrade_rules:
  - fixture_updates_required
target_asset: fixtures/index.yaml
affected_paths:
  - fixtures/valid/minimal-authoring-ir.json
expected_bundle_bump: {expected_bundle_bump}
"#
    )
}

fn incompatible_evolution_yaml(compatibility_class: &str, expected_bundle_bump: &str) -> String {
    format!(
        r#"kind: evolution-case
compatibility_class: {compatibility_class}
upgrade_rules:
  - migration_notes_required
  - executable_checks_required
target_asset: fixtures/index.yaml
affected_paths:
  - fixtures/invalid/missing-document-id.json
  - fixtures/expected/missing-document-id.report.json
expected_bundle_bump: {expected_bundle_bump}
"#
    )
}
