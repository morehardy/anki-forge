use contract_tools::{
    contract_manifest_path, fixtures::run_fixture_gates, manifest::load_manifest,
};
use serde_json::json;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

#[test]
fn fixture_gates_accept_the_bundled_catalog_and_fixtures() {
    run_fixture_gates(contract_manifest_path()).expect("bundled fixture gate should pass");
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
