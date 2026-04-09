# Phase 1 Foundation Platform Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the `Phase 1` CI-grade, schema-centered contract bundle for `anki-forge`, including normative contract assets, executable verification tooling, and CI/release gates without introducing product-grade APIs or writer logic.

**Architecture:** The repository becomes contract-first. `contracts/` is the normative source of truth, `contract_tools/` is a Rust workspace member that loads and verifies the bundle, `docs/` holds explanatory and governance material, and `.github/` runs the same checks in CI that local workers run through `contracts/manifest.yaml`. Every contract-affecting change must be represented by assets plus executable gates before it is allowed to merge.

**Tech Stack:** Rust 1.92 workspace, `serde`, `serde_json`, `serde_yaml`, `jsonschema`, `clap`, `thiserror`, `assert_cmd`, JSON Schema files, JSON/YAML fixtures, Markdown normative docs, GitHub Actions

---

## Scope Check

This plan covers a single coherent subsystem: `Phase 1: Foundation Platform`. Do not split this into separate implementation plans unless the user explicitly asks to peel off one of these areas into its own sub-project:

- contract asset authoring
- executable verification tooling
- CI/release automation

## Execution Rules

- `contracts/manifest.yaml` is updated incrementally only. Every manifest snippet below is a patch fragment to merge into the existing file, never a full-file overwrite.
- All contract asset paths declared inside `contracts/manifest.yaml` are resolved relative to the manifest directory.
- Tests and tooling must use the same manifest-driven path resolution helpers. Do not hardcode ad hoc `../contracts/...` paths once the resolver exists.
- Avoid implicit cross-file JSON Schema `$ref` resolution. Use local `$defs` or explicit resolver setup so schema validation remains deterministic.

## File Structure Map

### Repository root

- Create: `Cargo.toml` - workspace root for `contract_tools`
- Modify: `.gitignore` - ignore `target/`, `dist/`, and generated bundle artifacts
- Create: `README.md` - explain that `contracts/` is the source of truth and that `Phase 1` is not a product implementation

### Contract assets

- Create: `contracts/manifest.yaml` - single-entry bundle manifest with bundle version, component versions, compatibility claims, and asset paths
- Create: `contracts/schema/manifest.schema.json` - manifest shape contract
- Create: `contracts/schema/authoring-ir.schema.json` - `Authoring IR` shape contract
- Create: `contracts/schema/diagnostic-item.schema.json` - diagnostic item shape contract
- Create: `contracts/schema/validation-report.schema.json` - report contract
- Create: `contracts/schema/service-envelope.schema.json` - minimal envelope contract
- Create: `contracts/schema/error-registry.schema.json` - shape for `contracts/errors/error-registry.yaml`
- Create: `contracts/semantics/validation.md` - normative validation semantics
- Create: `contracts/semantics/path-conventions.md` - normative path syntax and interpretation
- Create: `contracts/semantics/compatibility.md` - compatibility, incompatibility, and upgrade semantics
- Create: `contracts/errors/error-registry.yaml` - stable error code registry
- Create: `contracts/versioning/policy.md` - public bundle version policy
- Create: `contracts/versioning/compatibility-classes.yaml` - machine-readable compatibility classes
- Create: `contracts/versioning/upgrade-rules.yaml` - machine-readable upgrade rule catalog
- Create: `contracts/fixtures/index.yaml` - machine-readable fixture catalog
- Create: `contracts/fixtures/valid/minimal-authoring-ir.json` - canonical valid IR case
- Create: `contracts/fixtures/invalid/missing-document-id.json` - canonical invalid IR case
- Create: `contracts/fixtures/expected/missing-document-id.report.json` - expected report for the invalid IR case
- Create: `contracts/fixtures/service-envelope/minimal-success.json` - canonical minimal envelope case
- Create: `contracts/fixtures/evolution/additive-compatible.yaml` - additive compatibility example
- Create: `contracts/fixtures/evolution/incompatible-path-change.yaml` - incompatible evolution example

### Governance docs

- Create: `docs/adr/README.md` - ADR process and numbering rules
- Create: `docs/adr/0001-contract-bundle-source-of-truth.md` - contract bundle as the normative source
- Create: `docs/adr/0002-bundle-version-is-public-axis.md` - bundle version as the only public compatibility axis
- Create: `docs/process/contract-change-policy.md` - governance for contract-affecting changes only
- Create: `docs/rfcs/README.md` - RFC entry criteria and review flow

### Contract tooling

- Create: `contract_tools/Cargo.toml` - Rust package for verification tooling
- Create: `contract_tools/src/lib.rs` - library entrypoint
- Create: `contract_tools/src/main.rs` - internal CLI entrypoint for `verify`, `summary`, and `package`
- Create: `contract_tools/src/manifest.rs` - manifest loader and validator
- Create: `contract_tools/src/schema.rs` - schema loading and JSON Schema validation helpers
- Create: `contract_tools/src/semantics.rs` - semantics document metadata loading and consistency checks
- Create: `contract_tools/src/registry.rs` - error registry loading and lifecycle checks
- Create: `contract_tools/src/fixtures.rs` - fixture catalog loading and pairing logic
- Create: `contract_tools/src/versioning.rs` - compatibility-class and upgrade-rule loading plus evolution-case checks
- Create: `contract_tools/src/gates.rs` - top-level verification gates
- Create: `contract_tools/src/package.rs` - bundle packaging and artifact manifesting
- Create: `contract_tools/src/summary.rs` - change and compatibility summary rendering
- Create: `contract_tools/tests/workspace_smoke_tests.rs` - repo skeleton smoke tests
- Create: `contract_tools/tests/manifest_tests.rs` - manifest loading tests
- Create: `contract_tools/tests/schema_gate_tests.rs` - schema integrity tests
- Create: `contract_tools/tests/registry_gate_tests.rs` - registry consistency tests
- Create: `contract_tools/tests/fixture_gate_tests.rs` - fixture conformance tests
- Create: `contract_tools/tests/versioning_gate_tests.rs` - compatibility-class and evolution-case tests
- Create: `contract_tools/tests/cli_tests.rs` - internal CLI tests
- Create: `contract_tools/tests/package_tests.rs` - package artifact tests

