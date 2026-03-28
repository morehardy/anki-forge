use contract_tools::{
    contract_manifest_path,
    manifest::{load_manifest, resolve_asset_path, resolve_contract_relative_path},
};
use serde_json::json;
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

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

fn write_bundle(root: &Path, bundle_version: &str, schema_bundle_version: &str) -> PathBuf {
    fs::create_dir_all(root.join("schema")).expect("create schema dir");
    fs::create_dir_all(root.join("versioning")).expect("create versioning dir");
    fs::create_dir_all(root.join("errors")).expect("create errors dir");
    fs::create_dir_all(root.join("semantics")).expect("create semantics dir");

    fs::write(root.join("manifest.yaml"), manifest_yaml(bundle_version)).expect("write manifest");
    fs::write(
        root.join("schema/manifest.schema.json"),
        manifest_schema(schema_bundle_version),
    )
    .expect("write manifest schema");
    fs::write(root.join("versioning/policy.md"), "# policy\n").expect("write policy");
    fs::write(
        root.join("schema/diagnostic-item.schema.json"),
        "{\n  \"$schema\": \"http://json-schema.org/draft-07/schema#\",\n  \"type\": \"object\"\n}\n",
    )
    .expect("write diagnostic item schema");
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
    fs::write(
        root.join("errors/error-registry.yaml"),
        "codes:\n  - id: AF0001\n    status: active\n    summary: example\n",
    )
    .expect("write error registry");
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

    root.join("manifest.yaml")
}

fn manifest_yaml(bundle_version: &str) -> String {
    format!(
        r#"bundle_version: "{bundle_version}"
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
  diagnostic_item_schema: schema/diagnostic-item.schema.json
  error_registry: errors/error-registry.yaml
  validation_semantics: semantics/validation.md
  path_semantics: semantics/path-conventions.md
  compatibility_semantics: semantics/compatibility.md
"#
    )
}

fn manifest_schema(bundle_version: &str) -> String {
    let schema = json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "required": ["bundle_version", "component_versions", "compatibility", "assets"],
        "additionalProperties": false,
        "properties": {
            "bundle_version": {
                "const": bundle_version
            },
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
                    "diagnostic_item_schema",
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
                    "diagnostic_item_schema": { "type": "string" },
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

#[test]
fn manifest_uses_bundle_version_as_the_only_public_axis() {
    let manifest = load_manifest(contract_manifest_path()).expect("manifest loads");
    assert_eq!(manifest.data.bundle_version, "0.1.0");
    assert_eq!(manifest.data.compatibility.public_axis, "bundle_version");
    assert!(manifest.data.component_versions.contains_key("schema"));
}

#[test]
fn manifest_resolves_asset_paths_relative_to_manifest_directory() {
    let manifest = load_manifest(contract_manifest_path()).expect("manifest loads");
    let schema_path = resolve_asset_path(&manifest, "manifest_schema").unwrap();
    assert_eq!(
        schema_path,
        manifest.contracts_root.join("schema/manifest.schema.json")
    );
}

#[test]
fn manifest_schema_is_loaded_from_each_manifest_root() {
    let root_a = temp_contract_root("root-a");
    let root_b = temp_contract_root("root-b");
    let manifest_a = load_manifest(write_bundle(&root_a, "1.0.0", "1.0.0")).expect("root A loads");
    let manifest_b = load_manifest(write_bundle(&root_b, "2.0.0", "2.0.0")).expect("root B loads");

    assert_eq!(manifest_a.data.bundle_version, "1.0.0");
    assert_eq!(manifest_b.data.bundle_version, "2.0.0");
}

#[test]
fn relative_contract_paths_reject_absolute_paths_and_escape_attempts() {
    let root = temp_contract_root("relative-paths");
    fs::create_dir_all(&root).expect("create temp root");
    fs::write(root.join("manifest.yaml"), "placeholder\n").expect("create manifest");

    let absolute_err = resolve_contract_relative_path(&root, "/tmp/evil")
        .expect_err("absolute paths are rejected");
    assert!(absolute_err
        .to_string()
        .contains("asset path must be relative"));

    let outside = root
        .parent()
        .expect("temp root should have a parent")
        .join("evil-outside-file");
    fs::write(&outside, "evil").expect("create outside file");

    let escape_err = resolve_contract_relative_path(&root, "../evil-outside-file")
        .expect_err("path traversal is rejected");
    assert!(escape_err
        .to_string()
        .contains("asset path must stay within contracts/"));
}

#[test]
fn relative_contract_paths_reject_directory_assets() {
    let root = temp_contract_root("directory-asset");
    fs::create_dir_all(root.join("versioning/upgrade-rules.yaml")).expect("create dir asset");

    let err = resolve_contract_relative_path(&root, "versioning/upgrade-rules.yaml")
        .expect_err("directory assets are rejected");
    assert!(err
        .to_string()
        .contains("asset path must resolve to a file"));
}

#[cfg(unix)]
#[test]
fn relative_contract_paths_reject_symlink_escape() {
    let root = temp_contract_root("symlink-escape");
    let outside = temp_contract_root("outside-target");
    fs::create_dir_all(&root).expect("create root");
    fs::create_dir_all(&outside).expect("create outside root");
    fs::write(outside.join("manifest.schema.json"), "escaped").expect("create outside file");
    fs::create_dir_all(root.join("schema")).expect("create schema dir");

    std::os::unix::fs::symlink(
        outside.join("manifest.schema.json"),
        root.join("schema/manifest.schema.json"),
    )
    .expect("create symlink");

    let err = resolve_contract_relative_path(&root, "schema/manifest.schema.json")
        .expect_err("symlink escape is rejected");
    assert!(err
        .to_string()
        .contains("asset path must stay within contracts/"));
}
