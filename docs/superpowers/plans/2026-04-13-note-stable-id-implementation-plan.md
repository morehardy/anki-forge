# Note Stable ID Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement deterministic `afid:v1:*` note identity for `Deck` notes using contract-backed recipes, strongly-typed selector APIs, persisted identity snapshots, and collision-safe add-time validation.

**Architecture:** Move note identity semantics out of ad-hoc Rust tests and into the contracts bundle (`contracts/semantics`, `contracts/schema`, `contracts/fixtures`) so the Rust resolver executes a published spec instead of inventing one. Land the resolver in three layers: typed policy/override surface, persisted `ResolvedIdentitySnapshot` plus deserialize-time runtime index rebuild, and stock recipes (`basic.core.v1`, `cloze.core.v2`, `io.core.v2`) with fixture-driven verification. Only after the recipes and round-trip invariants are stable do we replace `generated:*` fallback with default inferred AFIDs.

**Tech Stack:** Rust (`anyhow`, `serde`, `serde_json`, `serde_yaml`, `blake3`, `unicode-normalization`, `imagesize`), existing `anki_forge::deck` facade, contracts bundle fixtures and schemas, `contract_tools` fixture gates, Cargo test runner.

---

## Scope Check

This plan covers one subsystem: deck-layer note stable identity for the existing `Deck` API (`Basic`, `Cloze`, `Image Occlusion`).

This pass includes:

1. published AFID note identity semantics and fixture schema in `contracts/`
2. golden fixture cases for `basic.core.v1`, `cloze.core.v2`, and `io.core.v2`
3. strongly-typed field selectors and atomic note-level override construction
4. persisted `ResolvedIdentitySnapshot` on notes plus deserialize-time runtime index rebuild
5. recipe-specific canonical payload generation with fixed field order and stable wire names
6. `afid:v1` hashing and blocking duplicate/collision behavior at add-time and load-time
7. deck-local audit accessors and AFID diagnostics

This pass excludes:

1. artifact-level `writer_core` / APKG `inspect-report` enrichment with full recipe provenance and canonical payload
2. card-level identity policies
3. custom non-stock note recipes
4. parser-level HTML canonicalization
5. non-rect image occlusion authoring APIs (the spec leaves a tagged-union extension point for future shapes)

## Execution Prerequisite

Run this plan in a dedicated worktree:

```bash
git worktree add ../anki-forge-note-stable-id -b codex/note-stable-id
cd ../anki-forge-note-stable-id
```

## File Structure Map

- Modify: `contracts/manifest.yaml` - register a new `note_identity_fixture_schema` asset.
- Create: `contracts/semantics/note-stable-id.md` - normative AFID note identity semantics, recipe/version rules, and canonical payload contract.
- Create: `contracts/schema/note-identity-fixture.schema.json` - shared schema for language-neutral AFID golden fixtures.
- Modify: `contracts/fixtures/index.yaml` - register note-identity cases in the bundle catalog.
- Create: `contracts/fixtures/note-identity/basic-front-only.case.json` - `basic.core.v1` stock fixture.
- Create: `contracts/fixtures/note-identity/cloze-hint-ignored.case.json` - `cloze.core.v2` fixture.
- Create: `contracts/fixtures/note-identity/cloze-whitespace-significant.case.json` - `cloze.core.v2` whitespace boundary fixture.
- Create: `contracts/fixtures/note-identity/cloze-malformed.case.json` - malformed cloze error fixture.
- Create: `contracts/fixtures/note-identity/io-order-insensitive.case.json` - `io.core.v2` order-stable fixture.
- Create: `contracts/fixtures/note-identity/io-translation-different.case.json` - `io.core.v2` translation-sensitive fixture.
- Modify: `contracts/errors/error-registry.yaml` - add AFID error codes referenced by fixtures and validation.
- Modify: `contract_tools/src/fixtures.rs` - accept `note-identity` fixture category and schema-validate those cases.
- Modify: `contract_tools/tests/fixture_gate_tests.rs` - assert bundled fixture gates continue to pass with note-identity fixtures.
- Modify: `anki_forge/Cargo.toml` - add `blake3`, `unicode-normalization`, and `imagesize`.
- Create: `anki_forge/src/deck/identity.rs` - selector canonicalization, recipe resolvers, canonical payload hashing, cloze parser, IO canonicalizer, and fixture helpers.
- Modify: `anki_forge/src/deck/model.rs` - typed selector/override types, identity policy, `ResolvedIdentitySnapshot`, persisted/runtime deck split, and accessors.
- Modify: `anki_forge/src/deck/builders.rs` - add-time identity assignment, runtime index rebuild, duplicate/collision blocking, and note-level override warnings.
- Modify: `anki_forge/src/deck/media.rs` - attach optional raster image dimensions to registered media.
- Modify: `anki_forge/src/deck/mod.rs` - export new typed selector and snapshot APIs.
- Modify: `anki_forge/src/deck/validation.rs` - add AFID validation codes.
- Create: `anki_forge/tests/deck_identity_contract_tests.rs` - contract fixture loading and recipe expectation tests.
- Create: `anki_forge/tests/deck_identity_roundtrip_tests.rs` - serde round-trip and deserialize-time rebuild tests.
- Modify: `anki_forge/tests/deck_model_tests.rs` - typed selector API surface tests.
- Modify: `anki_forge/tests/deck_validation_tests.rs` - AFID duplicate/collision and warning diagnostics.
- Modify: `README.md` - describe AFID defaults, typed selector APIs, and round-trip guarantees.

## Implementation Notes

- Canonical payload field order must be fixed by a struct, not a `serde_json::Map`: `algo_version`, `recipe_id`, `notetype_family`, `notetype_key`, `components`.
- Identity text normalization is `NFC` plus `\r\n` / `\r` to `\n`. Do not `trim()` identity text.
- Selector order is not semantic. Canonicalize selector lists by enum order and deduplicate before serialization.
- Note-level overrides must be constructed atomically with both `fields` and `reason_code`.
- `ResolvedIdentitySnapshot` is persisted on each note. Deserialize-time rebuild must never re-run the current resolver for inferred notes.
- `cloze.core.v2` replaces the original `cloze.core.v1` plan and explicitly rejects nested clozes with `AFID.CLOZE_NESTED_UNSUPPORTED`. `io.core.v2` replaces the original `io.core.v1` plan. Do not silently change `v1` recipe semantics.
- `io.core.v2` uses source-image pixel space plus stable wire mode values, not `Debug` output and not bounding-box-relative quantization.
- Artifact-level `inspect-report` identity enrichment is intentionally deferred. This plan provides deck-local auditability via persisted snapshots and validation diagnostics.

### Task 1: Publish AFID Contracts And Golden Fixtures

**Files:**
- Modify: `contracts/manifest.yaml`
- Create: `contracts/semantics/note-stable-id.md`
- Create: `contracts/schema/note-identity-fixture.schema.json`
- Modify: `contracts/fixtures/index.yaml`
- Create: `contracts/fixtures/note-identity/basic-front-only.case.json`
- Create: `contracts/fixtures/note-identity/cloze-hint-ignored.case.json`
- Create: `contracts/fixtures/note-identity/cloze-whitespace-significant.case.json`
- Create: `contracts/fixtures/note-identity/cloze-malformed.case.json`
- Create: `contracts/fixtures/note-identity/io-order-insensitive.case.json`
- Create: `contracts/fixtures/note-identity/io-translation-different.case.json`
- Modify: `contracts/errors/error-registry.yaml`
- Modify: `contract_tools/src/fixtures.rs`
- Modify: `contract_tools/tests/fixture_gate_tests.rs`
- Test: `anki_forge/tests/deck_identity_contract_tests.rs`

- [ ] **Step 1: Write failing contract-facing tests**