### Future consumer placeholder

- Create: `implementations/rust/README.md` - note that Rust implementation work is intentionally deferred beyond stubs in Phase 1

### CI/release automation

- Create: `.github/workflows/contract-ci.yml` - run format, lint, tests, and `verify`
- Create: `.github/workflows/contract-release.yml` - package and publish bundle artifacts on tagged releases or manual dispatch

## Task 1: Bootstrap the Contract-First Repository Skeleton

**Files:**
- Create: `Cargo.toml`
- Modify: `.gitignore`
- Create: `README.md`
- Create: `contract_tools/Cargo.toml`
- Create: `contract_tools/src/lib.rs`
- Create: `contract_tools/src/main.rs`
- Create: `contract_tools/tests/workspace_smoke_tests.rs`
- Create: `contracts/manifest.yaml`
- Create: `implementations/rust/README.md`

- [ ] **Step 1: Write the failing workspace smoke test**

```rust
// contract_tools/tests/workspace_smoke_tests.rs
#[test]
fn repository_exposes_a_contract_bundle_entrypoint() {
    assert!(contract_tools::contract_manifest_path().exists());
}
```

- [ ] **Step 2: Run the smoke test to verify it fails**

Run: `cargo test -p contract_tools --test workspace_smoke_tests -v`

Expected: FAIL because the workspace package and `contracts/manifest.yaml` do not exist yet.

- [ ] **Step 3: Create the minimal workspace and stub tooling package**

```toml
# Cargo.toml
[workspace]
members = ["contract_tools"]
resolver = "2"

[workspace.package]
edition = "2021"
rust-version = "1.92"

[workspace.lints.rust]
unsafe_code = "forbid"
```

```toml
# contract_tools/Cargo.toml
[package]
name = "contract_tools"
version = "0.1.0"
edition = "2021"

[dependencies]

[dev-dependencies]
```

```rust
// contract_tools/src/lib.rs
use std::path::PathBuf;

pub fn contract_manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../contracts/manifest.yaml")
}
```

```rust
// contract_tools/src/main.rs
fn main() {
    eprintln!("contract_tools bootstrap");
}
```

```yaml
# contracts/manifest.yaml
bundle_version: "0.1.0"
component_versions: {}
compatibility:
  public_axis: bundle_version
assets: {}
```

- [ ] **Step 4: Add repository metadata and placeholder consumer docs**

```markdown
# README.md

`anki-forge` Phase 1 is a contract-first repository.
`contracts/` is the normative source of truth.
`contract_tools/` provides internal verification tooling only.
```

```markdown
# implementations/rust/README.md

Rust implementation work is intentionally deferred in Phase 1.
This directory may remain skeletal or empty until a later phase.
```

```gitignore
/target
/dist
```

- [ ] **Step 5: Run the smoke test to verify it passes**

Run: `cargo test -p contract_tools --test workspace_smoke_tests -v`

Expected: PASS

- [ ] **Step 6: Commit the repository bootstrap**

```bash
git add Cargo.toml .gitignore README.md contracts/manifest.yaml contract_tools/Cargo.toml contract_tools/src/lib.rs contract_tools/src/main.rs contract_tools/tests/workspace_smoke_tests.rs implementations/rust/README.md
git commit -m "chore: bootstrap phase 1 contract workspace"
```

## Task 2: Define the Bundle Manifest and Versioning Assets

**Files:**
- Modify: `contracts/manifest.yaml`
- Create: `contracts/schema/manifest.schema.json`
- Create: `contracts/versioning/policy.md`
- Create: `contracts/versioning/compatibility-classes.yaml`
- Create: `contracts/versioning/upgrade-rules.yaml`
- Modify: `contract_tools/Cargo.toml`
- Create: `contract_tools/src/manifest.rs`
- Modify: `contract_tools/src/lib.rs`
- Create: `contract_tools/tests/manifest_tests.rs`

- [ ] **Step 1: Write the failing manifest-loading test**

```rust
// contract_tools/tests/manifest_tests.rs
use contract_tools::{
    contract_manifest_path,
    manifest::{load_manifest, resolve_asset_path},
};

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
    assert!(schema_path.ends_with("contracts/schema/manifest.schema.json"));
}
```

- [ ] **Step 2: Run the manifest test to verify it fails**

Run: `cargo test -p contract_tools --test manifest_tests -v`

Expected: FAIL with unresolved import or missing loader implementation.

- [ ] **Step 3: Implement the manifest loader and schema**

```rust
// contract_tools/src/manifest.rs
use anyhow::{Context, ensure};
use serde::Deserialize;
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Deserialize)]
pub struct Compatibility {
    pub public_axis: String,
}

#[derive(Debug, Deserialize)]
pub struct Manifest {
    pub bundle_version: String,
    pub component_versions: BTreeMap<String, String>,
    pub compatibility: Compatibility,
    pub assets: BTreeMap<String, String>,
}

pub struct LoadedManifest {
    pub path: PathBuf,
    pub contracts_root: PathBuf,
    pub data: Manifest,
}

pub fn load_manifest(path: impl AsRef<Path>) -> anyhow::Result<LoadedManifest> {
    let path = path.as_ref().canonicalize()?;
    let contracts_root = path.parent().context("manifest must live under contracts/")?.to_path_buf();
    let raw = fs::read_to_string(&path)?;

    let schema_raw = fs::read_to_string(contracts_root.join("schema/manifest.schema.json"))?;
    let schema: Value = serde_json::from_str(&schema_raw)?;
    let validator = jsonschema::validator_for(&schema)?;
    let yaml_value: serde_yaml::Value = serde_yaml::from_str(&raw)?;
    let json_value = serde_json::to_value(yaml_value)?;
    validator.validate(&json_value)?;

    let data: Manifest = serde_yaml::from_str(&raw)?;
    let loaded = LoadedManifest { path, contracts_root, data };
    for key in loaded.data.assets.keys() {
        let _ = resolve_asset_path(&loaded, key)?;
    }
    Ok(loaded)
}

pub fn resolve_asset_path(manifest: &LoadedManifest, key: &str) -> anyhow::Result<PathBuf> {
    let rel = manifest.data.assets.get(key).context("missing asset key")?;
    let path = manifest.contracts_root.join(rel);
    ensure!(path.exists(), "asset path does not exist: {}", path.display());
    Ok(path)
}
```

