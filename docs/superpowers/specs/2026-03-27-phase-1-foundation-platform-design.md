# Anki Forge Phase 1 Foundation Platform Design

- Date: 2026-03-27
- Status: Approved in brainstorming, written for planning handoff
- Scope: `Phase 1: Foundation Platform`
- Parent spec: `2026-03-27-anki-forge-platform-phasing-design.md`

## 1. Purpose

`Phase 1` establishes the contract and governance foundation for `anki-forge`.

It is defined as:

> A CI-grade, schema-centered contract bundle for Anki Forge, plus the governance and verification machinery required to evolve it safely.

This phase exists to make future implementations obeyable, not to make early implementations impressive.

## 2. What Phase 1 Produces

The primary deliverable of `Phase 1` is not a Rust crate, a CLI product, or a language binding.
The primary deliverable is a language-neutral `contract bundle`.

That bundle must include:

- `IR schema`
- `ValidationReport / diagnostic payload / error reference / path` contract
- minimal `service envelope` contract
- normative semantic text
- error registry
- normative fixtures
- versioning and compatibility rules
- bundle manifest
- executable verification tooling
- CI/release gates that enforce the contract

Rust is only the first reference implementation for contract tooling and reference stubs.
Rust is not the source of truth for the contract itself.

## 3. What Phase 1 Does Not Produce

`Phase 1` explicitly does not deliver:

- operation-specific surface such as `validate`, `inspect`, `build`, or `normalize`
- product-grade Rust core implementation
- Node.js or Python bindings
- latest package writer
- full authoring model implementation
- end-user ergonomic API
- product CLI

If a change introduces operation-specific product semantics, it is out of scope for `Phase 1`.

## 4. Contract Strategy

The contract bundle is `JSON Schema-first` and `schema-centered`.

The source-of-truth package is language-neutral and includes:

- schema files
- normative semantic text
- normative fixtures in JSON/YAML
- version and compatibility rules
- executable verification

Complex semantics that cannot be fully expressed in JSON Schema must be defined by:

- normative text in `contracts/semantics/`
- executable fixtures
- contract verification gates

The contract bundle is `CI-grade`, but not a product-grade implementation.

## 5. Public Compatibility Model

`Phase 1` uses a two-level version model with one public compatibility axis.

### Public compatibility axis

The only public compatibility axis is `contract bundle version`.

External implementations may say:

- compatible with `anki-forge contract bundle vX.Y`

External implementations may not say:

- compatible with schema version only
- compatible with fixture version only
- compatible with service envelope version only

### Internal governed versions

The bundle may track internal component versions for:

- schema
- fixture set
- service envelope
- error registry

These component versions are internal governance metadata.
They support review, diff classification, release notes, and upgrade control.
They are not independent public compatibility targets.

### Compatibility classification

Every contract-affecting change must be classified.
Suggested classes include:

- additive compatible
- behavior-tightening compatible
- behavior-changing incompatible
- fixture-only non-semantic
- documentation-only normative clarification

### Upgrade rule

Each classified change must specify:

- whether migration notes are required
- whether fixtures must be updated
- whether executable verification must change
- whether old fixtures may coexist temporarily

## 6. Top-Level Repository Shape

`Phase 1` organizes the repository around normative assets first and implementation helpers second.

Recommended top-level structure:

```text
contracts/
contract_tools/
docs/
implementations/
.github/
```

Roles:

- `contracts/`: normative source of truth
- `contract_tools/`: official executable verification tooling
- `docs/`: explanatory docs, governance docs, planning docs
- `implementations/`: reference or future consumers
- `.github/`: automation integration only

`implementations/` may remain empty, skeletal, or limited to reference stubs in `Phase 1`.
Preallocating this directory does not create a scope commitment to product implementation.

## 7. Contract Bundle Asset Layout

Recommended layout inside `contracts/`:

```text
contracts/
  schema/
  semantics/
  errors/
  fixtures/
    valid/
    invalid/
    expected/
    service-envelope/
    evolution/
  versioning/
  manifest.yaml
```

Asset responsibilities:

- `schema/`: data-shape contracts
- `semantics/`: normative text for rules JSON Schema cannot fully express
- `errors/`: error registry and lifecycle rules
- `fixtures/`: normative examples and expected evidence
- `versioning/`: version semantics, compatibility classes, upgrade rules
- `manifest.yaml`: bundle identity and version metadata entrypoint

### Manifest boundary

`manifest.yaml` is the bundle root manifest.
It is a single-entry asset, not a broad asset family by default.

It records:

- contract bundle version
- internal component versions
- compatibility claims
- fixture set identity
- packaging metadata
- release metadata

