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
- Modify: `contract_tools/Cargo.toml`
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
rust-version = "1.92"

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

```toml
# contract_tools/Cargo.toml
[dev-dependencies]
tempfile = "=3.17.1"
authoring_core = { path = "../authoring_core" }
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
git add Cargo.toml contract_tools/Cargo.toml contract_tools/tests/workspace_smoke_tests.rs authoring_core/Cargo.toml authoring_core/src/lib.rs
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
    "comparison_context": {
      "anyOf": [
        { "type": "null" },
        { "$ref": "comparison-context.schema.json" }
      ]
    },
    "diagnostics": { "$ref": "normalization-diagnostics.schema.json" },
    "normalized_ir": { "$ref": "normalized-ir.schema.json" },
    "merge_risk_report": { "$ref": "merge-risk-report.schema.json" }
  },
  "allOf": [
    {
      "if": {
        "properties": { "comparison_context": { "type": "null" } }
      },
      "then": {
        "not": { "required": ["merge_risk_report"] }
      }
    },
    {
      "if": {
        "properties": { "comparison_context": { "type": "object" } }
      },
      "then": {
        "required": ["merge_risk_report"]
      }
    }
  ]
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

- [ ] **Step 1: Write failing pipeline tests for status behavior and contract-json envelope fields**

```rust
// authoring_core/tests/normalization_pipeline_tests.rs
use authoring_core::{normalize, AuthoringDocument, NormalizationRequest};
use serde_json::Value;

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
fn normalize_success_includes_required_contract_envelope_fields() {
    let input = AuthoringDocument {
        kind: "authoring-ir".into(),
        schema_version: "0.1.0".into(),
        metadata_document_id: "doc-1".into(),
    };

    let result = normalize(NormalizationRequest::new(input));
    assert_eq!(result.result_status, "success");
    assert!(result.normalized_ir.is_some());

    let value: Value = serde_json::to_value(result).unwrap();
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

    assert!(value.get("comparison_context").unwrap().is_null());
    assert!(value.get("merge_risk_report").unwrap().is_null());
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonContext {
    pub kind: String,
    pub baseline_kind: String,
    pub baseline_artifact_fingerprint: String,
    pub risk_policy_ref: String,
    pub comparison_mode: String,
}

#[derive(Debug, Clone)]
pub struct NormalizationRequest {
    pub input: AuthoringDocument,
    pub comparison_context: Option<ComparisonContext>,
}

impl NormalizationRequest {
    pub fn new(input: AuthoringDocument) -> Self {
        Self {
            input,
            comparison_context: None,
        }
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
pub struct PolicyRefs {
    pub identity_policy_ref: String,
    pub risk_policy_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeRiskReport {
    pub kind: String,
    pub comparison_status: String,
    pub overall_level: String,
    pub policy_version: String,
    pub baseline_artifact_fingerprint: String,
    pub current_artifact_fingerprint: String,
    pub comparison_reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizationResult {
    pub kind: String,
    pub result_status: String,
    pub tool_contract_version: String,
    pub policy_refs: PolicyRefs,
    pub comparison_context: Option<ComparisonContext>,
    pub diagnostics: NormalizationDiagnostics,
    pub normalized_ir: Option<NormalizedIr>,
    pub merge_risk_report: Option<MergeRiskReport>,
}
```

```rust
// authoring_core/src/normalize.rs
use crate::model::*;

pub fn normalize(request: NormalizationRequest) -> NormalizationResult {
    let policy_refs = PolicyRefs {
        identity_policy_ref: "identity-policy.default@1.0.0".into(),
        risk_policy_ref: request
            .comparison_context
            .as_ref()
            .map(|ctx| ctx.risk_policy_ref.clone()),
    };

    if request.input.metadata_document_id.trim().is_empty() {
        return NormalizationResult {
            kind: "normalization-result".into(),
            result_status: "invalid".into(),
            tool_contract_version: crate::tool_contract_version().into(),
            policy_refs,
            comparison_context: request.comparison_context,
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
            merge_risk_report: None,
        };
    }

    NormalizationResult {
        kind: "normalization-result".into(),
        result_status: "success".into(),
        tool_contract_version: crate::tool_contract_version().into(),
        policy_refs,
        comparison_context: request.comparison_context,
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
        merge_risk_report: None,
    }
}
```

```rust
// authoring_core/src/lib.rs
pub mod model;
pub mod normalize;

pub use model::{AuthoringDocument, ComparisonContext, NormalizationRequest};
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
git commit -m "feat: add phase2 normalization result envelope skeleton"
```

## Task 5: Implement `target_selector` grammar parser and resolver behavior

**Files:**
- Create: `authoring_core/src/selector.rs`
- Modify: `authoring_core/src/lib.rs`
- Modify: `authoring_core/src/normalize.rs`
- Create: `authoring_core/tests/selector_tests.rs`
- Create: `contracts/semantics/target-selector-grammar.md`
- Modify: `contracts/manifest.yaml`

- [ ] **Step 1: Write failing selector tests for index prohibition, zero-match, and multi-match errors**

```rust
// authoring_core/tests/selector_tests.rs
use std::collections::BTreeMap;
use authoring_core::selector::{
    parse_selector, resolve_selector, SelectorError, SelectorResolveError, SelectorTarget,
};

#[test]
fn selector_rejects_array_index_segments() {
    let err = parse_selector("note[id='n1']/fields[12]").unwrap_err();
    assert!(matches!(err, SelectorError::ArrayIndexNotAllowed));
}

#[test]
fn selector_accepts_kind_and_predicates() {
    let sel = parse_selector("note[id='n1']").unwrap();
    assert_eq!(sel.kind, "note");
    assert_eq!(sel.predicates.len(), 1);
}

#[test]
fn resolver_reports_zero_match() {
    let selector = parse_selector("note[id='missing']").unwrap();
    let targets = vec![SelectorTarget::new("note", [("id", "n1")])];
    let err = resolve_selector(&selector, &targets).unwrap_err();
    assert!(matches!(err, SelectorResolveError::Unmatched));
}

#[test]
fn resolver_reports_multi_match() {
    let selector = parse_selector("note[id='n1']").unwrap();
    let targets = vec![
        SelectorTarget::new("note", [("id", "n1")]),
        SelectorTarget::new("note", [("id", "n1")]),
    ];
    let err = resolve_selector(&selector, &targets).unwrap_err();
    assert!(matches!(err, SelectorResolveError::Ambiguous));
}
```

- [ ] **Step 2: Run selector tests to verify failure**

Run: `cargo test -p authoring_core --test selector_tests -v`
Expected: FAIL because selector module does not exist.

- [ ] **Step 3: Implement parser, resolver, and normalization error mapping**

```rust
// authoring_core/src/selector.rs
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Eq)]
pub enum SelectorError {
    Empty,
    ArrayIndexNotAllowed,
    InvalidPredicate,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SelectorResolveError {
    Unmatched,
    Ambiguous,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Selector {
    pub kind: String,
    pub predicates: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectorTarget {
    pub kind: String,
    pub keys: BTreeMap<String, String>,
}

impl SelectorTarget {
    pub fn new<const N: usize>(kind: &str, pairs: [(&str, &str); N]) -> Self {
        let mut keys = BTreeMap::new();
        for (k, v) in pairs {
            keys.insert(k.to_string(), v.to_string());
        }
        Self {
            kind: kind.to_string(),
            keys,
        }
    }
}

pub fn parse_selector(raw: &str) -> Result<Selector, SelectorError> {
    if raw.trim().is_empty() {
        return Err(SelectorError::Empty);
    }

    if raw
        .split(['/', '.'])
        .any(|segment| segment.starts_with('[') && segment.ends_with(']') && segment[1..segment.len() - 1].chars().all(|c| c.is_ascii_digit()))
    {
        return Err(SelectorError::ArrayIndexNotAllowed);
    }

    let (kind, rest) = raw
        .split_once('[')
        .ok_or(SelectorError::InvalidPredicate)?;
    let predicate = rest.strip_suffix(']').ok_or(SelectorError::InvalidPredicate)?;
    let predicates = predicate
        .split(',')
        .map(|part| {
            let (k, v) = part
                .split_once('=')
                .ok_or(SelectorError::InvalidPredicate)?;
            let value = v.trim().trim_matches('\'').to_string();
            Ok((k.trim().to_string(), value))
        })
        .collect::<Result<Vec<_>, SelectorError>>()?;

    Ok(Selector {
        kind: kind.trim().to_string(),
        predicates,
    })
}

pub fn resolve_selector(
    selector: &Selector,
    targets: &[SelectorTarget],
) -> Result<usize, SelectorResolveError> {
    let matches: Vec<usize> = targets
        .iter()
        .enumerate()
        .filter(|(_, target)| {
            target.kind == selector.kind
                && selector
                    .predicates
                    .iter()
                    .all(|(k, v)| target.keys.get(k) == Some(v))
        })
        .map(|(idx, _)| idx)
        .collect();

    match matches.as_slice() {
        [single] => Ok(*single),
        [] => Err(SelectorResolveError::Unmatched),
        _ => Err(SelectorResolveError::Ambiguous),
    }
}
```

```rust
// authoring_core/src/normalize.rs (selector error mapping excerpt)
use crate::selector::SelectorResolveError;

fn selector_error_code(err: &SelectorResolveError) -> &'static str {
    match err {
        SelectorResolveError::Unmatched => "PHASE2.SELECTOR_UNMATCHED",
        SelectorResolveError::Ambiguous => "PHASE2.SELECTOR_AMBIGUOUS",
    }
}
```

```markdown
<!-- contracts/semantics/target-selector-grammar.md -->
---
asset_refs:
  - schema/normalized-ir.schema.json
---

# Target Selector Grammar

Valid selector shape is `kind[key='value']` or `kind[k1='v1',k2='v2']`.

- array index selectors are forbidden
- selectors must be deterministic
- zero-match and multi-match are normalization errors (`PHASE2.SELECTOR_UNMATCHED`, `PHASE2.SELECTOR_AMBIGUOUS`)
```

- [ ] **Step 4: Run selector tests to verify pass**

Run: `cargo test -p authoring_core --test selector_tests -v`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add authoring_core/src/selector.rs authoring_core/src/lib.rs authoring_core/src/normalize.rs authoring_core/tests/selector_tests.rs contracts/semantics/target-selector-grammar.md contracts/manifest.yaml
git commit -m "feat: add target_selector parser and resolver contract semantics"
```

## Task 6: Implement identity policy resolution and canonicalization rules

**Files:**
- Modify: `authoring_core/Cargo.toml`
- Create: `authoring_core/src/identity.rs`
- Create: `authoring_core/src/canonical_json.rs`
- Modify: `authoring_core/src/model.rs`
- Modify: `authoring_core/src/normalize.rs`
- Modify: `authoring_core/src/lib.rs`
- Create: `contracts/semantics/canonical-serialization.md`
- Create: `contracts/semantics/identity.md`
- Modify: `contracts/manifest.yaml`
- Modify: `authoring_core/tests/normalization_pipeline_tests.rs`

- [ ] **Step 1: Write failing tests for deterministic default, random non-deterministic path, and required reason_code**

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
    assert!(result
        .normalized_ir
        .as_ref()
        .unwrap()
        .resolved_identity
        .starts_with("rnd:"));
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

#[test]
fn external_override_requires_external_id() {
    let input = AuthoringDocument {
        kind: "authoring-ir".into(),
        schema_version: "0.1.0".into(),
        metadata_document_id: "doc-1".into(),
    };
    let mut req = NormalizationRequest::new(input);
    req.identity_override_mode = Some("external".into());
    req.reason_code = Some("migration_keep_id".into());

    let result = normalize(req);
    assert_eq!(result.result_status, "invalid");
    assert!(result
        .diagnostics
        .items
        .iter()
        .any(|d| d.code == "PHASE2.EXTERNAL_ID_REQUIRED"));
}

#[test]
fn unmatched_target_selector_is_normalization_error() {
    let input = AuthoringDocument {
        kind: "authoring-ir".into(),
        schema_version: "0.1.0".into(),
        metadata_document_id: "doc-1".into(),
    };
    let mut req = NormalizationRequest::new(input);
    req.target_selector = Some("note[id='missing']".into());

    let result = normalize(req);
    assert_eq!(result.result_status, "invalid");
    assert!(result
        .diagnostics
        .items
        .iter()
        .any(|d| d.code == "PHASE2.SELECTOR_UNMATCHED"));
}
```

- [ ] **Step 2: Run tests to verify failure**

Run: `cargo test -p authoring_core --test normalization_pipeline_tests -v`
Expected: FAIL because override fields/behavior are not implemented.

- [ ] **Step 3: Implement identity policy handling (with real non-deterministic random path) and canonical serialization helper**

```toml
# authoring_core/Cargo.toml (dependencies excerpt)
[dependencies]
anyhow = "1"
rand = "0.8"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

```rust
// authoring_core/src/model.rs (request fields excerpt)
#[derive(Debug, Clone)]
pub struct NormalizationRequest {
    pub input: AuthoringDocument,
    pub comparison_context: Option<ComparisonContext>,
    pub identity_override_mode: Option<String>,
    pub target_selector: Option<String>,
    pub external_id: Option<String>,
    pub reason_code: Option<String>,
    pub reason: Option<String>,
}

impl NormalizationRequest {
    pub fn new(input: AuthoringDocument) -> Self {
        Self {
            input,
            comparison_context: None,
            identity_override_mode: None,
            target_selector: None,
            external_id: None,
            reason_code: None,
            reason: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedIr {
    pub kind: String,
    pub schema_version: String,
    pub document_id: String,
    pub resolved_identity: String,
}
```

```rust
// authoring_core/src/identity.rs
use crate::model::{DiagnosticItem, NormalizationRequest};

pub trait NonceSource {
    fn next_u64(&mut self) -> u64;
}

pub struct DefaultNonceSource;

impl NonceSource for DefaultNonceSource {
    fn next_u64(&mut self) -> u64 {
        rand::random::<u64>()
    }
}

pub fn resolve_identity(
    request: &NormalizationRequest,
    diagnostics: &mut Vec<DiagnosticItem>,
    nonce_source: &mut dyn NonceSource,
) -> anyhow::Result<String> {
    match request.identity_override_mode.as_deref() {
        None => Ok(format!("det:{}", request.input.metadata_document_id)),
        Some("external") => {
            anyhow::ensure!(request.reason_code.as_deref().is_some(), "reason_code required");
            let external = request
                .external_id
                .clone()
                .ok_or_else(|| anyhow::anyhow!("external_id required"))?;
            Ok(format!("ext:{external}"))
        }
        Some("random") => {
            anyhow::ensure!(request.reason_code.as_deref().is_some(), "reason_code required");
            diagnostics.push(DiagnosticItem {
                level: "warning".into(),
                code: "PHASE2.IDENTITY_RANDOM_OVERRIDE".into(),
                summary: "random override disables reproducible identity for targeted objects".into(),
            });
            let nonce = nonce_source.next_u64();
            Ok(format!("rnd:{nonce:016x}"))
        }
        Some(other) => anyhow::bail!("unsupported identity override mode: {other}"),
    }
}
```

```rust
// authoring_core/src/normalize.rs (identity integration excerpt)
use crate::selector::{parse_selector, resolve_selector, SelectorResolveError, SelectorTarget};
use crate::identity::{resolve_identity, DefaultNonceSource};

let mut diagnostics = vec![];
let policy_refs = PolicyRefs {
    identity_policy_ref: "identity-policy.default@1.0.0".into(),
    risk_policy_ref: request
        .comparison_context
        .as_ref()
        .map(|ctx| ctx.risk_policy_ref.clone()),
};

if let Some(raw_selector) = request.target_selector.as_deref() {
    let selector = match parse_selector(raw_selector) {
        Ok(sel) => sel,
        Err(_) => {
            diagnostics.push(DiagnosticItem {
                level: "error".into(),
                code: "PHASE2.SELECTOR_INVALID".into(),
                summary: "target_selector does not match grammar".into(),
            });
            return NormalizationResult {
                kind: "normalization-result".into(),
                result_status: "invalid".into(),
                tool_contract_version: crate::tool_contract_version().into(),
                policy_refs,
                comparison_context: request.comparison_context.clone(),
                diagnostics: NormalizationDiagnostics {
                    kind: "normalization-diagnostics".into(),
                    status: "invalid".into(),
                    items: diagnostics,
                },
                normalized_ir: None,
                merge_risk_report: None,
            };
        }
    };

    let targets = vec![SelectorTarget::new("document", [("id", request.input.metadata_document_id.as_str())])];
    if let Err(err) = resolve_selector(&selector, &targets) {
        let code = match err {
            SelectorResolveError::Unmatched => "PHASE2.SELECTOR_UNMATCHED",
            SelectorResolveError::Ambiguous => "PHASE2.SELECTOR_AMBIGUOUS",
        };
        diagnostics.push(DiagnosticItem {
            level: "error".into(),
            code: code.into(),
            summary: "target_selector resolution failed".into(),
        });
        return NormalizationResult {
            kind: "normalization-result".into(),
            result_status: "invalid".into(),
            tool_contract_version: crate::tool_contract_version().into(),
            policy_refs,
            comparison_context: request.comparison_context.clone(),
            diagnostics: NormalizationDiagnostics {
                kind: "normalization-diagnostics".into(),
                status: "invalid".into(),
                items: diagnostics,
            },
            normalized_ir: None,
            merge_risk_report: None,
        };
    }
}

let mut nonce_source = DefaultNonceSource;
let resolved_identity = match resolve_identity(&request, &mut diagnostics, &mut nonce_source) {
    Ok(id) => id,
    Err(_) => {
        diagnostics.push(DiagnosticItem {
            level: "error".into(),
            code: "PHASE2.EXTERNAL_ID_REQUIRED".into(),
            summary: "external override requires external_id and reason_code".into(),
        });
        return NormalizationResult {
            kind: "normalization-result".into(),
            result_status: "invalid".into(),
            tool_contract_version: crate::tool_contract_version().into(),
            policy_refs,
            comparison_context: request.comparison_context.clone(),
            diagnostics: NormalizationDiagnostics {
                kind: "normalization-diagnostics".into(),
                status: "invalid".into(),
                items: diagnostics,
            },
            normalized_ir: None,
            merge_risk_report: None,
        };
    }
};

let normalized = NormalizedIr {
    kind: "normalized-ir".into(),
    schema_version: request.input.schema_version.clone(),
    document_id: request.input.metadata_document_id.clone(),
    resolved_identity,
};
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
git add authoring_core/Cargo.toml authoring_core/src/model.rs authoring_core/src/normalize.rs authoring_core/src/identity.rs authoring_core/src/canonical_json.rs authoring_core/src/lib.rs authoring_core/tests/normalization_pipeline_tests.rs contracts/semantics/identity.md contracts/semantics/canonical-serialization.md contracts/manifest.yaml
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
fn risk_report_is_partial_for_best_effort_identity_index_context() {
    let input = AuthoringDocument {
        kind: "authoring-ir".into(),
        schema_version: "0.1.0".into(),
        metadata_document_id: "doc-1".into(),
    };
    let mut req = NormalizationRequest::new(input);
    req.comparison_context = Some(ComparisonContext {
        kind: "comparison-context".into(),
        baseline_kind: "identity_index".into(),
        baseline_artifact_fingerprint: "idx-1".into(),
        risk_policy_ref: "risk-policy.default@1.0.0".into(),
        comparison_mode: "best_effort".into(),
    });

    let report = normalize(req).merge_risk_report.expect("report required");
    assert_eq!(report.comparison_status, "partial");
    assert!(report
        .comparison_reasons
        .iter()
        .any(|r| r == "BASELINE_IDENTITY_INDEX_ONLY"));
}

#[test]
fn risk_report_is_unavailable_when_strict_context_lacks_baseline_fingerprint() {
    let input = AuthoringDocument {
        kind: "authoring-ir".into(),
        schema_version: "0.1.0".into(),
        metadata_document_id: "doc-1".into(),
    };
    let mut req = NormalizationRequest::new(input);
    req.comparison_context = Some(ComparisonContext {
        kind: "comparison-context".into(),
        baseline_kind: "normalized_ir".into(),
        baseline_artifact_fingerprint: "".into(),
        risk_policy_ref: "risk-policy.default@1.0.0".into(),
        comparison_mode: "strict".into(),
    });

    let report = normalize(req).merge_risk_report.expect("report required");
    assert_eq!(report.comparison_status, "unavailable");
    assert!(report
        .comparison_reasons
        .iter()
        .any(|r| r == "BASELINE_UNAVAILABLE"));
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
    pub comparison_reasons: Vec<String>,
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
    let (comparison_status, overall_level, comparison_reasons) = if ctx.comparison_mode == "strict"
        && ctx.baseline_artifact_fingerprint.trim().is_empty()
    {
        ("unavailable", "high", vec!["BASELINE_UNAVAILABLE".to_string()])
    } else if ctx.baseline_kind == "identity_index" {
        ("partial", "medium", vec!["BASELINE_IDENTITY_INDEX_ONLY".to_string()])
    } else {
        ("complete", "low", vec![])
    };

    let findings = if comparison_status == "unavailable" {
        vec![MergeRiskFinding {
            level: "high".into(),
            code: "PHASE2.RISK.BASELINE_UNAVAILABLE".into(),
            dimension: "references".into(),
            target_selector: "document[id='root']".into(),
            summary: "baseline context unavailable for strict comparison".into(),
        }]
    } else {
        vec![MergeRiskFinding {
            level: overall_level.into(),
            code: "PHASE2.RISK.COMPARISON_COMPLETE".into(),
            dimension: "structure".into(),
            target_selector: "document[id='root']".into(),
            summary: "risk comparison computed from provided context".into(),
        }]
    };

    Some(MergeRiskReport {
        kind: "merge-risk-report".into(),
        comparison_status: comparison_status.into(),
        overall_level: overall_level.into(),
        policy_version: ctx.risk_policy_ref.clone(),
        baseline_artifact_fingerprint: ctx.baseline_artifact_fingerprint.clone(),
        current_artifact_fingerprint: format!("current:{}", current.document_id),
        comparison_reasons,
        findings,
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
- Create: `contracts/fixtures/phase2/normalization/minimal-success.case.yaml`
- Create: `contracts/fixtures/phase2/normalization/identity-random-warning.case.yaml`
- Create: `contracts/fixtures/phase2/risk/complete-low.case.yaml`
- Create: `contracts/fixtures/phase2/risk/partial-high.case.yaml`
- Create: `contracts/fixtures/phase2/inputs/minimal-authoring-ir.json`
- Create: `contracts/fixtures/phase2/expected/minimal-success.result.json`
- Create: `contracts/fixtures/phase2/expected/complete-low.risk.json`
- Modify: `contract_tools/src/fixtures.rs`
- Modify: `contract_tools/tests/fixture_gate_tests.rs`
- Modify: `contract_tools/Cargo.toml`

- [ ] **Step 1: Write failing fixture gate tests for executable phase2 regression**

```rust
// contract_tools/tests/fixture_gate_tests.rs (add)
#[test]
fn fixture_gate_executes_phase2_normalize_cases_and_compares_expected_output() {
    let manifest = contract_tools::manifest::load_manifest(contract_tools::contract_manifest_path()).unwrap();
    contract_tools::fixtures::run_fixture_gates(&manifest.path).unwrap();
}
```

- [ ] **Step 2: Run fixture tests to verify failure**

Run: `cargo test -p contract_tools --test fixture_gate_tests -v`
Expected: FAIL once phase2 case files are added but executable runner is not implemented.

- [ ] **Step 3: Add phase2 case files and executable fixture runner**

```yaml
# contracts/fixtures/index.yaml (phase2 excerpt)
cases:
  - id: phase2-normalization-minimal-success
    category: phase2-normalization
    input: fixtures/phase2/normalization/minimal-success.case.yaml
    compatibility_class: additive_compatible
    upgrade_rules: [fixture_updates_required]
  - id: phase2-risk-complete-low
    category: phase2-risk
    input: fixtures/phase2/risk/complete-low.case.yaml
    compatibility_class: additive_compatible
    upgrade_rules: [fixture_updates_required]
```

```yaml
# contracts/fixtures/phase2/normalization/minimal-success.case.yaml
kind: phase2-normalization-case
authoring_input: fixtures/phase2/inputs/minimal-authoring-ir.json
comparison_context: null
expected_result: fixtures/phase2/expected/minimal-success.result.json
```

```json
// contracts/fixtures/phase2/inputs/minimal-authoring-ir.json
{
  "kind": "authoring-ir",
  "schema_version": "0.1.0",
  "metadata": { "document_id": "demo-doc" },
  "notetypes": [],
  "notes": []
}
```

```json
// contracts/fixtures/phase2/expected/minimal-success.result.json
{
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
    "resolved_identity": "det:demo-doc"
  },
  "merge_risk_report": null
}
```

```rust
// contract_tools/src/fixtures.rs (phase2 execution excerpt)
#[derive(Debug, serde::Deserialize)]
struct Phase2NormalizationCase {
    kind: String,
    authoring_input: String,
    comparison_context: Option<serde_json::Value>,
    expected_result: String,
}

fn run_phase2_normalization_case(
    manifest: &crate::manifest::LoadedManifest,
    case_path: &std::path::Path,
) -> anyhow::Result<()> {
    let raw = std::fs::read_to_string(case_path)?;
    let case: Phase2NormalizationCase = serde_yaml::from_str(&raw)?;
    anyhow::ensure!(case.kind == "phase2-normalization-case", "unexpected case kind");

    let input_path = resolve_contract_relative_path(&manifest.contracts_root, &case.authoring_input)?;
    let input_json: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(input_path)?)?;
    let authoring = authoring_core::AuthoringDocument {
        kind: input_json["kind"].as_str().unwrap_or_default().to_string(),
        schema_version: input_json["schema_version"].as_str().unwrap_or_default().to_string(),
        metadata_document_id: input_json["metadata"]["document_id"].as_str().unwrap_or_default().to_string(),
    };
    let mut req = authoring_core::NormalizationRequest::new(authoring);
    if let Some(ctx) = case.comparison_context {
        req.comparison_context = Some(serde_json::from_value(ctx)?);
    }

    let actual = authoring_core::normalize(req);
    let actual_text = authoring_core::canonical_json::to_canonical_json(&actual)?;
    let expected_path = resolve_contract_relative_path(&manifest.contracts_root, &case.expected_result)?;
    let expected_value: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(expected_path)?)?;
    let expected_text = authoring_core::canonical_json::to_canonical_json(&expected_value)?;

    anyhow::ensure!(actual_text == expected_text, "phase2 normalization output mismatch");
    Ok(())
}
```

- [ ] **Step 4: Run fixture tests to verify pass**

Run: `cargo test -p contract_tools --test fixture_gate_tests -v`
Expected: PASS with new phase2 catalog entries.

- [ ] **Step 5: Commit**

```bash
git add contracts/fixtures/index.yaml contracts/fixtures/phase2/normalization/minimal-success.case.yaml contracts/fixtures/phase2/normalization/identity-random-warning.case.yaml contracts/fixtures/phase2/risk/complete-low.case.yaml contracts/fixtures/phase2/risk/partial-high.case.yaml contracts/fixtures/phase2/inputs/minimal-authoring-ir.json contracts/fixtures/phase2/expected/minimal-success.result.json contracts/fixtures/phase2/expected/complete-low.risk.json contract_tools/Cargo.toml contract_tools/src/fixtures.rs contract_tools/tests/fixture_gate_tests.rs
git commit -m "feat: add executable phase2 fixture regression gates"
```

## Task 10: End-to-end contract verification and release-readiness proof

**Files:**
- Modify: `README.md`
- Create: `docs/superpowers/checklists/phase-2-exit-evidence.md`

- [ ] **Step 1: Add normalize command to operator docs**

```markdown
<!-- README.md (command excerpt) -->
cargo run -p contract_tools -- normalize --manifest "$(pwd)/contracts/manifest.yaml" --input "$(pwd)/contracts/fixtures/phase2/inputs/minimal-authoring-ir.json" --output contract-json
```

- [ ] **Step 2: Run focused crate tests**

Run: `cargo test -p authoring_core -v`
Expected: PASS.

Run: `cargo test -p contract_tools -v`
Expected: PASS.

- [ ] **Step 3: Run contract verification commands**

Run: `cargo run -p contract_tools -- verify --manifest "$(pwd)/contracts/manifest.yaml"`
Expected: `verification passed`.

Run: `cargo run -p contract_tools -- normalize --manifest "$(pwd)/contracts/manifest.yaml" --input "$(pwd)/contracts/fixtures/phase2/inputs/minimal-authoring-ir.json" --output contract-json`
Expected: valid JSON with required top-level keys.

- [ ] **Step 4: Update checklist evidence with exact command outputs and dates**

```markdown
<!-- docs/superpowers/checklists/phase-2-exit-evidence.md -->
# Phase 2 Exit Evidence

## 2026-04-03 Phase 2 core authoring model verification

- `cargo test -p authoring_core -v` : PASS
- `cargo test -p contract_tools -v` : PASS
- `cargo run -p contract_tools -- verify --manifest "$(pwd)/contracts/manifest.yaml"` : PASS
- `cargo run -p contract_tools -- normalize --manifest "$(pwd)/contracts/manifest.yaml" --input "$(pwd)/contracts/fixtures/phase2/inputs/minimal-authoring-ir.json" --output contract-json` : PASS
```

- [ ] **Step 5: Commit**

```bash
git add README.md docs/superpowers/checklists/phase-2-exit-evidence.md
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
- executable fixture regression gating (not schema-only): Task 9
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
