# Anki Forge Phase 3 Anki Compatibility, Inspection, and Writer Design

- Date: 2026-04-04
- Status: Approved in brainstorming, written for planning handoff
- Scope: `Phase 3: Anki Compatibility, Inspection & Writer`
- Parent spec: `2026-03-27-anki-forge-platform-phasing-design.md`

## 1. Purpose

`Phase 3` turns `Phase 2` normalized output into modern-Anki-compatible package artifacts and makes writer behavior observable enough to debug, diff, and gate safely.

This phase exists to solve two coupled problems together:

1. package artifacts must import correctly into modern Anki for the supported core behaviors
2. compatibility drift must be visible through stable, structured evidence rather than only through ad hoc import testing

`Phase 3` is therefore not only a package writer project.
It is a writer plus inspection and regression-evidence project.

## 2. Delivery Strategy Decision

Three delivery strategies were considered:

1. `writer-first`: prioritize `.apkg` generation and basic import success, with minimal inspection/diff support
2. `inspection-first`: prioritize `normalize -> build -> inspect -> diff` evidence flow, while keeping writer coverage focused on core compatibility paths
3. `minimal parallel`: build small versions of writer and inspection at the same time

The chosen strategy is `inspection-first`.

Reasons:

- the repository is already `contract-first` and deterministic-output-oriented
- a writer without stable inspection/diff evidence would make regressions slower to diagnose
- `Phase 2` already established machine-readable diagnostics and comparison-status patterns that `Phase 3` should extend rather than bypass
- modern-Anki compatibility is easier to evolve safely when writer behavior is visible as stable logical observations instead of opaque package blobs

This design therefore distinguishes two loops:

- `writer_core` sub-loop: `build -> inspect -> diff`
- full `Phase 3` system loop: `normalize -> build -> inspect -> diff`

The system loop is the main `Phase 3` verification story.
The writer sub-loop exists so writer correctness can be tested directly from `Normalized IR`.

## 3. Fixed Decisions

The following decisions are fixed for this phase.

1. `contracts/` remains the normative source of truth for all Phase 3 contracts and semantics.
2. `Phase 3` is `inspection-first`, not `.apkg-first`.
3. A staging representation is a first-class writer output and must be inspectable directly.
4. `.apkg` remains a required final artifact, but it is not the only observation surface.
5. `writer-policy` and `verification-policy` are separate assets and must not be collapsed into one policy model.
6. `writer_core` owns artifact construction semantics only; gate decisions remain outside the core writer.
7. `inspect-report` is a stable observation model, not a raw package dump.
8. `diff-report` is an analyzer that emits evidence and compatibility guidance, not a final pass/fail decision.
9. Reports describe status; gates decide consequences.
10. Fixture strategy is explicitly two-tier:
    - `Tier A`: writer-level fixtures driven by `Normalized IR`
    - `Tier B`: end-to-end fixtures driven by `Authoring IR`

## 4. Architecture and Ownership Boundaries

### 4.1 Contract source of truth

`contracts/` remains the only normative source of truth for:

- writer-facing schemas
- inspection and diff report schemas
- writer and verification policies
- normative compatibility semantics
- fixture catalog metadata and golden-regression rules

Implementation crates conform to these contracts.
They do not define them.

### 4.2 Module responsibilities

A new writer-focused implementation module or crate (`writer_core`) owns:

- `Normalized IR -> staging representation`
- staging representation -> `.apkg`
- package-layout and metadata mapping semantics
- artifact fingerprint generation
- build-time semantic diagnostics

`contract_tools` owns:

- command orchestration
- manifest loading and contract validation
- `build`, `inspect`, `diff`, and `verify` command surfaces
- fixture execution and regression gates
- human-readable rendering layered on top of stable machine outputs

`authoring_core` remains upstream and owns normalization semantics.
It is not redesigned in `Phase 3`.

### 4.3 Explicit non-goals

`Phase 3` does not own:

- Node.js or Python binding ergonomics
- high-level authoring helpers
- structured-data ingestion UX
- broad product API simplification
- legacy package-writing compatibility

