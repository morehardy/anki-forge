# Anki Forge Phase 4 Language Bindings and DX Design

- Date: 2026-04-06
- Status: Approved in brainstorming, written for planning handoff
- Scope: `Phase 4: Language Bindings & DX`
- Parent spec: `2026-03-27-anki-forge-platform-phasing-design.md`

## 1. Purpose

`Phase 4` turns the existing `Phase 2` and `Phase 3` core implementation into a disciplined multi-language access surface without changing the contract-defined semantics those phases already established.

This phase exists to solve four closely related problems together:

1. Rust needs a complete low-level public surface that is suitable as the repository's normative library implementation.
2. Node and Python need minimal-but-usable bindings that stay tightly constrained by shared conformance rather than inventing language-local semantics.
3. the CLI contract surface needs to be treated as a stable cross-language protocol, not only as an internal operator tool.
4. runtime discovery, failure classification, examples, and documentation need to become predictable enough that advanced users can reliably compose the platform from multiple languages inside the repository.

`Phase 4` is therefore not a product-SDK phase.
It is a semantics-preserving access-layer and developer-experience phase.

## 2. Delivery Strategy Decision

Three delivery strategies were considered:

1. `cli-first everywhere`: treat the CLI as the only real access surface and keep every language as a thin command wrapper.
2. `dual-layer canonical Rust plus CLI protocol wrappers`: make Rust the complete low-level normative implementation, keep the CLI as the stable protocol surface, and let Node/Python wrap the CLI with strict conformance.
3. `native-binding ready from day one`: design Rust, Node, and Python around future native bindings immediately, including early ABI-oriented decisions.

The chosen strategy is `dual-layer canonical Rust plus CLI protocol wrappers`.

Reasons:

- it matches the requirement that Rust be the complete low-level normative implementation first
- it preserves `contracts/` as the only semantic source of truth instead of letting wrapper ergonomics become a hidden contract surface
- it lets Node/Python become useful in `Phase 4` without prematurely freezing native binding or packaging decisions
- it keeps the CLI machine surface relevant and testable as the cross-language protocol boundary
- it leaves clean upgrade paths for future publishable packages and native bindings without forcing those commitments into the first release of this phase

The resulting layering is:

- `contracts/` defines semantics and machine-shape truth
- Rust facade and CLI implement those semantics as the normative access surfaces
- Node/Python wrappers converge on those normative surfaces through shared conformance

## 3. Fixed Decisions

The following decisions are fixed for this phase.

1. `contracts/` remains the only semantic baseline for schemas, policies, semantics, fixture meaning, and `contract-json` shape.
2. `Phase 4` does not redesign `Phase 2` or `Phase 3` semantics.
3. Rust is the first-class low-level implementation surface for this phase.
4. The CLI `normalize`, `build`, `inspect`, and `diff` commands with `--output contract-json` are stable protocol surfaces for cross-language use.
5. Node and Python bindings launch through the CLI in `Phase 4`; they do not define a new native ABI yet.
6. `workspace mode` is the normative runtime path in `Phase 4`.
7. `installed mode` remains an explicit extension point, not a first-release support promise.
8. Shared conformance is a first-class deliverable, not an incidental test byproduct.
9. Bindings must preserve protocol truth before adding ergonomic interpretation.
10. High-level wrappers may reduce boilerplate, but they must not hide contract source, runtime resolution, or failure type.

## 4. Architecture and Ownership Boundaries

### 4.1 Semantic source of truth

`contracts/` remains the only normative source of truth for:

- machine-readable schema and report shapes
- policy and build-context assets
- result, warning, and error semantics
- fixture corpus meaning
- contract-facing version axes

No language surface becomes a replacement semantic authority.

### 4.2 Core semantic implementations

`authoring_core` and `writer_core` remain the core semantic implementation crates.

They own the structured execution model for:

- `normalize`
- `build`
- `inspect`
- `diff`

They implement `contracts/`.
They do not replace `contracts/`.

### 4.3 Rust facade crate

`Phase 4` introduces a top-level Rust facade crate, referred to in this design as `anki_forge`.

This facade is not a third semantic core layer.
Its responsibilities are constrained to two roles:

1. `typed core facade`
   - organize and expose the existing structured capabilities of `authoring_core` and `writer_core`
   - rely mainly on re-export and light composition
   - avoid inventing a parallel primary data model
2. `runtime facade`
   - locate manifest and bundle roots
   - resolve policy and build-context assets
   - support workspace-mode runtime discovery
   - provide file-oriented orchestration helpers around the existing core operations

The facade adds organization and runtime access value.
It does not redefine the core semantic model.

### 4.4 CLI protocol and orchestration layer

`contract_tools` remains an independent CLI protocol layer and gate/orchestration layer.

It may reuse shared runtime-facade logic for:

- manifest resolution
- bundle-root resolution
- asset loading
- policy/build-context lookup
- execution orchestration

However, it must retain its distinct identity as:

- the CLI protocol surface
- the contract bundle governance surface
- the operator and verification entry point

It must not collapse into being only a thin shell around the facade crate.

### 4.5 Node and Python wrapper role

Node and Python bindings are constrained wrapper layers in `Phase 4`.

They do not define new semantics.
They wrap the CLI protocol surface and expose three layers:

1. `raw command layer`
2. `structured contract-json layer`
3. `helper/view layer`

The first two layers are normative access surfaces for conformance.
The helper/view layer is convenience-only.

### 4.6 Baseline relationship across layers

The baseline relationship for this phase is:

- semantic baseline: `contracts/`
- implementation baselines: Rust facade and CLI protocol surface
- convergence targets: Node and Python wrappers

Rust facade and CLI must both converge on `contracts/`.
Node and Python must converge on Rust facade, CLI protocol behavior, and `contracts/`.

## 5. API Shape and Data Flow

### 5.1 Rust public API layers

The Rust public API is divided into two formal surfaces.

`typed core facade` exposes structured operations directly:

- `normalize`
- `build`
- `inspect`
- `diff`

This layer should stay close to the existing `authoring_core` and `writer_core` models and result types.

`runtime facade` exposes runtime-oriented helpers:

- resolve manifest
- resolve bundle root
- resolve default and named policy/build-context assets
- orchestrate file-based execution paths
- expose workspace-mode discovery results

### 5.2 Rust calling styles

The Rust facade must support two valid calling styles.

`pure typed path`

- caller supplies structured inputs directly
- no manifest or runtime discovery is required
- best for tests, advanced embedding, and tight control over execution

`runtime orchestration path`

- caller supplies file paths, bundle selectors, or asset selectors
- facade performs manifest discovery, asset lookup, and file IO orchestration
- best for repository tooling and higher-level future SDK layers

### 5.3 Stable CLI protocol commands

The stable CLI protocol surface in `Phase 4` is:

- `normalize`
- `build`
- `inspect`
- `diff`

For those commands:

- `--output contract-json` is the normative machine surface
- stdout is the carrier for successful `contract-json` payloads
- non-zero exit codes are stable indicators of invocation failure
- stderr is useful diagnostic output but not the primary contract surface

The `human` output mode remains operator-facing and is not a shared-conformance target.

### 5.4 Node and Python API layers

Each wrapper exposes three layers with aligned semantics.

`raw command layer` is the closest surface to the CLI and owns:

- executable or launcher discovery
- command invocation
- preservation of stdout, stderr, and exit status
- exposure of resolved runtime metadata

`structured contract-json layer` is the default high-level entry point and owns:

- invoking the CLI with contract-json output
- parsing successful contract payloads
- returning structured results that stay close to the contract field names and meanings
- classifying invocation failures separately from protocol/parse failures

`helper/view layer` owns convenience-only derived views such as:

- status helpers
- warning counts
- artifact-path helpers
- resolved-mode summaries

It must not introduce new semantic states.

### 5.5 Result mapping rules

The structured wrapper layers must preserve contract semantics with only thin ergonomic interpretation.

Rules:

- preserve contract field names and meanings as closely as practical
- keep raw structured result access available
- allow helper-derived views only as projections of the protocol truth
- do not rename warning, error, or result semantics into language-local business terminology

### 5.6 Runtime resolution semantics

`Phase 4` defines runtime resolution semantics, not a single required concrete struct across all layers.