```toml
# contract_tools/Cargo.toml
[dependencies]
anyhow = "1"
jsonschema = "0.18"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"

[dev-dependencies]
```

```json
// contracts/schema/manifest.schema.json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "required": ["bundle_version", "component_versions", "compatibility", "assets"],
  "properties": {
    "bundle_version": { "type": "string" },
    "component_versions": { "type": "object", "additionalProperties": { "type": "string" } },
    "compatibility": {
      "type": "object",
      "required": ["public_axis"],
      "properties": {
        "public_axis": { "const": "bundle_version" }
      }
    },
    "assets": { "type": "object", "additionalProperties": { "type": "string" } }
  }
}
```

- [ ] **Step 4: Populate versioning policy assets and enrich the manifest**

```yaml
# contracts/manifest.yaml
bundle_version: "0.1.0"
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
```

```yaml
# contracts/versioning/compatibility-classes.yaml
classes:
  - additive_compatible
  - behavior_tightening_compatible
  - behavior_changing_incompatible
  - fixture_only_non_semantic
  - documentation_only_normative_clarification
```

```yaml
# contracts/versioning/upgrade-rules.yaml
rules:
  - id: migration_notes_required
  - id: fixture_updates_required
  - id: executable_checks_required
  - id: legacy_fixture_overlap_allowed
```

- [ ] **Step 5: Run the manifest test to verify it passes**

Run: `cargo test -p contract_tools --test manifest_tests -v`

Expected: PASS

- [ ] **Step 6: Commit the manifest and versioning foundation**

```bash
git add contracts/manifest.yaml contracts/schema/manifest.schema.json contracts/versioning/policy.md contracts/versioning/compatibility-classes.yaml contracts/versioning/upgrade-rules.yaml contract_tools/Cargo.toml contract_tools/src/lib.rs contract_tools/src/manifest.rs contract_tools/tests/manifest_tests.rs
git commit -m "feat: add bundle manifest and versioning assets"
```

## Task 3: Define the Core Contract Schemas

**Files:**
- Create: `contracts/schema/authoring-ir.schema.json`
- Create: `contracts/schema/diagnostic-item.schema.json`
- Create: `contracts/schema/validation-report.schema.json`
- Create: `contracts/schema/service-envelope.schema.json`
- Create: `contracts/schema/error-registry.schema.json`
- Modify: `contracts/manifest.yaml`
- Modify: `contract_tools/Cargo.toml`
- Create: `contract_tools/src/schema.rs`
- Modify: `contract_tools/src/lib.rs`
- Create: `contract_tools/tests/schema_gate_tests.rs`

- [ ] **Step 1: Write the failing schema gate tests**

```rust
// contract_tools/tests/schema_gate_tests.rs
use contract_tools::{
    contract_manifest_path,
    manifest::{load_manifest, resolve_asset_path},
    schema::{load_schema, validate_value},
};
use serde_json::json;

#[test]
fn authoring_ir_schema_accepts_the_minimal_valid_shape() {
    let manifest = load_manifest(contract_manifest_path()).unwrap();
    let schema = load_schema(resolve_asset_path(&manifest, "authoring_ir_schema").unwrap()).unwrap();
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
    let schema = load_schema(resolve_asset_path(&manifest, "validation_report_schema").unwrap()).unwrap();
    let value = json!({ "kind": "validation-report", "status": "invalid" });
    assert!(validate_value(&schema, &value).is_err());
}
```

- [ ] **Step 2: Run the schema tests to verify they fail**

Run: `cargo test -p contract_tools --test schema_gate_tests -v`

Expected: FAIL because the schemas and helpers do not exist yet.

- [ ] **Step 3: Implement schema loading helpers**

```rust
// contract_tools/src/schema.rs
use anyhow::Context;
use jsonschema::Validator;
use serde_json::Value;
use std::{fs, path::Path};

pub fn load_schema(path: impl AsRef<Path>) -> anyhow::Result<Validator> {
    let raw = fs::read_to_string(path)?;
    let schema: Value = serde_json::from_str(&raw)?;
    Ok(jsonschema::validator_for(&schema)?)
}

pub fn validate_value(schema: &Validator, value: &Value) -> anyhow::Result<()> {
    schema.validate(value)?;
    Ok(())
}

pub fn run_schema_gates(manifest_path: &str) -> anyhow::Result<()> {
    let manifest = crate::manifest::load_manifest(manifest_path)?;
    for key in [
        "authoring_ir_schema",
        "diagnostic_item_schema",
        "validation_report_schema",
        "service_envelope_schema",
        "error_registry_schema"
    ] {
        let path = manifest.data.assets.get(key).context("missing schema asset in manifest")?;
        load_schema(path)?;
    }
    Ok(())
}
```

```toml
# contract_tools/Cargo.toml
[dependencies]
anyhow = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
jsonschema = "0.18"
```

- [ ] **Step 4: Author the core schema files**

```json
// contracts/schema/authoring-ir.schema.json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "required": ["kind", "schema_version", "metadata", "notetypes", "notes"],
  "properties": {
    "kind": { "const": "authoring-ir" },
    "schema_version": { "type": "string" },
    "metadata": {
      "type": "object",
      "required": ["document_id"],
      "properties": {
        "document_id": { "type": "string", "minLength": 1 }
      }
    },
    "notetypes": { "type": "array" },
    "notes": { "type": "array" }
  }
}
```

