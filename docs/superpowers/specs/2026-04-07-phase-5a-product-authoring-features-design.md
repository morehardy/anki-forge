# Anki Forge Phase 5A Product Authoring Features Design

- Date: 2026-04-07
- Status: Approved in brainstorming, written for planning handoff
- Scope: `Phase 5A: Product Authoring Features`
- Parent spec: `2026-03-27-anki-forge-platform-phasing-design.md`
- Related specs:
  - `2026-04-03-phase-2-core-authoring-model-design.md`
  - `2026-04-04-phase-3-anki-compatibility-inspection-writer-design.md`
  - `2026-04-06-phase-4-language-bindings-dx-design.md`

## 1. Purpose

`Phase 5A` introduces the author-facing product layer on top of the existing `Phase 2` and `Phase 3` pipeline.

This phase exists to solve three related problems together:

1. authors need a default workflow centered on first-class note types rather than raw `Authoring IR`
2. template helpers, resource bundling, metadata, and override semantics need a product-layer home instead of being forced into pipeline-facing contracts
3. Rust needs a normative first implementation of product authoring semantics without prematurely freezing a cross-language product API

`Phase 5A` is therefore not a contract redesign phase and not a cross-language SDK alignment phase.
It is an authoring-facing product semantics and lowering phase.

## 2. Delivery Strategy Decision

Three delivery strategies were considered:

1. `Authoring IR overlay`: keep `Phase 5A` as a thin layer of convenience builders on top of the existing authoring model
2. `constrained product-layer compiler`: provide a high-level authoring API backed by an explicit product-layer model that lowers into `Authoring IR`
3. `product platform first`: build a large, highly extensible product layer with broad helper and asset platform semantics from the start

The chosen strategy is `constrained product-layer compiler`.

Reasons:

- it preserves the `Phase 2` and `Phase 3` pipeline as the stable downstream contract path
- it gives first-class note types, helpers, bundlers, and overrides a coherent semantic home
- it keeps `Authoring IR` pipeline-facing instead of turning it into a mirror of product API ergonomics
- it allows Rust to launch as the normative implementation without requiring `Phase 5A` to freeze three-language product APIs on day one
- it leaves room for future cross-language portability by defining product semantics and lowering rules explicitly instead of hiding them inside Rust-only ergonomics

The resulting layering is:

- `contracts/` remains the only normative source of truth for pipeline-facing contracts
- `product layer` becomes the author-facing semantic layer for `Phase 5A`
- `lowering` connects product semantics to `Authoring IR`
- `Authoring IR` remains the stable entry point into `Phase 2` normalization and `Phase 3` build/inspect/diff flows

## 3. Fixed Decisions

The following decisions are fixed for this phase.

1. `Phase 5A` introduces a distinct `product layer`; it is not only a set of convenience wrappers over `AuthoringDocument`.
2. `product layer` is `authoring-facing`; `Authoring IR` remains `pipeline-facing`.
3. The normative direction is `product layer -> Authoring IR`; `Authoring IR` does not define product API shape in reverse.
4. `product layer` is a documented and tested semantic layer, but it is not promoted into `contracts/` as a new public contract source in `Phase 5A`.
5. The normative status of `product layer` comes from this design, dedicated tests, and explicit lowering rules, not from `contracts/`.
6. Rust is the first implementation surface for `Phase 5A`.
7. Rust may define the most natural first API shape, but product semantics must not depend on Rust-only hidden behavior or type-system tricks.
8. `Basic`, `Cloze`, and `ImageOcclusion` are the default authoring path in `Block 1`.
9. generic/custom notetype support exists only as a light escape hatch in `Block 1`.
10. `Phase 5A` may extend `Authoring IR` and `contracts/` only when product semantics cannot be stably expressed otherwise.
11. `template helper system` is a first-class product-layer declaration system, not a general-purpose template language.
12. `bundler`, `field metadata`, and `browser/deck override` are formal product declarations and must flow through the same lowering boundary rather than through ad hoc side paths.

## 4. Architecture and Ownership Boundaries

### 4.1 Product layer status and semantic source

`product layer` is the formal author-facing semantic layer for `Phase 5A`.

It is not a new `contracts/` authority.
Its normative force comes from:

- the `Phase 5A` design document
- dedicated semantic and lowering tests
- explicit lowering rules and diagnostics

This distinction matters because `Phase 5A` must define real product semantics without collapsing repository governance away from `contracts/`.

### 4.2 One-way lowering boundary

The main boundary of `Phase 5A` is one-way lowering from `product layer` into `Authoring IR`.

This boundary is intentionally one-way:

- `product layer` is the source of authoring-facing semantics
- `Authoring IR` is the compile target for pipeline compatibility
- product API design must not be constrained into a raw `Authoring IR` mirror simply because the current pipeline is shaped a certain way

`Authoring IR` remains the target of lowering.
It does not become the design authority for product-layer API.

### 4.3 Relationship to existing phases

`Phase 5A` depends on earlier phases and must preserve their boundaries.

- `Phase 2` remains responsible for normalization semantics and validation of pipeline-facing authoring input
- `Phase 3` remains responsible for writer/build/inspect/diff correctness
- `Phase 4` remains responsible for cross-language access to stable pipeline surfaces

`Phase 5A` sits above those phases and compiles into them.
It does not replace them.

### 4.4 Rust-first, portability-constrained implementation

Rust is the normative first implementation surface for `Phase 5A`.

However, `Phase 5A` must be designed so that future Node/Python product surfaces can port the semantics without relying on Rust-only behavior.

This means:

- Rust can expose natural builders and DSL affordances
- the core product semantics must also be representable as data-driven cases, lowering fixtures, and diagnostics expectations
- future portability targets should be able to implement the semantics and lowering rules even if their public API shape differs from Rust

### 4.5 Phase 5A block boundaries

`Phase 5A` is an umbrella phase with three ordered blocks:

1. `Block 1: High-level Authoring API + Basic/Cloze/ImageOcclusion`
2. `Block 2: Template Helper System`
3. `Block 3: Media/Fonts Bundler + Field Metadata + Browser/Deck Override`

The order is part of the design, not a loose suggestion.
`Block 1` establishes the product-layer spine.
`Block 2` and `Block 3` attach to that spine through the same lowering boundary.

## 5. Block 1: High-level Authoring API and First-Class Note Types

### 5.1 Authoring root versus compilation context

`ProductDocument` is the authoring root.
It exists to express author intent, not to accumulate every compilation detail.

`ProductDocument` owns:

- document-level authoring semantics
- note type declarations
- note collections
- later `helper`, `bundler`, `metadata`, and `override` declarations

Lowering should create a distinct compilation context rather than writing all compilation state back onto the document object itself.

That compilation context owns:

- lowering mappings
- compilation decisions
- diagnostics aggregation
- links between product declarations and produced `Authoring IR`

This separation keeps the model clean:

- `ProductDocument` answers "what is the author trying to express?"
- compilation context answers "how was that expression turned into stable pipeline input?"

### 5.2 Product note type model

`ProductNoteType` in `Block 1` is based on a closed variant set, not an open inheritance hierarchy.

The primary variants are:

- `Basic`
- `Cloze`
- `ImageOcclusion`

Each variant owns a restricted configuration surface.
`Phase 5A` should not start with a wide generic note-type abstraction that immediately becomes the dominant product story.

This closed-variant design keeps:

- semantics explicit
- diagnostics clearer
- tests simpler
- future cross-language portability more realistic

### 5.3 Default authoring path

The default user journey in `Block 1` is:

1. create a `ProductDocument`
2. declare or register first-class note types
3. add notes through note-type-aware builder or DSL entry points
4. run explicit lowering
5. pass produced `Authoring IR` into the existing `Phase 2/3` pipeline

The primary authoring path therefore starts from product concepts rather than from hand-building `AuthoringDocument`.

### 5.4 First-class note type customization rules

`Basic`, `Cloze`, and `ImageOcclusion` are first-class and `constrained-customizable`.

Allowed customization is limited to presentation and bounded configuration, such as:

- display names
- author-facing field aliases or labels
- template-fragment entry points
- CSS or styling entry points
- small note-type-specific options

One rule is higher priority than the individual examples:

`Phase 5A` allows customization of presentation and limited configuration, but it does not allow a first-class note type to change semantic category.