```rust
// anki_forge/tests/deck_identity_contract_tests.rs
use serde_json::Value;
use serde_yaml::Value as YamlValue;
use std::{fs, path::PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

#[test]
fn bundled_catalog_declares_note_identity_cases() {
    let raw = fs::read_to_string(repo_root().join("contracts/fixtures/index.yaml"))
        .expect("read fixture catalog");
    let catalog: YamlValue = serde_yaml::from_str(&raw).expect("parse fixture catalog");
    let cases = catalog["cases"].as_sequence().expect("catalog cases");

    for case_id in [
        "note-identity-basic-front-only",
        "note-identity-cloze-hint-ignored",
        "note-identity-io-order-insensitive",
    ] {
        assert!(
            cases.iter().any(|case| case["id"].as_str() == Some(case_id)),
            "expected bundled catalog to declare note identity case {case_id}"
        );
    }
}

#[test]
fn note_identity_fixtures_exist_and_parse() {
    for rel in [
        "contracts/fixtures/note-identity/basic-front-only.case.json",
        "contracts/fixtures/note-identity/cloze-hint-ignored.case.json",
        "contracts/fixtures/note-identity/io-order-insensitive.case.json",
    ] {
        let raw = fs::read_to_string(repo_root().join(rel))
            .unwrap_or_else(|err| panic!("missing fixture {rel}: {err}"));
        let _: Value = serde_json::from_str(&raw)
            .unwrap_or_else(|err| panic!("invalid JSON fixture {rel}: {err}"));
    }
}
```

- [ ] **Step 2: Run the failing contract tests**

Run: `cargo test -p anki_forge --test deck_identity_contract_tests -v`  
Expected: FAIL because the new note-identity fixtures and catalog entries do not exist yet.

Run: `cargo test -p contract_tools --test fixture_gate_tests fixture_gates_accept_the_bundled_catalog_and_fixtures -v`  
Expected: PASS before fixture registration, then FAIL once the new unsupported `note-identity` category is added to the catalog.

- [ ] **Step 3: Add the contract spec, fixture schema, bundled cases, and fixture-gate support**

```yaml
# contracts/manifest.yaml (asset excerpt)
assets:
  fixture_catalog: fixtures/index.yaml
  error_registry: errors/error-registry.yaml
  note_identity_fixture_schema: schema/note-identity-fixture.schema.json
```

```markdown
<!-- contracts/semantics/note-stable-id.md -->
---
asset_refs:
  - schema/note-identity-fixture.schema.json
  - fixtures/index.yaml
  - errors/error-registry.yaml
---

# Note Stable ID Semantics

`afid:v1:*` note identity is computed from a canonical payload with fixed field order:

1. `algo_version`
2. `recipe_id`
3. `notetype_family`
4. `notetype_key`
5. `components`

All recipe text normalization uses Unicode NFC and newline normalization only.
Identity normalization must not trim leading or trailing whitespace.

Recipe ids are stable compatibility boundaries:

1. `basic.core.v1`
2. `cloze.core.v2`
3. `io.core.v2`

Changing the meaning of any recipe input, normalization rule, canonical field, or error behavior requires a new `recipe_id`.

`ResolvedIdentitySnapshot` persists the resolver output used at add-time:

1. `stable_id`
2. `recipe_id` when inferred
3. `provenance`
4. `canonical_payload` when inferred
5. `used_override`

Deserialize-time rebuild must use the persisted snapshot and must not re-resolve inferred identity under the current code.
```

```json
// contracts/schema/note-identity-fixture.schema.json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["recipe_id", "note_kind", "input", "expected"],
  "additionalProperties": false,
  "properties": {
    "recipe_id": { "type": "string", "minLength": 1 },
    "note_kind": { "enum": ["basic", "cloze", "image_occlusion"] },
    "input": { "type": "object" },
    "expected": {
      "oneOf": [
        {
          "type": "object",
          "required": ["canonical_payload", "stable_id", "provenance"],
          "additionalProperties": false,
          "properties": {
            "canonical_payload": { "type": "string", "minLength": 1 },
            "stable_id": { "type": "string", "minLength": 1 },
            "provenance": { "type": "string", "minLength": 1 }
          }
        },
        {
          "type": "object",
          "required": ["error_code"],
          "additionalProperties": false,
          "properties": {
            "error_code": { "type": "string", "minLength": 1 }
          }
        }
      ]
    }
  }
}
```

```json
// contracts/fixtures/note-identity/basic-front-only.case.json (shape example)
{
  "recipe_id": "basic.core.v1",
  "note_kind": "basic",
  "input": {
    "front": "hola",
    "back": "hello"
  },
  "expected": {
    "canonical_payload": "{\"algo_version\":1,\"recipe_id\":\"basic.core.v1\",\"notetype_family\":\"stock\",\"notetype_key\":\"basic\",\"components\":{\"selected_fields\":[{\"name\":\"front\",\"value\":\"hola\"}]}}",
    "stable_id": "afid:v1:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    "provenance": "stock_recipe"
  }
}
```

```yaml
# contracts/fixtures/index.yaml (entry excerpt)
cases:
  - id: note-identity-basic-front-only
    category: note-identity
    input: fixtures/note-identity/basic-front-only.case.json
  - id: note-identity-cloze-hint-ignored
    category: note-identity
    input: fixtures/note-identity/cloze-hint-ignored.case.json
  - id: note-identity-cloze-whitespace-significant
    category: note-identity
    input: fixtures/note-identity/cloze-whitespace-significant.case.json
  - id: note-identity-cloze-malformed
    category: note-identity
    input: fixtures/note-identity/cloze-malformed.case.json
  - id: note-identity-io-order-insensitive
    category: note-identity
    input: fixtures/note-identity/io-order-insensitive.case.json
  - id: note-identity-io-translation-different
    category: note-identity
    input: fixtures/note-identity/io-translation-different.case.json
```

```yaml
# contracts/errors/error-registry.yaml (entry excerpt)
codes:
  - id: AFID.IDENTITY_FIELDS_EMPTY
    status: active
    summary: typed identity selector list is empty
  - id: AFID.IDENTITY_FIELD_NOT_FOUND
    status: active
    summary: requested identity field does not exist for the note kind
  - id: AFID.NOTE_LEVEL_IDENTITY_OVERRIDE_REASON_REQUIRED
    status: active
    summary: note-level identity override requires a non-empty reason code
  - id: AFID.CLOZE_MALFORMED
    status: active
    summary: cloze text is malformed for identity parsing
  - id: AFID.CLOZE_ORD_INVALID
    status: active
    summary: cloze ordinal must be a non-zero positive integer
  - id: AFID.CLOZE_NESTED_UNSUPPORTED
    status: active
    summary: nested cloze syntax is not supported by the current recipe
  - id: AFID.IO_IMAGE_DIMENSIONS_MISSING
    status: active
    summary: image occlusion identity requires source image dimensions
```

```rust
// contract_tools/src/fixtures.rs (match arm excerpt)
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct NoteIdentityCase {
    recipe_id: String,
    note_kind: String,
    input: Value,
    expected: Value,
}

// inside run_fixture_gates()
let note_identity_fixture_schema = load_schema(&resolve_asset_path(
    &manifest,
    "note_identity_fixture_schema",
)?)?;

// inside the category match
"note-identity" => {
    let input_path = resolve_contract_relative_path(&catalog_path, &case.input)?;
    let value: Value = load_json_model(&input_path)?;
    validate_value(&note_identity_fixture_schema, &value)
        .with_context(|| format!("note-identity fixture failed schema validation: {}", case.id))?;
    let _: NoteIdentityCase = serde_json::from_value(value)
        .with_context(|| format!("note-identity fixture failed model decode: {}", case.id))?;
}
```

- [ ] **Step 4: Re-run contract and fixture-gate tests**

Run: `cargo test -p anki_forge --test deck_identity_contract_tests -v`  
Expected: PASS for bundled catalog and fixture existence tests.

Run: `cargo test -p contract_tools --test fixture_gate_tests -v`  
Expected: PASS with the new `note-identity` catalog entries and schema validation path.

- [ ] **Step 5: Commit**

```bash
git add contracts/manifest.yaml contracts/semantics/note-stable-id.md contracts/schema/note-identity-fixture.schema.json contracts/fixtures/index.yaml contracts/fixtures/note-identity contracts/errors/error-registry.yaml contract_tools/src/fixtures.rs contract_tools/tests/fixture_gate_tests.rs anki_forge/tests/deck_identity_contract_tests.rs
git commit -m "feat: publish afid note identity contracts and fixtures"
```

### Task 2: Add Strongly-Typed Selector And Override APIs

**Files:**
- Modify: `anki_forge/src/deck/model.rs`
- Modify: `anki_forge/src/deck/builders.rs`
- Modify: `anki_forge/src/deck/mod.rs`
- Test: `anki_forge/tests/deck_model_tests.rs`

- [ ] **Step 1: Write failing API tests for typed selectors and atomic overrides**