```json
// contracts/schema/diagnostic-item.schema.json
{
  "type": "object",
  "required": ["level", "code", "path", "message"],
  "properties": {
    "level": { "enum": ["warning", "error"] },
    "code": { "type": "string" },
    "path": { "type": "string" },
    "message": { "type": "string" }
  }
}
```

```json
// contracts/schema/validation-report.schema.json
{
  "type": "object",
  "$defs": {
    "diagnostic_item": {
      "type": "object",
      "required": ["level", "code", "path", "message"],
      "properties": {
        "level": { "enum": ["warning", "error"] },
        "code": { "type": "string" },
        "path": { "type": "string" },
        "message": { "type": "string" }
      }
    }
  },
  "required": ["kind", "status", "diagnostics"],
  "properties": {
    "kind": { "const": "validation-report" },
    "status": { "enum": ["valid", "invalid"] },
    "diagnostics": {
      "type": "array",
      "items": { "$ref": "#/$defs/diagnostic_item" }
    }
  }
}
```

```json
// contracts/schema/service-envelope.schema.json
{
  "type": "object",
  "required": ["kind", "request_id", "status"],
  "properties": {
    "kind": { "const": "service-envelope" },
    "request_id": { "type": "string", "minLength": 1 },
    "status": { "enum": ["ok", "error"] }
  }
}
```

```json
// contracts/schema/error-registry.schema.json
{
  "type": "object",
  "required": ["codes"],
  "properties": {
    "codes": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["id", "status", "summary"],
        "properties": {
          "id": { "type": "string" },
          "status": { "enum": ["active", "deprecated", "removed"] },
          "summary": { "type": "string" }
        }
      }
    }
  }
}
```

- [ ] **Step 5: Update the manifest asset map and run the schema tests**

```yaml
# contracts/manifest.yaml
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
```

Run: `cargo test -p contract_tools --test schema_gate_tests -v`

Expected: PASS

- [ ] **Step 6: Commit the core schemas**

```bash
git add contracts/schema/authoring-ir.schema.json contracts/schema/diagnostic-item.schema.json contracts/schema/validation-report.schema.json contracts/schema/service-envelope.schema.json contracts/schema/error-registry.schema.json contracts/manifest.yaml contract_tools/Cargo.toml contract_tools/src/lib.rs contract_tools/src/schema.rs contract_tools/tests/schema_gate_tests.rs
git commit -m "feat: add phase 1 core contract schemas"
```

## Task 4: Add Semantics, Governance Docs, and the Error Registry

**Files:**
- Create: `contracts/semantics/validation.md`
- Create: `contracts/semantics/path-conventions.md`
- Create: `contracts/semantics/compatibility.md`
- Create: `contracts/errors/error-registry.yaml`
- Modify: `contracts/manifest.yaml`
- Create: `docs/adr/README.md`
- Create: `docs/adr/0001-contract-bundle-source-of-truth.md`
- Create: `docs/adr/0002-bundle-version-is-public-axis.md`
- Create: `docs/process/contract-change-policy.md`
- Create: `docs/rfcs/README.md`
- Modify: `contract_tools/Cargo.toml`
- Create: `contract_tools/src/semantics.rs`
- Create: `contract_tools/src/registry.rs`
- Modify: `contract_tools/src/lib.rs`
- Create: `contract_tools/tests/registry_gate_tests.rs`

- [ ] **Step 1: Write the failing registry and semantics tests**

```rust
// contract_tools/tests/registry_gate_tests.rs
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
    assert!(registry.codes.iter().all(|code| matches!(code.status.as_str(), "active" | "deprecated" | "removed")));
}

#[test]
fn semantics_docs_declare_asset_references() {
    let manifest = load_manifest(contract_manifest_path()).unwrap();
    let doc = load_semantics_doc(resolve_asset_path(&manifest, "path_semantics").unwrap()).unwrap();
    assert!(doc.asset_refs.iter().any(|item| item == "schema/diagnostic-item.schema.json"));
}
```

- [ ] **Step 2: Run the registry test to verify it fails**

Run: `cargo test -p contract_tools --test registry_gate_tests -v`

Expected: FAIL because the registry asset and loader do not exist yet.

- [ ] **Step 3: Implement registry loading and semantics metadata checks**

```rust
// contract_tools/src/registry.rs
use serde::Deserialize;
use std::{fs, path::Path};

#[derive(Debug, Deserialize)]
pub struct ErrorCode {
    pub id: String,
    pub status: String,
    pub summary: String,
}

#[derive(Debug, Deserialize)]
pub struct ErrorRegistry {
    pub codes: Vec<ErrorCode>,
}

pub fn load_registry(path: impl AsRef<Path>) -> anyhow::Result<ErrorRegistry> {
    let raw = fs::read_to_string(path)?;
    Ok(serde_yaml::from_str(&raw)?)
}

pub fn run_registry_gates(manifest_path: &str) -> anyhow::Result<()> {
    let manifest = crate::manifest::load_manifest(manifest_path)?;
    let registry_path = manifest.data.assets.get("error_registry").expect("manifest includes error_registry");
    let registry = load_registry(registry_path)?;
    anyhow::ensure!(!registry.codes.is_empty(), "registry must contain at least one error code");
    Ok(())
}
```

