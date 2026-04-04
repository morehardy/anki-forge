# Anki Forge Phase 2 Core Authoring Model Design

- Date: 2026-04-03
- Status: Approved in brainstorming, written for planning handoff
- Scope: `Phase 2: Core Authoring Model`
- Parent spec: `2026-03-27-anki-forge-platform-phasing-design.md`

## 1. Purpose

`Phase 2` defines and implements the semantic core between authoring input and writer-facing normalized output.

This phase exists to provide a stable, update-friendly, analyzable model for later compatibility/writer work, while preserving the `contract-first` governance established in `Phase 1`.

## 2. Fixed Decisions

The following decisions are fixed for this phase.

1. Delivery target is near-`Phase 3` readiness for core semantics, not a minimal placeholder.
2. Architecture uses split responsibilities:
   - a new core crate (`authoring_core`) for semantic implementation
   - `contract_tools` as gate/orchestration tooling
3. `Normalized IR` uses a mixed strategy:
   - core fields align closely with Anki semantics
   - governance/trace fields remain Forge-neutral
4. Identity policy is dual-mode with strict defaults:
   - default path is deterministic
   - non-deterministic or external identity is object-level opt-in only
5. Merge risk is a graded analysis output:
   - high risk and invalidity are separate concepts
   - policy/gates decide enforcement

## 3. Architecture and Ownership Boundaries

### 3.1 Contract source of truth

`contracts/` remains the normative source of truth for `Phase 2` semantics.

`authoring_core` is a conforming implementation of Phase 2 contracts, not their source of truth.

### 3.2 Module responsibilities

`authoring_core` owns:

- `Authoring IR -> Normalized IR`
- identity policy resolution
- merge-risk assessment
- core semantic validation rules directly tied to normalization

`contract_tools` owns:

- contract/gate orchestration
- fixture execution and regression gates
- CLI invocation and rendering of contract-facing outputs

### 3.3 Non-goals in Phase 2

Phase 2 does not own:

- package emission/writer correctness (Phase 3)
- language binding ergonomics (Phase 4)
- product-level helper systems and ingestion UX (Phase 5)

## 4. Contract Assets Introduced/Extended

`Phase 2` extends `contracts/` with machine-readable and normative assets.

### 4.1 Priority closure set before planning handoff

The following six items are treated as priority contract closures and should be explicitly covered before `writing-plans` handoff:

1. diagnostics contract
2. policy assets
3. comparison-context schema
4. target-selector grammar
5. contract-json required fields
6. canonical serialization specification

### 4.2 Planned schemas

- `contracts/schema/normalized-ir.schema.json`
- `contracts/schema/merge-risk-report.schema.json`
- `contracts/schema/normalization-diagnostics.schema.json`
- `contracts/schema/comparison-context.schema.json`
- `contracts/schema/normalization-result.schema.json` (contract-stable CLI machine output envelope)

### 4.3 Planned policy assets

- `contracts/policies/identity-policy.default.yaml`
- `contracts/policies/risk-policy.default.yaml`
- `contracts/schema/identity-policy.schema.json`
- `contracts/schema/risk-policy.schema.json`

Policy assets should expose stable identifiers and versions so fixtures and reports can reference them by ID, not only by file path.

### 4.4 Planned semantics docs

- `contracts/semantics/normalization.md`
- `contracts/semantics/identity.md`
- `contracts/semantics/merge-risk.md`
- `contracts/semantics/target-selector-grammar.md`
- `contracts/semantics/canonical-serialization.md`

The manifest and schema gates must include these assets before implementation is considered complete.

## 5. Normalization Model and Pipeline

### 5.1 Inputs and outputs

Required inputs:

- `Authoring IR`
- `NormalizationPolicy` / `IdentityPolicy`

Optional comparison input (schema-governed):

- `ComparisonContext` (from `comparison-context.schema.json`)

Outputs:

- `Normalized IR` (when valid)
- `NormalizationDiagnostics`
- `MergeRiskReport` (when comparison context is provided)

### 5.1.1 ComparisonContext contract minimum

`ComparisonContext` defines the baseline used for risk assessment. It must not be implicit.

Minimum fields:

- `kind: comparison-context`
- `baseline_kind: normalized_ir | identity_index`
- `baseline_artifact_ref` or embedded baseline payload
- `baseline_artifact_fingerprint`
- `risk_policy_ref`
- `comparison_mode: strict | best_effort`

### 5.2 Execution order

To avoid identity drift from authoring-side representational noise, the pipeline order is:

1. `Shape validation`
2. `Semantic precheck`
3. `Canonicalization A` (independent of identity assignment)
4. `Reference linking`
5. `Identity resolution`
6. `Canonicalization B` (final convergence after identity injection)
7. `Semantic postcheck`
8. `Risk assessment`

### 5.3 Validity vs risk separation

- shape/core semantic invalidity blocks `Normalized IR` emission
- merge-risk grading does not invalidate normalized content by default
- downstream gates/policies decide whether high-risk findings should fail a workflow

### 5.4 Stability guarantee

Cross-language stability is defined as stable semantics and stable canonical ordering rules, not serializer-specific byte identity by default.

Canonical serialization rules are defined separately and used where stable machine diffs are required (fixtures/CI/contract-json output).

### 5.5 Diagnostics contract

`NormalizationDiagnostics` is a distinct output contract and is not merged into `MergeRiskReport`.

Minimum diagnostic structure:

- `kind: normalization-diagnostics`
- `status: valid | invalid`
- `items[]` with `level`, `code`, `summary`, and optional `target_selector`/`details`

Failure behavior:

- invalid shape/core semantic checks block `Normalized IR`
- diagnostics still return contract-stable machine output

## 6. Identity Policy Contract

### 6.1 Defaults and exceptions

Identity defaults to deterministic resolution.

Exceptions are object-level and explicit:

- `external`
- `random`

No implicit fallback to non-deterministic identity is allowed.

### 6.2 Selector model

Override targeting uses stable logical selectors, not array-position paths.

Contract field: `target_selector`.

Selectors must resolve against authoring-domain stable keys and must not depend on transient list indices.

Selector grammar is specified in `contracts/semantics/target-selector-grammar.md`.

Minimum grammar constraints:

- no array index addressing (for example `/notes/3`)
- selector resolves by stable keys (for example object kind + key predicates)
- selector resolution must be deterministic
- zero-match and multi-match are both contract errors

### 6.3 Exception metadata and auditability

For each non-default exception:

- `reason_code` is required
- `reason` is optional

For `external`, required fields include explicit external identity payload.

For `random`, a warning-level diagnostic is emitted even when configuration is valid.

### 6.4 Failure rules

Normalization fails when:

- selector is unmatched or ambiguous
- required external identity payload is missing or conflicts
- non-deterministic behavior occurs without explicit exception declaration

### 6.5 Stability caveat for random

Global reproducibility guarantees apply to paths without `random` overrides.

Objects using explicit `random` overrides are outside reproducible-identity guarantees; non-random objects remain governed by deterministic stability rules.

## 7. Merge Risk Assessment Contract

### 7.1 Responsibility

`MergeRiskReport` represents risk classification and evidence for evolution decisions.

It is not a substitute for validity diagnostics and not a workflow enforcer.

### 7.2 Inputs

Risk assessment requires explicit comparison context.

The report generator receives:

- current normalized artifact
- baseline comparison artifact/index from `ComparisonContext`
- risk policy reference/version

If comparison context is absent, risk assessment is skipped and no `MergeRiskReport` is emitted in contract-json output.

### 7.3 Output model

Planned report fields:

- `kind: merge-risk-report`
- `comparison_status: complete | partial | unavailable`
- `overall_level: low | medium | high`
- `findings[]`
- `policy_version`
- `baseline_artifact_fingerprint`
- `current_artifact_fingerprint`

Each finding includes:

- `level`
- `code`
- `dimension: identity | structure | references | renderability | exceptions`
- `target_selector`
- `summary`
- optional `details`
- `evidence` references

### 7.4 Evidence representation

Evidence should be stable logical references, not large raw content copies.

Preferred fields:

- `baseline_selector`
- `current_selector`
- `baseline_excerpt_hash`
- `current_excerpt_hash`
- optional short textual summary

### 7.5 Comparison incompleteness

When baseline coverage is insufficient, mapping is partial, or exceptions degrade comparability, report completeness must degrade via `comparison_status` and include explicit reasoning.

## 8. `contract_tools normalize` Contract-facing Interface

`contract_tools` adds a `normalize` command that calls `authoring_core` and supports at least:

- `--output contract-json` (machine/stable)
- `--output human` (readable)

`contract-json` is a stable interface governed by `normalization-result.schema.json`.

Required top-level fields:

- `kind: normalization-result`
- `result_status: success | invalid | error`
- `tool_contract_version`
- `policy_refs` (at minimum identity policy reference, and risk policy reference when comparison is enabled)
- `comparison_context` (explicitly `none` or populated metadata)
- `diagnostics`

Conditionally required fields:

- `normalized_ir` is required when `result_status=success`
- `merge_risk_report` is required when `comparison_context` is provided and `result_status=success`

Human output is convenience-only and must not be the only stable integration surface.

## 9. Test and Fixture Strategy

### 9.1 Test layering

`authoring_core` tests emphasize semantic structure assertions:

- normalization behavior
- identity behavior
- risk findings
- semantic rule outcomes

Serialization stability tests are separate and focused.

`contract_tools` tests and CI gates assert canonical machine output stability for contract surfaces.

### 9.2 Fixture organization

`fixtures/index.yaml` remains catalog-oriented (id/category/path/tags/policy refs).

Detailed test payloads live in dedicated case files under `fixtures/phase2/...`.

### 9.3 Merge-risk fixtures

Each merge-risk fixture carries explicit policy reference/version and comparison inputs so expected findings remain explainable across policy evolution.

### 9.4 Stable output rules for `contract-json`

Contract JSON output defines explicit rules for:

- key ordering strategy
- array ordering origin (semantic/canonical rules, not serializer accident)
- null/empty field omission behavior
- diagnostics/findings ordering
- deterministic map/object serialization order for canonical mode

The normative rules live in `contracts/semantics/canonical-serialization.md` and are used by fixtures and CI gates.

## 10. Governance Rule for Mainline Changes

For any contract-affecting mainline change, implementation-only updates are not sufficient.

Mainline acceptance requires synchronized updates to:

- contracts/spec assets
- fixtures
- executable gates/tests

This preserves contract-first discipline while allowing local exploratory implementation during development.

## 11. Phase 2 Exit Criteria

Phase 2 is ready to hand off when all are true:

1. `authoring -> normalized` is stable on representative fixtures.
2. deterministic identity is default and reproducible.
3. object-level identity exceptions are explicit and auditable.
4. merge-risk report generation is machine-readable and comparison-context aware.
5. `contract_tools normalize --output contract-json` is schema-governed and stable for downstream consumption.
6. diagnostics contract, policy assets, comparison-context schema, target-selector grammar, contract-json required fields, and canonical serialization spec are all merged and gated.
7. contracts/fixtures/gates are updated together for all merged contract-affecting changes.
