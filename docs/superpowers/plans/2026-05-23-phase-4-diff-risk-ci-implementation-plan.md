# Phase 4 Diff Risk CI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the latest `docs/api-design.md` Phase 4 so Product builds can compare against a previous APKG, classify import/update risk, apply `fail_on` policy, emit a stable JSON report, and run from CI.

**Architecture:** Keep `BuildReport` as the user-visible truth. Add focused `anki_forge::diff` and `anki_forge::risk` modules that transform existing inspect, writer diff, diagnostics, and update-safety evidence into Product-level report sections, while `Project::build` remains the orchestration entrypoint and `contract_tools product-build` calls the same Rust report path.

**Tech Stack:** Rust workspace (`anki_forge`, `writer_core`, `contract_tools`), `serde`/`serde_json`, existing canonical JSON helpers, JSON Schema contracts, APKG inspect/diff runtime, `cargo test`, shell CLI integration tests.

**Commit Rhythm:** One commit per task after that task's focused tests pass. Do not combine unrelated tasks in a single commit.

---

## Source Inputs

- Phase 4 design spec: `docs/superpowers/specs/2026-05-23-phase-4-diff-risk-ci-design.md`
- Roadmap source: `docs/api-design.md`, sections `10`, `10.1`, `11.3`, and `17.6`
- Current Product build path: `anki_forge/src/product/project.rs`
- Build API/report models: `anki_forge/src/build/options.rs`, `anki_forge/src/build/report.rs`, `anki_forge/src/build/mod.rs`
- Existing update-safety implementation: `anki_forge/src/update_safety/*`
- Existing inspect/diff runtime: `anki_forge/src/runtime/inspect.rs`, `anki_forge/src/runtime/diff.rs`, `writer_core/src/diff.rs`, `writer_core/src/model.rs`
- Existing CLI: `contract_tools/src/main.rs`, `contract_tools/src/build_cmd.rs`, `contract_tools/src/diff_cmd.rs`, `contract_tools/tests/cli_tests.rs`
- Contract manifest/schema assets: `contracts/manifest.yaml`, `contracts/schema/*`

## Scope Boundary

This plan implements the first Phase 4 slice: Rust Product API, `Project::diff_against_apkg`, stable BuildReport JSON projection, `contract_tools product-build`, CI docs, and tests. It does not implement Python or Node parity, APKG-to-editable-Project import, native Anki scheduling migration, or a new writer semantic model.

`writer_core` stays artifact-observation focused. Product risk semantics live in `anki_forge::risk`.

## Pre-Implementation API Surface Check

Run these checks before Task 1. They confirm the plan is aligned with the current workspace and that the final verification command already exists from earlier phases.

```bash
rg -n "pub fn facade_api_version|inspect_apkg|pub fn as_str|pub fn lower\\(&self\\)|pub fn default_deck_name|pub fn basic\\(|pub struct InspectReport|pub struct DiffReport|source_kind|observation_status|Command::Verify|verification passed" anki_forge/src writer_core/src contract_tools/src contract_tools/tests
rg -n "compare_to" anki_forge/src/build/options.rs
rg -n "tempfile" anki_forge/Cargo.toml
rg -n "derive\\(Debug, Clone, PartialEq, Eq, Serialize, Deserialize\\)|pub fn document_id\\(&self\\)" anki_forge/src/product/model.rs
rg -n "pub struct InspectObservations|pub struct DiffChange|pub struct DiffReport|pub source_kind|pub observation_status|pub references|pub metadata" writer_core/src/model.rs
rg -n "UPDATE\\.GUID_DERIVATION_DRIFT|UPDATE\\.BASELINE_CONFLICT_GUID|UPDATE\\.FIELD_MERGE_ID_CHANGED|UPDATE\\.TEMPLATE_MERGE_ID_CHANGED|UPDATE\\.TEMPLATE_ORD_CHANGED" anki_forge/src/update_safety anki_forge/tests
```

Expected paths:

```text
anki_forge/src/lib.rs: facade_api_version and inspect_apkg root re-export
anki_forge/src/diagnostics/mod.rs: DiagnosticCode::as_str and SourcePath::as_str
anki_forge/src/product/builders.rs: ProductDocument::default_deck_name and ProductDocument::lower
anki_forge/src/product/note.rs: Note::basic
writer_core/src/model.rs: InspectReport and DiffReport
writer_core/src/model.rs: InspectReport exposes public source_kind and observation_status fields
contract_tools/src/main.rs: Command::Verify
contract_tools/tests/cli_tests.rs: verification passed assertion
anki_forge/src/build/options.rs: compare_to field and builder exist
anki_forge/Cargo.toml: tempfile = "3"
anki_forge/src/product/model.rs: ProductDocument derives Clone and exposes document_id()
writer_core/src/model.rs: InspectObservations has public Vec<Value> notetypes/templates/fields/media/metadata/references fields
writer_core/src/model.rs: DiffChange and DiffReport are public with public fields used by Task 4
anki_forge/src/update_safety and tests: update-safety diagnostic codes used by Task 5 exist
```

## File Structure

Create these focused files:

- `docs/superpowers/checklists/phase-4-risk-evidence-matrix.md`: enabled/deferred risk-rule evidence matrix used before coding risk rules.
- `docs/oracles/phase-4-template-card-risk.md`: manual oracle record for template ord/removal card-risk behavior.
- `contracts/schema/build-report.schema.json`: first stable JSON schema for the Phase 4 report projection.
- `anki_forge/src/build/status.rs`: `BuildStatus`, `ComparisonStatus`, and stable serialization names.
- `anki_forge/src/build/policy.rs`: `RiskLevel`, `BuildPolicyStatus`, `BuildPolicyResult`, and policy threshold evaluation.
- `anki_forge/src/build/json_report.rs`: stable report projection structs and atomic JSON write helper.
- `anki_forge/src/diff/mod.rs`: public Product diff report types and `ProjectDiffError`.
- `anki_forge/src/diff/summary.rs`: transforms `writer_core::DiffReport` and inspect summaries into `BuildDiffSummary`.
- `anki_forge/src/risk/mod.rs`: risk public exports.
- `anki_forge/src/risk/model.rs`: `ImportRiskReport`, `ImportRiskFinding`, and `EvidenceRef`.
- `anki_forge/src/risk/rules.rs`: first-slice risk classification rules.
- `anki_forge/src/risk/policy.rs`: small facade that connects `ImportRiskReport` to `BuildPolicyResult`.
- `anki_forge/src/product/comparison.rs`: shared comparison assembler used by `Project::build(compare_to(...))` and `Project::diff_against_apkg(...)`.
- `anki_forge/src/runtime/product_build.rs`: runtime facade for `contract_tools product-build`.
- `contract_tools/src/product_build_cmd.rs`: CLI command implementation for ProductDocument build.
- `docs/ci/phase-4-product-build.md`: GitHub Actions and failure-mode examples.

Modify these existing files:

- `anki_forge/src/lib.rs`: expose `diff` and `risk`; keep Product risk semantics out of `writer_core`.
- `anki_forge/src/build/mod.rs`: export new build status, policy, and JSON projection types.
- `anki_forge/src/build/options.rs`: add `fail_on` and `report_json` builder methods.
- `anki_forge/src/build/report.rs`: add comparison, diff, risk, policy, typed status, and updated `ensure_success`.
- `anki_forge/src/product/mod.rs`: export `ProjectDiffReport`/`ProjectDiffError` only if the public API needs Product module access; prefer `anki_forge::diff`.
- `anki_forge/src/product/project.rs`: call shared comparison assembler, policy evaluator, JSON report writer, and add `diff_against_apkg`.
- `anki_forge/src/runtime/mod.rs`: export `build_product_document`.
- `contract_tools/src/lib.rs`: add `product_build_cmd`.
- `contract_tools/src/main.rs`: add `product-build` subcommand and exit code mapping.
- `contracts/manifest.yaml`: add `build_report_schema` asset.
- Tests in `anki_forge/tests`, `contract_tools/tests`, and targeted `writer_core/tests`.

---

### Task 1: Risk Evidence Matrix And Schema Gate

**Files:**
- Create: `docs/superpowers/checklists/phase-4-risk-evidence-matrix.md`
- Create: `docs/oracles/phase-4-template-card-risk.md`
- Create: `contracts/schema/build-report.schema.json`
- Modify: `contracts/manifest.yaml`
- Modify: `contract_tools/tests/schema_gate_tests.rs`

- [ ] **Step 1: Add the risk-rule evidence matrix**

Create `docs/superpowers/checklists/phase-4-risk-evidence-matrix.md` with this content:

```markdown
# Phase 4 Risk Rule Evidence Matrix

Date: 2026-05-23
Source spec: docs/superpowers/specs/2026-05-23-phase-4-diff-risk-ci-design.md

| Rule | Level | Status | Required evidence | Repo evidence refs | First-slice behavior |
| --- | --- | --- | --- | --- | --- |
| RISK.BASELINE_UNAVAILABLE | High | enabled | compare_to requested and previous APKG inspect is unavailable | source:anki_forge/src/update_safety/baseline.rs, source:anki_forge/src/runtime/inspect.rs | Emit finding, set comparison unavailable, allow fail_on to block. |
| RISK.NOTE_GUID_DRIFT | High | enabled | stable note id maps to different GUID through update-safety reconcile evidence | source:anki_forge/src/update_safety/reconcile.rs, roundtrip:update-safety-guid-preservation | Emit finding from UPDATE diagnostics or reconcile conflicts. |
| RISK.NOTETYPE_CONFIG_ID_DRIFT | High | enabled | field/template/notetype merge config id changed unexpectedly | source:anki_forge/src/update_safety/merge_safety.rs, source:writer_core/src/inspect.rs | Emit finding from existing update-safety merge diagnostics. |
| RISK.TEMPLATE_REORDER | High | enabled | template ordinal changes affect card ord update behavior | manual:phase4-template-card-risk, source:writer_core/src/inspect.rs | Emit finding when same template identity changes ord. |
| RISK.TEMPLATE_REMOVED | Critical | enabled | template/card ordinal disappeared from update path | manual:phase4-template-card-risk, source:writer_core/src/inspect.rs | Emit finding when a previous template identity is absent in current evidence. |
| RISK.FIELD_REMOVED_OR_RENAMED | Medium | enabled | field disappeared or rename cannot be proven safe by stable identity | source:anki_forge/src/product/lowering.rs, source:writer_core/src/inspect.rs | Emit finding for field removal/rename in the first slice. Inspect exposes `config_id`, but the first writer diff adapter lacks paired before/after field payloads, so safe-rename proof is not applied and the report records a limitation. |
| RISK.CARD_COUNT_CHANGED | Medium | enabled | current and previous card count differ | source:writer_core/src/inspect.rs, manual:card-count-change-review | Emit finding; promote to High when linked to RISK.TEMPLATE_REMOVED. |
| RISK.MEDIA_REFERENCE_BROKEN | High | enabled | current diagnostics or inspect references show missing/unresolved media | source:anki_forge/src/product/project.rs, source:authoring_core media diagnostics | Emit finding with no baseline required. |
| RISK.MEDIA_REMOVED | Medium | enabled | media filename present in previous artifact is absent from current artifact | source:writer_core/src/diff.rs, source:writer_core/src/inspect.rs | Emit finding from artifact diff media removal. |

## Oracle Reference Files

- manual:phase4-template-card-risk -> docs/oracles/phase-4-template-card-risk.md.
- manual:card-count-change-review -> docs/api-design.md section 10.1 item "card ord changed, existing scheduling may attach to wrong card".
- roundtrip:update-safety-guid-preservation -> anki_forge/tests/update_safety_build_tests.rs.
```

Create `docs/oracles/phase-4-template-card-risk.md`:

```markdown
# Phase 4 Template/Card Risk Oracle

This oracle records why template ordinal changes and template removal are treated as import/update risks.

Evidence source:

- Existing roadmap statement: docs/api-design.md section 10.1 says Anki cards are associated by note id plus card ordinal, so template order is import-sensitive.
- Repository observation source: writer_core/src/inspect.rs records template `ord`, card-count metadata, and card references from generated artifacts.
- Regression requirement: Phase 4 tests must build a previous APKG, build a current APKG with a removed or reordered template, and assert that the Product report emits `RISK.TEMPLATE_REMOVED` or `RISK.TEMPLATE_REORDER` with diff evidence refs.

Manual acceptance statement:

Changing or removing a template can change which existing cards are generated for the same note identity. Phase 4 therefore blocks high/critical template-card changes unless the user chooses a less strict `fail_on` threshold.
```

- [ ] **Step 2: Add a failing manifest/schema test**

Append these tests to `contract_tools/tests/schema_gate_tests.rs`:

```rust
#[test]
fn phase4_build_report_schema_is_registered_in_manifest() {
    let manifest =
        contract_tools::manifest::load_manifest(contract_tools::contract_manifest_path())
            .expect("repo manifest should load");
    let schema_path =
        contract_tools::manifest::resolve_asset_path(&manifest, "build_report_schema")
            .expect("build_report_schema should resolve");
    assert!(
        schema_path.ends_with("contracts/schema/build-report.schema.json"),
        "unexpected schema path: {}",
        schema_path.display()
    );
}

#[test]
fn phase4_build_report_schema_is_valid_json_schema() {
    let manifest =
        contract_tools::manifest::load_manifest(contract_tools::contract_manifest_path())
            .expect("repo manifest should load");
    let schema_path =
        contract_tools::manifest::resolve_asset_path(&manifest, "build_report_schema")
            .expect("build_report_schema should resolve");
    let raw = std::fs::read_to_string(schema_path).expect("read build report schema");
    let schema: serde_json::Value = serde_json::from_str(&raw).expect("schema JSON");
    jsonschema::JSONSchema::compile(&schema).expect("schema compiles");
}
```

- [ ] **Step 3: Run the failing schema test**

Run:

```bash
cargo test -p contract_tools phase4_build_report_schema --test schema_gate_tests
```

Expected:

```text
test phase4_build_report_schema_is_registered_in_manifest ... FAILED
test phase4_build_report_schema_is_valid_json_schema ... FAILED
```

- [ ] **Step 4: Register and create the schema**

Add this asset to `contracts/manifest.yaml` under `assets:`:

```yaml
  build_report_schema: schema/build-report.schema.json
```