```rust
// contract_tools/src/semantics.rs
use anyhow::Context;
use std::{fs, path::Path};

#[derive(Debug)]
pub struct SemanticsDoc {
    pub asset_refs: Vec<String>,
}

pub fn load_semantics_doc(path: impl AsRef<Path>) -> anyhow::Result<SemanticsDoc> {
    let raw = fs::read_to_string(path)?;
    let parts = raw.splitn(3, "---").collect::<Vec<_>>();
    anyhow::ensure!(parts.len() >= 3, "semantics docs must start with YAML frontmatter");
    let header = parts[1];
    let value: serde_yaml::Value = serde_yaml::from_str(&header)?;
    let refs = value["asset_refs"]
        .as_sequence()
        .into_iter()
        .flatten()
        .filter_map(|item| item.as_str().map(ToOwned::to_owned))
        .collect();
    Ok(SemanticsDoc { asset_refs: refs })
}

pub fn run_semantics_gates(manifest_path: &str) -> anyhow::Result<()> {
    let manifest = crate::manifest::load_manifest(manifest_path)?;
    for key in ["validation_semantics", "path_semantics", "compatibility_semantics"] {
        let doc = load_semantics_doc(crate::manifest::resolve_asset_path(&manifest, key)?)?;
        for asset_ref in doc.asset_refs {
            let asset_path = manifest.contracts_root.join(&asset_ref);
            anyhow::ensure!(asset_path.exists(), "semantic asset ref missing: {}", asset_ref);
        }
    }
    let _ = manifest.data.assets.get("compatibility_classes").context("missing compatibility_classes asset")?;
    let _ = manifest.data.assets.get("upgrade_rules").context("missing upgrade_rules asset")?;
    Ok(())
}
```

```markdown
<!-- contracts/semantics/path-conventions.md -->
---
asset_refs:
  - schema/diagnostic-item.schema.json
---

# Path Conventions

Paths use slash-prefixed logical locations such as `/metadata/document_id`.
```

- [ ] **Step 4: Author the registry and governance documents**

```yaml
# contracts/errors/error-registry.yaml
codes:
  - id: AF0001
    status: active
    summary: document_id is required
  - id: AF0002
    status: active
    summary: diagnostics array is required
```

```yaml
# contracts/manifest.yaml
assets:
  error_registry: errors/error-registry.yaml
  validation_semantics: semantics/validation.md
  path_semantics: semantics/path-conventions.md
  compatibility_semantics: semantics/compatibility.md
```

```markdown
# docs/process/contract-change-policy.md

This process applies to contract-affecting changes only.
Changes that alter contract meaning, evidence, or compatibility claims require ADR/RFC review and compatibility classification.
```

- [ ] **Step 5: Run the registry tests and a full test pass**

Run: `cargo test -p contract_tools --test registry_gate_tests -v`

Expected: PASS

Run: `cargo test -p contract_tools -v`

Expected: PASS

- [ ] **Step 6: Commit semantics, registry, and governance docs**

```bash
git add contracts/semantics/validation.md contracts/semantics/path-conventions.md contracts/semantics/compatibility.md contracts/errors/error-registry.yaml contracts/manifest.yaml docs/adr/README.md docs/adr/0001-contract-bundle-source-of-truth.md docs/adr/0002-bundle-version-is-public-axis.md docs/process/contract-change-policy.md docs/rfcs/README.md contract_tools/Cargo.toml contract_tools/src/lib.rs contract_tools/src/semantics.rs contract_tools/src/registry.rs contract_tools/tests/registry_gate_tests.rs
git commit -m "feat: add semantics, registry, and governance assets"
```

## Task 5: Add Normative Fixtures and Expected Evidence

**Files:**
- Create: `contracts/fixtures/index.yaml`
- Create: `contracts/fixtures/valid/minimal-authoring-ir.json`
- Create: `contracts/fixtures/invalid/missing-document-id.json`
- Create: `contracts/fixtures/expected/missing-document-id.report.json`
- Create: `contracts/fixtures/service-envelope/minimal-success.json`
- Create: `contracts/fixtures/evolution/additive-compatible.yaml`
- Create: `contracts/fixtures/evolution/incompatible-path-change.yaml`
- Modify: `contracts/manifest.yaml`
- Create: `contract_tools/src/fixtures.rs`
- Modify: `contract_tools/src/lib.rs`
- Create: `contract_tools/tests/fixture_gate_tests.rs`

- [ ] **Step 1: Write the failing fixture catalog tests**

```rust
// contract_tools/tests/fixture_gate_tests.rs
use contract_tools::{
    contract_manifest_path,
    fixtures::load_fixture_catalog,
    manifest::{load_manifest, resolve_asset_path},
};

#[test]
fn fixture_catalog_pairs_invalid_inputs_with_expected_reports() {
    let manifest = load_manifest(contract_manifest_path()).unwrap();
    let catalog = load_fixture_catalog(resolve_asset_path(&manifest, "fixture_catalog").unwrap()).unwrap();
    let case = catalog.cases.iter().find(|case| case.id == "missing-document-id").unwrap();
    assert_eq!(case.input, "fixtures/invalid/missing-document-id.json");
    assert_eq!(case.expected, Some("fixtures/expected/missing-document-id.report.json".into()));
}
```

- [ ] **Step 2: Run the fixture tests to verify they fail**

Run: `cargo test -p contract_tools --test fixture_gate_tests -v`

Expected: FAIL because the fixture catalog and loader do not exist yet.

- [ ] **Step 3: Implement fixture catalog loading**

```rust
// contract_tools/src/fixtures.rs
use serde::Deserialize;
use std::{fs, path::Path};

#[derive(Debug, Deserialize)]
pub struct FixtureCase {
    pub id: String,
    pub input: String,
    pub expected: Option<String>,
    pub category: String,
    pub compatibility_class: Option<String>,
    pub upgrade_rules: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct FixtureCatalog {
    pub cases: Vec<FixtureCase>,
}

pub fn load_fixture_catalog(path: impl AsRef<Path>) -> anyhow::Result<FixtureCatalog> {
    let raw = fs::read_to_string(path)?;
    Ok(serde_yaml::from_str(&raw)?)
}

pub fn run_fixture_gates(manifest_path: &str) -> anyhow::Result<()> {
    let manifest = crate::manifest::load_manifest(manifest_path)?;
    let fixture_catalog = crate::manifest::resolve_asset_path(&manifest, "fixture_catalog")?;
    let catalog = load_fixture_catalog(fixture_catalog)?;
    anyhow::ensure!(!catalog.cases.is_empty(), "fixture catalog must not be empty");
    anyhow::ensure!(
        catalog.cases.iter().any(|case| case.id == "additive-compatible") &&
        catalog.cases.iter().any(|case| case.id == "incompatible-path-change"),
        "fixture catalog must include compatible and incompatible evolution cases"
    );
    Ok(())
}
```

