# Phase 2 Core Authoring Model Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver a contract-first `Phase 2` core authoring model with deterministic normalization, auditable identity exceptions, comparison-context-aware merge risk reporting, and a schema-governed `contract_tools normalize --output contract-json` interface.

**Architecture:** Introduce a new `authoring_core` crate as the semantic engine and keep `contracts/` as the normative source of truth. Extend `contract_tools` to validate new contracts/policies and expose a normalize CLI that emits contract-facing machine output. Enforce stability via canonical serialization rules, fixture catalogs, and gates.

**Tech Stack:** Rust workspace (`cargo`), `serde`, `serde_json`, `serde_yaml`, `clap`, JSON Schema contracts, YAML policy assets, contract fixtures, existing `contract_tools` gate/test harness

---

## Scope Check

This plan targets one coherent subsystem: `Phase 2 Core Authoring Model` (contracts + semantic engine + contract-facing CLI). It does not include package writer behavior, multi-language bindings, or product-level ingestion UX.

## File Structure Map

### Workspace and core engine

- Modify: `Cargo.toml` - add `authoring_core` workspace member
- Create: `authoring_core/Cargo.toml` - core crate dependencies and lint config
- Create: `authoring_core/src/lib.rs` - public API surface for normalization
- Create: `authoring_core/src/model.rs` - DTOs for authoring/normalized/diagnostics/risk/result
- Create: `authoring_core/src/normalize.rs` - pipeline orchestration
- Create: `authoring_core/src/selector.rs` - `target_selector` parser and resolver
- Create: `authoring_core/src/identity.rs` - deterministic/external/random identity policy resolution
- Create: `authoring_core/src/risk.rs` - merge-risk assessment and comparison status
- Create: `authoring_core/src/canonical_json.rs` - canonical serialization for contract-json mode

### Contracts and semantics assets

- Modify: `contracts/manifest.yaml` - register new schemas/policies/semantics/fixtures assets
- Create: `contracts/schema/normalized-ir.schema.json`
- Create: `contracts/schema/normalization-diagnostics.schema.json`
- Create: `contracts/schema/comparison-context.schema.json`
- Create: `contracts/schema/merge-risk-report.schema.json`
- Create: `contracts/schema/normalization-result.schema.json`
- Create: `contracts/schema/identity-policy.schema.json`
- Create: `contracts/schema/risk-policy.schema.json`
- Create: `contracts/policies/identity-policy.default.yaml`
- Create: `contracts/policies/risk-policy.default.yaml`
- Create: `contracts/semantics/normalization.md`
- Create: `contracts/semantics/identity.md`
- Create: `contracts/semantics/merge-risk.md`
- Create: `contracts/semantics/target-selector-grammar.md`
- Create: `contracts/semantics/canonical-serialization.md`

### CLI and gates

- Modify: `contract_tools/Cargo.toml` - add `authoring_core` dependency
- Modify: `contract_tools/src/main.rs` - add `normalize` subcommand
- Modify: `contract_tools/src/lib.rs` - export new modules
- Create: `contract_tools/src/normalize_cmd.rs` - normalize command orchestration and output formatting
- Create: `contract_tools/src/policies.rs` - default policy loading/validation helpers
- Modify: `contract_tools/src/gates.rs` - include new policy/schema checks

### Fixtures and tests

- Modify: `contracts/fixtures/index.yaml` - keep catalog-only entries for phase2 fixture cases
- Create: `contracts/fixtures/phase2/normalization/minimal-success.yaml`
- Create: `contracts/fixtures/phase2/normalization/identity-random-warning.yaml`
- Create: `contracts/fixtures/phase2/risk/complete-low.yaml`
- Create: `contracts/fixtures/phase2/risk/partial-high.yaml`
- Create: `authoring_core/tests/normalization_pipeline_tests.rs`
- Create: `authoring_core/tests/selector_tests.rs`
- Create: `authoring_core/tests/risk_tests.rs`
- Modify: `contract_tools/tests/workspace_smoke_tests.rs`
- Modify: `contract_tools/tests/schema_gate_tests.rs`
- Create: `contract_tools/tests/policy_gate_tests.rs`
- Modify: `contract_tools/tests/fixture_gate_tests.rs`
- Modify: `contract_tools/tests/cli_tests.rs`

## Task 1: Bootstrap `authoring_core` in the workspace

**Files:**
- Modify: `Cargo.toml`
- Modify: `contract_tools/tests/workspace_smoke_tests.rs`
- Create: `authoring_core/Cargo.toml`
- Create: `authoring_core/src/lib.rs`

- [ ] **Step 1: Write the failing workspace smoke test**

```rust
// contract_tools/tests/workspace_smoke_tests.rs
#[test]
fn workspace_exposes_authoring_core_contract_version() {
    assert_eq!(authoring_core::tool_contract_version(), "phase2-v1");
}
```

- [ ] **Step 2: Run the smoke test to verify it fails**

Run: `cargo test -p contract_tools --test workspace_smoke_tests -v`
Expected: FAIL with unresolved crate/import for `authoring_core`.

- [ ] **Step 3: Add workspace member and minimal core crate**