```rust
// anki_forge/tests/deck_model_tests.rs
use anki_forge::{
    BasicIdentityField, BasicIdentityOverride, BasicIdentitySelection, BasicNote, Deck,
};
use serde_json::json;

#[test]
fn deck_builder_stores_canonicalized_typed_identity_fields() {
    let deck = Deck::builder("Spanish")
        .basic_identity(
            BasicIdentitySelection::new([
                BasicIdentityField::Back,
                BasicIdentityField::Front,
                BasicIdentityField::Back,
            ])
            .expect("selection"),
        )
        .build();

    let policy = deck.identity_policy();
    assert_eq!(
        policy.basic.as_ref().expect("basic policy").as_slice(),
        &[BasicIdentityField::Front, BasicIdentityField::Back]
    );
}

#[test]
fn note_level_identity_override_is_constructed_atomically() {
    let override_cfg = BasicIdentityOverride::new(
        [BasicIdentityField::Front],
        "homonym-disambiguation",
    )
    .expect("override");

    let note = BasicNote::new("hola", "hello").identity_override(override_cfg.clone());
    assert_eq!(note.identity_override_config(), Some(&override_cfg));
}

#[test]
fn typed_identity_override_uses_stable_wire_names() {
    let override_cfg = BasicIdentityOverride::new(
        [BasicIdentityField::Back, BasicIdentityField::Front],
        "sense-disambiguation",
    )
    .expect("override");

    let json_value = serde_json::to_value(&override_cfg).expect("serialize override");
    assert_eq!(
        json_value,
        json!({
            "fields": ["front", "back"],
            "reason_code": "sense-disambiguation"
        })
    );
}
```

- [ ] **Step 2: Run the model tests and confirm failure**

Run: `cargo test -p anki_forge --test deck_model_tests -v`  
Expected: FAIL with missing types and methods (`BasicIdentityField`, `BasicIdentitySelection`, `BasicIdentityOverride`, `DeckBuilder::basic_identity`, `BasicNote::identity_override`).

- [ ] **Step 3: Add typed field enums, canonicalized selections, and atomic override types**

```rust
// anki_forge/src/deck/model.rs (new typed selector surface)
use serde::{Deserialize, Serialize};

fn canonicalize_fields<F, I>(fields: I) -> anyhow::Result<Vec<F>>
where
    F: Copy + Ord,
    I: IntoIterator<Item = F>,
{
    let mut values: Vec<F> = fields.into_iter().collect();
    values.sort();
    values.dedup();
    anyhow::ensure!(!values.is_empty(), "AFID.IDENTITY_FIELDS_EMPTY");
    Ok(values)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BasicIdentityField {
    Front,
    Back,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClozeIdentityField {
    Text,
    Extra,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IoIdentityField {
    Image,
    Mode,
    Rects,
    Header,
    BackExtra,
    Comments,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentitySelection<F> {
    fields: Vec<F>,
}

impl<F> IdentitySelection<F>
where
    F: Copy + Ord,
{
    pub fn new<I>(fields: I) -> anyhow::Result<Self>
    where
        I: IntoIterator<Item = F>,
    {
        Ok(Self {
            fields: canonicalize_fields(fields)?,
        })
    }

    pub fn as_slice(&self) -> &[F] {
        &self.fields
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityOverride<F> {
    fields: Vec<F>,
    reason_code: String,
}

impl<F> IdentityOverride<F>
where
    F: Copy + Ord,
{
    pub fn new<I>(fields: I, reason_code: impl Into<String>) -> anyhow::Result<Self>
    where
        I: IntoIterator<Item = F>,
    {
        let reason_code = reason_code.into().trim().to_string();
        anyhow::ensure!(
            !reason_code.is_empty(),
            "AFID.NOTE_LEVEL_IDENTITY_OVERRIDE_REASON_REQUIRED"
        );
        Ok(Self {
            fields: canonicalize_fields(fields)?,
            reason_code,
        })
    }

    pub fn fields(&self) -> &[F] {
        &self.fields
    }

    pub fn reason_code(&self) -> &str {
        &self.reason_code
    }
}

pub type BasicIdentitySelection = IdentitySelection<BasicIdentityField>;
pub type ClozeIdentitySelection = IdentitySelection<ClozeIdentityField>;
pub type IoIdentitySelection = IdentitySelection<IoIdentityField>;

pub type BasicIdentityOverride = IdentityOverride<BasicIdentityField>;
pub type ClozeIdentityOverride = IdentityOverride<ClozeIdentityField>;
pub type IoIdentityOverride = IdentityOverride<IoIdentityField>;
```

```rust
// anki_forge/src/deck/model.rs (policy + note surface excerpt)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DeckIdentityPolicy {
    pub basic: Option<BasicIdentitySelection>,
    pub cloze: Option<ClozeIdentitySelection>,
    pub image_occlusion: Option<IoIdentitySelection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BasicNote {
    pub(crate) id: String,
    pub(crate) stable_id: Option<String>,
    pub(crate) front: String,
    pub(crate) back: String,
    pub(crate) tags: Vec<String>,
    pub(crate) generated: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) identity_override: Option<BasicIdentityOverride>,
}

impl BasicNote {
    pub fn identity_override(mut self, override_cfg: BasicIdentityOverride) -> Self {
        self.identity_override = Some(override_cfg);
        self
    }

    pub fn identity_override_config(&self) -> Option<&BasicIdentityOverride> {
        self.identity_override.as_ref()
    }
}
```

```rust
// anki_forge/src/deck/builders.rs (DeckBuilder excerpt)
pub struct DeckBuilder {
    name: String,
    stable_id: Option<String>,
    identity_policy: crate::deck::model::DeckIdentityPolicy,
}

impl DeckBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            stable_id: None,
            identity_policy: crate::deck::model::DeckIdentityPolicy::default(),
        }
    }

    pub fn build(self) -> Deck {
        Deck {
            name: self.name,
            stable_id: self.stable_id,
            notes: Vec::new(),
            next_generated_note_id: 1,
            media: Default::default(),
            identity_policy: self.identity_policy,
            used_note_ids: Default::default(),
            identity_snapshot_by_id: Default::default(),
        }
    }
}

impl DeckBuilder {
    pub fn basic_identity(mut self, selection: crate::deck::model::BasicIdentitySelection) -> Self {
        self.identity_policy.basic = Some(selection);
        self
    }

    pub fn cloze_identity(mut self, selection: crate::deck::model::ClozeIdentitySelection) -> Self {
        self.identity_policy.cloze = Some(selection);
        self
    }

    pub fn image_occlusion_identity(
        mut self,
        selection: crate::deck::model::IoIdentitySelection,
    ) -> Self {
        self.identity_policy.image_occlusion = Some(selection);
        self
    }
}

impl Deck {
    pub fn identity_policy(&self) -> &crate::deck::model::DeckIdentityPolicy {
        &self.identity_policy
    }
}
```

- [ ] **Step 4: Re-run the model tests**

Run: `cargo test -p anki_forge --test deck_model_tests -v`  
Expected: PASS for typed selector ordering, atomic override construction, and stable wire names.

- [ ] **Step 5: Commit**

```bash
git add anki_forge/src/deck/model.rs anki_forge/src/deck/builders.rs anki_forge/src/deck/mod.rs anki_forge/tests/deck_model_tests.rs
git commit -m "feat: add typed afid selector and override APIs"
```

### Task 3: Persist Resolved Identity Snapshots And Rebuild Runtime Indexes On Deserialize

**Files:**
- Modify: `anki_forge/src/deck/model.rs`
- Modify: `anki_forge/src/deck/builders.rs`
- Modify: `anki_forge/src/deck/mod.rs`
- Test: `anki_forge/tests/deck_identity_roundtrip_tests.rs`

- [ ] **Step 1: Write failing round-trip and load-validation tests**

