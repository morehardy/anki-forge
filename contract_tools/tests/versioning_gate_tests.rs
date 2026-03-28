use contract_tools::{contract_manifest_path, versioning::run_versioning_gates};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

#[test]
fn versioning_gates_accept_the_bundled_contract_bundle() {
    run_versioning_gates(contract_manifest_path()).expect("bundled versioning gate should pass");
}

#[test]
fn versioning_gates_reject_unknown_compatibility_classes() {
    let manifest = load_manifest(write_bundle(
        "behavior_changing_incompatible",
        "experimental_breaking",
    ))
    .expect("temp manifest loads");

    let err = run_versioning_gates(&manifest.path).expect_err("unknown classes should fail");
    assert!(err
        .to_string()
        .contains("unknown compatibility class in evolution fixture"));
}

#[test]
fn versioning_gates_reject_missing_compatible_or_incompatible_coverage() {
    let manifest = load_manifest(write_bundle("additive_compatible", "additive_compatible"))
        .expect("temp manifest loads");

    let err = run_versioning_gates(&manifest.path)
        .expect_err("missing incompatible coverage should fail");
    assert!(err.to_string().contains(
        "fixture catalog must include both compatible and incompatible evolution examples"
    ));
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

fn write_bundle(incompatible_class: &str, additive_class: &str) -> PathBuf {
    let root = temp_contract_root("versioning-gates");
    fs::create_dir_all(root.join("schema")).expect("create schema dir");
    fs::create_dir_all(root.join("versioning")).expect("create versioning dir");
    fs::create_dir_all(root.join("fixtures/evolution")).expect("create evolution dir");
    fs::create_dir_all(root.join("fixtures")).expect("create fixtures dir");

    fs::write(root.join("manifest.yaml"), manifest_yaml()).expect("write manifest");
    fs::write(root.join("schema/manifest.schema.json"), manifest_schema()).expect("write schema");
    fs::write(root.join("versioning/policy.md"), version_policy()).expect("write policy");
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
    fs::write(
        root.join("fixtures/index.yaml"),
        fixture_catalog_yaml(incompatible_class, additive_class),
    )
    .expect("write catalog");
    fs::write(root.join("fixtures/target.yaml"), "target: true\n").expect("write target");
    fs::write(root.join("fixtures/a.yaml"), "a: true\n").expect("write affected path");
    fs::write(root.join("fixtures/b.yaml"), "b: true\n").expect("write affected path");

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
  fixture_catalog: fixtures/index.yaml
"#
    .to_string()
}

fn manifest_schema() -> String {
    r#"{
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
        "fixture_catalog"
      ],
      "additionalProperties": false,
      "properties": {
        "manifest_schema": { "type": "string" },
        "version_policy": { "type": "string" },
        "compatibility_classes": { "type": "string" },
        "upgrade_rules": { "type": "string" },
        "fixture_catalog": { "type": "string" }
      }
    }
  }
}
"#
    .to_string()
}

fn version_policy() -> String {
    "# Bundle Versioning Policy\n\nThe bundle version is the only public compatibility axis for Anki Forge contracts.\nComponent versions are internal governance metadata only.\n".to_string()
}

fn compatibility_classes_yaml() -> String {
    r#"classes:
  - additive_compatible
  - behavior_tightening_compatible
  - behavior_changing_incompatible
"#
    .to_string()
}

fn upgrade_rules_yaml() -> String {
    r#"rules:
  - id: migration_notes_required
  - id: fixture_updates_required
  - id: executable_checks_required
"#
    .to_string()
}

fn fixture_catalog_yaml(incompatible_class: &str, additive_class: &str) -> String {
    let incompatible_bump = if incompatible_class == "behavior_changing_incompatible" {
        "major"
    } else {
        "minor"
    };
    format!(
        r#"cases:
  - id: additive-compatible
    category: evolution
    compatibility_class: {additive_class}
    upgrade_rules:
      - fixture_updates_required
    target_asset: fixtures/target.yaml
    affected_paths:
      - fixtures/a.yaml
    expected_bundle_bump: minor
    input: fixtures/evolution/additive.yaml
  - id: incompatible-path-change
    category: evolution
    compatibility_class: {incompatible_class}
    upgrade_rules:
      - migration_notes_required
      - executable_checks_required
    target_asset: fixtures/target.yaml
    affected_paths:
      - fixtures/b.yaml
    expected_bundle_bump: {incompatible_bump}
    input: fixtures/evolution/incompatible.yaml
"#
    )
}

fn load_manifest(path: PathBuf) -> anyhow::Result<contract_tools::manifest::LoadedManifest> {
    contract_tools::manifest::load_manifest(path)
}