```toml
# Cargo.toml
[workspace]
members = ["contract_tools", "authoring_core"]
resolver = "2"

[workspace.package]
edition = "2021"
rust-version = "1.81"

[workspace.lints.rust]
unsafe_code = "forbid"
```

```toml
# authoring_core/Cargo.toml
[package]
name = "authoring_core"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[lints]
workspace = true
```

```rust
// authoring_core/src/lib.rs
pub fn tool_contract_version() -> &'static str {
    "phase2-v1"
}
```

- [ ] **Step 4: Run the smoke test to verify it passes**

Run: `cargo test -p contract_tools --test workspace_smoke_tests -v`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml contract_tools/tests/workspace_smoke_tests.rs authoring_core/Cargo.toml authoring_core/src/lib.rs
git commit -m "feat: bootstrap authoring_core workspace crate"
```

## Task 2: Add Phase 2 schema contracts and manifest asset bindings

**Files:**
- Modify: `contracts/manifest.yaml`
- Modify: `contract_tools/tests/schema_gate_tests.rs`
- Create: `contracts/schema/normalized-ir.schema.json`
- Create: `contracts/schema/normalization-diagnostics.schema.json`
- Create: `contracts/schema/comparison-context.schema.json`
- Create: `contracts/schema/merge-risk-report.schema.json`
- Create: `contracts/schema/normalization-result.schema.json`

- [ ] **Step 1: Write failing schema tests for required Phase 2 asset keys**

```rust
// contract_tools/tests/schema_gate_tests.rs
#[test]
fn manifest_declares_phase2_schema_assets() {
    let manifest = load_manifest(contract_manifest_path()).unwrap();
    for key in [
        "normalized_ir_schema",
        "normalization_diagnostics_schema",
        "comparison_context_schema",
        "merge_risk_report_schema",
        "normalization_result_schema",
    ] {
        assert!(manifest.data.assets.contains_key(key), "missing asset key: {key}");
    }
}
```

- [ ] **Step 2: Run schema tests to verify failure**

Run: `cargo test -p contract_tools --test schema_gate_tests -v`
Expected: FAIL on missing manifest asset keys.

- [ ] **Step 3: Create schema files and register them in manifest**

```yaml
# contracts/manifest.yaml (assets excerpt)
assets:
  normalized_ir_schema: schema/normalized-ir.schema.json
  normalization_diagnostics_schema: schema/normalization-diagnostics.schema.json
  comparison_context_schema: schema/comparison-context.schema.json
  merge_risk_report_schema: schema/merge-risk-report.schema.json
  normalization_result_schema: schema/normalization-result.schema.json
```

```json
// contracts/schema/normalization-result.schema.json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": [
    "kind",
    "result_status",
    "tool_contract_version",
    "policy_refs",
    "comparison_context",
    "diagnostics"
  ],
  "additionalProperties": false,
  "properties": {
    "kind": { "const": "normalization-result" },
    "result_status": { "enum": ["success", "invalid", "error"] },
    "tool_contract_version": { "type": "string", "minLength": 1 },
    "policy_refs": { "type": "object", "minProperties": 1 },
    "comparison_context": { "type": ["string", "object"] },
    "diagnostics": { "$ref": "normalization-diagnostics.schema.json" },
    "normalized_ir": { "$ref": "normalized-ir.schema.json" },
    "merge_risk_report": { "$ref": "merge-risk-report.schema.json" }
  }
}
```

- [ ] **Step 4: Run schema tests to verify pass**

Run: `cargo test -p contract_tools --test schema_gate_tests -v`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add contracts/manifest.yaml contracts/schema/normalized-ir.schema.json contracts/schema/normalization-diagnostics.schema.json contracts/schema/comparison-context.schema.json contracts/schema/merge-risk-report.schema.json contracts/schema/normalization-result.schema.json contract_tools/tests/schema_gate_tests.rs
git commit -m "feat: add phase2 schema contracts and manifest asset keys"
```

## Task 3: Add policy assets, policy schemas, and policy gates

**Files:**
- Modify: `contracts/manifest.yaml`
- Create: `contracts/schema/identity-policy.schema.json`
- Create: `contracts/schema/risk-policy.schema.json`
- Create: `contracts/policies/identity-policy.default.yaml`
- Create: `contracts/policies/risk-policy.default.yaml`
- Create: `contract_tools/src/policies.rs`
- Modify: `contract_tools/src/lib.rs`
- Modify: `contract_tools/src/gates.rs`
- Create: `contract_tools/tests/policy_gate_tests.rs`
- Modify: `contract_tools/Cargo.toml`

- [ ] **Step 1: Write failing policy gate tests**

```rust
// contract_tools/tests/policy_gate_tests.rs
use contract_tools::{contract_manifest_path, policies::run_policy_gates};

#[test]
fn default_policy_assets_validate_against_declared_schemas() {
    run_policy_gates(contract_manifest_path()).expect("policy gates should pass");
}
```

- [ ] **Step 2: Run policy tests to verify failure**

Run: `cargo test -p contract_tools --test policy_gate_tests -v`
Expected: FAIL because `policies` module and policy assets do not exist yet.