- [ ] **Step 4: Author the canonical fixture set**

```yaml
# contracts/fixtures/index.yaml
cases:
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
    upgrade_rules: [fixture_updates_required]
    input: fixtures/evolution/additive-compatible.yaml
  - id: incompatible-path-change
    category: evolution
    compatibility_class: behavior_changing_incompatible
    upgrade_rules: [migration_notes_required, executable_checks_required]
    input: fixtures/evolution/incompatible-path-change.yaml
```

```json
// contracts/fixtures/valid/minimal-authoring-ir.json
{
  "kind": "authoring-ir",
  "schema_version": "0.1.0",
  "metadata": { "document_id": "demo-doc" },
  "notetypes": [],
  "notes": []
}
```

```json
// contracts/fixtures/expected/missing-document-id.report.json
{
  "kind": "validation-report",
  "status": "invalid",
  "diagnostics": [
    {
      "level": "error",
      "code": "AF0001",
      "path": "/metadata/document_id",
      "message": "document_id is required"
    }
  ]
}
```

```yaml
# contracts/manifest.yaml
assets:
  fixture_catalog: fixtures/index.yaml
```

- [ ] **Step 5: Run fixture tests and the full test suite**

Run: `cargo test -p contract_tools --test fixture_gate_tests -v`

Expected: PASS

Run: `cargo test -p contract_tools -v`

Expected: PASS

- [ ] **Step 6: Commit the normative fixtures**

```bash
git add contracts/fixtures/index.yaml contracts/fixtures/valid/minimal-authoring-ir.json contracts/fixtures/invalid/missing-document-id.json contracts/fixtures/expected/missing-document-id.report.json contracts/fixtures/service-envelope/minimal-success.json contracts/fixtures/evolution/additive-compatible.yaml contracts/fixtures/evolution/incompatible-path-change.yaml contracts/manifest.yaml contract_tools/src/lib.rs contract_tools/src/fixtures.rs contract_tools/tests/fixture_gate_tests.rs
git commit -m "feat: add normative phase 1 fixtures"
```

## Task 6: Build the Executable Verification Library and Internal CLI

**Files:**
- Modify: `contract_tools/Cargo.toml`
- Create: `contract_tools/src/gates.rs`
- Create: `contract_tools/src/summary.rs`
- Create: `contract_tools/src/versioning.rs`
- Modify: `contract_tools/src/main.rs`
- Modify: `contract_tools/src/lib.rs`
- Create: `contract_tools/tests/versioning_gate_tests.rs`
- Create: `contract_tools/tests/cli_tests.rs`

- [ ] **Step 1: Write the failing CLI verification tests**

```rust
// contract_tools/tests/cli_tests.rs
use assert_cmd::Command;

#[test]
fn verify_command_succeeds_for_the_repo_contract_bundle() {
    Command::cargo_bin("contract_tools")
        .unwrap()
        .args(["verify", "--manifest", contract_tools::contract_manifest_path().to_str().unwrap()])
        .assert()
        .success();
}
```

```rust
// contract_tools/tests/versioning_gate_tests.rs
use contract_tools::{contract_manifest_path, versioning::run_versioning_gates};

#[test]
fn versioning_gates_accept_known_evolution_classes_and_rules() {
    run_versioning_gates(contract_manifest_path().to_str().unwrap()).unwrap();
}
```

- [ ] **Step 2: Run the CLI test to verify it fails**

Run: `cargo test -p contract_tools --test cli_tests -v`

Expected: FAIL because the CLI does not expose the `verify` command yet.

Run: `cargo test -p contract_tools --test versioning_gate_tests -v`

Expected: FAIL because semantics/versioning/evolution verification is not implemented yet.

- [ ] **Step 3: Implement verification gates and the internal CLI**

```rust
// contract_tools/src/main.rs
use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Verify { #[arg(long)] manifest: String },
    Summary { #[arg(long)] manifest: String },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Verify { manifest } => contract_tools::gates::run_all(&manifest),
        Commands::Summary { manifest } => {
            println!("{}", contract_tools::summary::render(&manifest)?);
            Ok(())
        }
    }
}
```

```rust
// contract_tools/src/gates.rs
pub fn run_all(manifest_path: &str) -> anyhow::Result<()> {
    super::manifest::load_manifest(manifest_path)?;
    super::schema::run_schema_gates(manifest_path)?;
    super::semantics::run_semantics_gates(manifest_path)?;
    super::registry::run_registry_gates(manifest_path)?;
    super::fixtures::run_fixture_gates(manifest_path)?;
    super::versioning::run_versioning_gates(manifest_path)?;
    Ok(())
}
```

```rust
// contract_tools/src/summary.rs
pub fn render(manifest_path: &str) -> anyhow::Result<String> {
    let manifest = crate::manifest::load_manifest(manifest_path)?;
    Ok(format!(
        "bundle_version: {}\npublic_axis: {}",
        manifest.data.bundle_version, manifest.data.compatibility.public_axis
    ))
}
```