That means:

- `Basic` must remain semantically basic
- `Cloze` must remain cloze-driven
- `ImageOcclusion` must retain the core image-occlusion behavior model

The point of first-class note types is stable product semantics, not a generic builder with more defaults.

### 5.5 Generic/custom note types

generic/custom note types exist in `Block 1` only as a light escape hatch.

They are not:

- the default authoring path
- the primary teaching path
- an equally mature product surface compared to `Basic/Cloze/ImageOcclusion`

They exist mainly for:

- advanced notetype experiments
- coverage gaps before future product support lands
- transition scenarios where first-class coverage is not enough yet

### 5.6 Lowering plan as boundary evidence

`LoweringPlan` is not another long-lived semantic layer.
It is the reviewable evidence layer for the `product layer -> Authoring IR` boundary.

Its responsibilities include recording:

- how note type variants lower into `Authoring IR`
- how fields, template decisions, defaults, and restricted configuration are mapped
- which declarations produced which authoring structures
- what diagnostics were emitted during product validation and lowering

`LoweringPlan` exists so `Phase 5A` lowering is inspectable and explainable.
It does not replace `Authoring IR`.

### 5.7 Block 1 diagnostics

Diagnostics in `Block 1` are explicitly split into two layers.

`product-layer diagnostics` cover author intent problems, such as:

- invalid first-class note type usage
- illegal customization
- product-semantic conflicts

`lowering diagnostics` cover compile-boundary problems, such as:

- inability to express a product semantic stably in the current pipeline
- ambiguous mappings
- unacceptable information loss during lowering

This split prevents every failure from collapsing into a generic "compile failed" result.

## 6. Block 1 Lowering Rules and Contract Extension Policy

### 6.1 Lowering principles

The core lowering rules for `Block 1` are:

- `single direction`: product semantics lower into `Authoring IR`; there is no reverse authority
- `semantic preservation`: first-class semantics must survive lowering in a form that can still be validated and built correctly
- `explicit degradation`: any loss of product-layer detail must happen through written rules, not accidental implementation behavior
- `portable meaning`: lowering rules must describe semantics, not Rust-specific tricks

### 6.2 Preferred lowering strategy

`Block 1` should lower into the existing `Authoring IR` model wherever that model already expresses the required pipeline semantics cleanly.

In particular, the first implementation should prefer lowering to existing note kind support for:

- `basic`
- `cloze`
- `image_occlusion`

This reflects the current repository state:
the downstream pipeline already recognizes those note-type categories.

`Phase 5A` therefore starts by improving author-facing ergonomics and semantic clarity, not by rebuilding the whole pipeline contract.

### 6.3 What Authoring IR does and does not keep

In `Block 1`, `Authoring IR` is the stable pipeline target.
It does not need to preserve every author-facing convenience detail.

Product-layer information may therefore end up in one of three places:

- encoded directly in produced `Authoring IR`
- compiled into template, field, or style results that `Authoring IR` already carries
- retained only as lowering evidence or diagnostics rather than as long-lived contract fields

This prevents `Authoring IR` from becoming both product API and pipeline contract at once.

### 6.4 Contract extension gate

`Phase 5A` may extend `Authoring IR` and `contracts/` only when all of the following are true:

1. the product semantic cannot be expressed clearly and stably in the current contract shape
2. not extending the contract would cause real semantic loss, ambiguity, or unverifiability
3. the extension still serves the downstream pipeline rather than mirroring product API convenience directly
4. the extension can be specified cleanly in schema, semantics docs, fixtures, and diagnostics
5. the extension does not redefine the responsibilities of `Phase 2` or `Phase 3`

This allows constrained evolution without turning `contracts/` into a duplicate of the product layer.

### 6.5 Non-goals for Block 1

`Block 1` explicitly does not try to:

- create one-to-one `Authoring IR` fields for every product-layer convenience
- promote generic/custom note types to the main product story
- preserve every builder default as a long-lived contract field
- encode Rust API feel directly into contracts

## 7. Block 2: Template Helper System

### 7.1 Helper system role

The `template helper system` is a product-layer semantic helper system.
It is not:

- a general-purpose template language
- an unrestricted macro engine
- a scripting runtime