## 5. Build, Inspect, and Diff System Model

### 5.1 Full verification loop

The complete `Phase 3` verification story is:

1. `normalize`
2. `build`
3. `inspect`
4. `diff`

This loop exists so `Phase 2` normalized-output drift can be observed inside the same regression story as writer behavior.

### 5.2 Writer-focused sub-loop

The writer-focused sub-loop is:

1. `build`
2. `inspect`
3. `diff`

This loop exists so `writer_core` can be exercised directly from `Normalized IR` without requiring an upstream normalization run for every case.

### 5.3 Staging representation

`writer_core` must produce an inspectable staging representation before `.apkg` packaging.

The staging representation is:

- stable enough to support deterministic fingerprinting
- structured enough to support direct inspection
- close enough to final package semantics to serve as the primary writer observation surface

The staging representation must not be treated as an incidental temporary directory with ad hoc shape.
It is a contract-governed writer output model.

### 5.4 `.apkg` role

`.apkg` remains the final emitted artifact for compatibility validation and release-oriented workflows.

However:

- `.apkg` is not the primary diff surface
- `inspect` must support both staging and `.apkg` inputs
- compatibility gates compare staging and `.apkg` on stable semantic observations, not on literal packaging identity

## 6. Contract Assets Introduced in Phase 3

### 6.1 Planned schemas

The minimum new contract assets are:

- `contracts/schema/package-build-result.schema.json`
- `contracts/schema/inspect-report.schema.json`
- `contracts/schema/diff-report.schema.json`
- `contracts/schema/writer-policy.schema.json`
- `contracts/schema/verification-policy.schema.json`
- `contracts/schema/build-context.schema.json`

`build-context` is separate from `writer-policy`.
It captures command/runtime concerns rather than write semantics.

### 6.2 Planned semantics docs

The minimum new normative documents are:

- `contracts/semantics/build.md`
- `contracts/semantics/inspect.md`
- `contracts/semantics/diff.md`
- `contracts/semantics/golden-regression.md`

These documents define behavior that JSON Schema alone cannot express, especially:

- staging-model expectations
- observation stability rules
- diff classification semantics
- golden update and review discipline

## 7. Build Contract

### 7.1 Inputs

`build` accepts three distinct layers of input:

1. `Normalized IR`
2. `writer-policy`
3. `build-context`

These layers must remain separate.

`writer-policy` defines how artifacts should be written semantically.
`build-context` defines runtime options such as:

- whether `.apkg` should be emitted
- staging output location or staging materialization mode
- media resolution mode
- unresolved-asset handling mode
- reproducibility profile or fingerprint mode

### 7.2 Output model

`package-build-result` is the contract-facing machine output for `build`.

Required top-level fields:

- `kind: package-build-result`
- `result_status: success | invalid | error`
- `tool_contract_version`
- `writer_policy_ref`
- `build_context_ref`
- `diagnostics`

Conditionally required fields:

- `staging_ref` and `artifact_fingerprint` when staging materialization exists; both are required for `result_status=success`
- `package_fingerprint` when `.apkg` is emitted successfully
- `apkg_ref` when `.apkg` is emitted successfully

Field roles are distinct:

- `artifact_fingerprint`: fingerprint of the build observation domain, aligned with staging and inspection
- `package_fingerprint`: fingerprint of the final `.apkg` payload
- `staging_ref`: stable reference to the staging representation, not an arbitrary path string
- `build_context_ref`: stable reference to the build-time runtime/materialization context that produced this result

### 7.3 Diagnostic split

`package-build-result` diagnostics must distinguish semantic invalidity from execution failure.

For `result_status=invalid`:

- diagnostics are contract/semantic diagnostics
- items should include stable content-domain targeting where possible
- preferred location fields are `target_selector`, `path`, and `domain`

For `result_status=error`:

- diagnostics are execution diagnostics
- items may omit content-object targeting
- items must include execution context such as `stage`, `operation`, and relevant artifact references when available