Create `contracts/schema/build-report.schema.json`:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://anki-forge.local/schema/build-report.schema.json",
  "title": "Anki Forge Build Report",
  "type": "object",
  "required": [
    "kind",
    "schema_version",
    "tool_version",
    "status",
    "comparison",
    "counts",
    "media",
    "diagnostics",
    "metrics",
    "policy"
  ],
  "properties": {
    "kind": { "const": "anki-forge-build-report" },
    "schema_version": { "const": "phase4-build-report-v1" },
    "tool_version": { "type": "string", "minLength": 1 },
    "artifact": {
      "type": ["object", "null"],
      "required": ["path"],
      "properties": { "path": { "type": "string", "minLength": 1 } },
      "additionalProperties": false
    },
    "status": { "enum": ["success", "blocked", "invalid", "error"] },
    "comparison": { "enum": ["not_requested", "complete", "partial", "unavailable"] },
    "counts": {
      "type": "object",
      "required": ["notes", "cards", "media"],
      "properties": {
        "notes": { "type": "integer", "minimum": 0 },
        "cards": { "type": "integer", "minimum": 0 },
        "media": { "type": "integer", "minimum": 0 }
      },
      "additionalProperties": false
    },
    "media": {
      "type": "object",
      "required": [
        "objects",
        "bindings",
        "references",
        "missing_references",
        "unsafe_references",
        "unused_bindings",
        "unique_bytes"
      ],
      "properties": {
        "objects": { "type": "integer", "minimum": 0 },
        "bindings": { "type": "integer", "minimum": 0 },
        "references": { "type": "integer", "minimum": 0 },
        "missing_references": { "type": "integer", "minimum": 0 },
        "unsafe_references": { "type": "integer", "minimum": 0 },
        "unused_bindings": { "type": "integer", "minimum": 0 },
        "unique_bytes": { "type": "integer", "minimum": 0 }
      },
      "additionalProperties": false
    },
    "diagnostics": { "type": "array", "items": { "type": "object" } },
    "metrics": {
      "type": "object",
      "required": ["duration_ms"],
      "properties": { "duration_ms": { "type": "integer", "minimum": 0 } },
      "additionalProperties": false
    },
    "inspect": { "type": ["object", "null"] },
    "update_safety": { "type": ["object", "null"] },
    "diff": { "type": ["object", "null"] },
    "risk": { "type": ["object", "null"] },
    "policy": {
      "type": "object",
      "required": ["status", "threshold", "highest_risk", "blocking_findings"],
      "properties": {
        "status": { "enum": ["passed", "blocked", "not_evaluated"] },
        "threshold": { "type": ["string", "null"], "enum": ["info", "low", "medium", "high", "critical", null] },
        "highest_risk": { "type": ["string", "null"], "enum": ["info", "low", "medium", "high", "critical", null] },
        "blocking_findings": { "type": "array", "items": { "type": "string" } }
      },
      "additionalProperties": false
    }
  },
  "additionalProperties": false
}
```

- [ ] **Step 5: Run schema tests**

Run:

```bash
cargo test -p contract_tools phase4_build_report_schema --test schema_gate_tests
```

Expected:

```text
test phase4_build_report_schema_is_registered_in_manifest ... ok
test phase4_build_report_schema_is_valid_json_schema ... ok
```

- [ ] **Step 6: Commit**

```bash
git add docs/superpowers/checklists/phase-4-risk-evidence-matrix.md docs/oracles/phase-4-template-card-risk.md contracts/manifest.yaml contracts/schema/build-report.schema.json contract_tools/tests/schema_gate_tests.rs
git commit -m "docs: add phase 4 risk evidence matrix"
```

---

### Task 2: Build Status, Risk Level, Policy, And Report Shape

**Files:**
- Create: `anki_forge/src/build/status.rs`
- Create: `anki_forge/src/build/policy.rs`
- Create: `anki_forge/src/diff/mod.rs`
- Create: `anki_forge/src/risk/mod.rs`
- Modify: `anki_forge/src/build/mod.rs`
- Modify: `anki_forge/src/build/report.rs`
- Modify: `anki_forge/src/build/options.rs`
- Modify: `anki_forge/src/lib.rs`
- Modify: `anki_forge/tests/build_report_tests.rs`
- Modify: `anki_forge/tests/public_api_boundary_tests.rs`

- [ ] **Step 1: Add failing unit tests for typed status and policy**

Append to `anki_forge/tests/build_report_tests.rs`:

```rust
use anki_forge::build::{
    BuildPolicyResult, BuildPolicyStatus, BuildStatus, ComparisonStatus, RiskLevel,
};

#[test]
fn risk_level_order_matches_fail_on_thresholds() {
    assert!(RiskLevel::Critical >= RiskLevel::High);
    assert!(RiskLevel::High >= RiskLevel::Medium);
    assert!(RiskLevel::Medium >= RiskLevel::Low);
    assert!(RiskLevel::Low >= RiskLevel::Info);
}

#[test]
fn policy_blocks_at_or_above_threshold() {
    let result = BuildPolicyResult::evaluate(
        Some(RiskLevel::High),
        Some(RiskLevel::High),
        vec!["RISK.BASELINE_UNAVAILABLE".to_string()],
    );

    assert_eq!(result.status, BuildPolicyStatus::Blocked);
    assert_eq!(result.threshold, Some(RiskLevel::High));
    assert_eq!(result.highest_risk, Some(RiskLevel::High));
    assert_eq!(
        result.blocking_findings,
        vec!["RISK.BASELINE_UNAVAILABLE".to_string()]
    );
}

#[test]
fn policy_passes_below_threshold() {
    let result = BuildPolicyResult::evaluate(
        Some(RiskLevel::High),
        Some(RiskLevel::Medium),
        vec!["RISK.FIELD_REMOVED_OR_RENAMED".to_string()],
    );

    assert_eq!(result.status, BuildPolicyStatus::Passed);
    assert_eq!(result.blocking_findings, Vec::<String>::new());
}

#[test]
fn build_status_precedence_prefers_error_over_invalid_blocked_success() {
    let status = BuildStatus::highest([
        BuildStatus::Success,
        BuildStatus::Blocked,
        BuildStatus::Invalid,
        BuildStatus::Error,
    ]);

    assert_eq!(status, BuildStatus::Error);
}

#[test]
fn comparison_status_serializes_as_snake_case() {
    assert_eq!(
        serde_json::to_string(&ComparisonStatus::NotRequested).unwrap(),
        "\"not_requested\""
    );
    assert_eq!(
        serde_json::to_string(&ComparisonStatus::Unavailable).unwrap(),
        "\"unavailable\""
    );
}
```

Append to `anki_forge/tests/public_api_boundary_tests.rs`:

```rust
#[test]
fn build_api_exports_phase4_report_types() {
    use anki_forge::build::{
        BuildPolicyResult, BuildPolicyStatus, BuildStatus, ComparisonStatus, RiskLevel,
    };

    let _status = BuildStatus::Success;
    let _comparison = ComparisonStatus::NotRequested;
    let _level = RiskLevel::High;
    let _policy = BuildPolicyResult {
        status: BuildPolicyStatus::NotEvaluated,
        threshold: None,
        highest_risk: None,
        blocking_findings: Vec::new(),
    };
}
```

- [ ] **Step 2: Run the failing tests**

Run:

```bash
cargo test -p anki_forge --test build_report_tests risk_level_order_matches_fail_on_thresholds
cargo test -p anki_forge --test public_api_boundary_tests build_api_exports_phase4_report_types
```

Expected:

```text
error[E0432]: unresolved imports `anki_forge::build::BuildPolicyResult`, `anki_forge::build::BuildStatus`
error[E0432]: unresolved imports `anki_forge::build::BuildPolicyResult`, `anki_forge::build::BuildStatus`
```

The five new tests in Step 1 are all expected to be blocked by the same missing-type compile error before Step 3.

- [ ] **Step 3: Add minimal diff/risk carrier modules used by BuildReport**

Create `anki_forge/src/diff/mod.rs`:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildDiffSummary {
    pub artifact_diff: Option<ArtifactDiffSummary>,
    pub semantic_changes: Vec<SemanticDiffChange>,
    pub summary_counts: DiffSummaryCounts,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DiffSummaryCounts {
    pub added: usize,
    pub removed: usize,
    pub modified: usize,
    pub reordered: usize,
    pub uncompared_domains: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactDiffSummary {
    pub changes: Vec<ArtifactDiffChange>,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactDiffChange {
    pub category: String,
    pub domain: String,
    pub severity: String,
    pub selector: String,
    pub message: String,
    pub evidence_refs: Vec<EvidenceRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticDiffCategory {
    Notetype,
    Field,
    Template,
    NoteIdentity,
    CardCount,
    Media,
    Baseline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticDiffChangeKind {
    Added,
    Removed,
    Modified,
    Reordered,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticDiffChange {
    pub category: SemanticDiffCategory,
    pub selector: String,
    pub change_kind: SemanticDiffChangeKind,
    pub risk_codes: Vec<String>,
    pub message: String,
    pub source: Option<crate::diagnostics::SourcePath>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceRefKind {
    Diagnostic,
    DiffChange,
    InspectObservation,
    UpdateSafety,
    Oracle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceRef {
    pub kind: EvidenceRefKind,
    pub ref_id: String,
}
```

Create `anki_forge/src/risk/mod.rs`:

```rust
use crate::build::RiskLevel;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportRiskFinding {
    pub code: String,
    pub level: RiskLevel,
    pub category: String,
    pub message: String,
    pub source: Option<crate::diagnostics::SourcePath>,
    pub evidence_refs: Vec<crate::diff::EvidenceRef>,
    pub suggested_action: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImportRiskReport {
    pub highest_level: Option<RiskLevel>,
    pub findings: Vec<ImportRiskFinding>,
    pub limitations: Vec<String>,
}
```

Modify `anki_forge/src/lib.rs`:

```rust
pub mod diff;
pub mod risk;
```

- [ ] **Step 4: Add status and policy types**

Create `anki_forge/src/build/status.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildStatus {
    #[default]
    Success,
    Blocked,
    Invalid,
    Error,
}

impl BuildStatus {
    pub fn highest<I>(statuses: I) -> Self
    where
        I: IntoIterator<Item = BuildStatus>,
    {
        statuses
            .into_iter()
            .max()
            .unwrap_or(BuildStatus::Success)
    }

    pub fn is_success(self) -> bool {
        matches!(self, BuildStatus::Success)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonStatus {
    #[default]
    NotRequested,
    Complete,
    Partial,
    Unavailable,
}
```

Create `anki_forge/src/build/policy.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildPolicyStatus {
    Passed,
    Blocked,
    NotEvaluated,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildPolicyResult {
    pub status: BuildPolicyStatus,
    pub threshold: Option<RiskLevel>,
    pub highest_risk: Option<RiskLevel>,
    pub blocking_findings: Vec<String>,
}

impl Default for BuildPolicyResult {
    fn default() -> Self {
        Self {
            status: BuildPolicyStatus::NotEvaluated,
            threshold: None,
            highest_risk: None,
            blocking_findings: Vec::new(),
        }
    }
}

impl BuildPolicyResult {
    pub fn evaluate(
        threshold: Option<RiskLevel>,
        highest_risk: Option<RiskLevel>,
        candidate_findings: Vec<String>,
    ) -> Self {
        let Some(threshold) = threshold else {
            return Self {
                status: BuildPolicyStatus::NotEvaluated,
                threshold: None,
                highest_risk,
                blocking_findings: Vec::new(),
            };
        };

        let blocked = highest_risk
            .map(|level| level >= threshold)
            .unwrap_or(false);

        Self {
            status: if blocked {
                BuildPolicyStatus::Blocked
            } else {
                BuildPolicyStatus::Passed
            },
            threshold: Some(threshold),
            highest_risk,
            blocking_findings: if blocked {
                candidate_findings
            } else {
                Vec::new()
            },
        }
    }
}
```

- [ ] **Step 5: Export the new types**

Modify `anki_forge/src/build/mod.rs`:

```rust
pub mod options;
pub mod policy;
pub mod report;
pub mod status;

pub use options::{
    BuildOptions, ProjectDeclaredMimeMismatchBehavior, ProjectMediaDiagnosticBehavior,
    ProjectMediaPolicy, ProjectMediaPolicyError, ProjectNormalizeOptions, UpdateSafetyMode,
};
pub use policy::{BuildPolicyResult, BuildPolicyStatus, RiskLevel};
pub use report::{
    ApkgArtifact, BaselineSourceSummary, BuildCounts, BuildError, BuildFailureCause, BuildMetrics,
    BuildReport, InspectSummary, MediaSummary, UpdateSafetySummary,
};
pub use status::{BuildStatus, ComparisonStatus};
```

- [ ] **Step 6: Extend report and failure cause fields**

Modify `anki_forge/src/build/report.rs` imports and type definitions:

```rust
use crate::build::{
    BuildPolicyResult, BuildPolicyStatus, BuildStatus, ComparisonStatus,
};
use crate::diagnostics::{Diagnostic, Severity};
```

Change `BuildReport` and `BuildFailureCause`:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildReport {
    pub artifact: Option<ApkgArtifact>,
    pub counts: BuildCounts,
    pub media: MediaSummary,
    pub diagnostics: Vec<Diagnostic>,
    pub metrics: BuildMetrics,
    pub inspect: Option<InspectSummary>,
    pub update_safety: Option<UpdateSafetySummary>,
    pub comparison: ComparisonStatus,
    pub diff: Option<crate::diff::BuildDiffSummary>,
    pub risk: Option<crate::risk::ImportRiskReport>,
    pub policy: BuildPolicyResult,
    pub status: BuildStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildFailureCause {
    MissingArtifact,
    Diagnostics,
    PolicyBlocked,
    Invalid,
    Io,
    Internal,
}
```

Update `ensure_success`:

```rust
pub fn ensure_success(&self) -> Result<(), BuildError> {
    if self
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Error)
    {
        return Err(BuildError::new(
            self.clone(),
            BuildFailureCause::Diagnostics,
        ));
    }

    if self.artifact.is_none() {
        return Err(BuildError::new(
            self.clone(),
            BuildFailureCause::MissingArtifact,
        ));
    }

    if matches!(self.policy.status, BuildPolicyStatus::Blocked) {
        return Err(BuildError::new(
            self.clone(),
            BuildFailureCause::PolicyBlocked,
        ));
    }

    if !self.status.is_success() {
        let cause = match self.status {
            BuildStatus::Invalid => BuildFailureCause::Invalid,
            BuildStatus::Error => BuildFailureCause::Internal,
            BuildStatus::Blocked => BuildFailureCause::PolicyBlocked,
            BuildStatus::Success => BuildFailureCause::Internal,
        };
        return Err(BuildError::new(self.clone(), cause));
    }

    Ok(())
}
```

- [ ] **Step 7: Update existing report test fixtures**

Every `BuildReport { ... }` literal in `anki_forge/tests/build_report_tests.rs` must include:

```rust
comparison: ComparisonStatus::NotRequested,
diff: None,
risk: None,
policy: BuildPolicyResult::default(),
status: BuildStatus::Success,
```

For tests that previously used `status: "invalid".into()`, use:

```rust
status: BuildStatus::Invalid,
```

For tests that previously used `status: "error".into()`, use:

```rust
status: BuildStatus::Error,
```

- [ ] **Step 8: Run report tests**

Run:

```bash
cargo test -p anki_forge --test build_report_tests
```

Expected:

```text
test result: ok.
```

- [ ] **Step 9: Commit**

```bash
git add anki_forge/src/build/status.rs anki_forge/src/build/policy.rs anki_forge/src/diff/mod.rs anki_forge/src/risk/mod.rs anki_forge/src/build/mod.rs anki_forge/src/build/report.rs anki_forge/src/lib.rs anki_forge/tests/build_report_tests.rs anki_forge/tests/public_api_boundary_tests.rs
git commit -m "feat: add phase 4 build status and policy types"
```

---

### Task 3: JSON Report Projection And Report File Writing

**Files:**
- Create: `anki_forge/src/build/json_report.rs`
- Modify: `anki_forge/src/build/mod.rs`
- Modify: `anki_forge/src/build/options.rs`
- Modify: `anki_forge/src/diagnostics/mod.rs`
- Modify: `anki_forge/src/diff/mod.rs`
- Modify: `anki_forge/src/risk/mod.rs`
- Modify: `anki_forge/tests/build_report_tests.rs`
- Modify: `anki_forge/tests/public_api_boundary_tests.rs`

- [ ] **Step 1: Add failing projection and options tests**

Append to `anki_forge/tests/build_report_tests.rs`:

```rust
use anki_forge::build::{BuildReportJson, SerializableBuildReport};

