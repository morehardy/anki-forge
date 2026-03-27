# Anki Forge Platform Phasing Design

- Date: 2026-03-27
- Status: Approved in brainstorming, written for planning handoff
- Scope: program-level phase decomposition for `anki-forge`

## 1. Context

`anki-forge` is intended to become:

> A latest-only, update-friendly, strongly-validated note type / template / media / image-occlusion authoring toolkit for modern Anki.

The repository is currently at an early stage:

- top-level implementation code is not present on `main`
- `docs/source/rslib` is the primary reference baseline

That means this design must solve two problems at once:

1. define the product roadmap
2. define the platform and governance skeleton that lets the product evolve safely

## 2. Design Drivers

The design is driven by the following product goals.

### Functional goals

1. `latest package writer`
2. `stable IDs / GUID / merge-friendly evolution`
3. `strong validator`
4. `first-class Basic / Cloze / IO note types`
5. `template helper system`
6. `media + fonts bundler`
7. `browser/deck-override/field-metadata support`
8. `structured-data ingestion pipeline`

### Non-functional goals

1. modern engineering workflow: development, validation, release, GitHub CI, compatibility verification
2. simple, ergonomic API and usage flow for end users
3. cross-language reach beyond Rust, especially Node.js and Python

## 3. Strategic Decisions

The following decisions are fixed for this roadmap.

- The project is decomposed by platform layers, not by isolated feature buckets.
- `Phase 1` optimizes for foundation and DevOps, not end-user breadth.
- The cross-language strategy is `schema/protocol-first`.
- Both `IR schema` and `Service API` exist, but only `IR schema` is intended to stabilize first.
- Bindings must not force core architecture decisions prematurely.
- `docs/source/rslib` is the compatibility reference source, and alignment with modern Anki behavior has higher priority than inventing novel package semantics.

## 4. Platform Layer Model

The system is organized as six phases across five platform layers plus one split product layer.

### Phase 1: Foundation Platform

Purpose:

- establish the monorepo/workspace structure
- define `IR schema v0`
- define validation and diagnostic contracts
- define governance and release rules
- establish CI/CD and verification foundations

This phase owns:

- workspace and package topology
- schema evolution policy
- service API versioning policy
- error code registry
- fixture versioning rules
- ADR/RFC process
- validation framework and reporting format
- test pyramid and compatibility test skeleton
- release automation and repository hygiene

This phase explicitly does not own:

- full package writing
- full note-type product features
- production-ready bindings

Exit criteria:

- `IR schema v0` exists with explicit compatibility rules, not only a version field
- `ValidationReport`, `error code`, and `path` conventions are frozen at `v0`
- contract fixtures exist with clear upgrade rules
- schema/API change governance is defined through ADR/RFC process
- CI covers formatting, linting, tests, schema contract checks, and release verification skeletons

### Phase 2: Core Authoring Model

Purpose:

- define the authoring model users work with
- define the normalized model the writer/compat layers consume
- formalize identity and merge-risk semantics

This phase owns:

- `Authoring IR` vs `Normalized IR`
- normalization pipeline
- deterministic identity policy
- merge-risk metadata model
- strong validator core semantic rules
- update-friendly authoring semantics

This phase explicitly does not own:

- full modern Anki package emission
- full helper system or ingestion product UX

Exit criteria:

- `authoring -> normalized` pipeline exists and is testable
- IDs/GUID generation strategies are configurable and reproducible
- merge-risk metadata can distinguish safe from risky evolution paths
- normalized output is stable enough to feed compatibility/writer work

### Phase 3: Anki Compatibility, Inspection & Writer

Purpose:

- convert normalized data into artifacts that modern Anki can import correctly
- make compatibility differences visible and diagnosable

This phase owns:

- latest package writer
- compat reference fixtures
- inspect/diff/normalize tooling
- failure explanation tooling
- package layout, metadata mapping, and media manifest behavior
- Basic/Cloze/IO compatibility correctness at writer level

This phase explicitly does not own:

- full cross-language ergonomics
- product-level ingestion flows

Exit criteria:

- `build + inspect + golden diff` workflow is operational
- compatibility reference fixtures cover core modern Anki behaviors
- import failures and compatibility mismatches produce structured diagnostics
- writer output is validated against modern-Anki-oriented expectations

### Phase 4: Language Bindings & DX

Purpose:

- expose the platform safely to Rust, Node, and Python users
- ensure bindings remain conformant to the core contract

This phase owns:

- Rust, Node, and Python public entry points
- binding conformance kit
- shared contract fixture runner
- developer documentation and examples
- cross-language warning/error semantics

Governance rule:

- language-specific ergonomics policy must not back-propagate accidental complexity into core contracts

Exit criteria:

- Python and Node pass the shared conformance suite
- errors and warnings are semantically aligned across languages
- examples are sufficient to complete a minimal authoring flow from each supported language