```rust
// anki_forge/tests/deck_identity_roundtrip_tests.rs
use anki_forge::{BasicNote, Deck};
use serde_json::json;

#[test]
fn roundtrip_preserves_resolved_identity_snapshot_and_duplicate_detection() {
    let mut deck = Deck::new("Spanish");
    deck.add(BasicNote::new("hola", "hello"))
        .expect("add inferred note");

    let raw = serde_json::to_string(&deck).expect("serialize deck");
    let mut roundtripped: Deck = serde_json::from_str(&raw).expect("deserialize deck");

    let err = roundtripped
        .add(BasicNote::new("hola", "hello"))
        .expect_err("duplicate inferred identity should still be blocked");
    assert!(err.to_string().contains("AFID.IDENTITY_DUPLICATE_PAYLOAD"));
}

#[test]
fn inferred_afid_without_snapshot_fails_to_deserialize() {
    let err = serde_json::from_value::<Deck>(json!({
        "name": "Spanish",
        "stable_id": null,
        "notes": [
            {
                "Basic": {
                    "id": "afid:v1:deadbeef",
                    "stable_id": "afid:v1:deadbeef",
                    "front": "hola",
                    "back": "hello",
                    "tags": [],
                    "generated": false
                }
            }
        ],
        "next_generated_note_id": 1,
        "media": {},
        "identity_policy": {}
    }))
    .expect_err("missing inferred snapshot must fail");

    assert!(err.to_string().contains("AFID.IDENTITY_SNAPSHOT_MISSING"));
}
```

- [ ] **Step 2: Run the round-trip tests and confirm failure**

Run: `cargo test -p anki_forge --test deck_identity_roundtrip_tests -v`  
Expected: FAIL because notes do not persist resolved identity snapshots and deserialize-time rebuild does not exist.

- [ ] **Step 3: Add persisted deck shape, resolved identity snapshot, and deserialize-time rebuild**

```rust
// anki_forge/src/deck/model.rs (snapshot + persisted deck excerpt)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdentityProvenance {
    ExplicitStableId,
    InferredFromNoteFields,
    InferredFromNotetypeFields,
    InferredFromStockRecipe,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedIdentitySnapshot {
    pub stable_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recipe_id: Option<String>,
    pub provenance: IdentityProvenance,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_payload: Option<String>,
    #[serde(default)]
    pub used_override: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct PersistedDeck {
    name: String,
    stable_id: Option<String>,
    notes: Vec<DeckNote>,
    next_generated_note_id: u64,
    media: BTreeMap<String, RegisteredMedia>,
    #[serde(default)]
    identity_policy: DeckIdentityPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "PersistedDeck", into = "PersistedDeck")]
pub struct Deck {
    pub(crate) name: String,
    pub(crate) stable_id: Option<String>,
    pub(crate) notes: Vec<DeckNote>,
    pub(crate) next_generated_note_id: u64,
    pub(crate) media: BTreeMap<String, RegisteredMedia>,
    pub(crate) identity_policy: DeckIdentityPolicy,
    #[serde(skip, default)]
    pub(crate) used_note_ids: BTreeSet<String>,
    #[serde(skip, default)]
    pub(crate) identity_snapshot_by_id: BTreeMap<String, ResolvedIdentitySnapshot>,
}

impl TryFrom<PersistedDeck> for Deck {
    type Error = anyhow::Error;

    fn try_from(value: PersistedDeck) -> anyhow::Result<Self> {
        let mut deck = Deck {
            name: value.name,
            stable_id: value.stable_id,
            notes: value.notes,
            next_generated_note_id: value.next_generated_note_id,
            media: value.media,
            identity_policy: value.identity_policy,
            used_note_ids: BTreeSet::new(),
            identity_snapshot_by_id: BTreeMap::new(),
        };
        deck.rebuild_runtime_indexes()?;
        Ok(deck)
    }
}
```

```rust
// anki_forge/src/deck/model.rs (note snapshot fields excerpt)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BasicNote {
    pub(crate) id: String,
    pub(crate) stable_id: Option<String>,
    pub(crate) front: String,
    pub(crate) back: String,
    pub(crate) tags: Vec<String>,
    pub(crate) generated: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) identity_override: Option<BasicIdentityOverride>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) resolved_identity: Option<ResolvedIdentitySnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClozeNote {
    pub(crate) id: String,
    pub(crate) stable_id: Option<String>,
    pub(crate) text: String,
    pub(crate) extra: String,
    pub(crate) tags: Vec<String>,
    pub(crate) generated: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) identity_override: Option<ClozeIdentityOverride>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) resolved_identity: Option<ResolvedIdentitySnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IoNote {
    pub(crate) id: String,
    pub(crate) stable_id: Option<String>,
    pub(crate) image: MediaRef,
    pub(crate) mode: IoMode,
    pub(crate) rects: Vec<IoRect>,
    pub(crate) header: String,
    pub(crate) back_extra: String,
    pub(crate) comments: String,
    pub(crate) tags: Vec<String>,
    pub(crate) generated: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) identity_override: Option<IoIdentityOverride>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) resolved_identity: Option<ResolvedIdentitySnapshot>,
}
```

```rust
// anki_forge/src/deck/builders.rs (runtime rebuild excerpt)
impl DeckNote {
    pub(crate) fn resolved_identity_snapshot(
        &self,
    ) -> Option<&crate::deck::model::ResolvedIdentitySnapshot> {
        match self {
            DeckNote::Basic(note) => note.resolved_identity.as_ref(),
            DeckNote::Cloze(note) => note.resolved_identity.as_ref(),
            DeckNote::ImageOcclusion(note) => note.resolved_identity.as_ref(),
        }
    }

    pub(crate) fn assign_resolved_identity(
        &mut self,
        snapshot: crate::deck::model::ResolvedIdentitySnapshot,
    ) {
        match self {
            DeckNote::Basic(note) => note.resolved_identity = Some(snapshot),
            DeckNote::Cloze(note) => note.resolved_identity = Some(snapshot),
            DeckNote::ImageOcclusion(note) => note.resolved_identity = Some(snapshot),
        }
    }

    pub fn resolved_identity(&self) -> Option<&crate::deck::model::ResolvedIdentitySnapshot> {
        self.resolved_identity_snapshot()
    }
}

impl Deck {
    pub(crate) fn rebuild_runtime_indexes(&mut self) -> anyhow::Result<()> {
        self.used_note_ids.clear();
        self.identity_snapshot_by_id.clear();

        for note in &self.notes {
            let note_id = note.id().to_string();
            anyhow::ensure!(
                self.used_note_ids.insert(note_id.clone()),
                "AFID.STABLE_ID_DUPLICATE: {note_id}"
            );

            match note.resolved_identity_snapshot() {
                Some(snapshot) => {
                    anyhow::ensure!(
                        snapshot.stable_id == note_id,
                        "AFID.IDENTITY_SNAPSHOT_NOTE_ID_MISMATCH: {note_id}"
                    );
                    self.identity_snapshot_by_id
                        .insert(note_id.clone(), snapshot.clone());
                }
                None if note.requested_stable_id().is_some() => {
                    self.identity_snapshot_by_id.insert(
                        note_id.clone(),
                        crate::deck::model::ResolvedIdentitySnapshot {
                            stable_id: note_id.clone(),
                            recipe_id: None,
                            provenance: crate::deck::model::IdentityProvenance::ExplicitStableId,
                            canonical_payload: None,
                            used_override: false,
                        },
                    );
                }
                None if note_id.starts_with("generated:") => {}
                None => anyhow::bail!("AFID.IDENTITY_SNAPSHOT_MISSING: {note_id}"),
            }
        }

        Ok(())
    }
}
```

- [ ] **Step 4: Re-run the round-trip tests**

Run: `cargo test -p anki_forge --test deck_identity_roundtrip_tests -v`  
Expected: PASS with snapshot persistence and deserialize-time index rebuild.

- [ ] **Step 5: Commit**

```bash
git add anki_forge/src/deck/model.rs anki_forge/src/deck/builders.rs anki_forge/src/deck/mod.rs anki_forge/tests/deck_identity_roundtrip_tests.rs
git commit -m "feat: persist afid snapshots and rebuild indexes on deserialize"
```

### Task 4: Implement `basic.core.v1` Resolver And Fixture Runner

**Files:**
- Modify: `anki_forge/Cargo.toml`
- Create: `anki_forge/src/deck/identity.rs`
- Modify: `anki_forge/src/deck/builders.rs`
- Modify: `anki_forge/src/deck/mod.rs`
- Modify: `anki_forge/tests/deck_identity_contract_tests.rs`
- Modify: `anki_forge/tests/deck_validation_tests.rs`

- [ ] **Step 1: Write failing resolver tests for stock Basic inference and contract fixtures**