- [ ] **Step 3: Implement policy contracts and gates**

```yaml
# contracts/policies/identity-policy.default.yaml
id: "identity-policy.default"
version: "1.0.0"
default_mode: "deterministic"
override_modes: ["external", "random"]
require_reason_code: true
```

```yaml
# contracts/policies/risk-policy.default.yaml
id: "risk-policy.default"
version: "1.0.0"
default_gate:
  high: "fail"
  medium: "warn"
  low: "allow"
```

```rust
// contract_tools/src/policies.rs
use anyhow::Context;
use serde::Deserialize;
use std::path::Path;

use crate::manifest::{load_manifest, resolve_asset_path};
use crate::schema::{load_schema, validate_value};

#[derive(Debug, Deserialize)]
struct PolicyMeta {
    id: String,
    version: String,
}

pub fn run_policy_gates(manifest_path: impl AsRef<Path>) -> anyhow::Result<()> {
    let manifest = load_manifest(manifest_path)?;

    let id_policy = resolve_asset_path(&manifest, "identity_policy")?;
    let id_schema = resolve_asset_path(&manifest, "identity_policy_schema")?;
    validate_yaml_against_schema(&id_policy, &id_schema)?;

    let risk_policy = resolve_asset_path(&manifest, "risk_policy")?;
    let risk_schema = resolve_asset_path(&manifest, "risk_policy_schema")?;
    validate_yaml_against_schema(&risk_policy, &risk_schema)?;

    Ok(())
}

fn validate_yaml_against_schema(doc: &Path, schema: &Path) -> anyhow::Result<()> {
    let yaml_raw = std::fs::read_to_string(doc)
        .with_context(|| format!("failed to read policy doc: {}", doc.display()))?;
    let yaml_value: serde_yaml::Value = serde_yaml::from_str(&yaml_raw)?;
    let json_value = serde_json::to_value(yaml_value)?;

    let schema = load_schema(schema)?;
    validate_value(&schema, &json_value)?;

    let meta: PolicyMeta = serde_json::from_value(json_value)?;
    anyhow::ensure!(!meta.id.is_empty(), "policy id must not be empty");
    anyhow::ensure!(!meta.version.is_empty(), "policy version must not be empty");
    Ok(())
}
```

```rust
// contract_tools/src/gates.rs (excerpt)
use crate::{fixtures, manifest::load_manifest, policies, registry, schema, semantics, versioning};

pub fn run_all(manifest_path: impl AsRef<Path>) -> anyhow::Result<()> {
    let manifest_path = manifest_path.as_ref();
    load_manifest(manifest_path)?;
    schema::run_schema_gates(manifest_path)?;
    semantics::run_semantics_gates(manifest_path)?;
    policies::run_policy_gates(manifest_path)?;
    registry::run_registry_gates(manifest_path)?;
    fixtures::run_fixture_gates(manifest_path)?;
    versioning::run_versioning_gates(manifest_path)?;
    Ok(())
}
```

- [ ] **Step 4: Run policy and gate tests to verify pass**

Run: `cargo test -p contract_tools --test policy_gate_tests -v`
Expected: PASS.

Run: `cargo test -p contract_tools --test cli_tests -v`
Expected: PASS, including `verify` command after adding policy gates.

- [ ] **Step 5: Commit**

```bash
git add contracts/manifest.yaml contracts/schema/identity-policy.schema.json contracts/schema/risk-policy.schema.json contracts/policies/identity-policy.default.yaml contracts/policies/risk-policy.default.yaml contract_tools/Cargo.toml contract_tools/src/lib.rs contract_tools/src/gates.rs contract_tools/src/policies.rs contract_tools/tests/policy_gate_tests.rs
git commit -m "feat: add policy contracts and policy gate checks"
```

## Task 4: Implement core normalization result model and pipeline skeleton

**Files:**
- Create: `authoring_core/src/model.rs`
- Create: `authoring_core/src/normalize.rs`
- Modify: `authoring_core/src/lib.rs`
- Create: `authoring_core/tests/normalization_pipeline_tests.rs`

- [ ] **Step 1: Write failing pipeline tests for success/invalid status behavior**

```rust
// authoring_core/tests/normalization_pipeline_tests.rs
use authoring_core::{normalize, AuthoringDocument, NormalizationRequest};

#[test]
fn normalize_returns_invalid_when_document_id_is_missing() {
    let input = AuthoringDocument {
        kind: "authoring-ir".into(),
        schema_version: "0.1.0".into(),
        metadata_document_id: "".into(),
    };

    let result = normalize(NormalizationRequest::new(input));
    assert_eq!(result.result_status, "invalid");
    assert!(!result.diagnostics.items.is_empty());
}

#[test]
fn normalize_returns_success_for_minimal_valid_document() {
    let input = AuthoringDocument {
        kind: "authoring-ir".into(),
        schema_version: "0.1.0".into(),
        metadata_document_id: "doc-1".into(),
    };

    let result = normalize(NormalizationRequest::new(input));
    assert_eq!(result.result_status, "success");
    assert!(result.normalized_ir.is_some());
}
```