fn successful_report_fixture() -> BuildReport {
    BuildReport {
        artifact: Some(ApkgArtifact {
            path: PathBuf::from("out/success.apkg"),
        }),
        counts: BuildCounts {
            notes: 1,
            cards: 1,
            media: 0,
        },
        media: MediaSummary::default(),
        diagnostics: Vec::new(),
        metrics: BuildMetrics {
            duration: std::time::Duration::from_millis(1),
        },
        inspect: None,
        update_safety: None,
        comparison: ComparisonStatus::NotRequested,
        diff: None,
        risk: None,
        policy: BuildPolicyResult::default(),
        status: BuildStatus::Success,
    }
}

#[test]
fn build_report_projection_has_phase4_contract_header() {
    let report = successful_report_fixture();
    let projected = BuildReportJson::from_report(&report);

    assert_eq!(projected.kind, "anki-forge-build-report");
    assert_eq!(projected.schema_version, "phase4-build-report-v1");
    assert!(!projected.tool_version.is_empty());
    assert_eq!(projected.status, BuildStatus::Success);
    assert_eq!(projected.comparison, ComparisonStatus::NotRequested);
}

#[test]
fn build_report_projection_serializes_duration_as_milliseconds() {
    let mut report = successful_report_fixture();
    report.metrics = BuildMetrics {
        duration: std::time::Duration::from_millis(42),
    };

    let json = serde_json::to_value(BuildReportJson::from_report(&report)).unwrap();
    assert_eq!(json["metrics"]["duration_ms"], 42);
}

