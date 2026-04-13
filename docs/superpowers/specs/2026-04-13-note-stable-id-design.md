# Note Stable ID Design

- Date: 2026-04-13
- Status: Approved in brainstorming
- Scope: note-level stable identity resolution for default Rust authoring API
- Related specs:
  - `2026-04-10-rust-north-star-api-design.md`
  - `2026-04-03-phase-2-core-authoring-model-design.md`

## 1. Purpose

This document defines the stable note identity design for `anki_forge::Deck`
notes. The goal is update-friendly note evolution with deterministic identity
that is:

1. explicit-first
2. recipe-driven when explicit id is absent
3. collision-auditable
4. versioned for future migration

The design replaces generated fallback identity behavior for new note insertions
that require stable identity inference.

## 2. Scope and Non-Goals

In scope:

1. note-level stable id resolution pipeline
2. `identity_from_fields` layering and precedence
3. stock notetype default recipes (Basic, Basic reverse family, Cloze, IO)
4. custom notetype requirements
5. collision semantics, diagnostics, and provenance
6. hash payload canonicalization and algorithm versioning

Out of scope:

1. card-level identity policy
2. notetype field/template id migration mechanics
3. advanced `guid(...)` escape hatch implementation details

## 3. Decision Summary

1. Stable id resolution is explicit-first: `stable_id(...)` always wins.
2. If stable id is absent, identity is inferred from recipe components.
3. If inferred components are empty/invalid, resolution fails with error.
4. No fallback to `generated:*` when inference is required.
5. `identity_from_fields` supports notetype-level defaults and note-level
   overrides.
6. Custom notetype must declare `identity_from_fields` (note or notetype
   level). No universal custom default recipe is allowed.
7. Collision handling:
   1. same identity + same canonical payload -> duplicate warning
   2. same identity + different canonical payload -> collision error
8. Stable id format is versioned: `afid:v1:<blake3-hex>`.
9. Identity hash payload must be structured and canonicalized, never
   string-concatenated ad hoc.

## 4. Identity Resolution Pipeline

All note additions must pass a unified resolver:

1. If `note.stable_id` exists and non-empty, use it directly.
2. Else resolve identity recipe source in order:
   1. `note.identity_from_fields`
   2. `notetype.identity_from_fields`
   3. stock recipe for stock notetype families only
3. Build recipe components from note data.
4. Normalize components with canonical rules.
5. Validate component presence and non-emptiness.
6. Build canonical structured identity payload.
7. Hash payload with BLAKE3 and emit `afid:v1:<hex>`.
8. Run in-build collision classification using canonical payload.

If any step fails, note insertion fails.

## 5. Identity Sources and Provenance

Each resolved note identity records provenance:

1. `ExplicitStableId`
2. `InferredFromNoteFields`
3. `InferredFromNotetypeFields`
4. `InferredFromStockRecipe`

Validation/build reports should summarize:

1. identity counts by source
2. duplicate warning count
3. collision error count
4. component-empty and field-resolution error count

## 6. `identity_from_fields` Contract

## 6.1 Placement and precedence

Configuration layers:

1. note-level `identity_from_fields([...])` (override)
2. notetype-level `identity_from_fields([...])` (default)

For stock notetypes only, if neither is configured, stock recipe defaults apply.
For custom notetypes, missing both is an error.

## 6.2 Validation rules

`identity_from_fields` must satisfy:

1. list is non-empty
2. each field exists on target notetype
3. duplicate field names are de-duplicated deterministically
4. normalized selected values are not all empty

Validation failure must produce deterministic error diagnostics.

## 7. Default Recipes by Notetype Family

Identity is note-level, not card-level. Reverse card generation does not create
a separate note identity.

## 7.1 Basic

Recipe id: `basic.core.v1`

Components:

1. `primary_prompt`

`primary_prompt` is semantic role based, not "first field index" by assumption.

Default excluded fields:

1. back/explanation/extra
2. tags
3. deck path
4. style/template presentation fields

Expected stability:

1. back edits do not change identity
2. prompt semantic change changes identity

## 7.2 Basic Reverse / Optional Reverse

Recipe id: `basic_reverse.core.v1`

Components:

1. `primary_prompt`

Default excluded fields:

1. back
2. reverse toggle field
3. reverse template/style details