`versioning/` has a different role.

- `manifest.yaml` answers: what bundle is this?
- `versioning/` answers: how should version changes and compatibility be interpreted?

## 8. Module Model

`Phase 1` uses eight tightly bounded modules plus one explicit core asset.

### 8.1 Schema Module

Defines language-neutral structure contracts for:

- `Authoring IR`
- `ValidationReport`
- `DiagnosticItem`
- minimal `ServiceEnvelope`
- diagnostic payload schema
- error reference shape

This module defines structure only.
It does not define full error semantics or operation-specific behavior.

### 8.2 Semantics Module

Defines normative meaning that JSON Schema cannot fully express, including:

- logical constraints between fields
- `path` interpretation rules
- warning vs error semantic boundaries
- compatibility and incompatibility definitions
- fixture upgrade legality

The semantics module may supplement schema but may not contradict it.

### 8.3 Error Registry Module

Defines:

- error code registry
- naming rules
- classification rules
- lifecycle states
- deprecation and removal rules
- registry-to-report expectations

This module owns error meaning and lifecycle.
It does not redefine report structure.

### 8.4 Fixture Module

Produces normative examples for:

- valid IR
- invalid IR
- expected reports
- minimal service envelope examples
- compatibility and incompatibility cases
- fixture upgrade rule cases

Fixtures are contract assets.
They are not coupled to any particular runner implementation.

### 8.5 Executable Verification Module

Consumes:

- schema
- semantics
- error registry
- fixtures
- bundle manifest

It provides executable conformance evidence through:

- schema integrity checks
- semantic consistency checks
- registry consistency checks
- fixture conformance checks
- manifest and version checks

This module cannot become a new normative source.

### 8.6 Versioning Module

Defines:

- version semantics
- compatibility classes
- upgrade rules
- change-classification policy

It does not define business content.
It classifies changes across contract assets.

### 8.7 Governance Module

Defines:

- ADR/RFC requirements
- review requirements for contract-affecting changes
- release approval requirements
- compatibility-significance gates

Governance applies to contract-affecting changes only.
It does not own general repository process for unrelated implementation work.

### 8.8 CI/Release Module

Automates:

- verification execution
- compatibility checks
- drift detection
- packaging
- publishing

CI/Release executes previously defined rules.
It does not define or reinterpret them.

### 8.9 Bundle Manifest Asset

The bundle manifest is a first-class contract asset.
It is not promoted to a separate governance or semantics module, but it must be named explicitly.

It is consumed by:

- executable verification
- CI/release automation
- change classification and release tooling

## 9. Dependency Model

The internal dependency shape is intentionally one-directional.

### Core definition chain

`Schema -> Semantics -> Error Registry -> Fixtures`

Interpretation:

- schema defines structural boundaries
- semantics adds normative meaning without contradicting schema
- error registry defines stable error-space meaning within those boundaries
- fixtures instantiate the contract as normative evidence

### Official verification chain

`Schema + Semantics + Error Registry + Fixtures + Bundle Manifest -> Executable Verification`

Interpretation:

- executable verification explicitly depends on semantics, not merely references it
- runners and checkers consume contract inputs
- they do not define normative meaning

### Control plane

- `Versioning` classifies changes across all contract assets
- `Governance` controls how contract-affecting changes are proposed and approved
- `CI/Release` automates verification, compatibility checks, drift detection, packaging, and publishing

Guiding principle:

> Normative meaning flows downward; conformance evidence flows upward.

## 10. CI-Grade Contract Gates

`Phase 1` must enforce the contract through explicit CI gates.

### 10.1 Schema integrity gates

At minimum:

- schema files are valid
- schema references are valid
- report, diagnostic, service envelope, and error reference structures are parseable
- manifest-declared schema metadata matches actual assets

### 10.2 Semantics consistency gates

At minimum:

- normative semantic references point to real contract assets
- compatibility and upgrade statements map to fixtures or executable checks
- major contract meaning changes cannot exist as text-only edits

### 10.3 Error registry gates

At minimum:

- error codes are unique
- lifecycle states are valid
- deprecated/removed transitions are valid
- expected fixtures use valid registered codes
- error reference shape stays aligned with registry rules

### 10.4 Fixture conformance gates

At minimum:

- valid fixtures pass structure and semantic checks
- invalid fixtures produce expected reports or expected diagnostic references
- service-envelope fixtures match the minimal envelope contract
- evolution fixtures demonstrate compatible, incompatible, and upgrade-rule cases
- expected evidence remains paired with the input cases it proves

### 10.5 Versioning and manifest gates