## 8. Inspection Contract

### 8.1 Observation model

`inspect-report` is a stable logical observation model for regression comparison.

It is explicitly not:

- a byte-for-byte package dump
- a raw SQLite row dump
- a serializer-accidental view of package order

Observation design must filter out noise such as:

- row ordering artifacts
- compression/container differences
- transient file layout details that do not affect compatibility semantics

### 8.2 Top-level fields

Required top-level fields are:

- `kind: inspect-report`
- `observation_model_version`
- `source_kind: staging | apkg`
- `source_ref`
- `artifact_fingerprint`
- `observation_status: complete | degraded | unavailable`
- `missing_domains`
- `degradation_reasons`
- `observations`

### 8.3 Observation domains

`observations` is a fixed top-level container with stable subdomains.
At minimum it contains:

- `notetypes`
- `templates`
- `fields`
- `media`
- `metadata`
- `references`

New domains may be added only through contract-governed evolution.
It must not become an unconstrained free-form map.

### 8.4 Degraded inspection

Inspection incompleteness is first-class contract state.

When observation is not fully available:

- `observation_status` must degrade from `complete`
- `missing_domains[]` must list absent domains
- `degradation_reasons[]` must state why coverage is reduced

These fields are part of the schema, not only implementation logging or human-readable output.

## 9. Diff Contract

### 9.1 Responsibility

`diff-report` analyzes changes between two inspection reports and provides structured evidence plus compatibility guidance.

It does not decide whether a workflow should fail.
That remains the responsibility of gate/policy evaluation.

### 9.2 Top-level fields

Required top-level fields are:

- `kind: diff-report`
- `comparison_status: complete | partial | unavailable`
- `left_fingerprint`
- `right_fingerprint`
- `left_observation_model_version`
- `right_observation_model_version`
- `summary`
- `uncompared_domains`
- `comparison_limitations`
- `changes[]`

### 9.3 Change model

Each change entry includes at minimum:

- `category`
- `domain`
- `severity`
- `selector`
- `message`
- `compatibility_hint`

Optional evidence fields should prefer stable logical references over raw large payload copies.
Preferred fields include stable `evidence_refs[]` or equivalent logical evidence references tied to observation domains.

### 9.4 Incomplete comparison

Partial comparison is first-class contract state.

When comparison is incomplete:

- `comparison_status` must degrade from `complete`
- `uncompared_domains[]` must identify the uncovered comparison domains
- `comparison_limitations[]` must explain the limiting factors

Typical causes include:

- mismatched observation-model versions
- source-kind differences that reduce comparability
- degraded or unavailable inspection inputs
- missing domain coverage on either side

## 10. Policies and Consequences

### 10.1 Writer policy

`writer-policy` governs artifact-writing behavior only.

It may include:

- compatibility target mode such as `latest-only`
- note-type mapping rules
- metadata emission choices
- media handling semantics that affect produced artifacts

It must not include CI or golden-regression verdict thresholds.

### 10.2 Verification policy

`verification-policy` governs gate consequences, not writer behavior.

It may include:

- acceptable `comparison_status` levels by gate type
- severity thresholds for diff findings
- whether degraded inspection is allowed in a given workflow
- which compatibility hints are treated as blocking for a gate

### 10.3 Governing principle

The governing principle for this phase is:

`reports describe status; gates decide consequences.`

In practical terms:

- `build`, `inspect`, and `diff` outputs describe evidence and status
- gate layers decide whether `degraded`, `partial`, or specific findings should fail a workflow
- report states must not be silently treated as implicit verdicts

## 11. Command Interface

`contract_tools` adds or extends these command-facing contract surfaces:

- `normalize` - existing upstream contract-facing normalization step
- `build` - emits `package-build-result` via a stable `contract-json` surface
- `inspect` - emits `inspect-report` from `staging` or `.apkg` via a stable `contract-json` surface
- `diff` - emits `diff-report` via a stable `contract-json` surface
- `verify` - orchestrates the full system loop or the writer-focused sub-loop depending on fixture tier