Rationale: reverse behavior is card-generation policy, not note identity.

## 7.3 Cloze

Recipe id: `cloze.core.v1`

Components:

1. `base_text_skeleton`
2. `deletions[{ord,text,slot}]`

Rules:

1. cloze text must be parsed to a structured form
2. skeleton replaces cloze spans with placeholders
3. deletions include:
   1. `ord` (group index, e.g. c1/c2)
   2. normalized deleted `text`
   3. `slot` position in skeleton sequence
4. hint is excluded by default

Expected stability:

1. hint changes do not change identity
2. changes to ord/text/slot do change identity

## 7.4 Image Occlusion

Recipe id: `io.core.v1`

Components:

1. `image_anchor`
2. `occlusion_mode`
3. `normalized_masks`

`image_anchor` priority:

1. explicit asset stable key (if present)
2. media bytes hash (default fallback anchor)

`normalized_masks` requirements:

1. geometry normalized relative to source image dimensions
2. canonicalized independently of editor insertion order
3. excludes non-semantic editor state (selection, color, temporary UI state)
4. includes semantic shape/type/group/mode attributes where applicable

### 7.4.1 Integer-only hash payload rule

To prevent cross-language floating-point drift, hash payload must contain no
floating-point numbers.

Allowed processing model:

1. internal computation may use float
2. before payload emission, values must be quantized into integers

Quantization:

1. clamp value into `[0.0, 1.0]`
2. `q(v) = round(v * 10000)`
3. encode as unsigned integer coordinates in payload

Examples:

1. rect -> `[x_q, y_q, w_q, h_q]`
2. polygon -> `[[x1_q, y1_q], [x2_q, y2_q], ...]`

Canonical sorting/deduplication must run on quantized integer geometry.

## 7.5 Custom notetype

Custom notetypes must provide `identity_from_fields` at note or notetype level.
Missing configuration is a hard error. There is no inferred universal custom
recipe.

## 8. Canonical Normalization Rules

Shared normalization baseline for textual components:

1. Unicode NFC
2. newline normalization (`\r\n`/`\r` -> `\n`)
3. trim outer whitespace
4. minimal HTML normalization when component is HTML

Defaults must not perform:

1. unconditional lowercasing
2. punctuation stripping
3. aggressive semantic rewrites

## 9. Collision Classification

Within one build/session, if two notes produce the same stable id:

1. if canonical payloads are equal -> emit duplicate warning
2. if canonical payloads differ -> emit collision error and fail insertion/build

This rule avoids blocking true duplicates while preventing identity-overwrite
risk.

## 10. Diagnostics

Recommended diagnostic codes:

Errors:

1. `AFID.CUSTOM_IDENTITY_FIELDS_REQUIRED`
2. `AFID.IDENTITY_FIELD_NOT_FOUND`
3. `AFID.IDENTITY_FIELDS_EMPTY`
4. `AFID.IDENTITY_COMPONENT_EMPTY`
5. `AFID.IDENTITY_COLLISION`
6. `AFID.STABLE_ID_DUPLICATE`

Warnings:

1. `AFID.IDENTITY_DUPLICATE_NOTE`

Diagnostics should include recipe id, source, and note id context to simplify
user remediation.

## 11. Canonical Payload and Hash Versioning

Identity payload must be structured and include version metadata:

```json
{
  "algo_version": 1,
  "recipe_id": "cloze.core.v1",
  "notetype_family": "cloze",
  "components": { "..." : "..." }
}
```

Output id format:

```text
afid:v1:<blake3-hex>
```

Recipe IDs are version-frozen (`*.v1`). Any semantic change to recipe behavior
must introduce a new recipe/algo version, not a silent mutation.

## 12. Migration Strategy

When introducing `v2` identity behavior:

1. require explicit opt-in via config/feature flag
2. provide migration report listing changed note identities and reasons
3. support side-by-side evaluation mode (`v1` vs `v2`) for audit before cutover

## 13. Acceptance Criteria

Design is accepted when implementation can satisfy:

1. no generated fallback for inference-required notes
2. deterministic cross-run ids with unchanged inputs
3. deterministic collision classification
4. custom notetype hard-fail without `identity_from_fields`
5. IO hash payload contains only integers for geometric coordinates
6. provenance visible in validation/build outputs