### Phase 5A: Product Authoring Features

Purpose:

- deliver the high-level authoring toolkit experience

This phase owns:

- first-class Basic / Cloze / IO note types
- template helper system
- media + fonts bundler
- field metadata
- browser/deck override support
- high-level authoring API ergonomics

Exit criteria:

- product-facing authoring APIs are substantially simpler than raw IR construction
- major note types are first-class, not ad hoc wrappers
- helpers and bundlers behave predictably and are validator-aware
- metadata and overrides are represented consistently across authoring and writer layers

### Phase 5B: Structured Data Pipelines

Purpose:

- turn external structured data into repeatable deck/package build workflows

This phase owns:

- CSV/JSON/YAML ingestion
- mapping and transforms
- batch build flows
- pipeline-focused CLI/API

Exit criteria:

- structured-data inputs can be converted into stable build flows
- mapping and transform behavior is testable and debuggable
- pipeline-oriented interfaces support repeatable batch production workflows

## 5. Why Platform-Layer Decomposition

Three decomposition styles were considered:

1. capability-domain split
2. platform-layer split
3. end-user workflow split

The chosen model is `platform-layer split`.

Reasons:

- it matches the requirement that `Phase 1` focus on skeleton and DevOps
- it supports a `schema/protocol-first` multi-language strategy
- it keeps `IR schema` stabilization ahead of binding ergonomics
- it prevents early feature work from forcing unstable cross-language API choices
- it gives each phase a clearer dependency boundary

## 6. Goal-to-Phase Mapping

### Functional goals

| Goal | Primary phase | Supporting phases | Notes |
| --- | --- | --- | --- |
| latest package writer | `Phase 3` | `Phase 1-2` | depends on normalized model and compatibility strategy |
| stable IDs / GUID / merge-friendly evolution | `Phase 2` | `Phase 1` | belongs in core semantics, not only writer logic |
| strong validator | `Phase 1-2` | `Phase 3-5` | framework first, semantic depth second, product-specific rules later |
| first-class Basic / Cloze / IO note types | `Phase 5A` | `Phase 3` | writer correctness lands first, product ergonomics later |
| template helper system | `Phase 5A` | `Phase 2-3` | should build on stable normalized semantics |
| media + fonts bundler | `Phase 5A` | `Phase 3` | depends on correct package/media handling |
| browser/deck-override/field-metadata support | `Phase 5A` | `Phase 2-3` | needs metadata modeling before product exposure |
| structured-data ingestion pipeline | `Phase 5B` | `Phase 2-4` | depends on stable IR, writer, and language access surface |

### Non-functional goals

| Goal | Primary phase | Supporting phases | Notes |
| --- | --- | --- | --- |
| modern engineering workflow | `Phase 1` | all | foundational governance and automation work |
| ergonomic API and usage flow | `Phase 4-5` | `Phase 2-3` | should mature after core semantics stabilize |
| cross-language support | `Phase 4` | `Phase 1-3` | bindings follow stable contract, not vice versa |

## 7. Phase Ownership Boundaries

To avoid scope bleed, the following boundaries are explicit.

- `Phase 1` freezes contracts and governance, not end-user ergonomics.
- `Phase 2` owns semantic correctness and normalization, not package emission.
- `Phase 3` owns compatibility correctness, inspection, and failure explanation, not binding UX.
- `Phase 4` owns language access and conformance, not core semantic redesign.
- `Phase 5A` owns author-facing product ergonomics.
- `Phase 5B` owns pipeline-facing product ergonomics.

## 8. Key Invariants

The roadmap assumes these invariants remain true unless an ADR/RFC explicitly changes them.

- The toolkit is `latest-only`; legacy package-writing compatibility is not a primary design goal.
- Validation is a first-class product surface, not only an internal safeguard.
- Diagnostics must be structured enough to survive across Rust, Node, and Python.
- Authoring data and normalized data are distinct representations with distinct responsibilities.
- Determinism matters: identity generation, normalization, fixtures, and golden outputs must be reproducible.
- Inspection and diff tooling are part of correctness, not optional developer convenience.

## 9. Planning Implications

The next planning cycle should start with `Phase 1`, because later phases depend on decisions made there:

- workspace/package topology
- initial crate/module map
- schema and fixture storage strategy
- validation report contract
- release and compatibility automation
- ADR/RFC and versioning governance

`Phase 1` should be planned as a deliverable platform foundation, not as a placeholder bootstrap.

## 10. Out of Scope for This Design

This document does not yet define:

- the exact workspace crate list
- the exact schema format choice
- the exact Node/Python binding implementation technology
- the exact writer storage model
- concrete API signatures

Those belong to subsequent implementation planning, beginning with a dedicated `Phase 1` plan.