Authors declare intent to use documented helper capabilities.
They do not author arbitrary executable helper logic.

### 7.2 Closed helper families with restricted declarations

`Block 2` should begin with closed, documented helper families plus limited configuration surfaces.

Each helper should define:

- valid parameters
- valid scope
- compatible note types
- deterministic lowering behavior

The author declares "use this helper with these options".
The author does not define new helper implementations in `Phase 5A`.

### 7.3 Helper scope model

Helper declarations should be structured product-layer declarations, not magic embedded only inside template strings.

The minimum scope model is:

- `document-level helper defaults`
- `note-type-level helper declarations`

Template-slot-level usage may exist where necessary, but the scope system should remain shallow in the first release.

### 7.4 Relationship to first-class note types

Helpers extend first-class note types.
They do not replace them or erase semantic category boundaries.

For example:

- a `Cloze` helper may refine cloze-oriented presentation
- it must not turn `Cloze` into a fully generic, non-cloze note type in disguise

This keeps helper power subordinate to note-type semantics.

### 7.5 Helper lowering

Helper declarations lower through the same explicit compilation boundary used in `Block 1`.

Outputs may include:

- template-fragment expansions
- style additions
- lowering evidence recorded in `LoweringPlan`

`Phase 5A` does not introduce a writer-side helper runtime.
Helpers lower into pipeline-consumable results before `Authoring IR` enters downstream phases.

### 7.6 Helper diagnostics

Helper-related diagnostics also split into product and lowering layers.

`product-helper diagnostics` include problems such as:

- helper not valid for the selected note type
- invalid helper parameters
- conflicting helper declarations

`helper-lowering diagnostics` include problems such as:

- helper semantics cannot be lowered stably to the current pipeline
- helper expansion conflicts with other produced template or style output

### 7.7 Non-goals for Block 2

`Block 2` does not try to:

- create a fully general template language
- support arbitrary user-defined helper implementations
- add a runtime helper interpreter to the writer pipeline
- provide a universal escape hatch around first-class note-type semantics

## 8. Block 3: Bundler, Field Metadata, and Browser/Deck Override

### 8.1 Shared phase boundary, distinct semantic categories

`Block 3` groups three related capabilities because all three are author declarations that must lower into pipeline-facing representations.

They are still distinct semantic categories:

- `bundler` is primarily resource declaration and packaging intent
- `field metadata` and `browser/deck override` are primarily display, scope, and authoring-organization semantics

They should therefore share a lowering framework without being collapsed into one undifferentiated object model.

### 8.2 Bundler role

The `media/fonts bundler` is a declarative asset-bundling intent layer.
It is not:

- a remote-fetch platform
- a general build pipeline
- a broad asset compiler with arbitrary transforms

The author declares:

- which assets are required
- where they come from
- what authoring feature or intent they support
- how they should participate in deck/package production

### 8.3 Bundler resource model

The first release should support declaration-oriented asset references such as:

- local file or directory sources
- in-memory provided asset content
- supported resource categories such as images and fonts

It should not support:

- remote URL fetching
- general content cache infrastructure
- large transformation pipelines
- platform-scale asset fingerprint or rewrite systems

### 8.4 Bundler lowering

Bundler output must lower into stable writer-consumable artifact inputs rather than remain as a live runtime abstraction.

That lowering may produce:

- media entries or references needed by downstream build inputs
- deterministic file naming or placement decisions
- evidence in `LoweringPlan` linking product-level asset declarations to produced outputs

### 8.5 Field metadata role

`field metadata` is a formal product-layer semantic feature, not a loose bucket of optional flags.

It expresses author-facing information such as:

- field labels and display descriptions
- field role hints used by note types or helpers
- browser-facing presentation hints that can be lowered stably

Only metadata that can be lowered clearly and stably should ship in the first release.

### 8.6 Browser and deck overrides

`browser override` and `deck override` are scope-aware product declarations.

They express things like:

- browser-facing presentation or field emphasis behavior
- controlled deck placement or deck selection overrides beyond document defaults

They must be governed by explicit scope and conflict rules rather than silent precedence accidents.

### 8.7 Scope model and conflict rules