#[test]
fn build_report_projection_serializes_diagnostics_as_strings() {
    let mut report = successful_report_fixture();
    report.diagnostics.push(Diagnostic {
        code: DiagnosticCode::new("PROJECT.NORMALIZE_FAILED"),
        severity: Severity::Error,
        message: "normalization failed".to_string(),
        source: Some(SourcePath::new("project")),
        help: Some("inspect the Product input".to_string()),
    });

    let json = serde_json::to_value(BuildReportJson::from_report(&report)).unwrap();
    assert_eq!(json["diagnostics"][0]["code"], "PROJECT.NORMALIZE_FAILED");
    assert_eq!(json["diagnostics"][0]["severity"], "error");
    assert_eq!(json["diagnostics"][0]["source"], "project");
}
```

Append to `anki_forge/tests/public_api_boundary_tests.rs`:

```rust
#[test]
fn build_options_expose_phase4_builder_methods() {
    use anki_forge::build::{BuildOptions, RiskLevel};

    let options = BuildOptions::new()
        .fail_on(RiskLevel::High)
        .report_json("build-report.json");

    assert_eq!(options.fail_on, Some(RiskLevel::High));
    assert_eq!(
        options.report_json.as_deref(),
        Some(std::path::Path::new("build-report.json"))
    );
}
```

- [ ] **Step 2: Run the failing tests**

Run:

```bash
cargo test -p anki_forge --test build_report_tests build_report_projection_has_phase4_contract_header
cargo test -p anki_forge --test public_api_boundary_tests build_options_expose_phase4_builder_methods
```

Expected:

```text
error[E0432]: unresolved imports `anki_forge::build::BuildReportJson`
error[E0432]: unresolved import `anki_forge::build::RiskLevel` or no field `fail_on`
```

- [ ] **Step 3: Make diagnostics serializable for report projection**

Modify `anki_forge/src/diagnostics/mod.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticCode(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourcePath(String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub severity: Severity,
    pub message: String,
    pub source: Option<SourcePath>,
    pub help: Option<String>,
}
```

Keep the existing `impl` blocks unchanged.

- [ ] **Step 4: Make the initial diff/risk report carriers serializable**

Modify `anki_forge/src/diff/mod.rs` by adding:

```rust
use serde::{Deserialize, Serialize};
```

Add `Serialize, Deserialize` to each struct and enum derive in that file. Add snake-case serde names to `SemanticDiffCategory`, `SemanticDiffChangeKind`, and `EvidenceRefKind`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceRefKind {
    Diagnostic,
    DiffChange,
    InspectObservation,
    UpdateSafety,
    Oracle,
}
```

Modify `anki_forge/src/risk/mod.rs` by adding:

```rust
use serde::{Deserialize, Serialize};
```

Add `Serialize, Deserialize` to `ImportRiskFinding` and `ImportRiskReport`.

- [ ] **Step 5: Add BuildOptions fields and methods**

Modify `anki_forge/src/build/options.rs`:

```rust
use crate::build::RiskLevel;
use std::path::PathBuf;
```

Add fields to `BuildOptions`:

```rust
pub fail_on: Option<RiskLevel>,
pub report_json: Option<PathBuf>,
```

Set defaults:

```rust
fail_on: None,
report_json: None,
```

Add builder methods:

```rust
pub fn fail_on(mut self, level: RiskLevel) -> Self {
    self.fail_on = Some(level);
    self
}

pub fn report_json(mut self, path: impl Into<PathBuf>) -> Self {
    self.report_json = Some(path.into());
    self
}
```

- [ ] **Step 6: Add JSON projection module**

Create `anki_forge/src/build/json_report.rs`:

```rust
use serde::Serialize;
use std::path::Path;

use crate::build::{
    ApkgArtifact, BuildCounts, BuildMetrics, BuildPolicyResult, BuildReport, BuildStatus,
    ComparisonStatus, InspectSummary, MediaSummary, UpdateSafetySummary,
};

#[derive(Debug, Clone, Serialize)]
pub struct BuildReportJson {
    pub kind: &'static str,
    pub schema_version: &'static str,
    pub tool_version: String,
    pub artifact: Option<ApkgArtifactJson>,
    pub status: BuildStatus,
    pub comparison: ComparisonStatus,
    pub counts: BuildCountsJson,
    pub media: MediaSummaryJson,
    pub diagnostics: Vec<crate::diagnostics::Diagnostic>,
    pub metrics: BuildMetricsJson,
    pub inspect: Option<InspectSummaryJson>,
    pub update_safety: Option<UpdateSafetySummary>,
    pub diff: Option<crate::diff::BuildDiffSummary>,
    pub risk: Option<crate::risk::ImportRiskReport>,
    pub policy: BuildPolicyResult,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApkgArtifactJson {
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BuildCountsJson {
    pub notes: usize,
    pub cards: usize,
    pub media: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct MediaSummaryJson {
    pub objects: usize,
    pub bindings: usize,
    pub references: usize,
    pub missing_references: usize,
    pub unsafe_references: usize,
    pub unused_bindings: usize,
    pub unique_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct BuildMetricsJson {
    pub duration_ms: u128,
}

#[derive(Debug, Clone, Serialize)]
pub struct InspectSummaryJson {
    pub source_kind: String,
    pub observation_status: String,
    pub notes: usize,
    pub cards: usize,
    pub notetypes: usize,
    pub templates: usize,
    pub fields: usize,
    pub media: usize,
}

pub trait SerializableBuildReport {
    fn to_report_json(&self) -> BuildReportJson;
}

impl SerializableBuildReport for BuildReport {
    fn to_report_json(&self) -> BuildReportJson {
        BuildReportJson::from_report(self)
    }
}

impl BuildReportJson {
    pub fn from_report(report: &BuildReport) -> Self {
        Self {
            kind: "anki-forge-build-report",
            schema_version: "phase4-build-report-v1",
            tool_version: crate::facade_api_version().to_string(),
            artifact: report.artifact.as_ref().map(ApkgArtifactJson::from),
            status: report.status,
            comparison: report.comparison,
            counts: BuildCountsJson::from(report.counts),
            media: MediaSummaryJson::from(report.media),
            diagnostics: report.diagnostics.clone(),
            metrics: BuildMetricsJson::from(report.metrics),
            inspect: report.inspect.as_ref().map(InspectSummaryJson::from),
            update_safety: report.update_safety.clone(),
            diff: report.diff.clone(),
            risk: report.risk.clone(),
            policy: report.policy.clone(),
        }
    }
}

impl From<&ApkgArtifact> for ApkgArtifactJson {
    fn from(value: &ApkgArtifact) -> Self {
        Self {
            path: value.path.display().to_string(),
        }
    }
}

impl From<BuildCounts> for BuildCountsJson {
    fn from(value: BuildCounts) -> Self {
        Self {
            notes: value.notes,
            cards: value.cards,
            media: value.media,
        }
    }
}

impl From<MediaSummary> for MediaSummaryJson {
    fn from(value: MediaSummary) -> Self {
        Self {
            objects: value.objects,
            bindings: value.bindings,
            references: value.references,
            missing_references: value.missing_references,
            unsafe_references: value.unsafe_references,
            unused_bindings: value.unused_bindings,
            unique_bytes: value.unique_bytes,
        }
    }
}

impl From<BuildMetrics> for BuildMetricsJson {
    fn from(value: BuildMetrics) -> Self {
        Self {
            duration_ms: value.duration.as_millis(),
        }
    }
}

impl From<&InspectSummary> for InspectSummaryJson {
    fn from(value: &InspectSummary) -> Self {
        Self {
            source_kind: value.source_kind.clone(),
            observation_status: value.observation_status.clone(),
            notes: value.notes,
            cards: value.cards,
            notetypes: value.notetypes,
            templates: value.templates,
            fields: value.fields,
            media: value.media,
        }
    }
}

pub fn write_report_json_atomic(path: &Path, report: &BuildReport) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let temp_path = path.with_extension("tmp");
    let bytes = serde_json::to_vec_pretty(&BuildReportJson::from_report(report))?;
    std::fs::write(&temp_path, bytes)?;
    std::fs::rename(&temp_path, path)?;
    Ok(())
}
```

- [ ] **Step 7: Export projection helpers**

Modify `anki_forge/src/build/mod.rs`:

```rust
pub mod json_report;
pub mod options;
pub mod policy;
pub mod report;
pub mod status;

pub use json_report::{
    write_report_json_atomic, BuildReportJson, SerializableBuildReport,
};
```

Keep the existing `pub use` groups for options, policy, report, and status.

- [ ] **Step 8: Run projection and API tests**

Run:

```bash
cargo test -p anki_forge --test build_report_tests build_report_projection
cargo test -p anki_forge --test public_api_boundary_tests build_options_expose_phase4_builder_methods
```

Expected:

```text
test result: ok.
test result: ok.
```

- [ ] **Step 9: Commit**

```bash
git add anki_forge/src/build/json_report.rs anki_forge/src/build/mod.rs anki_forge/src/build/options.rs anki_forge/src/diagnostics/mod.rs anki_forge/src/diff/mod.rs anki_forge/src/risk/mod.rs anki_forge/tests/build_report_tests.rs anki_forge/tests/public_api_boundary_tests.rs
git commit -m "feat: add build report json projection"
```

---

### Task 4: Product Diff And Risk Data Models

**Files:**
- Modify: `anki_forge/src/diff/mod.rs`
- Create: `anki_forge/src/diff/summary.rs`
- Modify: `anki_forge/src/risk/mod.rs`
- Create: `anki_forge/src/risk/model.rs`
- Create: `anki_forge/src/risk/policy.rs`
- Create: `anki_forge/src/risk/rules.rs`
- Modify: `anki_forge/tests/build_report_tests.rs`
- Create: `anki_forge/tests/phase4_diff_risk_model_tests.rs`

- [ ] **Step 1: Add failing model tests**

Create `anki_forge/tests/phase4_diff_risk_model_tests.rs`:

```rust
use anki_forge::build::RiskLevel;
use anki_forge::diff::{
    ArtifactDiffChange, ArtifactDiffSummary, BuildDiffSummary, DiffSummaryCounts, EvidenceRef,
    EvidenceRefKind, SemanticDiffChange, SemanticDiffCategory, SemanticDiffChangeKind,
};
use anki_forge::risk::{ImportRiskFinding, ImportRiskReport};

#[test]
fn diff_summary_counts_artifact_and_semantic_changes() {
    let summary = BuildDiffSummary {
        artifact_diff: Some(ArtifactDiffSummary {
            changes: vec![ArtifactDiffChange {
                category: "removed".to_string(),
                domain: "templates".to_string(),
                severity: "high".to_string(),
                selector: "notetype:jp/template:Recognition".to_string(),
                message: "template removed".to_string(),
                evidence_refs: vec![EvidenceRef {
                    kind: EvidenceRefKind::DiffChange,
                    ref_id: "diff:templates:0".to_string(),
                }],
            }],
            limitations: Vec::new(),
        }),
        semantic_changes: vec![SemanticDiffChange {
            category: SemanticDiffCategory::Template,
            selector: "notetype:jp/template:Recognition".to_string(),
            change_kind: SemanticDiffChangeKind::Removed,
            risk_codes: vec!["RISK.TEMPLATE_REMOVED".to_string()],
            message: "template Recognition was removed".to_string(),
            source: None,
        }],
        summary_counts: DiffSummaryCounts {
            added: 0,
            removed: 1,
            modified: 0,
            reordered: 0,
            uncompared_domains: 0,
        },
        limitations: Vec::new(),
    };

    assert_eq!(summary.summary_counts.removed, 1);
    assert_eq!(summary.semantic_changes[0].risk_codes[0], "RISK.TEMPLATE_REMOVED");
}

#[test]
fn import_risk_report_computes_highest_level() {
    let report = ImportRiskReport::from_findings(vec![
        ImportRiskFinding {
            code: "RISK.FIELD_REMOVED_OR_RENAMED".to_string(),
            level: RiskLevel::Medium,
            category: "field".to_string(),
            message: "field removed".to_string(),
            source: None,
            evidence_refs: Vec::new(),
            suggested_action: Some("restore the stable field key or confirm the rename".to_string()),
        },
        ImportRiskFinding {
            code: "RISK.TEMPLATE_REMOVED".to_string(),
            level: RiskLevel::Critical,
            category: "template".to_string(),
            message: "template removed".to_string(),
            source: None,
            evidence_refs: Vec::new(),
            suggested_action: Some("restore the template or migrate existing cards".to_string()),
        },
    ]);

    assert_eq!(report.highest_level, Some(RiskLevel::Critical));
}

#[test]
fn writer_diff_summary_maps_template_removal_to_semantic_risk() {
    let writer = writer_core::DiffReport {
        kind: "inspect-diff".to_string(),
        comparison_status: "complete".to_string(),
        left_fingerprint: "left".to_string(),
        right_fingerprint: "right".to_string(),
        left_observation_model_version: "v1".to_string(),
        right_observation_model_version: "v1".to_string(),
        summary: "1 change".to_string(),
        uncompared_domains: Vec::new(),
        comparison_limitations: Vec::new(),
        changes: vec![writer_core::DiffChange {
            category: "removed".to_string(),
            domain: "templates".to_string(),
            severity: "medium".to_string(),
            selector: "notetype[jp] template[Recognition]".to_string(),
            message: "template removed".to_string(),
            compatibility_hint: "review import behavior".to_string(),
            evidence_refs: Vec::new(),
        }],
    };

    let summary = anki_forge::diff::summarize_writer_diff(&writer);
    assert_eq!(summary.summary_counts.removed, 1);
    assert_eq!(summary.semantic_changes.len(), 1);
    assert_eq!(summary.semantic_changes[0].category, SemanticDiffCategory::Template);
    assert_eq!(
        summary.semantic_changes[0].risk_codes,
        vec!["RISK.TEMPLATE_REMOVED".to_string()]
    );
}
```

- [ ] **Step 2: Run the failing tests**

Run:

```bash
cargo test -p anki_forge --test phase4_diff_risk_model_tests
```

Expected:

```text
error[E0432]: unresolved import `anki_forge::diff`
```

- [ ] **Step 3: Replace the minimal diff carrier with serializable diff module types**

Replace `anki_forge/src/diff/mod.rs` with:

```rust
pub mod summary;

use serde::{Deserialize, Serialize};

pub use summary::summarize_writer_diff;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildDiffSummary {
    pub artifact_diff: Option<ArtifactDiffSummary>,
    pub semantic_changes: Vec<SemanticDiffChange>,
    pub summary_counts: DiffSummaryCounts,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffSummaryCounts {
    pub added: usize,
    pub removed: usize,
    pub modified: usize,
    pub reordered: usize,
    pub uncompared_domains: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactDiffSummary {
    pub changes: Vec<ArtifactDiffChange>,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactDiffChange {
    pub category: String,
    pub domain: String,
    pub severity: String,
    pub selector: String,
    pub message: String,
    pub evidence_refs: Vec<EvidenceRef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticDiffCategory {
    Notetype,
    Field,
    Template,
    NoteIdentity,
    CardCount,
    Media,
    Baseline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticDiffChangeKind {
    Added,
    Removed,
    Modified,
    Reordered,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticDiffChange {
    pub category: SemanticDiffCategory,
    pub selector: String,
    pub change_kind: SemanticDiffChangeKind,
    pub risk_codes: Vec<String>,
    pub message: String,
    pub source: Option<crate::diagnostics::SourcePath>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceRefKind {
    Diagnostic,
    DiffChange,
    InspectObservation,
    UpdateSafety,
    Oracle,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceRef {
    pub kind: EvidenceRefKind,
    pub ref_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectDiffReport {
    pub status: crate::build::BuildStatus,
    pub comparison: crate::build::ComparisonStatus,
    pub diagnostics: Vec<crate::diagnostics::Diagnostic>,
    pub current_inspect: Option<crate::build::InspectSummary>,
    pub previous_inspect: Option<crate::build::InspectSummary>,
    pub update_safety: Option<crate::build::UpdateSafetySummary>,
    pub diff: Option<BuildDiffSummary>,
    pub risk: Option<crate::risk::ImportRiskReport>,
    pub metrics: ComparisonMetrics,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComparisonMetrics {
    pub duration_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectDiffError {
    pub report: Box<ProjectDiffReport>,
    pub cause: crate::build::BuildFailureCause,
}

impl ProjectDiffError {
    pub fn new(report: ProjectDiffReport, cause: crate::build::BuildFailureCause) -> Self {
        Self {
            report: Box::new(report),
            cause,
        }
    }
}
```

- [ ] **Step 4: Replace the minimal risk carrier with serializable risk model types**

Replace `anki_forge/src/risk/mod.rs` with:

```rust
pub mod model;
pub mod policy;
pub mod rules;

pub use model::{ImportRiskFinding, ImportRiskReport};
pub use policy::policy_from_risk_report;
pub use rules::classify_import_risk;
```

Create `anki_forge/src/risk/model.rs`:

```rust
use serde::{Deserialize, Serialize};

use crate::build::RiskLevel;
use crate::diff::EvidenceRef;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportRiskFinding {
    pub code: String,
    pub level: RiskLevel,
    pub category: String,
    pub message: String,
    pub source: Option<crate::diagnostics::SourcePath>,
    pub evidence_refs: Vec<EvidenceRef>,
    pub suggested_action: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportRiskReport {
    pub highest_level: Option<RiskLevel>,
    pub findings: Vec<ImportRiskFinding>,
    pub limitations: Vec<String>,
}

impl ImportRiskReport {
    pub fn from_findings(findings: Vec<ImportRiskFinding>) -> Self {
        let highest_level = findings.iter().map(|finding| finding.level).max();
        Self {
            highest_level,
            findings,
            limitations: Vec::new(),
        }
    }

    pub fn blocking_codes_at_or_above(&self, threshold: RiskLevel) -> Vec<String> {
        self.findings
            .iter()
            .filter(|finding| finding.level >= threshold)
            .map(|finding| finding.code.clone())
            .collect()
    }
}
```

Create `anki_forge/src/risk/policy.rs`:

```rust
use crate::build::{BuildPolicyResult, RiskLevel};
use crate::risk::ImportRiskReport;

pub fn policy_from_risk_report(
    threshold: Option<RiskLevel>,
    risk: Option<&ImportRiskReport>,
) -> BuildPolicyResult {
    let highest = risk.and_then(|report| report.highest_level);
    let blocking_codes = match (threshold, risk) {
        (Some(threshold), Some(report)) => report.blocking_codes_at_or_above(threshold),
        _ => Vec::new(),
    };
    BuildPolicyResult::evaluate(threshold, highest, blocking_codes)
}
```

Create an empty first implementation in `anki_forge/src/risk/rules.rs`:

```rust
use crate::risk::ImportRiskReport;

#[derive(Debug, Clone)]
pub struct RiskInput<'a> {
    pub diagnostics: &'a [crate::diagnostics::Diagnostic],
    pub comparison: crate::build::ComparisonStatus,
    pub diff: Option<&'a crate::diff::BuildDiffSummary>,
    pub current_inspect: Option<&'a crate::build::InspectSummary>,
    pub previous_inspect: Option<&'a crate::build::InspectSummary>,
    pub update_safety: Option<&'a crate::build::UpdateSafetySummary>,
}

pub fn classify_import_risk(input: RiskInput<'_>) -> ImportRiskReport {
    let _ = input;
    ImportRiskReport::default()
}
```

- [ ] **Step 5: Add a writer diff summary adapter skeleton**

Create `anki_forge/src/diff/summary.rs`:

```rust
use crate::diff::{
    ArtifactDiffChange, ArtifactDiffSummary, BuildDiffSummary, DiffSummaryCounts, EvidenceRef,
    EvidenceRefKind, SemanticDiffCategory, SemanticDiffChange, SemanticDiffChangeKind,
};

pub fn summarize_writer_diff(report: &writer_core::DiffReport) -> BuildDiffSummary {
    let changes = report
        .changes
        .iter()
        .enumerate()
        .map(|(index, change)| ArtifactDiffChange {
            category: change.category.clone(),
            domain: change.domain.clone(),
            severity: change.severity.clone(),
            selector: change.selector.clone(),
            message: change.message.clone(),
            evidence_refs: vec![EvidenceRef {
                kind: EvidenceRefKind::DiffChange,
                ref_id: format!("diff:{}:{index}", change.domain),
            }],
        })
        .collect::<Vec<_>>();

    let mut counts = DiffSummaryCounts {
        uncompared_domains: report.uncompared_domains.len(),
        ..DiffSummaryCounts::default()
    };
    for change in &changes {
        match change.category.as_str() {
            "added" => counts.added += 1,
            "removed" => counts.removed += 1,
            "modified" => counts.modified += 1,
            _ => counts.modified += 1,
        }
    }

    let semantic_changes = report
        .changes
        .iter()
        .filter_map(semantic_change_from_writer_change)
        .collect::<Vec<_>>();
    let mut limitations = report.comparison_limitations.clone();
    if semantic_changes
        .iter()
        .any(|change| change.risk_codes.iter().any(|code| code == "RISK.FIELD_REMOVED_OR_RENAMED"))
    {
        limitations.push(
            "field_safe_rename_proof_not_applied: inspect exposes field config_id, but the first Product diff adapter does not receive paired before/after field payloads; field removal or rename is treated as unsafe"
                .to_string(),
        );
    }

    BuildDiffSummary {
        artifact_diff: Some(ArtifactDiffSummary {
            changes,
            limitations: limitations.clone(),
        }),
        semantic_changes,
        summary_counts: counts,
        limitations,
    }
}

fn semantic_change_from_writer_change(change: &writer_core::DiffChange) -> Option<SemanticDiffChange> {
    let (category, change_kind, risk_code) = match (change.domain.as_str(), change.category.as_str()) {
        ("templates", "removed") => (
            SemanticDiffCategory::Template,
            SemanticDiffChangeKind::Removed,
            "RISK.TEMPLATE_REMOVED",
        ),
        ("templates", "modified") if change.message.contains("ord") || change.selector.contains("ord") => (
            SemanticDiffCategory::Template,
            SemanticDiffChangeKind::Reordered,
            "RISK.TEMPLATE_REORDER",
        ),
        ("fields", "removed") => (
            SemanticDiffCategory::Field,
            SemanticDiffChangeKind::Removed,
            "RISK.FIELD_REMOVED_OR_RENAMED",
        ),
        ("media", "removed") => (
            SemanticDiffCategory::Media,
            SemanticDiffChangeKind::Removed,
            "RISK.MEDIA_REMOVED",
        ),
        ("metadata", "modified") if change.selector.contains("card_count") => (
            SemanticDiffCategory::CardCount,
            SemanticDiffChangeKind::Modified,
            "RISK.CARD_COUNT_CHANGED",
        ),
        _ => return None,
    };

    Some(SemanticDiffChange {
        category,
        selector: change.selector.clone(),
        change_kind,
        risk_codes: vec![risk_code.to_string()],
        message: change.message.clone(),
        source: None,
    })
}
```

- [ ] **Step 6: Confirm modules remain exported from crate root**

Run:

```bash
rg -n "pub mod diff;|pub mod risk;" anki_forge/src/lib.rs
```

Expected:

```text
anki_forge/src/lib.rs: contains pub mod diff;
anki_forge/src/lib.rs: contains pub mod risk;
```

- [ ] **Step 7: Run model tests**

Run:

```bash
cargo test -p anki_forge --test phase4_diff_risk_model_tests
```

Expected:

```text
test result: ok.
```

- [ ] **Step 8: Commit**

```bash
git add anki_forge/src/diff anki_forge/src/risk anki_forge/tests/phase4_diff_risk_model_tests.rs anki_forge/tests/build_report_tests.rs
git commit -m "feat: add phase 4 diff and risk models"
```

---

### Task 5: Risk Rules

**Files:**
- Modify: `anki_forge/src/risk/rules.rs`
- Modify: `anki_forge/src/risk/model.rs`
- Create: `anki_forge/tests/phase4_risk_rules_tests.rs`

- [ ] **Step 1: Add failing risk-rule tests**

Create `anki_forge/tests/phase4_risk_rules_tests.rs`:

```rust
use anki_forge::build::{ComparisonStatus, RiskLevel};
use anki_forge::diagnostics::{Diagnostic, DiagnosticCode, Severity};
use anki_forge::diff::{
    ArtifactDiffChange, ArtifactDiffSummary, BuildDiffSummary, DiffSummaryCounts, EvidenceRef,
    EvidenceRefKind, SemanticDiffChange, SemanticDiffCategory, SemanticDiffChangeKind,
};
use anki_forge::risk::rules::{classify_import_risk, RiskInput};

fn diagnostic(code: &str, severity: Severity) -> Diagnostic {
    Diagnostic {
        code: DiagnosticCode::new(code),
        severity,
        message: format!("{code} message"),
        source: None,
        help: None,
    }
}

fn diff_with_semantic_change(code: &str, category: SemanticDiffCategory) -> BuildDiffSummary {
    BuildDiffSummary {
        artifact_diff: Some(ArtifactDiffSummary {
            changes: vec![ArtifactDiffChange {
                category: "removed".to_string(),
                domain: "templates".to_string(),
                severity: "high".to_string(),
                selector: "selector:1".to_string(),
                message: "artifact change".to_string(),
                evidence_refs: vec![EvidenceRef {
                    kind: EvidenceRefKind::DiffChange,
                    ref_id: "diff:templates:0".to_string(),
                }],
            }],
            limitations: Vec::new(),
        }),
        semantic_changes: vec![SemanticDiffChange {
            category,
            selector: "selector:1".to_string(),
            change_kind: SemanticDiffChangeKind::Removed,
            risk_codes: vec![code.to_string()],
            message: "semantic change".to_string(),
            source: None,
        }],
        summary_counts: DiffSummaryCounts {
            added: 0,
            removed: 1,
            modified: 0,
            reordered: 0,
            uncompared_domains: 0,
        },
        limitations: Vec::new(),
    }
}

#[test]
fn baseline_unavailable_emits_high_risk() {
    let report = classify_import_risk(RiskInput {
        diagnostics: &[],
        comparison: ComparisonStatus::Unavailable,
        diff: None,
        current_inspect: None,
        previous_inspect: None,
        update_safety: None,
    });

    let finding = report
        .findings
        .iter()
        .find(|finding| finding.code == "RISK.BASELINE_UNAVAILABLE")
        .expect("baseline finding");
    assert_eq!(finding.level, RiskLevel::High);
}

#[test]
fn broken_media_reference_emits_high_risk_without_baseline() {
    let diagnostics = vec![diagnostic("MEDIA.MISSING_REFERENCE", Severity::Error)];
    let report = classify_import_risk(RiskInput {
        diagnostics: &diagnostics,
        comparison: ComparisonStatus::NotRequested,
        diff: None,
        current_inspect: None,
        previous_inspect: None,
        update_safety: None,
    });

    let finding = report
        .findings
        .iter()
        .find(|finding| finding.code == "RISK.MEDIA_REFERENCE_BROKEN")
        .expect("media finding");
    assert_eq!(finding.level, RiskLevel::High);
}

#[test]
fn template_removed_emits_critical_and_promotes_card_count() {
    let mut diff = diff_with_semantic_change(
        "RISK.TEMPLATE_REMOVED",
        SemanticDiffCategory::Template,
    );
    diff.semantic_changes.push(SemanticDiffChange {
        category: SemanticDiffCategory::CardCount,
        selector: "cards".to_string(),
        change_kind: SemanticDiffChangeKind::Modified,
        risk_codes: vec!["RISK.CARD_COUNT_CHANGED".to_string()],
        message: "card count changed".to_string(),
        source: None,
    });

    let report = classify_import_risk(RiskInput {
        diagnostics: &[],
        comparison: ComparisonStatus::Complete,
        diff: Some(&diff),
        current_inspect: None,
        previous_inspect: None,
        update_safety: None,
    });

    let template = report
        .findings
        .iter()
        .find(|finding| finding.code == "RISK.TEMPLATE_REMOVED")
        .expect("template finding");
    let card = report
        .findings
        .iter()
        .find(|finding| finding.code == "RISK.CARD_COUNT_CHANGED")
        .expect("card finding");

    assert_eq!(template.level, RiskLevel::Critical);
    assert_eq!(card.level, RiskLevel::High);
    assert_eq!(report.highest_level, Some(RiskLevel::Critical));
}
```

- [ ] **Step 2: Run failing risk tests**

Run:

```bash
cargo test -p anki_forge --test phase4_risk_rules_tests
```

Expected:

```text
baseline_unavailable_emits_high_risk ... FAILED
broken_media_reference_emits_high_risk_without_baseline ... FAILED
template_removed_emits_critical_and_promotes_card_count ... FAILED
```

- [ ] **Step 3: Implement risk rules**

Replace `classify_import_risk` in `anki_forge/src/risk/rules.rs` with:

```rust
use crate::build::{ComparisonStatus, RiskLevel};
use crate::diagnostics::Severity;
use crate::diff::{EvidenceRef, EvidenceRefKind, SemanticDiffCategory};
use crate::risk::{ImportRiskFinding, ImportRiskReport};

#[derive(Debug, Clone)]
pub struct RiskInput<'a> {
    pub diagnostics: &'a [crate::diagnostics::Diagnostic],
    pub comparison: ComparisonStatus,
    pub diff: Option<&'a crate::diff::BuildDiffSummary>,
    pub current_inspect: Option<&'a crate::build::InspectSummary>,
    pub previous_inspect: Option<&'a crate::build::InspectSummary>,
    pub update_safety: Option<&'a crate::build::UpdateSafetySummary>,
}

pub fn classify_import_risk(input: RiskInput<'_>) -> ImportRiskReport {
    let mut findings = Vec::new();
    let _update_safety_evidence_is_carried_by_diagnostics = input.update_safety;

    if matches!(input.comparison, ComparisonStatus::Unavailable) {
        findings.push(finding(
            "RISK.BASELINE_UNAVAILABLE",
            RiskLevel::High,
            "baseline",
            "compare_to was requested, but the previous APKG could not be inspected completely",
            vec![EvidenceRef {
                kind: EvidenceRefKind::Oracle,
                ref_id: "manual-doc:docs-api-design-phase4-baseline".to_string(),
            }],
            "verify the previous APKG path and rebuild with a readable baseline",
        ));
    }

    for (index, diagnostic) in input.diagnostics.iter().enumerate() {
        let code = diagnostic.code.as_str();
        if matches!(code, "MEDIA.MISSING_REFERENCE" | "MEDIA.UNSAFE_REFERENCE")
            && diagnostic.severity == Severity::Error
        {
            let mut item = finding(
                "RISK.MEDIA_REFERENCE_BROKEN",
                RiskLevel::High,
                "media",
                "current project contains a broken or unsafe media reference",
                vec![EvidenceRef {
                    kind: EvidenceRefKind::Diagnostic,
                    ref_id: format!("diagnostic:{index}:{code}"),
                }],
                "register the media file or update the Product content reference",
            );
            item.source = diagnostic.source.clone();
            findings.push(item);
        }

        if matches!(
            code,
            "UPDATE.GUID_DERIVATION_DRIFT"
                | "UPDATE.BASELINE_CONFLICT_GUID"
                | "UPDATE.GUID_DUPLICATE_AT_RECONCILE"
                | "UPDATE.GUID_DUPLICATE_IN_BASELINE"
        ) {
            findings.push(finding(
                "RISK.NOTE_GUID_DRIFT",
                RiskLevel::High,
                "identity",
                "same stable note identity maps to a different Anki GUID",
                vec![EvidenceRef {
                    kind: EvidenceRefKind::Diagnostic,
                    ref_id: format!("diagnostic:{index}:{code}"),
                }],
                "restore the previous stable id mapping or review the intentional GUID migration",
            ));
        }

        if matches!(
            code,
            "UPDATE.FIELD_MERGE_ID_CHANGED"
                | "UPDATE.TEMPLATE_MERGE_ID_CHANGED"
                | "UPDATE.FIELD_ORD_CHANGED"
                | "UPDATE.TEMPLATE_ORD_CHANGED"
                | "UPDATE.NOTETYPE_SET_CHANGED"
                | "UPDATE.TEMPLATE_SET_CHANGED"
        ) {
            findings.push(finding(
                "RISK.NOTETYPE_CONFIG_ID_DRIFT",
                RiskLevel::High,
                "notetype",
                "notetype, field, or template merge identity changed unexpectedly",
                vec![EvidenceRef {
                    kind: EvidenceRefKind::Diagnostic,
                    ref_id: format!("diagnostic:{index}:{code}"),
                }],
                "restore stable field/template keys or regenerate the previous package intentionally",
            ));
        }
    }

    if let Some(diff) = input.diff {
        for (index, change) in diff.semantic_changes.iter().enumerate() {
            for code in &change.risk_codes {
                let level = level_for_semantic_code(code);
                let evidence = vec![EvidenceRef {
                    kind: EvidenceRefKind::DiffChange,
                    ref_id: format!("semantic:{index}:{}", change.selector),
                }];
                findings.push(ImportRiskFinding {
                    code: code.clone(),
                    level,
                    category: semantic_category_name(change.category).to_string(),
                    message: change.message.clone(),
                    source: change.source.clone(),
                    evidence_refs: evidence,
                    suggested_action: suggested_action_for_code(code).map(str::to_string),
                });
            }
        }
    }

    promote_card_count_with_template_removed(&mut findings);
    ImportRiskReport::from_findings(findings)
}

fn finding(
    code: &str,
    level: RiskLevel,
    category: &str,
    message: &str,
    evidence_refs: Vec<EvidenceRef>,
    suggested_action: &str,
) -> ImportRiskFinding {
    ImportRiskFinding {
        code: code.to_string(),
        level,
        category: category.to_string(),
        message: message.to_string(),
        source: None,
        evidence_refs,
        suggested_action: Some(suggested_action.to_string()),
    }
}

fn level_for_semantic_code(code: &str) -> RiskLevel {
    match code {
        "RISK.TEMPLATE_REMOVED" => RiskLevel::Critical,
        "RISK.TEMPLATE_REORDER" => RiskLevel::High,
        "RISK.FIELD_REMOVED_OR_RENAMED" => RiskLevel::Medium,
        "RISK.CARD_COUNT_CHANGED" => RiskLevel::Medium,
        "RISK.MEDIA_REMOVED" => RiskLevel::Medium,
        "RISK.NOTE_GUID_DRIFT" => RiskLevel::High,
        "RISK.NOTETYPE_CONFIG_ID_DRIFT" => RiskLevel::High,
        "RISK.MEDIA_REFERENCE_BROKEN" => RiskLevel::High,
        "RISK.BASELINE_UNAVAILABLE" => RiskLevel::High,
        _ => RiskLevel::Low,
    }
}

fn suggested_action_for_code(code: &str) -> Option<&'static str> {
    match code {
        "RISK.TEMPLATE_REMOVED" => Some("restore the template or document the card migration"),
        "RISK.TEMPLATE_REORDER" => Some("preserve template keys and ordinals for existing cards"),
        "RISK.FIELD_REMOVED_OR_RENAMED" => Some("preserve the field key/config id or confirm the migration"),
        "RISK.CARD_COUNT_CHANGED" => Some("review expected card generation changes before importing"),
        "RISK.MEDIA_REMOVED" => Some("restore removed media or verify no notes reference it"),
        _ => None,
    }
}

fn semantic_category_name(category: SemanticDiffCategory) -> &'static str {
    match category {
        SemanticDiffCategory::Notetype => "notetype",
        SemanticDiffCategory::Field => "field",
        SemanticDiffCategory::Template => "template",
        SemanticDiffCategory::NoteIdentity => "note_identity",
        SemanticDiffCategory::CardCount => "card_count",
        SemanticDiffCategory::Media => "media",
        SemanticDiffCategory::Baseline => "baseline",
    }
}

fn promote_card_count_with_template_removed(findings: &mut [ImportRiskFinding]) {
    let template_removed = findings
        .iter()
        .any(|finding| finding.code == "RISK.TEMPLATE_REMOVED");
    if !template_removed {
        return;
    }

    for finding in findings {
        if finding.code == "RISK.CARD_COUNT_CHANGED" {
            finding.level = RiskLevel::High;
            finding.evidence_refs.push(EvidenceRef {
                kind: EvidenceRefKind::DiffChange,
                ref_id: "linked:RISK.TEMPLATE_REMOVED".to_string(),
            });
        }
    }
}
```

- [ ] **Step 4: Run risk-rule tests**

Run:

```bash
cargo test -p anki_forge --test phase4_risk_rules_tests
```

Expected:

```text
test result: ok.
```

- [ ] **Step 5: Commit**

```bash
git add anki_forge/src/risk/rules.rs anki_forge/src/risk/model.rs anki_forge/tests/phase4_risk_rules_tests.rs
git commit -m "feat: classify phase 4 import risks"
```

---

### Task 6: Comparison Assembler

**Files:**
- Create: `anki_forge/src/product/comparison.rs`
- Modify: `anki_forge/src/product/mod.rs`
- Create: `anki_forge/tests/phase4_comparison_tests.rs`

- [ ] **Step 1: Add failing comparison assembler tests**

Create `anki_forge/tests/phase4_comparison_tests.rs`:

```rust
use std::time::Instant;

use anki_forge::build::{BuildOptions, ComparisonStatus};
use anki_forge::prelude::*;
use tempfile::tempdir;

fn basic_project(front: &str) -> Project {
    let mut project = Project::new("Phase4 Basic")
        .stable_id("phase4-basic")
        .default_deck("Phase4");
    project
        .add_note(Note::basic(front, "back").stable_id("note-1"))
        .expect("add note");
    project
}

#[test]
fn comparison_assembler_compares_two_built_apkgs() {
    let temp = tempdir().expect("tempdir");
    let previous = temp.path().join("previous.apkg");
    let current = temp.path().join("current.apkg");

    basic_project("front")
        .build(BuildOptions::new().output(&previous))
        .expect("previous build");

    basic_project("changed front")
        .build(BuildOptions::new().output(&current))
        .expect("current build");

    let comparison = anki_forge::product::comparison::assemble_comparison(
        anki_forge::product::comparison::ComparisonInput {
            current_artifact: &current,
            previous_artifact: Some(&previous),
            diagnostics: &[],
            update_safety: None,
            started: Instant::now(),
        },
    );

    assert_eq!(comparison.comparison, ComparisonStatus::Complete);
    assert!(comparison.current_inspect.is_some(), "current inspect exists");
    assert!(comparison.previous_inspect.is_some(), "previous inspect exists");
    assert!(comparison.diff.is_some(), "diff summary exists");
    assert!(comparison.risk.is_some(), "risk report exists");
}

#[test]
fn comparison_assembler_reports_unavailable_baseline() {
    let temp = tempdir().expect("tempdir");
    let current = temp.path().join("current.apkg");
    let missing = temp.path().join("missing.apkg");

    basic_project("front")
        .build(BuildOptions::new().output(&current))
        .expect("current build");

    let comparison = anki_forge::product::comparison::assemble_comparison(
        anki_forge::product::comparison::ComparisonInput {
            current_artifact: &current,
            previous_artifact: Some(&missing),
            diagnostics: &[],
            update_safety: None,
            started: Instant::now(),
        },
    );

    assert_eq!(comparison.comparison, ComparisonStatus::Unavailable);
    assert!(comparison
        .risk
        .as_ref()
        .expect("risk")
        .findings
        .iter()
        .any(|finding| finding.code == "RISK.BASELINE_UNAVAILABLE"));
}
```

- [ ] **Step 2: Run the failing comparison tests**

Run:

```bash
cargo test -p anki_forge --test phase4_comparison_tests
```

Expected:

```text
error[E0433]: failed to resolve: could not find `comparison` in `product`
```

- [ ] **Step 3: Add comparison assembler types**

Create `anki_forge/src/product/comparison.rs`:

```rust
use std::path::Path;
use std::time::{Duration, Instant};

use crate::build::{
    BuildStatus, ComparisonStatus, InspectSummary, UpdateSafetySummary,
};
use crate::diagnostics::{Diagnostic, DiagnosticCode, Severity, SourcePath};
use crate::diff::{summarize_writer_diff, BuildDiffSummary};
use crate::risk::rules::{classify_import_risk, RiskInput};
use crate::risk::ImportRiskReport;

#[derive(Debug, Clone)]
pub struct ComparisonInput<'a> {
    pub current_artifact: &'a Path,
    pub previous_artifact: Option<&'a Path>,
    pub diagnostics: &'a [Diagnostic],
    pub update_safety: Option<&'a UpdateSafetySummary>,
    pub started: Instant,
}

#[derive(Debug, Clone)]
pub struct ComparisonOutput {
    pub comparison: ComparisonStatus,
    pub current_inspect: Option<InspectSummary>,
    pub previous_inspect: Option<InspectSummary>,
    pub diff: Option<BuildDiffSummary>,
    pub risk: Option<ImportRiskReport>,
    pub diagnostics: Vec<Diagnostic>,
    pub status: BuildStatus,
    pub duration: Duration,
}

pub fn assemble_comparison(input: ComparisonInput<'_>) -> ComparisonOutput {
    let mut diagnostics = input.diagnostics.to_vec();
    let current = inspect_summary(input.current_artifact);
    let Some(previous_artifact) = input.previous_artifact else {
        let risk = classify_import_risk(RiskInput {
            diagnostics: &diagnostics,
            comparison: ComparisonStatus::NotRequested,
            diff: None,
            current_inspect: current.as_ref(),
            previous_inspect: None,
            update_safety: input.update_safety,
        });
        return ComparisonOutput {
            comparison: ComparisonStatus::NotRequested,
            current_inspect: current,
            previous_inspect: None,
            diff: None,
            risk: Some(risk),
            diagnostics,
            status: BuildStatus::Success,
            duration: input.started.elapsed(),
        };
    };

    let previous = inspect_summary(previous_artifact);
    if previous.is_none() {
        diagnostics.push(Diagnostic {
            code: DiagnosticCode::new("COMPARE.BASELINE_UNAVAILABLE"),
            severity: Severity::Error,
            message: format!("previous APKG could not be inspected: {}", previous_artifact.display()),
            source: Some(SourcePath::new(previous_artifact.display().to_string())),
            help: Some("verify the previous APKG path and package contents".to_string()),
        });
    }

    let comparison = if current.is_some() && previous.is_some() {
        ComparisonStatus::Complete
    } else {
        ComparisonStatus::Unavailable
    };

    let mut comparison = comparison;
    let diff = if comparison == ComparisonStatus::Complete {
        match writer_diff(input.current_artifact, previous_artifact) {
            Ok((summary, writer_status)) => {
                comparison = writer_status;
                Some(summary)
            }
            Err(message) => {
                diagnostics.push(Diagnostic {
                    code: DiagnosticCode::new("COMPARE.DIFF_FAILED"),
                    severity: Severity::Error,
                    message,
                    source: Some(SourcePath::new("compare.diff")),
                    help: Some("inspect both APKG files before comparing".to_string()),
                });
                None
            }
        }
    } else {
        None
    };

    let risk = classify_import_risk(RiskInput {
        diagnostics: &diagnostics,
        comparison,
        diff: diff.as_ref(),
        current_inspect: current.as_ref(),
        previous_inspect: previous.as_ref(),
        update_safety: input.update_safety,
    });

    let status = if comparison == ComparisonStatus::Unavailable {
        BuildStatus::Invalid
    } else {
        BuildStatus::Success
    };

    ComparisonOutput {
        comparison,
        current_inspect: current,
        previous_inspect: previous,
        diff,
        risk: Some(risk),
        diagnostics,
        status,
        duration: input.started.elapsed(),
    }
}

fn inspect_summary(path: &Path) -> Option<InspectSummary> {
    crate::inspect_apkg(path).ok().map(|report| InspectSummary {
        notes: inspect_metadata_count(&report, "note_count"),
        cards: inspect_metadata_count(&report, "card_count"),
        source_kind: report.source_kind,
        observation_status: report.observation_status,
        notetypes: report.observations.notetypes.len(),
        templates: report.observations.templates.len(),
        fields: report.observations.fields.len(),
        media: report.observations.media.len(),
    })
}

fn inspect_metadata_count(report: &writer_core::InspectReport, key: &str) -> usize {
    report
        .observations
        .metadata
        .iter()
        .find_map(|value| value.get(key).and_then(serde_json::Value::as_u64))
        .unwrap_or_default() as usize
}

fn writer_diff(current: &Path, previous: &Path) -> Result<(BuildDiffSummary, ComparisonStatus), String> {
    let current_report = crate::inspect_apkg(current).map_err(|err| err.to_string())?;
    let previous_report = crate::inspect_apkg(previous).map_err(|err| err.to_string())?;
    let report = writer_core::diff_reports(&previous_report, &current_report);
    let status = writer_comparison_status(&report, &previous_report, &current_report);
    let mut summary = summarize_writer_diff(&report);
    match (
        card_evidence_status(&previous_report),
        card_evidence_status(&current_report),
    ) {
        (CardEvidenceStatus::Full, CardEvidenceStatus::Full) => {}
        (CardEvidenceStatus::Missing, _) | (_, CardEvidenceStatus::Missing) => {
            summary.limitations.push("card_evidence missing on at least one side".to_string());
        }
        (CardEvidenceStatus::Degraded, _) | (_, CardEvidenceStatus::Degraded) => {
            summary.limitations.push("card_evidence degraded: card_count exists, but card/template ordinal references are incomplete".to_string());
        }
    }
    Ok((summary, status))
}

fn writer_comparison_status(
    report: &writer_core::DiffReport,
    previous: &writer_core::InspectReport,
    current: &writer_core::InspectReport,
) -> ComparisonStatus {
    let core_missing = report
        .uncompared_domains
        .iter()
        .any(|domain| matches!(domain.as_str(), "notetypes" | "templates" | "fields" | "metadata" | "card_evidence"));
    let card_previous = card_evidence_status(previous);
    let card_current = card_evidence_status(current);
    if report.comparison_status == "unavailable"
        || core_missing
        || matches!(card_previous, CardEvidenceStatus::Missing)
        || matches!(card_current, CardEvidenceStatus::Missing)
    {
        return ComparisonStatus::Unavailable;
    }
    if report.comparison_status == "partial"
        || !report.uncompared_domains.is_empty()
        || !report.comparison_limitations.is_empty()
        || matches!(card_previous, CardEvidenceStatus::Degraded)
        || matches!(card_current, CardEvidenceStatus::Degraded)
    {
        return ComparisonStatus::Partial;
    }
    ComparisonStatus::Complete
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CardEvidenceStatus {
    Full,
    Degraded,
    Missing,
}

fn card_evidence_status(report: &writer_core::InspectReport) -> CardEvidenceStatus {
    let has_card_count = report
        .observations
        .metadata
        .iter()
        .any(|value| value.get("card_count").and_then(serde_json::Value::as_u64).is_some());
    if !has_card_count {
        return CardEvidenceStatus::Missing;
    }

    let has_card_references = report
        .observations
        .references
        .iter()
        .any(|value| value
            .get("selector")
            .and_then(serde_json::Value::as_str)
            .map(|selector| selector.starts_with("card["))
            .unwrap_or(false));
    let has_template_ordinals = report
        .observations
        .templates
        .iter()
        .any(|value| value.get("ord").and_then(serde_json::Value::as_u64).is_some());

    if has_card_references && has_template_ordinals {
        CardEvidenceStatus::Full
    } else {
        CardEvidenceStatus::Degraded
    }
}
```

- [ ] **Step 4: Export comparison module inside product**

Modify `anki_forge/src/product/mod.rs`:

```rust
pub mod comparison;
```

- [ ] **Step 5: Run comparison assembler tests**

Run:

```bash
cargo test -p anki_forge --test phase4_comparison_tests
```

Expected:

```text
test result: ok.
```

- [ ] **Step 6: Commit**

```bash
git add anki_forge/src/product/comparison.rs anki_forge/src/product/mod.rs anki_forge/tests/phase4_comparison_tests.rs
git commit -m "feat: add phase 4 comparison assembler"
```

---

### Task 7: Product Build Report Wiring

**Files:**
- Modify: `anki_forge/src/product/project.rs`
- Modify: `anki_forge/src/build/report.rs`
- Create: `anki_forge/tests/phase4_product_build_tests.rs`

- [ ] **Step 1: Add failing Product build tests**

Create `anki_forge/tests/phase4_product_build_tests.rs`:

```rust
use anki_forge::build::{BuildFailureCause, BuildOptions, BuildStatus, ComparisonStatus, RiskLevel};
use anki_forge::prelude::*;
use tempfile::tempdir;

fn basic_project(front: &str) -> Project {
    let mut project = Project::new("Phase4 Basic")
        .stable_id("phase4-basic")
        .default_deck("Phase4");
    project
        .add_note(Note::basic(front, "back").stable_id("note-1"))
        .expect("add note");
    project
}

#[test]
fn product_build_compare_to_attaches_diff_risk_and_policy() {
    let temp = tempdir().expect("tempdir");
    let previous = temp.path().join("previous.apkg");
    let current = temp.path().join("current.apkg");

    basic_project("front")
        .build(BuildOptions::new().output(&previous))
        .expect("previous build");

    let report = basic_project("changed front")
        .build(
            BuildOptions::new()
                .output(&current)
                .compare_to(&previous)
                .fail_on(RiskLevel::Critical),
        )
        .expect("current build");

    assert_eq!(report.comparison, ComparisonStatus::Complete);
    assert!(report.diff.is_some(), "diff summary should exist");
    assert!(report.risk.is_some(), "risk report should exist");
    assert_eq!(report.status, BuildStatus::Success);
}

#[test]
fn product_build_unreadable_baseline_returns_invalid_report_with_risk() {
    let temp = tempdir().expect("tempdir");
    let current = temp.path().join("current.apkg");
    let missing = temp.path().join("missing.apkg");

    let err = basic_project("front")
        .build(
            BuildOptions::new()
                .output(&current)
                .compare_to(&missing)
                .fail_on(RiskLevel::High),
        )
        .expect_err("unreadable baseline should fail");

    assert_eq!(err.report.status, BuildStatus::Invalid);
    assert_eq!(err.report.comparison, ComparisonStatus::Unavailable);
    assert!(err
        .report
        .risk
        .as_ref()
        .expect("risk")
        .findings
        .iter()
        .any(|finding| finding.code == "RISK.BASELINE_UNAVAILABLE"));
    assert!(matches!(
        err.cause,
        BuildFailureCause::Diagnostics | BuildFailureCause::Invalid
    ));
}
```

- [ ] **Step 2: Run the failing Product build tests**

Run:

```bash
cargo test -p anki_forge --test phase4_product_build_tests
```

Expected:

```text
product_build_compare_to_attaches_diff_risk_and_policy ... FAILED
product_build_unreadable_baseline_returns_invalid_report_with_risk ... FAILED
```

- [ ] **Step 3: Wire final Product build report fields**

In `anki_forge/src/product/project.rs`, update the final successful report construction at the existing anchor:

```rust
let update_safety = Some(update_safety_summary_val);
let report = BuildReport {
```

Replace that final `let report = BuildReport { ... }` block with:

```rust
let comparison_output = if let Some(artifact_ref) = artifact.as_ref() {
    crate::product::comparison::assemble_comparison(
        crate::product::comparison::ComparisonInput {
            current_artifact: &artifact_ref.path,
            previous_artifact: options.compare_to.as_deref(),
            diagnostics: &diagnostics,
            update_safety: Some(&update_safety_summary_val),
            started,
        },
    )
} else {
    crate::product::comparison::ComparisonOutput {
        comparison: ComparisonStatus::Unavailable,
        current_inspect: None,
        previous_inspect: None,
        diff: None,
        risk: None,
        diagnostics: diagnostics.clone(),
        status: BuildStatus::Invalid,
        duration: started.elapsed(),
    }
};

diagnostics = comparison_output.diagnostics;
let policy = crate::risk::policy_from_risk_report(
    options.fail_on,
    comparison_output.risk.as_ref(),
);
let status = BuildStatus::highest([
    comparison_output.status,
    if matches!(policy.status, BuildPolicyStatus::Blocked) {
        BuildStatus::Blocked
    } else {
        BuildStatus::Success
    },
    if diagnostics.iter().any(|diagnostic| diagnostic.severity == Severity::Error) {
        BuildStatus::Invalid
    } else {
        BuildStatus::Success
    },
]);

let report = BuildReport {
    artifact,
    counts,
    media,
    diagnostics,
    metrics: BuildMetrics {
        duration: started.elapsed(),
    },
    inspect: comparison_output.current_inspect.or(inspect),
    update_safety,
    comparison: comparison_output.comparison,
    diff: comparison_output.diff,
    risk: comparison_output.risk,
    policy,
    status,
};
```

Add imports:

```rust
use crate::build::{
    BuildPolicyStatus, BuildStatus, ComparisonStatus,
};
```

- [ ] **Step 4: Update all earlier partial BuildReport literals**

First enumerate the return sites:

```bash
rg -n "BuildReport \\{|failure_report\\(|BuildError::new\\(" anki_forge/src/product/project.rs
```

Expected current Phase 3 anchors to classify:

```text
line ~364: normalize failure -> invalid diagnostics
line ~389: validation diagnostics after normalization -> invalid diagnostics
line ~411: current_dir failure through failure_report -> error io
line ~418: runtime defaults failure through failure_report -> error io
line ~441: update-safety option error -> invalid diagnostics
line ~501: current identity diagnostics -> invalid diagnostics
line ~548: disabled-mode reconcile failure -> invalid diagnostics
line ~683: strict compare_to unreadable previous APKG -> invalid diagnostics
line ~720: reconcile duplicate failure -> invalid diagnostics
line ~758: strict update-safety blocking diagnostics -> invalid diagnostics
line ~809: writer failure through failure_report -> error internal
line ~849: artifact ref failure through failure_report -> error io
line ~857: output directory failure through failure_report -> error io
line ~864: output copy failure through failure_report -> error io
line ~891: missing project stable id before lockfile write -> error diagnostics
line ~938: lockfile write failure -> error io
line ~994: final report -> status from comparison/policy/diagnostics
line ~2215: artifact workspace failure through failure_report -> error io
line ~2450: failure_report helper -> error io/internal by caller
```

Every early `BuildReport { ... }` in `anki_forge/src/product/project.rs` must include these Phase 4 fields:

```rust
comparison: ComparisonStatus::NotRequested,
diff: None,
risk: None,
policy: BuildPolicyResult::default(),
status: BuildStatus::Invalid,
```

Use `BuildStatus::Error` for IO/runtime failures and `BuildStatus::Invalid` for validation/baseline/user-input failures.

- [ ] **Step 5: Run Product build tests**

Run:

```bash
cargo test -p anki_forge --test phase4_product_build_tests
```

Expected:

```text
test result: ok.
```

- [ ] **Step 6: Run update-safety regressions**

Run:

```bash
cargo test -p anki_forge update_safety --tests
```

Expected:

```text
test result: ok.
```

- [ ] **Step 7: Commit**

```bash
git add anki_forge/src/product/project.rs anki_forge/src/build/report.rs anki_forge/tests/phase4_product_build_tests.rs
git commit -m "feat: attach phase 4 comparison to product builds"
```

---

### Task 8: Report JSON Wiring In Product Build

**Files:**
- Modify: `anki_forge/src/product/project.rs`
- Modify: `anki_forge/tests/phase4_product_build_tests.rs`

- [ ] **Step 1: Add failing report_json integration tests**

Append to `anki_forge/tests/phase4_product_build_tests.rs`:

```rust
#[test]
fn product_build_report_json_writes_success_report() {
    let temp = tempdir().expect("tempdir");
    let apkg = temp.path().join("deck.apkg");
    let report_json = temp.path().join("build-report.json");

    let report = basic_project("front")
        .build(BuildOptions::new().output(&apkg).report_json(&report_json))
        .expect("build succeeds");

    assert_eq!(report.status, BuildStatus::Success);
    let written: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&report_json).expect("read report"))
            .expect("report JSON");
    assert_eq!(written["kind"], "anki-forge-build-report");
    assert_eq!(written["status"], "success");
}

#[test]
fn product_build_report_json_writes_invalid_baseline_report() {
    let temp = tempdir().expect("tempdir");
    let apkg = temp.path().join("deck.apkg");
    let report_json = temp.path().join("build-report.json");
    let missing = temp.path().join("missing.apkg");

    let err = basic_project("front")
        .build(
            BuildOptions::new()
                .output(&apkg)
                .compare_to(&missing)
                .fail_on(RiskLevel::High)
                .report_json(&report_json),
        )
        .expect_err("invalid baseline should fail");

    assert_eq!(err.report.status, BuildStatus::Invalid);
    let written: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&report_json).expect("read report"))
            .expect("report JSON");
    assert_eq!(written["status"], "invalid");
    assert_eq!(written["comparison"], "unavailable");
    assert_eq!(written["policy"]["status"], "blocked");
}
```

- [ ] **Step 2: Run failing report_json tests**

Run:

```bash
cargo test -p anki_forge report_json --test phase4_product_build_tests
```

Expected:

```text
product_build_report_json_writes_success_report ... FAILED
product_build_report_json_writes_invalid_baseline_report ... FAILED
```

- [ ] **Step 3: Write report_json on every report path that has a report**

Use the same return-site anchors listed in Task 7 Step 4. Every `return Err(BuildError::new(BuildReport { ... }, cause))` site in `Project::build` must route through `return_report_error(&options, report, cause)` after the report object exists. The `failure_report(...)` helper itself does not receive `options`, so callers that immediately wrap `failure_report(...)` in `BuildError::new(...)` should be converted to:

```rust
let report = failure_report(started, "PROJECT.CURRENT_DIR_FAILED", err.to_string());
return return_report_error(&options, report, BuildFailureCause::Io);
```

Add this helper near the bottom of `anki_forge/src/product/project.rs`:

```rust
fn maybe_write_report_json(
    options: &BuildOptions,
    mut report: BuildReport,
) -> Result<BuildReport, BuildError> {
    let Some(path) = options.report_json.as_ref() else {
        return Ok(report);
    };

    if let Err(err) = crate::build::write_report_json_atomic(path, &report) {
        report.diagnostics.push(Diagnostic {
            code: DiagnosticCode::new("REPORT.JSON_WRITE_FAILED"),
            severity: Severity::Error,
            message: err.to_string(),
            source: Some(SourcePath::new(path.display().to_string())),
            help: Some("verify that the report_json path is writable".to_string()),
        });
        report.status = BuildStatus::Error;
        return Err(BuildError::new(report, BuildFailureCause::Io));
    }

    Ok(report)
}
```

Before `report.ensure_success()?`, call:

```rust
let report = maybe_write_report_json(&options, report)?;
```

For early error reports, wrap report construction through a small local `return_report_error(options, report, cause)` helper so `report_json` is written when possible. The helper keeps the original cause unless JSON writing fails, in which case it returns `BuildFailureCause::Io`.

Add the helper next to `maybe_write_report_json`:

```rust
fn return_report_error(
    options: &BuildOptions,
    report: BuildReport,
    cause: BuildFailureCause,
) -> Result<BuildReport, BuildError> {
    match maybe_write_report_json(options, report) {
        Ok(report) => Err(BuildError::new(report, cause)),
        Err(err) => Err(err),
    }
}
```

Replace early returns shaped like this:

```rust
return Err(BuildError::new(report, BuildFailureCause::Diagnostics));
```

with:

```rust
return return_report_error(&options, report, BuildFailureCause::Diagnostics);
```

- [ ] **Step 4: Run report_json tests**

Run:

```bash
cargo test -p anki_forge report_json --test phase4_product_build_tests
```

Expected:

```text
test result: ok.
```

- [ ] **Step 5: Run Product build and update-safety regressions**

Run:

```bash
cargo test -p anki_forge --test phase4_product_build_tests
cargo test -p anki_forge update_safety --tests
```

Expected:

```text
test result: ok.
test result: ok.
```

- [ ] **Step 6: Commit**

```bash
git add anki_forge/src/product/project.rs anki_forge/tests/phase4_product_build_tests.rs
git commit -m "feat: write product build reports"
```

---

### Task 9: Project::diff_against_apkg

**Files:**
- Modify: `anki_forge/src/product/project.rs`
- Modify: `anki_forge/src/diff/mod.rs`
- Modify: `anki_forge/tests/phase4_product_build_tests.rs`

- [ ] **Step 1: Add failing standalone diff tests**

Append to `anki_forge/tests/phase4_product_build_tests.rs`:

```rust
#[test]
fn project_diff_against_apkg_matches_build_comparison_sections() {
    let temp = tempdir().expect("tempdir");
    let previous = temp.path().join("previous.apkg");
    let current = temp.path().join("current.apkg");

    basic_project("front")
        .build(BuildOptions::new().output(&previous))
        .expect("previous build");

    let project = basic_project("changed front");
    let build_report = project
        .build(BuildOptions::new().output(&current).compare_to(&previous))
        .expect("build comparison");
    let diff_report = project
        .diff_against_apkg(&previous)
        .expect("standalone diff");

    assert_eq!(diff_report.comparison, build_report.comparison);
    assert_eq!(diff_report.diff, build_report.diff);
    assert_eq!(diff_report.risk, build_report.risk);
    assert!(diff_report.current_inspect.is_some());
    assert!(diff_report.previous_inspect.is_some());
}

#[test]
fn project_diff_against_apkg_unreadable_baseline_returns_invalid_report() {
    let temp = tempdir().expect("tempdir");
    let missing = temp.path().join("missing.apkg");
    let err = basic_project("front")
        .diff_against_apkg(&missing)
        .expect_err("missing baseline should fail");

    assert_eq!(err.report.status, BuildStatus::Invalid);
    assert_eq!(err.report.comparison, ComparisonStatus::Unavailable);
    assert!(err
        .report
        .risk
        .as_ref()
        .expect("risk")
        .findings
        .iter()
        .any(|finding| finding.code == "RISK.BASELINE_UNAVAILABLE"));
}
```

- [ ] **Step 2: Run failing standalone diff tests**

Run:

```bash
cargo test -p anki_forge project_diff_against_apkg --test phase4_product_build_tests
```

Expected:

```text
error[E0599]: no method named `diff_against_apkg` found for struct `Project`
```

- [ ] **Step 3: Implement standalone diff through temporary materialization**

Add to `impl Project` in `anki_forge/src/product/project.rs`:

```rust
pub fn diff_against_apkg(
    &self,
    path: impl AsRef<Path>,
) -> Result<crate::diff::ProjectDiffReport, crate::diff::ProjectDiffError> {
    let started = Instant::now();
    let temp = tempfile::Builder::new()
        .prefix("anki-forge-project-diff-")
        .tempdir()
        .map_err(|err| {
            let report = crate::diff::ProjectDiffReport {
                status: BuildStatus::Error,
                comparison: ComparisonStatus::Unavailable,
                diagnostics: vec![Diagnostic {
                    code: DiagnosticCode::new("DIFF.TEMP_DIR_FAILED"),
                    severity: Severity::Error,
                    message: err.to_string(),
                    source: Some(SourcePath::new("project.diff_against_apkg")),
                    help: Some("verify that the system temporary directory is writable".to_string()),
                }],
                current_inspect: None,
                previous_inspect: None,
                update_safety: None,
                diff: None,
                risk: None,
                metrics: crate::diff::ComparisonMetrics { duration_ms: 0 },
            };
            crate::diff::ProjectDiffError::new(report, BuildFailureCause::Io)
        })?;
    let current_path = temp.path().join("current.apkg");

    let build = self.build(BuildOptions::new().output(&current_path).inspect(true));
    let build_report = match build {
        Ok(report) => report,
        Err(err) => {
            let report = crate::diff::ProjectDiffReport {
                status: err.report.status,
                comparison: ComparisonStatus::Unavailable,
                diagnostics: err.report.diagnostics.clone(),
                current_inspect: err.report.inspect.clone(),
                previous_inspect: None,
                update_safety: err.report.update_safety.clone(),
                diff: None,
                risk: err.report.risk.clone(),
                metrics: crate::diff::ComparisonMetrics {
                    duration_ms: started.elapsed().as_millis(),
                },
            };
            return Err(crate::diff::ProjectDiffError::new(report, err.cause));
        }
    };

    let Some(artifact) = build_report.artifact.as_ref() else {
        let report = crate::diff::ProjectDiffReport {
            status: BuildStatus::Invalid,
            comparison: ComparisonStatus::Unavailable,
            diagnostics: build_report.diagnostics.clone(),
            current_inspect: build_report.inspect.clone(),
            previous_inspect: None,
            update_safety: build_report.update_safety.clone(),
            diff: None,
            risk: build_report.risk.clone(),
            metrics: crate::diff::ComparisonMetrics {
                duration_ms: started.elapsed().as_millis(),
            },
        };
        return Err(crate::diff::ProjectDiffError::new(
            report,
            BuildFailureCause::Invalid,
        ));
    };

    let comparison = crate::product::comparison::assemble_comparison(
        crate::product::comparison::ComparisonInput {
            current_artifact: &artifact.path,
            previous_artifact: Some(path.as_ref()),
            diagnostics: &build_report.diagnostics,
            update_safety: build_report.update_safety.as_ref(),
            started,
        },
    );
    let report = crate::diff::ProjectDiffReport {
        status: comparison.status,
        comparison: comparison.comparison,
        diagnostics: comparison.diagnostics,
        current_inspect: comparison.current_inspect,
        previous_inspect: comparison.previous_inspect,
        update_safety: build_report.update_safety,
        diff: comparison.diff,
        risk: comparison.risk,
        metrics: crate::diff::ComparisonMetrics {
            duration_ms: started.elapsed().as_millis(),
        },
    };

    if report.status == BuildStatus::Success {
        Ok(report)
    } else {
        let cause = if report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == Severity::Error)
        {
            BuildFailureCause::Diagnostics
        } else {
            BuildFailureCause::Invalid
        };
        Err(crate::diff::ProjectDiffError::new(report, cause))
    }
}
```

- [ ] **Step 4: Run standalone diff tests**

Run:

```bash
cargo test -p anki_forge project_diff_against_apkg --test phase4_product_build_tests
```

Expected:

```text
test result: ok.
```

- [ ] **Step 5: Commit**

```bash
git add anki_forge/src/product/project.rs anki_forge/src/diff/mod.rs anki_forge/tests/phase4_product_build_tests.rs
git commit -m "feat: add project diff against apkg"
```

---

### Task 10: contract_tools product-build

**Files:**
- Create: `anki_forge/src/runtime/product_build.rs`
- Modify: `anki_forge/src/runtime/mod.rs`
- Create: `contract_tools/src/product_build_cmd.rs`
- Modify: `contract_tools/src/lib.rs`
- Modify: `contract_tools/src/main.rs`
- Modify: `contract_tools/tests/cli_tests.rs`

- [ ] **Step 1: Add failing CLI tests**

Append to `contract_tools/tests/cli_tests.rs`:

```rust
fn write_basic_product_document(temp_dir: &Path) -> PathBuf {
    let input = temp_dir.join("basic.product.json");
    let value = serde_json::json!({
        "document_id": "phase4-cli",
        "note_types": [
            { "Basic": { "id": "basic-main", "name": "Basic" } }
        ],
        "notes": [
            {
                "Basic": {
                    "id": "note-1",
                    "note_type_id": "basic-main",
                    "deck_name": "Default",
                    "front": "front",
                    "back": "back",
                    "tags": []
                }
            }
        ]
    });
    fs::write(&input, serde_json::to_string_pretty(&value).unwrap()).expect("write product");
    input
}

#[test]
fn product_build_command_writes_apkg_and_report_json() {
    let temp = tempdir().expect("tempdir");
    let manifest = contract_tools::contract_manifest_path();
    let input = write_basic_product_document(temp.path());
    let apkg = temp.path().join("deck.apkg");
    let report_json = temp.path().join("build-report.json");

    let output = run_cli(&[
        "product-build",
        "--manifest",
        manifest.to_str().unwrap(),
        "--product-input",
        input.to_str().unwrap(),
        "--apkg-out",
        apkg.to_str().unwrap(),
        "--report-json",
        report_json.to_str().unwrap(),
        "--output",
        "contract-json",
    ]);

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(apkg.exists(), "APKG output should exist");
    assert!(report_json.exists(), "report JSON should exist");
    let stdout_report: Value = serde_json::from_slice(&output.stdout).expect("stdout JSON");
    let file_report: Value =
        serde_json::from_str(&fs::read_to_string(report_json).expect("read report")).unwrap();
    assert_eq!(stdout_report["kind"], "anki-forge-build-report");
    assert_eq!(stdout_report["status"], "success");
    assert_eq!(stdout_report, file_report);
}

#[test]
fn product_build_command_returns_invalid_exit_for_missing_baseline() {
    let temp = tempdir().expect("tempdir");
    let manifest = contract_tools::contract_manifest_path();
    let input = write_basic_product_document(temp.path());
    let apkg = temp.path().join("deck.apkg");
    let missing = temp.path().join("missing.apkg");

    let output = run_cli(&[
        "product-build",
        "--manifest",
        manifest.to_str().unwrap(),
        "--product-input",
        input.to_str().unwrap(),
        "--apkg-out",
        apkg.to_str().unwrap(),
        "--compare-to",
        missing.to_str().unwrap(),
        "--fail-on",
        "high",
        "--output",
        "contract-json",
    ]);

    assert_eq!(output.status.code(), Some(3));
    let report: Value = serde_json::from_slice(&output.stdout).expect("stdout JSON");
    assert_eq!(report["status"], "invalid");
    assert_eq!(report["comparison"], "unavailable");
    assert_eq!(report["policy"]["status"], "blocked");
}
```

- [ ] **Step 2: Run failing CLI tests**

Run:

```bash
cargo test -p contract_tools product_build_command --test cli_tests
```

Expected:

```text
error: unrecognized subcommand 'product-build'
```

- [ ] **Step 3: Add runtime product build facade**

Create `anki_forge/src/runtime/product_build.rs`:

```rust
use crate::build::{BuildOptions, BuildReport};
use crate::product::ProductDocument;

pub fn build_product_document(
    document: ProductDocument,
    options: BuildOptions,
) -> Result<BuildReport, crate::build::BuildError> {
    crate::product::Project::from_product_document(document).build(options)
}
```

Modify `anki_forge/src/product/project.rs` so `Project` can hold a serialized ProductDocument source without converting it through the builder-only fields:

```rust
#[derive(Debug, Clone)]
pub struct Project {
    name: String,
    stable_id: Option<String>,
    default_deck: Option<String>,
    note_types: Vec<NoteType>,
    notes: Vec<Note>,
    media: crate::product::MediaRegistry,
    deck_source: Option<crate::deck::Deck>,
    product_document_source: Option<ProductDocument>,
}
```

Set the new field in `Project::new`:

```rust
product_document_source: None,
```

Add this constructor to `impl Project`:

```rust
pub fn from_product_document(document: ProductDocument) -> Self {
    let name = document.document_id().to_string();
    let default_deck = document.default_deck_name().map(str::to_string);
    Self {
        name: name.clone(),
        stable_id: Some(name),
        default_deck,
        note_types: Vec::new(),
        notes: Vec::new(),
        media: crate::product::MediaRegistry::default(),
        deck_source: None,
        product_document_source: Some(document),
    }
}
```

At the top of `Project::lower`, before the deck-source branch, add:

```rust
if let Some(product) = &self.product_document_source {
    return product
        .lower()
        .map_err(|err| anyhow::anyhow!("lower product document: {:?}", err));
}
```

At the start of `Project::validate`, add this guard so builder state cannot be mixed with direct ProductDocument state:

```rust
if self.product_document_source.is_some()
    && (!self.notes.is_empty()
        || !self.note_types.is_empty()
        || self.deck_source.is_some())
{
    diagnostics.push(Diagnostic {
        code: DiagnosticCode::new("PROJECT.PRODUCT_DOCUMENT_SOURCE_MIXED"),
        severity: Severity::Error,
        message: "ProductDocument-backed projects cannot mix direct Project notes, note types, or deck sources".to_string(),
        source: Some(SourcePath::new("project")),
        help: Some("build either a ProductDocument-backed Project or a builder-backed Project".to_string()),
    });
    return ValidationReport { diagnostics };
}
```

This preserves all current `ProductDocument` lowering behavior for the CLI path, including helpers, metadata, and bundled assets.

Modify `anki_forge/src/runtime/mod.rs`:

```rust
pub mod product_build;
pub use product_build::build_product_document;
```

- [ ] **Step 4: Run ProductDocument and Project regressions before adding CLI**

Run:

```bash
cargo test -p anki_forge --test product_portability_tests
cargo test -p anki_forge --test product_pipeline_tests
cargo test -p anki_forge --test project_api_tests
```

Expected:

```text
test result: ok.
test result: ok.
test result: ok.
```

- [ ] **Step 5: Add CLI command implementation**

Create `contract_tools/src/product_build_cmd.rs`:

```rust
use std::path::PathBuf;

use anki_forge::build::{BuildOptions, BuildReportJson, BuildStatus, RiskLevel};
use anki_forge::product::ProductDocument;

pub enum ProductBuildOutcome {
    Success(String),
    ReportFailure { json: String, exit_code: i32 },
}

pub fn run(
    manifest: &str,
    product_input: &str,
    apkg_out: &str,
    compare_to: Option<&str>,
    fail_on: Option<&str>,
    report_json: Option<&str>,
    output: &str,
) -> anyhow::Result<ProductBuildOutcome> {
    let manifest = contract_tools::manifest::load_manifest(manifest)?;
    contract_tools::manifest::resolve_asset_path(&manifest, "build_report_schema")?;
    let raw = std::fs::read_to_string(product_input)?;
    let document: ProductDocument = serde_json::from_str(&raw)?;

    let mut options = BuildOptions::new().output(PathBuf::from(apkg_out));
    if let Some(compare_to) = compare_to {
        options = options.compare_to(compare_to);
    }
    if let Some(fail_on) = fail_on {
        options = options.fail_on(parse_risk_level(fail_on)?);
    }
    if let Some(report_json) = report_json {
        options = options.report_json(report_json);
    }

    let result = anki_forge::runtime::build_product_document(document, options);

    match result {
        Ok(report) => {
            let body = render(&report, output)?;
            Ok(ProductBuildOutcome::Success(body))
        }
        Err(err) => {
            let body = render(&err.report, output)?;
            let exit_code = exit_code_for_status(err.report.status);
            Ok(ProductBuildOutcome::ReportFailure { json: body, exit_code })
        }
    }
}

fn render(report: &anki_forge::build::BuildReport, output: &str) -> anyhow::Result<String> {
    match output {
        "contract-json" => Ok(serde_json::to_string_pretty(&BuildReportJson::from_report(report))?),
        "human" => Ok(format!(
            "status: {:?}\ncomparison: {:?}\nhighest_risk: {:?}\n",
            report.status,
            report.comparison,
            report.risk.as_ref().and_then(|risk| risk.highest_level)
        )),
        other => anyhow::bail!("unsupported product-build output mode: {other}"),
    }
}

fn parse_risk_level(value: &str) -> anyhow::Result<RiskLevel> {
    match value {
        "info" => Ok(RiskLevel::Info),
        "low" => Ok(RiskLevel::Low),
        "medium" => Ok(RiskLevel::Medium),
        "high" => Ok(RiskLevel::High),
        "critical" => Ok(RiskLevel::Critical),
        other => anyhow::bail!("unsupported fail-on level: {other}"),
    }
}

fn exit_code_for_status(status: BuildStatus) -> i32 {
    match status {
        BuildStatus::Success => 0,
        BuildStatus::Blocked => 2,
        BuildStatus::Invalid => 3,
        BuildStatus::Error => 4,
    }
}
```

Modify `contract_tools/src/lib.rs`:

```rust
pub mod product_build_cmd;
```

- [ ] **Step 6: Add clap subcommand and exit mapping**

Modify `contract_tools/src/main.rs` `Command` enum:

```rust
ProductBuild {
    #[arg(long)]
    manifest: String,
    #[arg(long)]
    product_input: String,
    #[arg(long)]
    apkg_out: String,
    #[arg(long)]
    compare_to: Option<String>,
    #[arg(long)]
    fail_on: Option<String>,
    #[arg(long)]
    report_json: Option<String>,
    #[arg(long, default_value = "contract-json")]
    output: String,
},
```

Add match arm:

```rust
Command::ProductBuild {
    manifest,
    product_input,
    apkg_out,
    compare_to,
    fail_on,
    report_json,
    output,
} => {
    match contract_tools::product_build_cmd::run(
        &manifest,
        &product_input,
        &apkg_out,
        compare_to.as_deref(),
        fail_on.as_deref(),
        report_json.as_deref(),
        &output,
    )? {
        contract_tools::product_build_cmd::ProductBuildOutcome::Success(body) => {
            print!("{body}");
        }
        contract_tools::product_build_cmd::ProductBuildOutcome::ReportFailure {
            json,
            exit_code,
        } => {
            print!("{json}");
            std::process::exit(exit_code);
        }
    }
}
```

- [ ] **Step 7: Run CLI tests**

Run:

```bash
cargo test -p contract_tools product_build_command --test cli_tests
```

Expected:

```text
test result: ok.
```

- [ ] **Step 8: Commit**

```bash
git add anki_forge/src/runtime/product_build.rs anki_forge/src/runtime/mod.rs anki_forge/src/product/project.rs contract_tools/src/product_build_cmd.rs contract_tools/src/lib.rs contract_tools/src/main.rs contract_tools/tests/cli_tests.rs
git commit -m "feat: add product build cli report"
```

---

### Task 11: Oracle-Backed Template/Card Risk Tests

**Files:**
- Modify: `anki_forge/tests/phase4_product_build_tests.rs`
- Modify: `docs/oracles/phase-4-template-card-risk.md`

- [ ] **Step 1: Add failing oracle-backed template/card tests**

Append to `anki_forge/tests/phase4_product_build_tests.rs`:

```rust
fn custom_template_project(template_names: &[&str]) -> Project {
    let mut note_type = NoteType::custom("jp-vocab")
        .name("Japanese Vocabulary")
        .field(Field::new("Expression").key("expr").identity())
        .field(Field::new("Meaning").key("meaning"))
        .identity(IdentityRecipe::fields(["expr"]));

    for template_name in template_names {
        note_type = note_type.template(
            Template::new(*template_name)
                .key(template_name.to_ascii_lowercase().replace(' ', "-"))
                .front("{{Expression}}")
                .back("{{Meaning}}")
                .generate_when(GenerationRule::all(["expr"])),
        );
    }

    let mut project = Project::new("Phase4 Template Oracle")
        .stable_id("phase4-template-oracle")
        .default_deck("Phase4");
    project.add_notetype(note_type).expect("add notetype");
    project
        .add_note(
            Note::new("jp-vocab")
                .stable_id("jp-vocab:taberu")
                .text("Expression", "食べる")
                .text("Meaning", "to eat"),
        )
        .expect("add note");
    project
}

#[test]
fn oracle_template_removed_emits_critical_risk_with_evidence() {
    let temp = tempdir().expect("tempdir");
    let previous = temp.path().join("previous.apkg");
    let current = temp.path().join("current.apkg");

    custom_template_project(&["Recognition", "Production"])
        .build(BuildOptions::new().output(&previous))
        .expect("previous build");

    let err = custom_template_project(&["Recognition"])
        .build(
            BuildOptions::new()
                .output(&current)
                .compare_to(&previous)
                .fail_on(RiskLevel::Critical),
        )
        .expect_err("template removal should block at critical");

    assert_eq!(err.report.policy.status, BuildPolicyStatus::Blocked);
    let finding = err
        .report
        .risk
        .as_ref()
        .expect("risk")
        .findings
        .iter()
        .find(|finding| finding.code == "RISK.TEMPLATE_REMOVED")
        .expect("template removed finding");
    assert_eq!(finding.level, RiskLevel::Critical);
    assert!(
        finding.evidence_refs.iter().any(|evidence| evidence.ref_id.contains("semantic")),
        "finding should link to semantic diff evidence"
    );
}

#[test]
fn oracle_template_reorder_emits_high_risk_with_evidence() {
    let temp = tempdir().expect("tempdir");
    let previous = temp.path().join("previous.apkg");
    let current = temp.path().join("current.apkg");

    custom_template_project(&["Recognition", "Production"])
        .build(BuildOptions::new().output(&previous))
        .expect("previous build");

    let report = custom_template_project(&["Production", "Recognition"])
        .build(
            BuildOptions::new()
                .output(&current)
                .compare_to(&previous)
                .fail_on(RiskLevel::Critical),
        )
        .expect("template reorder is high risk but below critical threshold");

    let finding = report
        .risk
        .as_ref()
        .expect("risk")
        .findings
        .iter()
        .find(|finding| finding.code == "RISK.TEMPLATE_REORDER")
        .expect("template reorder finding");
    assert_eq!(finding.level, RiskLevel::High);
    assert!(!finding.evidence_refs.is_empty());
}
```

Also add `BuildPolicyStatus` to the existing import at the top of the test file:

```rust
use anki_forge::build::{
    BuildFailureCause, BuildOptions, BuildPolicyStatus, BuildStatus, ComparisonStatus, RiskLevel,
};
```

- [ ] **Step 2: Run failing oracle tests**

Run:

```bash
cargo test -p anki_forge oracle_template --test phase4_product_build_tests
```

Expected:

```text
oracle_template_removed_emits_critical_risk_with_evidence ... FAILED
oracle_template_reorder_emits_high_risk_with_evidence ... FAILED
```

- [ ] **Step 3: Connect template-ordinal diagnostics to template reorder risk**

Modify `anki_forge/src/risk/rules.rs` inside the diagnostics loop before the broader config-id drift branch:

```rust
if code == "UPDATE.TEMPLATE_ORD_CHANGED" {
    findings.push(finding(
        "RISK.TEMPLATE_REORDER",
        RiskLevel::High,
        "template",
        "template ordinal changed and may affect existing card scheduling",
        vec![EvidenceRef {
            kind: EvidenceRefKind::Diagnostic,
            ref_id: format!("diagnostic:{index}:{code}"),
        }],
        "preserve template order for existing cards or review the migration before import",
    ));
    continue;
}
```

The `continue` is required so one `UPDATE.TEMPLATE_ORD_CHANGED` diagnostic does not also emit `RISK.NOTETYPE_CONFIG_ID_DRIFT`.

- [ ] **Step 4: Run oracle tests**

Run:

```bash
cargo test -p anki_forge oracle_template --test phase4_product_build_tests
```

Expected:

```text
test result: ok.
```

- [ ] **Step 5: Update oracle document with test names**

Append to `docs/oracles/phase-4-template-card-risk.md`:

```markdown
## Automated Regression Tests

- `oracle_template_removed_emits_critical_risk_with_evidence`
- `oracle_template_reorder_emits_high_risk_with_evidence`
```

- [ ] **Step 6: Commit**

```bash
git add anki_forge/src/risk/rules.rs anki_forge/tests/phase4_product_build_tests.rs docs/oracles/phase-4-template-card-risk.md
git commit -m "test: add phase 4 template risk oracles"
```

---

### Task 12: Contract JSON Validation And CI Documentation

**Files:**
- Modify: `contract_tools/tests/cli_tests.rs`
- Modify: `contract_tools/tests/schema_gate_tests.rs`
- Create: `docs/ci/phase-4-product-build.md`
- Modify: `docs/api-design.md` only if implementation exposes a narrower first-slice CLI contract than the approved spec.

- [ ] **Step 1: Validate CLI report against schema**

Append to `contract_tools/tests/cli_tests.rs`:

```rust
#[test]
fn product_build_report_validates_against_build_report_schema() {
    let temp = tempdir().expect("tempdir");
    let manifest = contract_tools::contract_manifest_path();
    let input = write_basic_product_document(temp.path());
    let apkg = temp.path().join("deck.apkg");

    let output = run_cli(&[
        "product-build",
        "--manifest",
        manifest.to_str().unwrap(),
        "--product-input",
        input.to_str().unwrap(),
        "--apkg-out",
        apkg.to_str().unwrap(),
        "--output",
        "contract-json",
    ]);

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let report: Value = serde_json::from_slice(&output.stdout).expect("report JSON");
    let manifest =
        contract_tools::manifest::load_manifest(contract_tools::contract_manifest_path())
            .expect("manifest");
    let schema_path =
        contract_tools::manifest::resolve_asset_path(&manifest, "build_report_schema")
            .expect("schema path");
    let schema_value: Value =
        serde_json::from_str(&fs::read_to_string(schema_path).expect("schema")).unwrap();
    let schema = jsonschema::JSONSchema::compile(&schema_value).expect("schema compiles");
    let errors = schema
        .validate(&report)
        .err()
        .map(|errors| errors.map(|error| error.to_string()).collect::<Vec<_>>())
        .unwrap_or_default();
    assert!(errors.is_empty(), "schema errors: {errors:#?}");
}
```

- [ ] **Step 2: Run schema validation test**

Run:

```bash
cargo test -p contract_tools product_build_report_validates_against_build_report_schema --test cli_tests
```

Expected:

```text
test result: ok.
```

- [ ] **Step 3: Add CI docs**

Create `docs/ci/phase-4-product-build.md`:

```markdown
# Phase 4 Product Build CI

Phase 4 CI uses the Product build report as the single machine-readable result.

```bash
contract_tools product-build \
  --manifest contracts/manifest.yaml \
  --product-input project.product.json \
  --apkg-out deck.apkg \
  --compare-to previous.apkg \
  --fail-on high \
  --report-json build-report.json \
  --output contract-json
```

Exit codes:

| Code | Meaning |
| --- | --- |
| 0 | success |
| 1 | invocation failure before a BuildReport exists |
| 2 | policy blocked |
| 3 | invalid Product input, invalid baseline, or validation diagnostics |
| 4 | infrastructure or execution error |

When an invalid baseline also triggers `fail_on`, the top-level `status` is `invalid`, the exit code is `3`, and `policy.status` is `blocked`.

## GitHub Actions Example

```yaml
- name: Download previous APKG
  uses: actions/download-artifact@v4
  with:
    name: previous-apkg
    path: .

- name: Build Anki package
  run: |
    contract_tools product-build \
      --manifest contracts/manifest.yaml \
      --product-input project.product.json \
      --apkg-out deck.apkg \
      --compare-to previous.apkg \
      --fail-on high \
      --report-json build-report.json \
      --output contract-json

- name: Upload build report
  uses: actions/upload-artifact@v4
  with:
    name: anki-forge-build-report
    path: build-report.json
```

## Failure Modes

- Build failure: `status = error`, exit `4`, diagnostics explain the execution failure.
- Invalid baseline: `status = invalid`, `comparison = unavailable`, `risk.findings` includes `RISK.BASELINE_UNAVAILABLE`.
- Policy-blocked update: `policy.status = blocked`, `policy.blocking_findings` lists risk codes at or above the threshold.
- Warning-only build: `status = success`, exit `0`, warning diagnostics remain in the report.
```

- [ ] **Step 4: Run docs-related checks**

Run:

```bash
cargo test -p contract_tools product_build_report_validates_against_build_report_schema --test cli_tests
cargo test -p contract_tools phase4_build_report_schema --test schema_gate_tests
```

Expected:

```text
test result: ok.
test result: ok.
```

- [ ] **Step 5: Commit**

```bash
git add contract_tools/tests/cli_tests.rs contract_tools/tests/schema_gate_tests.rs docs/ci/phase-4-product-build.md
git commit -m "docs: add phase 4 product build ci contract"
```

---

### Task 13: Full Verification And Final Phase 4 Acceptance

**Files:**
- Modify only files needed to resolve failures found by the commands in this task.

- [ ] **Step 1: Run focused Phase 4 tests**

Run:

```bash
cargo test -p anki_forge --test build_report_tests
cargo test -p anki_forge --test phase4_diff_risk_model_tests
cargo test -p anki_forge --test phase4_risk_rules_tests
cargo test -p anki_forge --test phase4_comparison_tests
cargo test -p anki_forge --test phase4_product_build_tests
cargo test -p anki_forge oracle_template --test phase4_product_build_tests
cargo test -p contract_tools product_build_command --test cli_tests
cargo test -p contract_tools product_build_report_validates_against_build_report_schema --test cli_tests
```

Expected:

```text
test result: ok.
test result: ok.
test result: ok.
test result: ok.
test result: ok.
test result: ok.
test result: ok.
test result: ok.
```

- [ ] **Step 2: Run update-safety and writer diff regressions**

Run:

```bash
cargo test -p anki_forge update_safety --tests
cargo test -p writer_core diff --tests
```

Expected:

```text
test result: ok.
test result: ok.
```

- [ ] **Step 3: Run contract tool gates**

Run:

```bash
cargo test -p contract_tools --tests
cargo run -p contract_tools -- verify --manifest contracts/manifest.yaml
```

Expected:

```text
test result: ok.
verification passed
```

- [ ] **Step 4: Run touched workspace crate tests if focused suites pass**

Run:

```bash
cargo test -p anki_forge -p writer_core -p contract_tools
```

Expected:

```text
test result: ok.
```

- [ ] **Step 5: Confirm Phase 4 acceptance criteria**

Check these directly before final handoff:

```bash
rg -l "pub fn diff_against_apkg|fail_on|report_json|product-build|build_report_schema|RISK.BASELINE_UNAVAILABLE|RISK.TEMPLATE_REMOVED" anki_forge contract_tools contracts docs
```

Expected:

```text
anki_forge/src/product/project.rs
anki_forge/src/build/options.rs
contract_tools/src/main.rs
contracts/manifest.yaml
anki_forge/src/risk/rules.rs
```

- [ ] **Step 6: Commit final fixes**

If Step 1 through Step 5 required fixes, commit them:

```bash
git add anki_forge contract_tools contracts docs
git commit -m "test: verify phase 4 diff risk ci"
```

If no files changed after verification, do not create an empty commit.

## Acceptance Checklist

- [ ] `Project::build(BuildOptions::compare_to(...).fail_on(...))` returns `BuildReport` with `comparison`, `diff`, `risk`, `policy`, and typed `status`.
- [ ] `Project::diff_against_apkg(...)` returns a read-only `ProjectDiffReport` using the same comparison assembler and no published final APKG.
- [ ] `BuildReport::ensure_success()` fails for error diagnostics, missing artifact, blocked policy, invalid status, and error status.
- [ ] `report_json` writes the same stable projection emitted by CLI `contract-json`.
- [ ] `contract_tools product-build` reads a serialized `ProductDocument`, writes APKG, writes optional report JSON, and maps exit codes `0`, `2`, `3`, `4`.
- [ ] `contracts/manifest.yaml` exposes `build_report_schema`, and CLI report output validates against `contracts/schema/build-report.schema.json`.
- [ ] Enabled high/critical risk rules have tests and evidence references in `docs/superpowers/checklists/phase-4-risk-evidence-matrix.md`.
- [ ] Template/card high-risk behavior has oracle-backed Product build tests listed in `docs/oracles/phase-4-template-card-risk.md`.
- [ ] Product risk semantics are absent from `writer_core`; writer diff tests remain artifact-observation only.
- [ ] Existing Phase 3 update-safety tests still pass.