Each command must support a machine-stable mode suitable for fixtures and CI.
Human output is convenience-only and must not be the only stable interface.

## 12. Fixture and Golden Strategy

### 12.1 Two-tier fixtures

`Phase 3` uses two fixture tiers.

`Tier A` writer fixtures:

- input: `Normalized IR`
- flow: `build -> inspect -> diff`
- purpose: direct writer correctness

`Tier B` end-to-end fixtures:

- input: `Authoring IR`
- flow: `normalize -> build -> inspect -> diff`
- purpose: cross-phase regression detection

### 12.2 Golden assets are case-derived

Golden artifacts must remain logically attached to fixture cases.

Each case declares its own expected artifacts, such as:

- expected `inspect-report`
- optional expected `diff-report`
- expected diagnostics or status constraints

Even if golden files are stored in a dedicated physical directory, they must not become a free-floating asset pool detached from case identity.

### 12.3 Core fixture coverage

Core compatibility fixtures must cover at least:

- Basic writer behavior
- Cloze writer behavior
- Image Occlusion writer behavior for a scoped supported compatibility lane, not the full future feature matrix
- media-manifest and media-reference behavior
- structured failure explanation for import-relevant mismatches

## 13. Gate and Test Strategy

### 13.1 Gate layers

`Phase 3` uses three gate layers.

`writer fast gate`:

- executes Tier A fixtures broadly
- validates `build -> inspect -> diff`
- is intended for day-to-day development feedback

`system gate`:

- executes a core Tier B set
- validates `normalize -> build -> inspect -> diff`
- catches upstream normalized-output drift inside the same regression story

`compat gate`:

- executes Tier B full compatibility coverage
- checks that staging and `.apkg` are semantically consistent across comparable observation domains
- includes at least one controlled import validation path or compatibility oracle for each supported core scenario
- is appropriate for nightly or release-preparation validation

### 13.2 Semantic consistency rule

The staging-versus-`.apkg` check is a semantic-consistency check, not a report-text equality check.

The comparison target is:

- stable observation meaning in comparable domains

The check is not:

- strict equality across all report fields

Fields such as `source_kind`, packaging-layer fingerprints, and source references may differ legitimately.

### 13.3 Test layering

Testing follows four layers:

1. unit tests for writer mapping, media handling, staging construction, and fingerprint generation
2. contract tests for schemas, semantics assets, policy loading, and fixture catalog integrity
3. regression tests for golden `inspect-report` and `diff-report` outputs
4. system consistency tests for the end-to-end loop and staging-versus-`.apkg` semantic alignment

## 14. Governance Rule for Mainline Changes

For contract-affecting `Phase 3` changes, implementation-only updates are not sufficient.

Mainline acceptance requires synchronized updates to:

- contracts/spec assets
- fixtures and golden expectations
- executable gates/tests

This keeps `Phase 3` aligned with the same contract-first discipline already established in `Phase 1` and `Phase 2`.

## 15. Phase 3 Exit Criteria

`Phase 3` is ready to hand off when all are true:

1. the full `normalize -> build -> inspect -> diff` loop is operational for the core fixture set
2. the direct writer `build -> inspect -> diff` loop is operational for Tier A fixtures
3. `package-build-result`, `inspect-report`, and `diff-report` are schema-governed and exposed through stable `contract-json` command surfaces for machine consumption
4. staging is a first-class inspectable writer output, not only an internal temporary artifact
5. `writer-policy`, `verification-policy`, and `build-context` are separated by responsibility and governed by contracts
6. inspection degradation and diff incompleteness are represented explicitly in schema-governed status fields
7. staging and `.apkg` observation surfaces can be checked for semantic consistency across comparable domains
8. compatibility fixtures cover the core modern-Anki writer behaviors required by this phase
9. import failures and compatibility mismatches produce structured evidence suitable for CI and downstream tooling
10. supported core scenarios have at least one controlled import validation path or compatibility oracle in the acceptance story