The preferred scope ladder for `Block 3` is:

- `document-level defaults`
- `note-type-level declarations`
- `note-level overrides` only where the semantics remain clear and stable

Conflict handling rules should include:

- narrower scope wins over broader scope
- narrower scope still may not violate semantic category or stable lowering guarantees
- conflicts should emit structured diagnostics instead of silently overriding in surprising ways

### 8.8 Block 3 diagnostics

`product declaration diagnostics` include issues such as:

- incomplete asset declarations
- invalid metadata scope
- override conflicts
- incompatible asset usage declarations

`lowering diagnostics` include issues such as:

- inability to lower an asset declaration into stable media/build inputs
- inability to preserve requested metadata or override semantics in the current pipeline
- ambiguous merge of multi-scope declarations

### 8.9 Non-goals for Block 3

`Block 3` does not try to:

- create a remote asset acquisition platform
- build a full asset compilation or cache subsystem
- support unlimited override inheritance depth
- add metadata that cannot be expressed stably in downstream contracts

## 9. Testing Strategy

`Phase 5A` verification should be organized into four layers with explicit primary objects.

### 9.1 Product semantics tests

Primary objects:

- `ProductDocument`
- `ProductNoteType`
- `ProductNote`

These tests verify author-facing semantics, including:

- default authoring flows
- first-class note type behavior
- restricted customization
- helper declarations
- bundler declarations
- metadata and override declarations

### 9.2 Lowering boundary tests

Primary objects:

- `LoweringPlan`
- layered diagnostics
- produced `Authoring IR`

These tests verify:

- mappings and default expansion
- conflict handling
- information degradation behavior
- explicit failure, rejection, or controlled downgrade when product semantics cannot be represented stably in the current pipeline

### 9.3 Pipeline compatibility tests

Primary objects:

- lowered `Authoring IR` running through `normalize -> build -> inspect -> diff`

These tests are not meant to re-prove all `Phase 2` and `Phase 3` behavior.
They exist to prove that `Phase 5A` lowering does not break the existing pipeline contract story.

### 9.4 Portability constraint tests

Primary objects:

- data-driven product semantic cases
- lowering fixtures
- diagnostics expectations

These tests verify that important product semantics are not validated only through Rust builder feel.
Key semantics should be expressible as data-driven cases and expected lowering outcomes that future implementations can port.

## 10. Success Criteria

`Phase 5A` succeeds only if all of the following are true:

- the default authoring path is centered on `Basic/Cloze/ImageOcclusion`, not raw `Authoring IR`
- `product layer` is clearly documented, testable, and semantically coherent
- `Authoring IR` remains pipeline-facing rather than turning into a mirror of product API convenience
- helpers, bundlers, metadata, and overrides all attach through the same lowering boundary
- no product feature bypasses the lowering boundary by writing directly to `Authoring IR` as an ad hoc side path
- released product semantics can lower stably into the existing or minimally extended `Phase 2/3` pipeline
- generic/custom note types remain escape hatches rather than the dominant product surface
- Rust is a natural first implementation without becoming the only environment capable of expressing the semantics correctly

## 11. Recommended Implementation Order

The implementation order should follow the design order:

1. `Block 1`
   - establish `ProductDocument`
   - establish the closed first-class note-type set
   - establish the default authoring path
   - establish explicit lowering and `LoweringPlan`
   - establish layered diagnostics
2. `Block 2`
   - attach helper declarations to the existing product model
   - lower helpers through the same compilation boundary
3. `Block 3`
   - attach bundler declarations, field metadata, and browser/deck overrides
   - reuse the same lowering framework and scope rules

This order matters because `Block 2` and `Block 3` should extend a stable product-layer spine rather than compete to define it.

## 12. Planning Implications

The implementation plan should not treat `Phase 5A` as one flat task.

At minimum, planning should distinguish:

- `Phase 5A umbrella infrastructure`
- `Block 1`
- `Block 2`
- `Block 3`

Each planning unit should specify:

- code ownership and module boundaries
- any contract changes required
- fixture and diagnostics strategy
- risks and constraints
- exit evidence for completion

That plan decomposition is necessary to keep the umbrella spec actionable without allowing unrelated product work to sprawl together.