```rust
// anki_forge/tests/deck_identity_contract_tests.rs
use anki_forge::{BasicNote, Deck};
use serde::Deserialize;
use std::{fs, path::PathBuf};

#[derive(Debug, Deserialize)]
struct NoteIdentityFixture {
    recipe_id: String,
    note_kind: String,
    input: serde_json::Value,
    expected: serde_json::Value,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn load_case(path: &str) -> NoteIdentityFixture {
    let raw = fs::read_to_string(repo_root().join(path)).expect("read fixture");
    serde_json::from_str(&raw).expect("parse fixture")
}

#[test]
fn basic_front_only_contract_case_matches_expected_output() {
    let fixture = load_case("contracts/fixtures/note-identity/basic-front-only.case.json");
    let mut deck = Deck::new("Spanish");
    deck.add(BasicNote::new("hola", "hello"))
        .expect("add inferred basic");

    let snapshot = deck.notes()[0]
        .resolved_identity()
        .expect("resolved identity snapshot");
    assert_eq!(snapshot.recipe_id.as_deref(), Some(fixture.recipe_id.as_str()));
    assert_eq!(snapshot.canonical_payload.as_deref(), fixture.expected["canonical_payload"].as_str());
    assert_eq!(snapshot.stable_id, fixture.expected["stable_id"]);
}

#[test]
fn inferred_basic_note_uses_afid_instead_of_generated_id() {
    let mut deck = Deck::new("Spanish");
    deck.add(BasicNote::new("hola", "hello"))
        .expect("add inferred note");

    assert!(deck.notes()[0].id().starts_with("afid:v1:"));
}
```

- [ ] **Step 2: Run the contract tests and confirm failure**

Run: `cargo test -p anki_forge --test deck_identity_contract_tests -v`  
Expected: FAIL because non-explicit notes still receive `generated:*` ids and no `resolved_identity()` accessor exists.

- [ ] **Step 3: Add the Basic resolver, fixed canonical payload hashing, and fixture-aware identity output**

```toml
# anki_forge/Cargo.toml
[dependencies]
blake3 = "1"
unicode-normalization = "0.1"
```

```rust
// anki_forge/src/deck/identity.rs (core excerpt)
use blake3::Hasher;
use serde::Serialize;
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedIdentity {
    pub stable_id: String,
    pub recipe_id: String,
    pub canonical_payload: String,
    pub provenance: crate::deck::model::IdentityProvenance,
    pub used_override: bool,
}

#[derive(Debug, Serialize)]
struct CanonicalIdentityPayload<'a, T: Serialize> {
    algo_version: u8,
    recipe_id: &'a str,
    notetype_family: &'a str,
    notetype_key: &'a str,
    components: T,
}

pub fn normalize_field_text_for_identity(value: &str) -> String {
    value
        .nfc()
        .collect::<String>()
        .replace("\r\n", "\n")
        .replace('\r', "\n")
}

pub fn hash_payload<T: Serialize>(
    recipe_id: &str,
    notetype_family: &str,
    notetype_key: &str,
    components: T,
) -> anyhow::Result<(String, String)> {
    let payload = CanonicalIdentityPayload {
        algo_version: 1,
        recipe_id,
        notetype_family,
        notetype_key,
        components,
    };
    let canonical_payload = serde_json::to_string(&payload)?;
    let mut hasher = Hasher::new();
    hasher.update(canonical_payload.as_bytes());
    Ok((
        format!("afid:v1:{}", hasher.finalize().to_hex()),
        canonical_payload,
    ))
}

#[derive(Debug, Serialize)]
struct BasicFieldComponent {
    name: &'static str,
    value: String,
}

#[derive(Debug, Serialize)]
struct BasicComponents {
    selected_fields: Vec<BasicFieldComponent>,
}

pub fn resolve_basic_identity(
    deck: &crate::deck::model::Deck,
    note: &crate::deck::model::BasicNote,
) -> anyhow::Result<ResolvedIdentity> {
    let (fields, provenance, used_override) = if let Some(override_cfg) = note.identity_override_config() {
        (
            override_cfg.fields().to_vec(),
            crate::deck::model::IdentityProvenance::InferredFromNoteFields,
            true,
        )
    } else if let Some(selection) = deck.identity_policy.basic.as_ref() {
        (
            selection.as_slice().to_vec(),
            crate::deck::model::IdentityProvenance::InferredFromNotetypeFields,
            false,
        )
    } else {
        (
            vec![crate::deck::model::BasicIdentityField::Front],
            crate::deck::model::IdentityProvenance::InferredFromStockRecipe,
            false,
        )
    };

    let components = BasicComponents {
        selected_fields: fields
            .into_iter()
            .map(|field| match field {
                crate::deck::model::BasicIdentityField::Front => BasicFieldComponent {
                    name: "front",
                    value: normalize_field_text_for_identity(&note.front),
                },
                crate::deck::model::BasicIdentityField::Back => BasicFieldComponent {
                    name: "back",
                    value: normalize_field_text_for_identity(&note.back),
                },
            })
            .collect(),
    };

    let (stable_id, canonical_payload) =
        hash_payload("basic.core.v1", "stock", "basic", components)?;
    Ok(ResolvedIdentity {
        stable_id,
        recipe_id: "basic.core.v1".into(),
        canonical_payload,
        provenance,
        used_override,
    })
}

pub fn resolve_inferred_identity(
    deck: &crate::deck::model::Deck,
    note: &crate::deck::model::DeckNote,
) -> anyhow::Result<ResolvedIdentity> {
    match note {
        crate::deck::model::DeckNote::Basic(note) => resolve_basic_identity(deck, note),
        crate::deck::model::DeckNote::Cloze(note) => resolve_cloze_identity(deck, note),
        crate::deck::model::DeckNote::ImageOcclusion(note) => resolve_io_identity(deck, note),
    }
}
```

```rust
// anki_forge/src/deck/builders.rs (assign excerpt)
fn assign_identity(deck: &mut Deck, note: &mut DeckNote) -> anyhow::Result<()> {
    deck.rebuild_runtime_indexes()?;
    let requested = note.requested_stable_id().map(str::trim);

    match requested {
        Some("") => anyhow::bail!("stable_id must not be blank"),
        Some(stable_id) => {
            anyhow::ensure!(
                !deck.used_note_ids.contains(stable_id),
                "AFID.STABLE_ID_DUPLICATE: {stable_id}"
            );
            note.assign_stable_id(stable_id.to_string());
            note.assign_resolved_identity(crate::deck::model::ResolvedIdentitySnapshot {
                stable_id: stable_id.to_string(),
                recipe_id: None,
                provenance: crate::deck::model::IdentityProvenance::ExplicitStableId,
                canonical_payload: None,
                used_override: false,
            });
        }
        None => {
            let resolved = crate::deck::identity::resolve_inferred_identity(deck, note)?;
            note.assign_stable_id(resolved.stable_id.clone());
            note.assign_resolved_identity(crate::deck::model::ResolvedIdentitySnapshot {
                stable_id: resolved.stable_id,
                recipe_id: Some(resolved.recipe_id),
                provenance: resolved.provenance,
                canonical_payload: Some(resolved.canonical_payload),
                used_override: resolved.used_override,
            });
        }
    }

    deck.rebuild_runtime_indexes()?;
    Ok(())
}
```

- [ ] **Step 4: Re-run the contract tests**

Run: `cargo test -p anki_forge --test deck_identity_contract_tests -v`  
Expected: PASS for `basic.core.v1` fixture matching and `afid:v1:*` assignment.

- [ ] **Step 5: Commit**

```bash
git add anki_forge/Cargo.toml anki_forge/src/deck/identity.rs anki_forge/src/deck/builders.rs anki_forge/src/deck/mod.rs anki_forge/tests/deck_identity_contract_tests.rs anki_forge/tests/deck_validation_tests.rs
git commit -m "feat: add basic afid resolver and fixture-backed hashing"
```

### Task 5: Replace The Cloze Recipe With `cloze.core.v2`

**Files:**
- Modify: `contracts/fixtures/note-identity/cloze-hint-ignored.case.json`
- Modify: `contracts/fixtures/note-identity/cloze-whitespace-significant.case.json`
- Modify: `contracts/fixtures/note-identity/cloze-malformed.case.json`
- Modify: `anki_forge/src/deck/identity.rs`
- Modify: `anki_forge/tests/deck_identity_contract_tests.rs`
- Modify: `anki_forge/tests/deck_validation_tests.rs`

- [ ] **Step 1: Add failing tests for whitespace significance, malformed cloze, and nested support**