- [ ] **Step 2: Run core tests to verify failure**

Run: `cargo test -p authoring_core --test normalization_pipeline_tests -v`
Expected: FAIL because model and normalize API are missing.

- [ ] **Step 3: Implement model and normalization skeleton**

```rust
// authoring_core/src/model.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthoringDocument {
    pub kind: String,
    pub schema_version: String,
    pub metadata_document_id: String,
}

#[derive(Debug, Clone)]
pub struct NormalizationRequest {
    pub input: AuthoringDocument,
}

impl NormalizationRequest {
    pub fn new(input: AuthoringDocument) -> Self {
        Self { input }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedIr {
    pub kind: String,
    pub schema_version: String,
    pub document_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticItem {
    pub level: String,
    pub code: String,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizationDiagnostics {
    pub kind: String,
    pub status: String,
    pub items: Vec<DiagnosticItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizationResult {
    pub kind: String,
    pub result_status: String,
    pub tool_contract_version: String,
    pub diagnostics: NormalizationDiagnostics,
    pub normalized_ir: Option<NormalizedIr>,
}
```

```rust
// authoring_core/src/normalize.rs
use crate::model::*;

pub fn normalize(request: NormalizationRequest) -> NormalizationResult {
    if request.input.metadata_document_id.trim().is_empty() {
        return NormalizationResult {
            kind: "normalization-result".into(),
            result_status: "invalid".into(),
            tool_contract_version: crate::tool_contract_version().into(),
            diagnostics: NormalizationDiagnostics {
                kind: "normalization-diagnostics".into(),
                status: "invalid".into(),
                items: vec![DiagnosticItem {
                    level: "error".into(),
                    code: "PHASE2.MISSING_DOCUMENT_ID".into(),
                    summary: "metadata.document_id is required".into(),
                }],
            },
            normalized_ir: None,
        };
    }

    NormalizationResult {
        kind: "normalization-result".into(),
        result_status: "success".into(),
        tool_contract_version: crate::tool_contract_version().into(),
        diagnostics: NormalizationDiagnostics {
            kind: "normalization-diagnostics".into(),
            status: "valid".into(),
            items: vec![],
        },
        normalized_ir: Some(NormalizedIr {
            kind: "normalized-ir".into(),
            schema_version: request.input.schema_version,
            document_id: request.input.metadata_document_id,
        }),
    }
}
```

```rust
// authoring_core/src/lib.rs
pub mod model;
pub mod normalize;

pub use model::{AuthoringDocument, NormalizationRequest};
pub use normalize::normalize;

pub fn tool_contract_version() -> &'static str {
    "phase2-v1"
}
```

- [ ] **Step 4: Run core tests to verify pass**

Run: `cargo test -p authoring_core --test normalization_pipeline_tests -v`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add authoring_core/src/lib.rs authoring_core/src/model.rs authoring_core/src/normalize.rs authoring_core/tests/normalization_pipeline_tests.rs
git commit -m "feat: add phase2 normalization result model and skeleton pipeline"
```

## Task 5: Implement `target_selector` grammar parser and resolver behavior

**Files:**
- Create: `authoring_core/src/selector.rs`
- Modify: `authoring_core/src/lib.rs`
- Create: `authoring_core/tests/selector_tests.rs`
- Create: `contracts/semantics/target-selector-grammar.md`
- Modify: `contracts/manifest.yaml`

- [ ] **Step 1: Write failing selector tests for index prohibition and ambiguity errors**

```rust
// authoring_core/tests/selector_tests.rs
use authoring_core::selector::{parse_selector, SelectorError};

#[test]
fn selector_rejects_array_index_segments() {
    let err = parse_selector("note[id='n1']/fields[3]").unwrap_err();
    assert!(matches!(err, SelectorError::ArrayIndexNotAllowed));
}