Rust runtime surfaces should expose information such as:

- resolved mode
- resolved manifest path
- resolved bundle root
- resolved policy and build-context assets

Node/Python wrapper surfaces should additionally expose information such as:

- resolved launcher or executable
- resolved launch prefix
- resolved manifest path
- resolved bundle root
- resolved mode

The semantics are aligned.
The exact object shape may vary by layer and language responsibility.

### 5.7 Command data-flow support

The supported inputs and outputs for the stable path are:

`normalize`

- inputs:
  - structured authoring object
  - authoring JSON file path
  - JSON-able object in wrapper layers
- output:
  - `NormalizationResult`

`build`

- inputs:
  - structured `NormalizedIr`
  - normalized JSON file path
  - JSON-able object in wrapper layers
  - writer-policy selector or explicit policy
  - build-context selector or explicit context
  - artifact root
- output:
  - `PackageBuildResult`

`inspect`

- inputs:
  - staging manifest path
  - `.apkg` path
- output:
  - `InspectReport`

`diff`

- inputs:
  - structured `InspectReport`
  - inspect-report JSON file paths
- output:
  - `DiffReport`

No Phase 4 wrapper is required to introduce a larger workflow object that hides these four distinct steps.

## 6. Conformance, Semantic Alignment, and Runtime Resolution Rules

### 6.1 Shared conformance corpus

The shared conformance suite should reuse the existing `Phase 2` and `Phase 3` contract corpus as much as possible, then add a small binding/runtime corpus for wrapper- and locator-specific cases.

That means the suite should combine:

- semantic corpus from existing fixtures and goldens
- binding/runtime corpus for discovery, invocation failure, protocol failure, and version-probing behavior

### 6.2 Conformance object hierarchy

The primary conformance objects are:

- Rust typed facade
- Rust runtime facade
- CLI contract-json surface
- Node structured contract-json layer
- Python structured contract-json layer

The secondary conformance objects are:

- Node raw command layer
- Python raw command layer

Helper/view layers are not covered by shared field-stability guarantees.

The primary objects must align on result semantics.
The secondary objects must align on runtime metadata truthfulness and invocation-failure behavior.

### 6.3 Result-level states versus invocation failures

Result-level states are part of protocol truth.
Invocation failures are failures to obtain protocol truth.

As a result:

- if the CLI successfully returns a contract-valid result object, wrapper layers must treat it as a structured result even when the result represents failure semantics
- if a `build` result returns `result_status=error`, but it is still a valid contract result delivered successfully, wrappers must return it as a result-level failure rather than as an invocation exception
- Node/Python structured layers must return structured results for states such as `invalid`, `degraded`, and `partial`

Invocation failure only means the CLI did not successfully deliver a contract result.

### 6.4 Failure classes in structured wrapper layers

Wrapper failure must be split into at least two categories.

`runtime invocation failure`

- executable or launcher not found
- runtime locator failure
- manifest or bundle root not found
- CLI returned non-zero exit status
- filesystem or OS-level invocation error

`protocol or parse failure`

- exit `0` but stdout is not valid `contract-json`
- stdout JSON shape is inconsistent with expected contract shape
- required protocol fields are missing
- expected version or protocol metadata cannot be validated

These categories must be surfaced distinctly in Node and Python.

### 6.5 Raw-layer conformance obligations

The raw layers do not participate in stable business-result fields, but they do participate in a minimal conformance surface for call authenticity.

At minimum they must align on:

- resolved runtime metadata
- launcher or argv shape explainability
- faithful propagation of stdout, stderr, and exit status
- invocation-failure classification

### 6.6 Semantic comparison units

For commands that successfully return `contract-json`, shared conformance should compare:

- `kind`
- result-status fields
- diagnostics and change payloads
- fingerprint and ref fields
- version fields
- canonical JSON contract payloads

For discovery-only cases or invocation-failure cases, shared conformance should compare structured failure objects and runtime metadata rather than forcing those cases into a contract-payload shape.

### 6.7 Warning and diagnostic alignment

Warnings and diagnostics must remain visible inside structured results.

Language layers must preserve access to:

- diagnostic level
- diagnostic code
- domain
- path
- selector
- stage
- operation

Derived helper views may summarize this data, but they must not redefine it.

### 6.8 Runtime resolution priority

Runtime resolution should follow this priority order:

1. per-call explicit override
2. client or wrapper explicit configuration
3. workspace-mode auto-discovery
4. installed-mode extension point

`workspace mode` is the normative first-release path.
`installed mode` is only a defined extension point in this phase.

### 6.9 Version source separation

`Phase 4` must keep two kinds of version information separate.

`result-embedded versions`

- `tool_contract_version`
- `observation_model_version`
- other versions carried in a successful contract result

`runtime-detected versions`

- detected executable version
- detected bundle version
- detected manifest path
- detected bundle root

Wrappers must not confuse runtime-detected metadata with result-embedded protocol versions.

## 7. Examples, Docs, Packaging Boundaries, and Exit Criteria

### 7.1 Examples and documentation

Each language must include at least one minimal authoring-flow example that runs:

1. `normalize`
2. `build`
3. `inspect`
4. `diff`

Each example must show:

- resolved runtime information
- minimal input sourcing
- structured result inspection
- at least one diagnostics, warning, or state read

Default onboarding paths are fixed as:

- Rust examples use the facade first
- Node examples use the structured wrapper first
- Python examples use the structured wrapper first

Raw-layer examples may exist, but they are not the primary onboarding path.

### 7.2 Repository layout for future publishability

The repository should organize the new surfaces so they are close to future publishable boundaries even if no external publish happens in `Phase 4`.

Recommended additions:

- a top-level Rust facade crate such as `anki_forge/`
- a Node binding directory such as `bindings/node/`
- a Python binding directory such as `bindings/python/`

Those paths should be treated as future package boundaries in embryonic form.

They should not be hidden under the existing `implementations/rust/` placeholder path, which belongs to earlier-phase deferment context rather than the real multi-language access surface now being designed.

### 7.3 Shared conformance suite as an explicit artifact

The shared conformance suite must exist as a distinct, discoverable, and independently runnable test layer.

That means:

- it has its own directory or clearly marked test layer
- it has its own entry command or runner identity
- it can be executed intentionally rather than only by "run everything"

Its coverage split is:

- primary objects: Rust facade, CLI, Node structured, Python structured
- secondary objects: Node raw and Python raw for runtime metadata and invocation-truth checks
- excluded from field-stability guarantees: helper/view layers

### 7.4 Explicit non-goals

`Phase 4` does not own:

- first-release publishing to `crates.io`
- first-release publishing to `npm`
- first-release publishing to `PyPI`
- polished installed-mode runtime distribution
- Node or Python native ABI bindings
- high-level product authoring helpers
- wrapper surfaces beyond `normalize`, `build`, `inspect`, and `diff`

### 7.5 Phase 4 exit criteria

`Phase 4` is complete when all of the following are true:

- a Rust facade crate exists and is usable inside the repository
- the Rust facade provides both typed-core and runtime-oriented access layers
- the CLI `normalize`, `build`, `inspect`, and `diff` commands with `--output contract-json` are documented as stable protocol surfaces
- Node and Python wrappers are usable inside the repository
- Node and Python wrappers provide raw, structured, and helper/view layers
- Node and Python structured layers cover `normalize`, `build`, `inspect`, and `diff`
- shared conformance exists as a distinct, independently runnable suite and passes
- Rust facade, CLI, Node structured, and Python structured layers align on result semantics
- Node and Python raw layers align on runtime metadata truth, invocation-failure classification, and stdout/stderr/exit-status preservation
- Node and Python structured layers return structured results for result-level states such as `invalid`, `degraded`, and `partial` rather than turning those states into invocation exceptions
- runtime invocation failure and protocol-or-parse failure are classified distinctly and consistently across Node and Python
- runtime locator behavior is explicit and inspectable, including resolved mode, manifest, and bundle root, plus launcher or executable information where applicable
- every supported language surface includes a minimal authoring-flow example that exposes structured diagnostics or status reading
- naming, directory boundaries, and version hooks are compatible with future package publishing even though publishing itself is out of scope for this phase