```rust
// contract_tools/src/versioning.rs
use serde::Deserialize;
use std::{collections::BTreeSet, fs};

#[derive(Debug, Deserialize)]
pub struct CompatibilityClasses {
    pub classes: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpgradeRules {
    pub rules: Vec<UpgradeRule>,
}

#[derive(Debug, Deserialize)]
pub struct UpgradeRule {
    pub id: String,
}

pub fn run_versioning_gates(manifest_path: &str) -> anyhow::Result<()> {
    let manifest = crate::manifest::load_manifest(manifest_path)?;
    let classes: CompatibilityClasses = serde_yaml::from_str(&fs::read_to_string(
        crate::manifest::resolve_asset_path(&manifest, "compatibility_classes")?,
    )?)?;
    let rules: UpgradeRules = serde_yaml::from_str(&fs::read_to_string(
        crate::manifest::resolve_asset_path(&manifest, "upgrade_rules")?,
    )?)?;
    let catalog = crate::fixtures::load_fixture_catalog(
        crate::manifest::resolve_asset_path(&manifest, "fixture_catalog")?,
    )?;

    let class_ids = classes.classes.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let rule_ids = rules.rules.iter().map(|rule| rule.id.as_str()).collect::<BTreeSet<_>>();

    for case in catalog.cases.iter().filter(|case| case.category == "evolution") {
        let class_id = case.compatibility_class.as_deref().expect("evolution cases require compatibility_class");
        anyhow::ensure!(class_ids.contains(class_id), "unknown compatibility class: {class_id}");
        for rule_id in case.upgrade_rules.clone().unwrap_or_default() {
            anyhow::ensure!(rule_ids.contains(rule_id.as_str()), "unknown upgrade rule: {rule_id}");
        }
    }

    anyhow::ensure!(
        catalog.cases.iter().any(|case| case.compatibility_class.as_deref() == Some("additive_compatible")) &&
        catalog.cases.iter().any(|case| case.compatibility_class.as_deref() == Some("behavior_changing_incompatible")),
        "catalog must include both compatible and incompatible evolution examples"
    );
    Ok(())
}
```

```toml
# contract_tools/Cargo.toml
[dependencies]
anyhow = "1"
clap = { version = "4", features = ["derive"] }
jsonschema = "0.18"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"

[dev-dependencies]
assert_cmd = "2"
predicates = "3"
```

- [ ] **Step 4: Add a failure-path CLI test and then run all CLI tests**

```rust
#[test]
fn verify_command_fails_when_manifest_is_missing() {
    Command::cargo_bin("contract_tools")
        .unwrap()
        .args(["verify", "--manifest", "contracts/missing.yaml"])
        .assert()
        .failure();
}
```

Run: `cargo test -p contract_tools --test cli_tests -v`

Expected: PASS

- [ ] **Step 5: Run a repository-level verify command**

Run: `cargo run -p contract_tools -- verify --manifest "$(pwd)/contracts/manifest.yaml"`

Expected: exit code `0` with a short verification success summary.

- [ ] **Step 6: Commit the verification library and CLI**

```bash
git add contract_tools/Cargo.toml contract_tools/src/gates.rs contract_tools/src/summary.rs contract_tools/src/versioning.rs contract_tools/src/main.rs contract_tools/src/lib.rs contract_tools/tests/versioning_gate_tests.rs contract_tools/tests/cli_tests.rs
git commit -m "feat: add contract verification cli"
```

## Task 7: Add Packaging, Artifact Checks, and GitHub Workflows

**Files:**
- Modify: `contract_tools/Cargo.toml`
- Create: `contract_tools/src/package.rs`
- Modify: `contract_tools/src/main.rs`
- Modify: `contract_tools/src/lib.rs`
- Create: `contract_tools/tests/package_tests.rs`
- Create: `.github/workflows/contract-ci.yml`
- Create: `.github/workflows/contract-release.yml`
- Modify: `README.md`

- [ ] **Step 1: Write the failing package artifact test**

```rust
// contract_tools/tests/package_tests.rs
use flate2::read::GzDecoder;
use std::fs::File;
use tar::Archive;
use tempfile::tempdir;

#[test]
fn package_command_emits_a_bundle_artifact_with_manifest_and_contract_assets() {
    let dir = tempdir().unwrap();
    contract_tools::package::build_artifact(contract_tools::contract_manifest_path().to_str().unwrap(), dir.path()).unwrap();
    let artifact = dir.path().join("anki-forge-contract-bundle-0.1.0.tar.gz");
    let file = File::open(artifact).unwrap();
    let mut archive = Archive::new(GzDecoder::new(file));
    let entries = archive.entries().unwrap().map(|entry| entry.unwrap().path().unwrap().display().to_string()).collect::<Vec<_>>();
    assert!(entries.iter().any(|path| path.ends_with("contracts/manifest.yaml")));
    assert!(entries.iter().any(|path| path.ends_with("contracts/schema/authoring-ir.schema.json")));
}
```

- [ ] **Step 2: Run the package test to verify it fails**

Run: `cargo test -p contract_tools --test package_tests -v`

Expected: FAIL because packaging support does not exist yet.

- [ ] **Step 3: Implement artifact packaging and expose the CLI command**

```rust
// contract_tools/src/main.rs
#[derive(Subcommand)]
enum Commands {
    Verify { #[arg(long)] manifest: String },
    Summary { #[arg(long)] manifest: String },
    Package { #[arg(long)] manifest: String, #[arg(long)] out_dir: String },
}
```

```rust
// contract_tools/src/package.rs
pub fn build_artifact(manifest_path: &str, out_dir: &std::path::Path) -> anyhow::Result<()> {
    let manifest = crate::manifest::load_manifest(manifest_path)?;
    let out = out_dir.join(format!("anki-forge-contract-bundle-{}.tar.gz", manifest.data.bundle_version));
    let file = std::fs::File::create(out)?;
    let encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
    let mut builder = tar::Builder::new(encoder);
    builder.append_path_with_name(&manifest.path, "contracts/manifest.yaml")?;
    for asset_path in manifest.data.assets.values() {
        let source = manifest.contracts_root.join(asset_path);
        builder.append_path_with_name(&source, format!("contracts/{asset_path}"))?;
    }
    builder.finish()?;
    Ok(())
}
```

```toml
# contract_tools/Cargo.toml
[dependencies]
anyhow = "1"
clap = { version = "4", features = ["derive"] }
flate2 = "1"
jsonschema = "0.18"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
tar = "0.4"

[dev-dependencies]
assert_cmd = "2"
predicates = "3"
tempfile = "3"
```

- [ ] **Step 4: Add CI and release workflows**