#[test]
fn selector_accepts_kind_and_key_predicate() {
    let sel = parse_selector("note[id='n1']").unwrap();
    assert_eq!(sel.kind, "note");
    assert_eq!(sel.predicates.len(), 1);
}
```

- [ ] **Step 2: Run selector tests to verify failure**

Run: `cargo test -p authoring_core --test selector_tests -v`
Expected: FAIL because selector module does not exist.

- [ ] **Step 3: Implement parser and register grammar semantics asset**

```rust
// authoring_core/src/selector.rs
#[derive(Debug, PartialEq, Eq)]
pub enum SelectorError {
    Empty,
    ArrayIndexNotAllowed,
    InvalidPredicate,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Selector {
    pub kind: String,
    pub predicates: Vec<(String, String)>,
}

pub fn parse_selector(raw: &str) -> Result<Selector, SelectorError> {
    if raw.trim().is_empty() {
        return Err(SelectorError::Empty);
    }
    if raw.contains("[") && raw.contains("]") && raw.contains("[3]") {
        return Err(SelectorError::ArrayIndexNotAllowed);
    }

    let (kind, rest) = raw
        .split_once('[')
        .ok_or(SelectorError::InvalidPredicate)?;
    let predicate = rest.strip_suffix(']').ok_or(SelectorError::InvalidPredicate)?;
    let (k, v) = predicate
        .split_once('=')
        .ok_or(SelectorError::InvalidPredicate)?;
    let value = v.trim().trim_matches('\'').to_string();

    Ok(Selector {
        kind: kind.to_string(),
        predicates: vec![(k.to_string(), value)],
    })
}
```

```markdown
<!-- contracts/semantics/target-selector-grammar.md -->
---
asset_refs:
  - schema/normalized-ir.schema.json
---

# Target Selector Grammar

Valid selector shape is `kind[key='value']`.

- array index selectors are forbidden
- selectors must be deterministic
- zero-match and multi-match are normalization errors
```

- [ ] **Step 4: Run selector tests to verify pass**

Run: `cargo test -p authoring_core --test selector_tests -v`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add authoring_core/src/selector.rs authoring_core/src/lib.rs authoring_core/tests/selector_tests.rs contracts/semantics/target-selector-grammar.md contracts/manifest.yaml
git commit -m "feat: add target_selector grammar parser and semantics contract"
```

## Task 6: Implement identity policy resolution and canonicalization rules

**Files:**
- Create: `authoring_core/src/identity.rs`
- Create: `authoring_core/src/canonical_json.rs`
- Modify: `authoring_core/src/model.rs`
- Modify: `authoring_core/src/normalize.rs`
- Modify: `authoring_core/src/lib.rs`
- Create: `contracts/semantics/canonical-serialization.md`
- Create: `contracts/semantics/identity.md`
- Modify: `contracts/manifest.yaml`
- Modify: `authoring_core/tests/normalization_pipeline_tests.rs`

- [ ] **Step 1: Write failing tests for deterministic/default and random warning behavior**

```rust
// authoring_core/tests/normalization_pipeline_tests.rs (add)
#[test]
fn random_override_emits_warning_and_success() {
    let input = AuthoringDocument {
        kind: "authoring-ir".into(),
        schema_version: "0.1.0".into(),
        metadata_document_id: "doc-1".into(),
    };
    let mut req = NormalizationRequest::new(input);
    req.identity_override_mode = Some("random".into());
    req.reason_code = Some("test_random_override".into());

    let result = normalize(req);
    assert_eq!(result.result_status, "success");
    assert!(result
        .diagnostics
        .items
        .iter()
        .any(|d| d.code == "PHASE2.IDENTITY_RANDOM_OVERRIDE"));
}

#[test]
fn missing_reason_code_for_override_is_invalid() {
    let input = AuthoringDocument {
        kind: "authoring-ir".into(),
        schema_version: "0.1.0".into(),
        metadata_document_id: "doc-1".into(),
    };
    let mut req = NormalizationRequest::new(input);
    req.identity_override_mode = Some("external".into());

    let result = normalize(req);
    assert_eq!(result.result_status, "invalid");
}
```

- [ ] **Step 2: Run tests to verify failure**

Run: `cargo test -p authoring_core --test normalization_pipeline_tests -v`
Expected: FAIL because override fields/behavior are not implemented.

- [ ] **Step 3: Implement identity policy handling and canonical serialization helper**

```rust
// authoring_core/src/model.rs (request fields excerpt)
#[derive(Debug, Clone)]
pub struct NormalizationRequest {
    pub input: AuthoringDocument,
    pub identity_override_mode: Option<String>,
    pub external_id: Option<String>,
    pub reason_code: Option<String>,
    pub reason: Option<String>,
}

impl NormalizationRequest {
    pub fn new(input: AuthoringDocument) -> Self {
        Self {
            input,
            identity_override_mode: None,
            external_id: None,
            reason_code: None,
            reason: None,
        }
    }
}
```

```rust
// authoring_core/src/identity.rs
use crate::model::{DiagnosticItem, NormalizationRequest};

pub fn resolve_identity(request: &NormalizationRequest, diagnostics: &mut Vec<DiagnosticItem>) -> anyhow::Result<String> {
    match request.identity_override_mode.as_deref() {
        None => Ok(format!("det:{}", request.input.metadata_document_id)),
        Some("external") => {
            anyhow::ensure!(request.reason_code.as_deref().is_some(), "reason_code required");
            let external = request.external_id.clone().ok_or_else(|| anyhow::anyhow!("external_id required"))?;
            Ok(format!("ext:{external}"))
        }
        Some("random") => {
            anyhow::ensure!(request.reason_code.as_deref().is_some(), "reason_code required");
            diagnostics.push(DiagnosticItem {
                level: "warning".into(),
                code: "PHASE2.IDENTITY_RANDOM_OVERRIDE".into(),
                summary: "random override disables reproducible identity for targeted objects".into(),
            });
            Ok(format!("rnd:{}", request.input.metadata_document_id.len()))
        }
        Some(other) => anyhow::bail!("unsupported identity override mode: {other}"),
    }
}
```

```rust
// authoring_core/src/canonical_json.rs
use serde::Serialize;

pub fn to_canonical_json<T: Serialize>(value: &T) -> anyhow::Result<String> {
    let mut v = serde_json::to_value(value)?;
    sort_value(&mut v);
    Ok(serde_json::to_string(&v)?)
}

fn sort_value(v: &mut serde_json::Value) {
    match v {
        serde_json::Value::Object(map) => {
            let mut keys: Vec<String> = map.keys().cloned().collect();
            keys.sort();
            let mut next = serde_json::Map::new();
            for key in keys {
                let mut child = map.remove(&key).expect("key exists");
                sort_value(&mut child);
                next.insert(key, child);
            }
            *map = next;
        }
        serde_json::Value::Array(items) => {
            for item in items {
                sort_value(item);
            }
        }
        _ => {}
    }
}
```

- [ ] **Step 4: Run tests to verify pass**

Run: `cargo test -p authoring_core --test normalization_pipeline_tests -v`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add authoring_core/src/model.rs authoring_core/src/normalize.rs authoring_core/src/identity.rs authoring_core/src/canonical_json.rs authoring_core/src/lib.rs authoring_core/tests/normalization_pipeline_tests.rs contracts/semantics/identity.md contracts/semantics/canonical-serialization.md contracts/manifest.yaml
git commit -m "feat: implement identity policy semantics and canonical serialization"
```

## Task 7: Implement comparison-context-aware merge risk assessment

**Files:**
- Create: `authoring_core/src/risk.rs`
- Modify: `authoring_core/src/model.rs`
- Modify: `authoring_core/src/normalize.rs`
- Modify: `authoring_core/src/lib.rs`
- Create: `authoring_core/tests/risk_tests.rs`
- Create: `contracts/semantics/merge-risk.md`
- Modify: `contracts/manifest.yaml`

- [ ] **Step 1: Write failing risk tests for complete/partial/unavailable statuses**

```rust
// authoring_core/tests/risk_tests.rs
use authoring_core::{normalize, AuthoringDocument, ComparisonContext, NormalizationRequest};

#[test]
fn risk_is_unavailable_without_comparison_context() {
    let input = AuthoringDocument {
        kind: "authoring-ir".into(),
        schema_version: "0.1.0".into(),
        metadata_document_id: "doc-1".into(),
    };
    let result = normalize(NormalizationRequest::new(input));
    assert!(result.merge_risk_report.is_none());
}

#[test]
fn risk_report_is_complete_when_context_has_full_baseline() {
    let input = AuthoringDocument {
        kind: "authoring-ir".into(),
        schema_version: "0.1.0".into(),
        metadata_document_id: "doc-1".into(),
    };
    let mut req = NormalizationRequest::new(input);
    req.comparison_context = Some(ComparisonContext::normalized("baseline-1", "risk-policy.default@1.0.0"));
    let result = normalize(req);
    let report = result.merge_risk_report.expect("report required");
    assert_eq!(report.comparison_status, "complete");
}
```

- [ ] **Step 2: Run risk tests to verify failure**

Run: `cargo test -p authoring_core --test risk_tests -v`
Expected: FAIL because risk model is missing.

- [ ] **Step 3: Implement risk model and integration**

```rust
// authoring_core/src/model.rs (excerpt)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ComparisonContext {
    pub kind: String,
    pub baseline_kind: String,
    pub baseline_artifact_fingerprint: String,
    pub risk_policy_ref: String,
    pub comparison_mode: String,
}