At minimum:

- `manifest.yaml` is complete and parseable
- bundle version and internal component versions are internally consistent
- contract-affecting changes are compatibility-classified
- required upgrade notes or evolution fixtures are present

### 10.6 Release readiness gates

At minimum:

- a versioned contract artifact can be packaged
- required normative assets are present in the artifact
- release metadata is complete
- compatibility or change summary generation is possible

Global rule:

> Every contract-affecting change must fail closed unless its new meaning is captured by assets and gates.

## 11. Exit Criteria

`Phase 1` is complete only when all of the following are true.

### 11.1 Contract bundle exists as the primary deliverable

The repository contains a real contract bundle rooted at `contracts/manifest.yaml`, and that manifest identifies:

- schema assets
- semantics assets
- error registry
- fixture sets
- version metadata
- compatibility claims
- release metadata

### 11.2 IR schema v0 exists with compatibility rules

`Authoring IR`, `ValidationReport`, `DiagnosticItem`, and minimal `ServiceEnvelope` all exist as `v0` structure contracts with explicit compatibility meaning.

### 11.3 Validation contract is frozen at v0

The `ValidationReport / diagnostic payload / error reference / path` contract is frozen at `v0`, meaning changes require:

- compatibility classification
- supporting fixtures or executable evidence
- explicit governance

### 11.4 Error registry is stable and governable

A formal registry exists with:

- uniqueness rules
- classification rules
- lifecycle states
- deprecation/removal rules
- normative mapping to report expectations

### 11.5 Normative fixtures cover all Phase 1 contract surfaces

Fixtures cover:

- valid IR cases
- invalid IR cases
- expected reports
- minimal service-envelope cases
- compatibility/incompatibility evolution cases
- fixture upgrade rule cases

### 11.6 Executable verification is CI-grade

Official tooling in `contract_tools/` can run all required contract checks in CI and produce authoritative pass/fail outcomes.

### 11.7 Governance is active for contract-affecting changes

ADR/RFC and review rules are in force for contract-affecting changes, including explicit triggers for:

- bundle-version-impacting changes
- migration-relevant changes
- evolution-fixture-required changes

### 11.8 Release and consumption model is proven

The bundle can be packaged, versioned, released, and consumed as a contract artifact.
Implementations declare compatibility using bundle version only.

Completion definition:

> Phase 1 is done when contract meaning, contract evidence, and contract governance are all machine-checkable at bundle scope.

## 12. Risks and Drift Controls

### 12.1 Reference implementation drift

Risk:

- Rust tooling becomes the de facto normative source

Control:

- normative meaning lives under `contracts/`

### 12.2 Schema-only false confidence

Risk:

- schema exists, but semantics and evidence do not

Control:

- important semantic meaning must also appear in semantics and fixtures/gates

### 12.3 Fixture-runner coupling

Risk:

- fixture formats are distorted to satisfy one runner

Control:

- fixtures are normative assets; runners consume but do not own them

### 12.4 Version fragmentation

Risk:

- implementations claim compatibility with internal sub-versions instead of bundle version

Control:

- public compatibility claims are bundle-version-only

### 12.5 Governance sprawl

Risk:

- ADR/RFC expands to unrelated repository work

Control:

- governance applies only to contract-affecting changes

### 12.6 Premature product pressure

Risk:

- product semantics sneak into `Phase 1`

Control:

- `Phase 1` defines contracts and verification only, not product operations

## 13. Implementation Constraints

The following constraints apply throughout `Phase 1`.

1. Normative assets first.
If a capability cannot first be defined in `contracts/`, it should not first appear in implementation code.

2. Executable evidence required.
Contract-affecting changes require fixtures, expected evidence, or executable gates.

3. No hidden public surface.
Tooling interfaces in `Phase 1` do not imply future product API commitments.

4. Manifest-centered consumption.
Tools and CI enter through `contracts/manifest.yaml`, not by guessing from ad hoc directory scans.

5. Explanatory docs are non-normative by default.
`docs/` explains; `contracts/` defines.

6. Implementations may remain skeletal.
`implementations/` may stay empty or near-empty without constituting failure.

7. Contract discipline beats feature momentum.
If a change improves short-term demo value while weakening contract clarity, it should be rejected in `Phase 1`.

## 14. Planning Handoff

The next step after this spec is a dedicated implementation plan for `Phase 1`.
That plan should turn this design into:

- concrete repository layout decisions
- concrete file-format choices within the approved boundaries
- concrete CI workflows and contract tooling milestones
- concrete exit-check mapping to work items

This spec intentionally stops before operation-specific API or product implementation planning.