```yaml
# .github/workflows/contract-ci.yml
name: contract-ci

on:
  pull_request:
  push:
    branches: [main]

jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo fmt --all -- --check
      - run: cargo clippy -p contract_tools --all-targets -- -D warnings
      - run: cargo test -p contract_tools
      - run: cargo run -p contract_tools -- verify --manifest "$GITHUB_WORKSPACE/contracts/manifest.yaml"
```

```yaml
# .github/workflows/contract-release.yml
name: contract-release

on:
  workflow_dispatch:

jobs:
  package:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo run -p contract_tools -- package --manifest "$GITHUB_WORKSPACE/contracts/manifest.yaml" --out-dir dist
```

- [ ] **Step 5: Run packaging and the full repository verification sequence**

Run: `cargo test -p contract_tools --test package_tests -v`

Expected: PASS

Run: `cargo fmt --all -- --check`

Expected: PASS

Run: `cargo clippy -p contract_tools --all-targets -- -D warnings`

Expected: PASS

Run: `cargo test -p contract_tools -v`

Expected: PASS

Run: `cargo run -p contract_tools -- package --manifest "$(pwd)/contracts/manifest.yaml" --out-dir dist`

Expected: package artifact created in `dist/`

- [ ] **Step 6: Commit packaging and CI automation**

```bash
git add contract_tools/Cargo.toml contract_tools/src/package.rs contract_tools/src/main.rs contract_tools/src/lib.rs contract_tools/tests/package_tests.rs .github/workflows/contract-ci.yml .github/workflows/contract-release.yml README.md
git commit -m "feat: add contract packaging and ci workflows"
```

## Task 8: Finalize Release Readiness and Exit-Criteria Evidence

**Files:**
- Modify: `contracts/manifest.yaml`
- Modify: `README.md`
- Modify: `docs/process/contract-change-policy.md`
- Create: `docs/superpowers/checklists/phase-1-exit-evidence.md`
- Modify: `.github/workflows/contract-ci.yml`
- Modify: `contract_tools/src/summary.rs`

- [ ] **Step 1: Write the failing readiness smoke check**

```rust
// contract_tools/tests/cli_tests.rs
#[test]
fn summary_command_prints_bundle_version_and_public_axis() {
    use assert_cmd::Command;

    Command::cargo_bin("contract_tools")
        .unwrap()
        .args(["summary", "--manifest", contract_tools::contract_manifest_path().to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicates::str::contains("bundle_version: 0.1.0"))
        .stdout(predicates::str::contains("public_axis: bundle_version"))
        .stdout(predicates::str::contains("component_versions:"))
        .stdout(predicates::str::contains("fixture_catalog: fixtures/index.yaml"));
}
```

- [ ] **Step 2: Run the readiness smoke check to verify it fails**

Run: `cargo test -p contract_tools --test cli_tests summary_command_prints_bundle_version_and_public_axis -v`

Expected: FAIL because Task 6 summary output does not yet include component versions or key asset entries.

- [ ] **Step 3: Stabilize release evidence and exit-criteria documentation**

```rust
// contract_tools/src/summary.rs
pub fn render(manifest_path: &str) -> anyhow::Result<String> {
    let manifest = crate::manifest::load_manifest(manifest_path)?;
    let component_versions = manifest
        .data
        .component_versions
        .iter()
        .map(|(name, version)| format!("{name}={version}"))
        .collect::<Vec<_>>()
        .join(", ");

    Ok(format!(
        "bundle_version: {}\npublic_axis: {}\ncomponent_versions: {}\nfixture_catalog: {}",
        manifest.data.bundle_version,
        manifest.data.compatibility.public_axis,
        component_versions,
        manifest.data.assets.get("fixture_catalog").cloned().unwrap_or_default()
    ))
}
```

```markdown
# docs/superpowers/checklists/phase-1-exit-evidence.md

- [ ] `contracts/manifest.yaml` declares bundle and component versions
- [ ] all schema files validate
- [ ] registry gates pass
- [ ] fixture conformance passes
- [ ] `cargo run -p contract_tools -- verify --manifest "$(pwd)/contracts/manifest.yaml"` passes
- [ ] package artifact can be built
```

```markdown
# README.md

## Phase 1 verification

Run:

`cargo run -p contract_tools -- verify --manifest "$(pwd)/contracts/manifest.yaml"`
```

- [ ] **Step 4: Run the full release-readiness sequence**

Run: `cargo test -p contract_tools --test cli_tests -v`

Expected: PASS

Run: `cargo test -p contract_tools -v`

Expected: PASS

Run: `cargo run -p contract_tools -- summary --manifest "$(pwd)/contracts/manifest.yaml"`

Expected: prints bundle version, component versions, and public compatibility axis.

- [ ] **Step 5: Commit the exit-evidence pass**

```bash
git add contracts/manifest.yaml README.md docs/process/contract-change-policy.md docs/superpowers/checklists/phase-1-exit-evidence.md .github/workflows/contract-ci.yml contract_tools/src/summary.rs contract_tools/tests/cli_tests.rs
git commit -m "docs: record phase 1 exit evidence"
```

## Final Verification Sequence

Run these commands before calling `Phase 1` complete:

```bash
cargo fmt --all -- --check
cargo clippy -p contract_tools --all-targets -- -D warnings
cargo test -p contract_tools -v
cargo run -p contract_tools -- verify --manifest "$(pwd)/contracts/manifest.yaml"
cargo run -p contract_tools -- summary --manifest "$(pwd)/contracts/manifest.yaml"
cargo run -p contract_tools -- package --manifest "$(pwd)/contracts/manifest.yaml" --out-dir dist
```

Expected final state:

- all commands pass
- `contracts/` remains the normative source of truth
- `contract_tools/` only verifies and packages the contract bundle
- no operation-specific product surface has been introduced

## Guardrails During Execution

- Do not add writer logic, authoring builder APIs, or binding-specific surfaces.
- Do not move normative meaning into Rust types or CLI behavior.
- Do not let fixture layout depend on one test runner's private assumptions.
- Do not let internal component versions become external compatibility claims.
- Keep commits small and aligned to the task boundaries above.