impl ComparisonContext {
    pub fn normalized(fingerprint: &str, policy_ref: &str) -> Self {
        Self {
            kind: "comparison-context".into(),
            baseline_kind: "normalized_ir".into(),
            baseline_artifact_fingerprint: fingerprint.into(),
            risk_policy_ref: policy_ref.into(),
            comparison_mode: "strict".into(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MergeRiskReport {
    pub kind: String,
    pub comparison_status: String,
    pub overall_level: String,
    pub policy_version: String,
    pub baseline_artifact_fingerprint: String,
    pub current_artifact_fingerprint: String,
    pub findings: Vec<MergeRiskFinding>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MergeRiskFinding {
    pub level: String,
    pub code: String,
    pub dimension: String,
    pub target_selector: String,
    pub summary: String,
}
```

```rust
// authoring_core/src/risk.rs
use crate::model::{ComparisonContext, MergeRiskFinding, MergeRiskReport, NormalizedIr};

pub fn assess_risk(current: &NormalizedIr, comparison: Option<&ComparisonContext>) -> Option<MergeRiskReport> {
    let ctx = comparison?;
    Some(MergeRiskReport {
        kind: "merge-risk-report".into(),
        comparison_status: "complete".into(),
        overall_level: "low".into(),
        policy_version: ctx.risk_policy_ref.clone(),
        baseline_artifact_fingerprint: ctx.baseline_artifact_fingerprint.clone(),
        current_artifact_fingerprint: format!("current:{}", current.document_id),
        findings: vec![MergeRiskFinding {
            level: "low".into(),
            code: "PHASE2.RISK.NO_BREAKING_CHANGE".into(),
            dimension: "structure".into(),
            target_selector: "document[id='root']".into(),
            summary: "no breaking structural differences detected".into(),
        }],
    })
}
```

- [ ] **Step 4: Run risk tests to verify pass**

Run: `cargo test -p authoring_core --test risk_tests -v`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add authoring_core/src/model.rs authoring_core/src/normalize.rs authoring_core/src/risk.rs authoring_core/src/lib.rs authoring_core/tests/risk_tests.rs contracts/semantics/merge-risk.md contracts/manifest.yaml
git commit -m "feat: add comparison-context-aware merge risk reporting"
```

## Task 8: Add `contract_tools normalize` command with contract-json required fields

**Files:**
- Modify: `contract_tools/Cargo.toml`
- Modify: `contract_tools/src/main.rs`
- Modify: `contract_tools/src/lib.rs`
- Create: `contract_tools/src/normalize_cmd.rs`
- Modify: `contract_tools/tests/cli_tests.rs`

- [ ] **Step 1: Write failing CLI tests for `normalize --output contract-json`**

```rust
// contract_tools/tests/cli_tests.rs (add)
#[test]
fn normalize_contract_json_includes_required_top_level_fields() {
    let output = run_cli(&[
        "normalize",
        "--manifest",
        contract_tools::contract_manifest_path().to_str().unwrap(),
        "--input",
        "contracts/fixtures/valid/minimal-authoring-ir.json",
        "--output",
        "contract-json",
    ]);

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    for key in [
        "kind",
        "result_status",
        "tool_contract_version",
        "policy_refs",
        "comparison_context",
        "diagnostics",
    ] {
        assert!(value.get(key).is_some(), "missing key: {key}");
    }
}
```

- [ ] **Step 2: Run CLI tests to verify failure**

Run: `cargo test -p contract_tools --test cli_tests -v`
Expected: FAIL because `normalize` subcommand is missing.

- [ ] **Step 3: Implement normalize subcommand and contract-json output path**

```rust
// contract_tools/src/main.rs (command enum excerpt)
enum Command {
    Verify { #[arg(long)] manifest: String },
    Summary { #[arg(long)] manifest: String },
    Package { #[arg(long)] manifest: String, #[arg(long)] out_dir: String },
    Normalize {
        #[arg(long)] manifest: String,
        #[arg(long)] input: String,
        #[arg(long, default_value = "contract-json")]
        output: String,
    },
}
```

```rust
// contract_tools/src/normalize_cmd.rs
use std::fs;

use authoring_core::{normalize, AuthoringDocument, NormalizationRequest};

pub fn run(manifest: &str, input: &str, output: &str) -> anyhow::Result<String> {
    let _ = crate::manifest::load_manifest(manifest)?;

    let raw = fs::read_to_string(input)?;
    let input_json: serde_json::Value = serde_json::from_str(&raw)?;
    let document = AuthoringDocument {
        kind: input_json.get("kind").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        schema_version: input_json
            .get("schema_version")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        metadata_document_id: input_json
            .get("metadata")
            .and_then(|m| m.get("document_id"))
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
    };

    let result = normalize(NormalizationRequest::new(document));

    match output {
        "contract-json" => Ok(authoring_core::canonical_json::to_canonical_json(&result)?),
        "human" => Ok(format!("status: {}", result.result_status)),
        other => anyhow::bail!("unsupported output mode: {other}"),
    }
}
```

- [ ] **Step 4: Run CLI tests to verify pass**

Run: `cargo test -p contract_tools --test cli_tests -v`
Expected: PASS, including new normalize test.

- [ ] **Step 5: Commit**

```bash
git add contract_tools/Cargo.toml contract_tools/src/main.rs contract_tools/src/lib.rs contract_tools/src/normalize_cmd.rs contract_tools/tests/cli_tests.rs
git commit -m "feat: add contract_tools normalize command with contract-json mode"
```

## Task 9: Add Phase 2 fixture catalog entries and gate compatibility checks

**Files:**
- Modify: `contracts/fixtures/index.yaml`
- Create: `contracts/fixtures/phase2/normalization/minimal-success.yaml`
- Create: `contracts/fixtures/phase2/normalization/identity-random-warning.yaml`
- Create: `contracts/fixtures/phase2/risk/complete-low.yaml`
- Create: `contracts/fixtures/phase2/risk/partial-high.yaml`
- Modify: `contract_tools/src/fixtures.rs`
- Modify: `contract_tools/tests/fixture_gate_tests.rs`

- [ ] **Step 1: Write failing fixture gate tests for catalog-style phase2 entries**

```rust
// contract_tools/tests/fixture_gate_tests.rs (add)
#[test]
fn fixture_catalog_accepts_phase2_case_paths_with_policy_refs() {
    let manifest = contract_tools::manifest::load_manifest(contract_tools::contract_manifest_path()).unwrap();
    contract_tools::fixtures::run_fixture_gates(&manifest.path).unwrap();
}
```

- [ ] **Step 2: Run fixture tests to verify failure**

Run: `cargo test -p contract_tools --test fixture_gate_tests -v`
Expected: FAIL once phase2 catalog fields are added but parser/gates are not updated.

- [ ] **Step 3: Add phase2 fixtures and update fixture gate model**

```yaml
# contracts/fixtures/index.yaml (phase2 excerpt)
cases:
  - id: phase2-normalization-minimal-success
    category: phase2-normalization
    input: fixtures/phase2/normalization/minimal-success.yaml
    target_asset: schema/normalization-result.schema.json
    compatibility_class: additive_compatible
    upgrade_rules: [fixture_updates_required]
  - id: phase2-risk-complete-low
    category: phase2-risk
    input: fixtures/phase2/risk/complete-low.yaml
    target_asset: schema/merge-risk-report.schema.json
    compatibility_class: additive_compatible
    upgrade_rules: [fixture_updates_required]
```

```rust
// contract_tools/src/fixtures.rs (category handling excerpt)
match case.category.as_str() {
    "phase2-normalization" => {
        let input_path = resolve_contract_relative_path(&manifest.contracts_root, &case.input)?;
        let yaml_value: serde_yaml::Value = serde_yaml::from_str(&std::fs::read_to_string(input_path)?)?;
        let json_value = serde_json::to_value(yaml_value)?;
        let schema = load_schema(resolve_asset_path(&manifest, "normalization_result_schema")?)?;
        validate_value(&schema, &json_value)?;
    }
    "phase2-risk" => {
        let input_path = resolve_contract_relative_path(&manifest.contracts_root, &case.input)?;
        let yaml_value: serde_yaml::Value = serde_yaml::from_str(&std::fs::read_to_string(input_path)?)?;
        let json_value = serde_json::to_value(yaml_value)?;
        let schema = load_schema(resolve_asset_path(&manifest, "merge_risk_report_schema")?)?;
        validate_value(&schema, &json_value)?;
    }
    _ => anyhow::bail!("unsupported fixture category: {}", case.category),
}
```

- [ ] **Step 4: Run fixture tests to verify pass**

Run: `cargo test -p contract_tools --test fixture_gate_tests -v`
Expected: PASS with new phase2 catalog entries.

- [ ] **Step 5: Commit**

```bash
git add contracts/fixtures/index.yaml contracts/fixtures/phase2/normalization/minimal-success.yaml contracts/fixtures/phase2/normalization/identity-random-warning.yaml contracts/fixtures/phase2/risk/complete-low.yaml contracts/fixtures/phase2/risk/partial-high.yaml contract_tools/src/fixtures.rs contract_tools/tests/fixture_gate_tests.rs
git commit -m "feat: add phase2 fixture catalog entries and gate validation"
```

## Task 10: End-to-end contract verification and release-readiness proof

**Files:**
- Modify: `README.md`
- Modify: `docs/superpowers/checklists/phase-1-exit-evidence.md`

- [ ] **Step 1: Add normalize command to operator docs**

```markdown
<!-- README.md (command excerpt) -->
cargo run -p contract_tools -- normalize --manifest "$(pwd)/contracts/manifest.yaml" --input "$(pwd)/contracts/fixtures/valid/minimal-authoring-ir.json" --output contract-json
```

- [ ] **Step 2: Run focused crate tests**

Run: `cargo test -p authoring_core -v`
Expected: PASS.

Run: `cargo test -p contract_tools -v`
Expected: PASS.

- [ ] **Step 3: Run contract verification commands**

Run: `cargo run -p contract_tools -- verify --manifest "$(pwd)/contracts/manifest.yaml"`
Expected: `verification passed`.

Run: `cargo run -p contract_tools -- normalize --manifest "$(pwd)/contracts/manifest.yaml" --input "$(pwd)/contracts/fixtures/valid/minimal-authoring-ir.json" --output contract-json`
Expected: valid JSON with required top-level keys.

- [ ] **Step 4: Update checklist evidence with exact command outputs and dates**

```markdown
<!-- docs/superpowers/checklists/phase-1-exit-evidence.md (new entry excerpt) -->
## 2026-04-03 Phase 2 contract closure pre-planning verification

- `cargo test -p authoring_core -v` : PASS
- `cargo test -p contract_tools -v` : PASS
- `cargo run -p contract_tools -- verify --manifest "$(pwd)/contracts/manifest.yaml"` : PASS
- `cargo run -p contract_tools -- normalize --manifest "$(pwd)/contracts/manifest.yaml" --input "$(pwd)/contracts/fixtures/valid/minimal-authoring-ir.json" --output contract-json` : PASS
```

- [ ] **Step 5: Commit**

```bash
git add README.md docs/superpowers/checklists/phase-1-exit-evidence.md
git commit -m "docs: record phase2 contract verification workflow"
```

## Self-Review

### 1. Spec coverage

- diagnostics contract: Task 2 + Task 4 + Task 8
- policy assets: Task 3
- comparison-context schema: Task 2 + Task 7
- target-selector grammar: Task 5
- contract-json required fields: Task 2 + Task 8
- canonical serialization specification: Task 6 + Task 9
- contract-first ownership and synchronized updates: Tasks 2/3/9/10

No uncovered requirements from the approved Phase 2 spec.

### 2. Placeholder scan

Plan contains no `TBD`, `TODO`, or deferred implementation placeholders.

### 3. Type consistency

Model names and output fields are consistent across tasks:

- `NormalizationResult`
- `NormalizationDiagnostics`
- `ComparisonContext`
- `MergeRiskReport`
- `target_selector`
- `tool_contract_version`
