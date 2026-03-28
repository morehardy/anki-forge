use contract_tools::{
    contract_manifest_path,
    manifest::load_manifest,
    registry::run_registry_gates,
    semantics::run_semantics_gates,
};
use serde_json::json;
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn error_registry_gate_runs_against_the_bundled_manifest() {
    run_registry_gates(contract_manifest_path()).unwrap();
}

#[test]
fn semantics_gate_runs_against_the_bundled_manifest() {
    run_semantics_gates(contract_manifest_path()).unwrap();
}

#[test]
fn error_registry_gate_rejects_schema_violations() {
    let manifest = load_manifest(write_invalid_registry_bundle()).unwrap();
    let err = run_registry_gates(&manifest.path).expect_err("schema violations should fail");

    assert!(err
        .to_string()
        .contains("error registry schema validation failed"));
}

#[test]
fn semantics_gate_rejects_unregistered_asset_refs() {
    let manifest = load_manifest(write_unregistered_semantics_bundle()).unwrap();
    let err = run_semantics_gates(&manifest.path).expect_err("undeclared refs should fail");

    assert!(err
        .to_string()
        .contains("semantic asset ref must be declared in manifest"));
}

fn temp_contract_root(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be monotonic")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "anki-forge-contract-tools-{}-{}-{}",
        label,
        std::process::id(),
        unique
    ))
}

fn write_invalid_registry_bundle() -> PathBuf {
    let root = temp_contract_root("invalid-registry");
    write_bundle(
        &root,
        "codes:\n  - id: AF0001\n    status: active\n    summary: example\n    unexpected: true\n",
        "schema/diagnostic-item.schema.json",
        false,
    )
}

fn write_unregistered_semantics_bundle() -> PathBuf {
    let root = temp_contract_root("unregistered-semantics");
    write_bundle(
        &root,
        "codes:\n  - id: AF0001\n    status: active\n    summary: example\n",
        "schema/undeclared-item.schema.json",
        true,
    )
}

fn write_bundle(
    root: &Path,
    registry_yaml: &str,
    semantics_ref: &str,
    include_undeclared_asset: bool,
) -> PathBuf {
    fs::create_dir_all(root.join("schema")).expect("create schema dir");
    fs::create_dir_all(root.join("versioning")).expect("create versioning dir");
    fs::create_dir_all(root.join("errors")).expect("create errors dir");
    fs::create_dir_all(root.join("semantics")).expect("create semantics dir");

    fs::write(root.join("manifest.yaml"), manifest_yaml()).expect("write manifest");
    fs::write(
        root.join("schema/manifest.schema.json"),
        manifest_schema(),
    )
    .expect("write manifest schema");
    fs::write(
        root.join("schema/error-registry.schema.json"),
        error_registry_schema(),
    )
    .expect("write error registry schema");
    fs::write(root.join("versioning/policy.md"), "# policy\n").expect("write policy");
    fs::write(
        root.join("versioning/compatibility-classes.yaml"),
        "classes:\n  - additive_compatible\n",
    )
    .expect("write compatibility classes");
    fs::write(
        root.join("versioning/upgrade-rules.yaml"),
        "rules:\n  - id: migration_notes_required\n",
    )
    .expect("write upgrade rules");
    fs::write(root.join("errors/error-registry.yaml"), registry_yaml).expect("write registry");
    fs::write(
        root.join("semantics/validation.md"),
        "---\nasset_refs:\n  - versioning/compatibility-classes.yaml\n---\n# Validation\n",
    )
    .expect("write validation semantics");
    fs::write(
        root.join("semantics/path-conventions.md"),
        format!(
            "---\nasset_refs:\n  - {semantics_ref}\n---\n# Path Conventions\n"
        ),
    )
    .expect("write path semantics");
    fs::write(
        root.join("semantics/compatibility.md"),
        "---\nasset_refs:\n  - versioning/upgrade-rules.yaml\n---\n# Compatibility\n",
    )
    .expect("write compatibility semantics");
    if include_undeclared_asset {
        fs::write(root.join(semantics_ref), "# undeclared asset\n").expect("write undeclared asset");
    }

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
  error_registry_schema: schema/error-registry.schema.json
  error_registry: errors/error-registry.yaml
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
                    "error_registry_schema",
                    "error_registry",
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
                    "error_registry_schema": { "type": "string" },
                    "error_registry": { "type": "string" },
                    "validation_semantics": { "type": "string" },
                    "path_semantics": { "type": "string" },
                    "compatibility_semantics": { "type": "string" }
                }
            }
        }
    });

    serde_json::to_string_pretty(&schema).expect("serialize manifest schema")
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