```rust
// anki_forge/tests/deck_identity_contract_tests.rs
use anki_forge::{ClozeNote, Deck};

#[test]
fn cloze_hint_change_does_not_change_identity() {
    let mut deck_a = Deck::new("Geo");
    deck_a
        .add(ClozeNote::new("Capital of {{c1::France::country}} is {{c2::Paris::city}}"))
        .expect("deck a");

    let mut deck_b = Deck::new("Geo");
    deck_b
        .add(ClozeNote::new("Capital of {{c1::France::nation}} is {{c2::Paris::place}}"))
        .expect("deck b");

    assert_eq!(deck_a.notes()[0].id(), deck_b.notes()[0].id());
}

#[test]
fn cloze_boundary_whitespace_changes_identity() {
    let mut deck_a = Deck::new("Geo");
    deck_a
        .add(ClozeNote::new("A {{c1::B}} C"))
        .expect("deck a");

    let mut deck_b = Deck::new("Geo");
    deck_b
        .add(ClozeNote::new("A{{c1::B}}C"))
        .expect("deck b");

    assert_ne!(deck_a.notes()[0].id(), deck_b.notes()[0].id());
}

#[test]
fn malformed_cloze_reports_afid_error() {
    let mut deck = Deck::new("Geo");
    let err = deck
        .add(ClozeNote::new("Capital of {{c1::France is Paris"))
        .expect_err("malformed cloze must fail");

    assert!(err.to_string().contains("AFID.CLOZE_MALFORMED"));
}

#[test]
fn nested_cloze_reports_explicit_unsupported_error() {
    let mut deck = Deck::new("Geo");
    let err = deck
        .add(ClozeNote::new("{{c1::outer {{c2::inner}} body}}"))
        .expect_err("nested cloze must fail explicitly");

    assert!(err
        .to_string()
        .contains("AFID.CLOZE_NESTED_UNSUPPORTED"));
}
```

- [ ] **Step 2: Run the cloze tests and confirm failure**

Run: `cargo test -p anki_forge --test deck_identity_contract_tests cloze_ -v`  
Expected: FAIL because the current parser trims text fragments and uses delimiter scanning that cannot preserve boundary whitespace or reject nested syntax deterministically.

- [ ] **Step 3: Implement a real cloze parser, AST traversal, and `cloze.core.v2` canonicalization**

```rust
// anki_forge/src/deck/identity.rs (cloze excerpt)
use std::num::NonZeroU32;

#[derive(Debug, Clone, PartialEq, Eq)]
enum ClozeSegment {
    Text(String),
    Deletion {
        ord: NonZeroU32,
        body: String,
        hint: Option<String>,
        slot: usize,
    },
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct ClozeDeletion {
    ord: u32,
    body: String,
    slot: usize,
}

#[derive(Debug, Serialize)]
struct ClozeComponents {
    text_skeleton: String,
    deletions: Vec<ClozeDeletion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    extra: Option<String>,
}

fn normalize_deleted_text_for_identity(value: &str) -> String {
    normalize_field_text_for_identity(value)
}

fn parse_cloze_segments(input: &str) -> anyhow::Result<Vec<ClozeSegment>> {
    let mut segments = Vec::new();
    let mut cursor = 0usize;
    let mut slot = 0usize;

    while let Some(start_rel) = input[cursor..].find("{{c") {
        let start = cursor + start_rel;
        if start > cursor {
            segments.push(ClozeSegment::Text(input[cursor..start].to_string()));
        }

        let mut idx = start + 3;
        let ord_start = idx;
        while idx < input.len() && input.as_bytes()[idx].is_ascii_digit() {
            idx += 1;
        }
        anyhow::ensure!(
            idx > ord_start && input[idx..].starts_with("::"),
            "AFID.CLOZE_MALFORMED"
        );

        let ord_value = input[ord_start..idx].parse::<u32>()?;
        let ord = NonZeroU32::new(ord_value)
            .ok_or_else(|| anyhow::anyhow!("AFID.CLOZE_ORD_INVALID"))?;
        idx += 2;

        let mut body = String::new();
        let mut hint = String::new();
        let mut in_hint = false;
        let mut closed = false;

        while idx < input.len() {
            if input[idx..].starts_with("{{c") {
                anyhow::bail!("AFID.CLOZE_NESTED_UNSUPPORTED");
            }
            if input[idx..].starts_with("::") && !in_hint {
                in_hint = true;
                idx += 2;
                continue;
            }
            if input[idx..].starts_with("}}") {
                idx += 2;
                closed = true;
                break;
            }

            let ch = input[idx..]
                .chars()
                .next()
                .ok_or_else(|| anyhow::anyhow!("AFID.CLOZE_MALFORMED"))?;
            if in_hint {
                hint.push(ch);
            } else {
                body.push(ch);
            }
            idx += ch.len_utf8();
        }

        anyhow::ensure!(closed, "AFID.CLOZE_MALFORMED");
        anyhow::ensure!(!body.is_empty(), "AFID.CLOZE_MALFORMED");

        segments.push(ClozeSegment::Deletion {
            ord,
            body,
            hint: (!hint.is_empty()).then_some(hint),
            slot,
        });
        slot += 1;
        cursor = idx;
    }

    if cursor < input.len() {
        segments.push(ClozeSegment::Text(input[cursor..].to_string()));
    }

    anyhow::ensure!(
        segments
            .iter()
            .any(|segment| matches!(segment, ClozeSegment::Deletion { .. })),
        "AFID.IDENTITY_COMPONENT_EMPTY: cloze deletions"
    );
    Ok(segments)
}

fn canonicalize_cloze_segments(
    segments: &[ClozeSegment],
    skeleton: &mut String,
    deletions: &mut Vec<ClozeDeletion>,
) {
    for segment in segments {
        match segment {
            ClozeSegment::Text(text) => skeleton.push_str(text),
            ClozeSegment::Deletion { ord, body, slot, .. } => {
                skeleton.push_str("[[CLOZE]]");
                deletions.push(ClozeDeletion {
                    ord: ord.get(),
                    body: normalize_deleted_text_for_identity(body),
                    slot: *slot,
                });
            }
        }
    }
}

fn resolve_cloze_identity(
    deck: &crate::deck::model::Deck,
    note: &crate::deck::model::ClozeNote,
) -> anyhow::Result<ResolvedIdentity> {
    let segments = parse_cloze_segments(&normalize_field_text_for_identity(&note.text))?;
    let mut text_skeleton = String::new();
    let mut deletions = Vec::new();
    canonicalize_cloze_segments(&segments, &mut text_skeleton, &mut deletions);

    let components = ClozeComponents {
        text_skeleton,
        deletions,
        extra: deck
            .identity_policy
            .cloze
            .as_ref()
            .filter(|selection| selection.as_slice().contains(&crate::deck::model::ClozeIdentityField::Extra))
            .map(|_| normalize_field_text_for_identity(&note.extra)),
    };

    let (stable_id, canonical_payload) =
        hash_payload("cloze.core.v2", "stock", "cloze", components)?;
    Ok(ResolvedIdentity {
        stable_id,
        recipe_id: "cloze.core.v2".into(),
        canonical_payload,
        provenance: crate::deck::model::IdentityProvenance::InferredFromStockRecipe,
        used_override: false,
    })
}
```

- [ ] **Step 4: Re-run the cloze tests**

Run: `cargo test -p anki_forge --test deck_identity_contract_tests cloze_ -v`  
Expected: PASS for hint-insensitive matching, whitespace-sensitive matching, malformed-cloze errors, and explicit nested-cloze rejection.

- [ ] **Step 5: Commit**

```bash
git add contracts/fixtures/note-identity/cloze-hint-ignored.case.json contracts/fixtures/note-identity/cloze-whitespace-significant.case.json contracts/fixtures/note-identity/cloze-malformed.case.json anki_forge/src/deck/identity.rs anki_forge/tests/deck_identity_contract_tests.rs anki_forge/tests/deck_validation_tests.rs
git commit -m "feat: replace cloze identity with contract-backed v2 parser"
```

### Task 6: Replace The IO Recipe With `io.core.v2` Pixel-Space Geometry

**Files:**
- Modify: `anki_forge/Cargo.toml`
- Modify: `contracts/fixtures/note-identity/io-order-insensitive.case.json`
- Modify: `contracts/fixtures/note-identity/io-translation-different.case.json`
- Modify: `anki_forge/src/deck/model.rs`
- Modify: `anki_forge/src/deck/media.rs`
- Modify: `anki_forge/src/deck/identity.rs`
- Modify: `anki_forge/tests/deck_identity_contract_tests.rs`
- Modify: `anki_forge/tests/deck_validation_tests.rs`

- [ ] **Step 1: Add failing IO tests for order stability, translation sensitivity, and missing dimensions**

```rust
// anki_forge/tests/deck_identity_contract_tests.rs
use anki_forge::{Deck, IoMode, MediaSource};
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

#[test]
fn io_mask_order_does_not_change_identity() {
    let image_path = repo_root()
        .join("contracts/fixtures/phase3/manual-desktop-v1/S03_io_minimal/assets/occlusion-heart.png");

    let mut deck_a = Deck::new("Anatomy");
    let image_a = deck_a
        .media()
        .add(MediaSource::from_file(&image_path))
        .expect("image a");
    deck_a
        .image_occlusion()
        .note(image_a)
        .mode(IoMode::HideAllGuessOne)
        .rect(10, 20, 30, 40)
        .rect(100, 120, 30, 40)
        .add()
        .expect("io a");

    let mut deck_b = Deck::new("Anatomy");
    let image_b = deck_b
        .media()
        .add(MediaSource::from_file(&image_path))
        .expect("image b");
    deck_b
        .image_occlusion()
        .note(image_b)
        .mode(IoMode::HideAllGuessOne)
        .rect(100, 120, 30, 40)
        .rect(10, 20, 30, 40)
        .add()
        .expect("io b");

    assert_eq!(deck_a.notes()[0].id(), deck_b.notes()[0].id());
}

#[test]
fn io_translation_changes_identity() {
    let image_path = repo_root()
        .join("contracts/fixtures/phase3/manual-desktop-v1/S03_io_minimal/assets/occlusion-heart.png");

    let mut deck_a = Deck::new("Anatomy");
    let image_a = deck_a
        .media()
        .add(MediaSource::from_file(&image_path))
        .expect("image a");
    deck_a
        .image_occlusion()
        .note(image_a)
        .mode(IoMode::HideAllGuessOne)
        .rect(10, 20, 30, 40)
        .add()
        .expect("io a");

    let mut deck_b = Deck::new("Anatomy");
    let image_b = deck_b
        .media()
        .add(MediaSource::from_file(&image_path))
        .expect("image b");
    deck_b
        .image_occlusion()
        .note(image_b)
        .mode(IoMode::HideAllGuessOne)
        .rect(11, 20, 30, 40)
        .add()
        .expect("io b");

    assert_ne!(deck_a.notes()[0].id(), deck_b.notes()[0].id());
}

#[test]
fn invalid_raster_without_dimensions_fails_identity_resolution() {
    let mut deck = Deck::new("Anatomy");
    let image = deck
        .media()
        .add(MediaSource::from_bytes("broken.png", vec![1, 2, 3]))
        .expect("register media");

    let err = deck
        .image_occlusion()
        .note(image)
        .mode(IoMode::HideAllGuessOne)
        .rect(1, 2, 3, 4)
        .add()
        .expect_err("missing dimensions must fail");

    assert!(err.to_string().contains("AFID.IO_IMAGE_DIMENSIONS_MISSING"));
}
```

- [ ] **Step 2: Run the IO tests and confirm failure**

Run: `cargo test -p anki_forge --test deck_identity_contract_tests io_ -v`  
Expected: FAIL because IO identity is still based on bounding-box-relative geometry and does not require source image dimensions.

- [ ] **Step 3: Add raster image metadata and pixel-space IO canonicalization**

```toml
# anki_forge/Cargo.toml
[dependencies]
imagesize = "0.13"
```

```rust
// anki_forge/src/deck/model.rs (media metadata excerpt)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RasterImageMetadata {
    pub width_px: u32,
    pub height_px: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisteredMedia {
    pub(crate) name: String,
    pub(crate) mime: String,
    pub(crate) data_base64: String,
    pub(crate) sha1_hex: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) raster_image: Option<RasterImageMetadata>,
}
```

```rust
// anki_forge/src/deck/media.rs (metadata extraction excerpt)
fn raster_image_metadata(name: &str, bytes: &[u8]) -> Option<crate::deck::model::RasterImageMetadata> {
    match mime_from_name(name).as_str() {
        "image/png" | "image/jpeg" => imagesize::blob_size(bytes).ok().map(|size| {
            crate::deck::model::RasterImageMetadata {
                width_px: size.width as u32,
                height_px: size.height as u32,
            }
        }),
        _ => None,
    }
}

impl RegisteredMedia {
    pub fn from_source(source: MediaSource) -> anyhow::Result<Self> {
        let (name, bytes) = match source {
            MediaSource::File { path } => {
                let name = path
                    .file_name()
                    .and_then(|item| item.to_str())
                    .ok_or_else(|| anyhow::anyhow!("media path must end in a valid filename"))?
                    .to_string();
                (name, std::fs::read(path)?)
            }
            MediaSource::Bytes { name, bytes } => (name, bytes),
        };
        validate_media_filename(&name)?;

        let sha1_hex = hex::encode(sha1::Sha1::digest(&bytes));
        let raster_image = raster_image_metadata(&name, &bytes);
        Ok(Self {
            name: name.clone(),
            mime: mime_from_name(&name),
            data_base64: base64::engine::general_purpose::STANDARD.encode(bytes),
            sha1_hex,
            raster_image,
        })
    }
}
```

```rust
// anki_forge/src/deck/identity.rs (io excerpt)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum IoModeWire {
    HideAllGuessOne,
    HideOneGuessOne,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "shape", rename_all = "snake_case")]
enum IoShapeComponent {
    Rect { x_px: u32, y_px: u32, w_px: u32, h_px: u32 },
}

#[derive(Debug, Serialize)]
struct IoComponents {
    image_anchor: String,
    image_width_px: u32,
    image_height_px: u32,
    occlusion_mode: IoModeWire,
    shapes: Vec<IoShapeComponent>,
}

fn resolve_io_identity(
    deck: &crate::deck::model::Deck,
    note: &crate::deck::model::IoNote,
) -> anyhow::Result<ResolvedIdentity> {
    let media = deck
        .media
        .get(note.image.name())
        .ok_or_else(|| anyhow::anyhow!("AFID.IDENTITY_COMPONENT_EMPTY: missing io media"))?;
    let raster = media
        .raster_image
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("AFID.IO_IMAGE_DIMENSIONS_MISSING: {}", note.image.name()))?;

    let mut shapes = note
        .rects
        .iter()
        .map(|rect| IoShapeComponent::Rect {
            x_px: rect.x,
            y_px: rect.y,
            w_px: rect.width,
            h_px: rect.height,
        })
        .collect::<Vec<_>>();
    shapes.sort_by(|left, right| format!("{left:?}").cmp(&format!("{right:?}")));

    let mode = match note.mode {
        crate::deck::model::IoMode::HideAllGuessOne => IoModeWire::HideAllGuessOne,
        crate::deck::model::IoMode::HideOneGuessOne => IoModeWire::HideOneGuessOne,
    };

    let components = IoComponents {
        image_anchor: media.sha1_hex.clone(),
        image_width_px: raster.width_px,
        image_height_px: raster.height_px,
        occlusion_mode: mode,
        shapes,
    };

    let (stable_id, canonical_payload) =
        hash_payload("io.core.v2", "stock", "image_occlusion", components)?;
    Ok(ResolvedIdentity {
        stable_id,
        recipe_id: "io.core.v2".into(),
        canonical_payload,
        provenance: crate::deck::model::IdentityProvenance::InferredFromStockRecipe,
        used_override: false,
    })
}
```

- [ ] **Step 4: Re-run the IO tests**

Run: `cargo test -p anki_forge --test deck_identity_contract_tests io_ -v`  
Expected: PASS for order-insensitive mask identity, translation-sensitive geometry, and missing-dimensions errors.

- [ ] **Step 5: Commit**

```bash
git add anki_forge/Cargo.toml contracts/fixtures/note-identity/io-order-insensitive.case.json contracts/fixtures/note-identity/io-translation-different.case.json anki_forge/src/deck/model.rs anki_forge/src/deck/media.rs anki_forge/src/deck/identity.rs anki_forge/tests/deck_identity_contract_tests.rs anki_forge/tests/deck_validation_tests.rs
git commit -m "feat: replace io identity with pixel-space v2 recipe"
```

### Task 7: Wire AFID Defaults, Blocking Diagnostics, And Docs

**Files:**
- Modify: `anki_forge/src/deck/builders.rs`
- Modify: `anki_forge/src/deck/validation.rs`
- Modify: `anki_forge/src/deck/mod.rs`
- Modify: `anki_forge/tests/deck_validation_tests.rs`
- Modify: `anki_forge/tests/deck_identity_roundtrip_tests.rs`
- Modify: `README.md`

- [ ] **Step 1: Write failing tests for duplicate/collision blocking and override warnings**

```rust
// anki_forge/tests/deck_validation_tests.rs
use anki_forge::{
    BasicIdentityField, BasicIdentityOverride, BasicNote, ClozeNote, Deck, ValidationCode,
};

#[test]
fn inferred_duplicate_payload_is_error() {
    let mut deck = Deck::new("Spanish");
    deck.add(BasicNote::new("hola", "hello")).expect("first");

    let err = deck
        .add(BasicNote::new("hola", "hello"))
        .expect_err("duplicate payload must fail");
    assert!(err.to_string().contains("AFID.IDENTITY_DUPLICATE_PAYLOAD"));
}

#[test]
fn cross_notetype_same_visible_text_produces_different_afids() {
    let mut deck = Deck::new("Mixed");
    deck.add(BasicNote::new("Paris", "city")).expect("basic");
    deck.add(ClozeNote::new("{{c1::Paris}} is a city"))
        .expect("cloze");

    assert_ne!(deck.notes()[0].id(), deck.notes()[1].id());
}

#[test]
fn note_level_override_emits_warning_diagnostic() {
    let mut deck = Deck::new("Spanish");
    let override_cfg = BasicIdentityOverride::new(
        [BasicIdentityField::Front, BasicIdentityField::Back],
        "sense-disambiguation",
    )
    .expect("override");

    deck.add(BasicNote::new("bank", "river").identity_override(override_cfg))
        .expect("override note");

    let report = deck.validate_report().expect("report");
    assert!(report
        .diagnostics()
        .iter()
        .any(|item| item.code == ValidationCode::NoteLevelIdentityOverrideUsed));
}
```

- [ ] **Step 2: Run the validation tests and confirm failure**

Run: `cargo test -p anki_forge --test deck_validation_tests -v`  
Expected: FAIL because duplicate inferred payloads are not yet classified against persisted snapshots and the override warning code does not exist.

- [ ] **Step 3: Finalize add-time default inference, duplicate/collision classifier, audit accessors, and docs**

```rust
// anki_forge/src/deck/validation.rs
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ValidationCode {
    MissingStableId,
    DuplicateStableId,
    BlankStableId,
    EmptyIoMasks,
    UnknownMediaRef,
    NoteLevelIdentityOverrideUsed,
    IdentityDuplicatePayload,
    IdentityCollision,
    StableIdDuplicate,
}
```

```rust
// anki_forge/src/deck/builders.rs (duplicate/collision excerpt)
fn classify_duplicate(deck: &Deck, snapshot: &crate::deck::model::ResolvedIdentitySnapshot) -> anyhow::Result<()> {
    if let Some(existing) = deck.identity_snapshot_by_id.get(&snapshot.stable_id) {
        match (&existing.canonical_payload, &snapshot.canonical_payload) {
            (Some(left), Some(right)) if left == right => {
                anyhow::bail!("AFID.IDENTITY_DUPLICATE_PAYLOAD: {}", snapshot.stable_id);
            }
            (Some(_), Some(_)) => {
                anyhow::bail!("AFID.IDENTITY_COLLISION: {}", snapshot.stable_id);
            }
            _ => {
                anyhow::bail!("AFID.STABLE_ID_DUPLICATE: {}", snapshot.stable_id);
            }
        }
    }
    Ok(())
}

impl Deck {
    pub fn validate_report(&self) -> anyhow::Result<ValidationReport> {
        let mut diagnostics = Vec::new();
        let mut seen_ids = std::collections::BTreeSet::new();

        for note in &self.notes {
            if !seen_ids.insert(note.id().to_string()) {
                diagnostics.push(ValidationDiagnostic {
                    code: ValidationCode::StableIdDuplicate,
                    message: format!("id '{}' is duplicated", note.id()),
                    severity: "error".into(),
                });
            }

            if note
                .resolved_identity()
                .map(|snapshot| snapshot.used_override)
                .unwrap_or(false)
            {
                diagnostics.push(ValidationDiagnostic {
                    code: ValidationCode::NoteLevelIdentityOverrideUsed,
                    message: format!("note '{}' used note-level identity override", note.id()),
                    severity: "warning".into(),
                });
            }
        }

        Ok(ValidationReport::new(diagnostics))
    }
}
```

```markdown
<!-- README.md -->
### Stable note identity defaults

`Deck` notes now use explicit-first AFID resolution:

1. `stable_id("es-hola")` is preserved as-is.
2. If omitted, stock note kinds infer deterministic `afid:v1:*` ids from contract-backed recipes.

Typed policy and override APIs:

1. `Deck::builder("Spanish").basic_identity(BasicIdentitySelection::new([BasicIdentityField::Front])?)`
2. `BasicNote::new("bank", "river").identity_override(BasicIdentityOverride::new([BasicIdentityField::Front, BasicIdentityField::Back], "sense-disambiguation")?)`

Round-trip guarantee:

1. inferred notes persist `ResolvedIdentitySnapshot`
2. deserialize-time rebuild restores duplicate detection without re-running the current resolver
3. duplicate payloads and collisions are blocking errors
```

- [ ] **Step 4: Run focused tests, full crate tests, and lint/format checks**

Run: `cargo test -p anki_forge --test deck_model_tests --test deck_identity_contract_tests --test deck_identity_roundtrip_tests --test deck_validation_tests -v`  
Expected: PASS for typed selectors, fixture-backed recipes, round-trip rebuild, and duplicate/collision diagnostics.

Run: `cargo test -p contract_tools --test fixture_gate_tests -v`  
Expected: PASS with the bundled note-identity fixtures.

Run: `cargo test -p anki_forge -v`  
Expected: PASS with no regression in deck, export, or runtime facade tests.

Run: `cargo fmt --all --check`  
Expected: PASS.

Run: `cargo clippy -p anki_forge --tests -- -D warnings`  
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add anki_forge/src/deck/builders.rs anki_forge/src/deck/validation.rs anki_forge/src/deck/mod.rs anki_forge/tests/deck_validation_tests.rs anki_forge/tests/deck_identity_roundtrip_tests.rs README.md
git commit -m "feat: enable afid defaults with blocking diagnostics"
```

## Plan Self-Review

### 1. Spec coverage check

- Shared AFID contract source of truth: Task 1.
- Strongly-typed selector and override APIs: Task 2.
- Serde-stable runtime rebuild and persisted snapshots: Task 3.
- `basic.core.v1`: Task 4.
- `cloze.core.v2`: Task 5.
- `io.core.v2`: Task 6.
- Blocking duplicate/collision behavior and docs: Task 7.
- Explicit deferral of artifact-level `inspect-report` identity blocks: scope exclusion, not a missing task.

### 2. Placeholder scan

- No `TODO`, `TBD`, or “similar to Task N” placeholders remain.
- Every task lists exact files, concrete test commands, and code snippets for the main API and resolver changes.
- Fixture paths and recipe ids are explicit and versioned.

### 3. Type/signature consistency check

- Public selector API uses `IdentitySelection<F>` / `IdentityOverride<F>` plus stock field enums throughout Tasks 2-7.
- Persisted identity shape is always `ResolvedIdentitySnapshot`.
- Resolver outputs always carry `stable_id`, `recipe_id`, `canonical_payload`, `provenance`, and `used_override`.
- Recipe versions stay consistent across contracts and code: `basic.core.v1`, `cloze.core.v2`, `io.core.v2`.

Plan complete and saved to `docs/superpowers/plans/2026-04-13-note-stable-id-implementation-plan.md`. Two execution options:

1. Subagent-Driven (recommended) - I dispatch a fresh subagent per task, review between tasks, fast iteration
2. Inline Execution - Execute tasks in this session using executing-plans, batch execution with checkpoints

Which approach?
