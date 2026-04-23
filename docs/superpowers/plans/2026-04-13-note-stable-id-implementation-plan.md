# Note Stable ID Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement deterministic note stable-id inference for `Deck` notes with stock recipes, notetype-aware canonical payload hashing, blocking duplicate/collision rules, and auditable identity diagnostics.

**Architecture:** Introduce a dedicated `deck::identity` module that resolves note identity from explicit ids or recipe-derived components, then hashes canonical payloads to `afid:v1:<blake3-hex>`. Store deck-level notetype identity policy, note-level override escape hatches (with required reason code), and enforce collision/duplicate rules at add-time and validation-time with explicit AFID diagnostics.

**Tech Stack:** Rust (`anyhow`, `serde`, `serde_json`, `blake3`, `unicode-normalization`), existing `anki_forge::deck` facade, fixture-driven integration tests in `anki_forge/tests`, Cargo test runner.

---

## Scope Check

This plan covers one subsystem: default-layer note stable identity resolution for the existing `Deck` API (`Basic`, `Cloze`, `Image Occlusion`).

This pass includes:

1. explicit-first identity resolution (`stable_id` wins)
2. notetype-level `identity_from_fields` defaults
3. note-level `identity_from_fields` escape hatch with required `reason_code`
4. stock fallback recipes (`basic.core.v1`, `cloze.core.v1`, `io.core.v1`)
5. canonical payload with `notetype_key`
6. `afid:v1` hashing and blocking duplicate/collision behavior
7. text-only normalization and IO integer quantization
8. AFID diagnostics and provenance reporting

This pass excludes:

1. card-level identity policies
2. default-layer custom-note authoring API surface
3. parser-level HTML canonicalization

## Execution Prerequisite

Run this plan in a dedicated worktree:

```bash
git worktree add ../anki-forge-note-stable-id -b codex/note-stable-id
cd ../anki-forge-note-stable-id
```

## File Structure Map

- Modify: `anki_forge/Cargo.toml` - add hashing and unicode-normalization dependencies.
- Create: `anki_forge/src/deck/identity.rs` - canonical payload model, normalization helpers, recipe resolution, hashing, collision classifier.
- Modify: `anki_forge/src/deck/mod.rs` - export identity policy and provenance types.
- Modify: `anki_forge/src/deck/model.rs` - add deck-level identity policy, note-level override metadata, identity provenance, and runtime collision index.
- Modify: `anki_forge/src/deck/builders.rs` - replace generated-id fallback with resolver flow, enforce override policy, enforce blocking duplicate/collision behavior.
- Modify: `anki_forge/src/deck/validation.rs` - add AFID diagnostic codes and keep severity semantics.
- Modify: `anki_forge/tests/deck_model_tests.rs` - cover API defaults and identity policy storage behavior.
- Modify: `anki_forge/tests/deck_validation_tests.rs` - replace generated-id expectations with inferred-id and AFID collision behavior.
- Create: `anki_forge/tests/deck_identity_tests.rs` - focused tests for recipe behavior (`Basic`, `Cloze`, `IO`), notetype key separation, and quantized masks.
- Modify: `README.md` - update default identity behavior to inference-first + blocking duplicate/collision.

## Implementation Notes

- Keep explicit `stable_id` behavior unchanged except for stricter duplicate handling.
- Do not silently keep both notes when inferred identities clash.
- Canonical payload must include both `notetype_family` and `notetype_key`.
- `identity_from_fields` should use user-facing stock names:
  - Basic: `Front`, `Back`
  - Cloze: `Text`, `Extra`
  - IO: `Image`, `Mode`, `Rects`, `Header`, `Back Extra`, `Comments`
- Note-level overrides are escape hatches; emit warning diagnostic when used.
- IO hash payload must be integer-only after quantization (`q(v) = round(clamp(v, 0.0, 1.0) * 10000)`).

### Task 1: Add Identity Policy Surface To Deck/Notes

**Files:**
- Modify: `anki_forge/src/deck/model.rs`
- Modify: `anki_forge/src/deck/builders.rs`
- Modify: `anki_forge/src/deck/mod.rs`
- Test: `anki_forge/tests/deck_model_tests.rs`

- [ ] **Step 1: Write failing API tests for notetype policy + note override metadata**

```rust
// anki_forge/tests/deck_model_tests.rs
use anki_forge::{BasicNote, Deck};

#[test]
fn deck_builder_stores_notetype_identity_fields() {
    let deck = Deck::builder("Spanish")
        .basic_identity_from_fields(["Front"])
        .cloze_identity_from_fields(["Text"])
        .image_occlusion_identity_from_fields(["Image", "Rects", "Mode"])
        .build();

    let policy = deck.identity_policy();
    assert_eq!(policy.basic.as_deref(), Some(&["Front".to_string()][..]));
    assert_eq!(policy.cloze.as_deref(), Some(&["Text".to_string()][..]));
    assert_eq!(
        policy.image_occlusion.as_deref(),
        Some(&["Image".to_string(), "Rects".to_string(), "Mode".to_string()][..])
    );
}

#[test]
fn basic_note_can_store_note_level_identity_override_reason() {
    let note = BasicNote::new("hola", "hello")
        .identity_from_fields(["Front"])
        .identity_override_reason_code("homonym-disambiguation");

    let override_cfg = note.identity_override().expect("identity override");
    assert_eq!(override_cfg.fields, vec!["Front".to_string()]);
    assert_eq!(override_cfg.reason_code, "homonym-disambiguation");
}
```

- [ ] **Step 2: Run model tests to confirm failure**

Run: `cargo test -p anki_forge --test deck_model_tests -v`  
Expected: FAIL with missing methods (`basic_identity_from_fields`, `identity_policy`, `identity_from_fields`, `identity_override_reason_code`).

- [ ] **Step 3: Add model and builder API for identity policy and note override metadata**

```rust
// anki_forge/src/deck/model.rs (new types + new fields)
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub struct DeckIdentityPolicy {
    pub basic: Option<Vec<String>>,
    pub cloze: Option<Vec<String>>,
    pub image_occlusion: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NoteIdentityOverride {
    pub fields: Vec<String>,
    pub reason_code: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum IdentityProvenance {
    ExplicitStableId,
    InferredFromNoteFields,
    InferredFromNotetypeFields,
    InferredFromStockRecipe,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
    pub(crate) identity_payload_by_id: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct BasicNote {
    pub(crate) id: String,
    pub(crate) stable_id: Option<String>,
    pub(crate) front: String,
    pub(crate) back: String,
    pub(crate) tags: Vec<String>,
    pub(crate) generated: bool,
    #[serde(default)]
    pub(crate) identity_override: Option<NoteIdentityOverride>,
    #[serde(default)]
    pub(crate) provenance: Option<IdentityProvenance>,
}

impl Deck {
    pub fn identity_policy(&self) -> &DeckIdentityPolicy {
        &self.identity_policy
    }
}

impl BasicNote {
    pub fn identity_from_fields<T, I>(mut self, fields: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        let values: Vec<String> = fields.into_iter().map(Into::into).collect();
        self.identity_override = Some(NoteIdentityOverride {
            fields: values,
            reason_code: String::new(),
        });
        self
    }

    pub fn identity_override_reason_code(mut self, reason_code: impl Into<String>) -> Self {
        let reason_code = reason_code.into();
        let override_cfg = self.identity_override.get_or_insert(NoteIdentityOverride {
            fields: Vec::new(),
            reason_code: String::new(),
        });
        override_cfg.reason_code = reason_code;
        self
    }

    pub fn identity_override(&self) -> Option<&NoteIdentityOverride> {
        self.identity_override.as_ref()
    }
}
```

```rust
// anki_forge/src/deck/builders.rs (DeckBuilder additions)
impl DeckBuilder {
    pub fn basic_identity_from_fields<T, I>(mut self, fields: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        self.identity_policy.basic = Some(fields.into_iter().map(Into::into).collect());
        self
    }

    pub fn cloze_identity_from_fields<T, I>(mut self, fields: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        self.identity_policy.cloze = Some(fields.into_iter().map(Into::into).collect());
        self
    }

    pub fn image_occlusion_identity_from_fields<T, I>(mut self, fields: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        self.identity_policy.image_occlusion = Some(fields.into_iter().map(Into::into).collect());
        self
    }
}
```

- [ ] **Step 4: Re-run model tests**

Run: `cargo test -p anki_forge --test deck_model_tests -v`  
Expected: PASS for the two new identity-policy tests.

- [ ] **Step 5: Commit**

```bash
git add anki_forge/src/deck/model.rs anki_forge/src/deck/builders.rs anki_forge/src/deck/mod.rs anki_forge/tests/deck_model_tests.rs
git commit -m "feat: add deck identity policy and note override metadata"
```

### Task 2: Build Identity Resolver Core (`afid:v1`) And Wire Add-Time Assignment

**Files:**
- Modify: `anki_forge/Cargo.toml`
- Create: `anki_forge/src/deck/identity.rs`
- Modify: `anki_forge/src/deck/mod.rs`
- Modify: `anki_forge/src/deck/builders.rs`
- Test: `anki_forge/tests/deck_identity_tests.rs`

- [ ] **Step 1: Write failing tests for deterministic inferred id and provenance**

```rust
// anki_forge/tests/deck_identity_tests.rs
use anki_forge::{BasicNote, Deck};

#[test]
fn basic_without_explicit_stable_id_gets_deterministic_afid() {
    let mut deck_a = Deck::new("Spanish");
    deck_a
        .add(BasicNote::new("hola", "hello"))
        .expect("add note a");

    let mut deck_b = Deck::new("Spanish");
    deck_b
        .add(BasicNote::new("hola", "different back"))
        .expect("add note b");

    let id_a = deck_a.notes()[0].id().to_string();
    let id_b = deck_b.notes()[0].id().to_string();
    assert!(id_a.starts_with("afid:v1:"));
    assert_eq!(id_a, id_b);
}

#[test]
fn explicit_stable_id_remains_unchanged() {
    let mut deck = Deck::new("Spanish");
    deck.add(BasicNote::new("hola", "hello").stable_id("es-hola"))
        .expect("add explicit");
    assert_eq!(deck.notes()[0].id(), "es-hola");
}
```

- [ ] **Step 2: Run identity tests and confirm failure**

Run: `cargo test -p anki_forge --test deck_identity_tests -v`  
Expected: FAIL because non-explicit notes still produce `generated:*` instead of `afid:v1:*`.

- [ ] **Step 3: Add resolver core and replace generated fallback**

```rust
// anki_forge/Cargo.toml
[dependencies]
blake3 = "1"
unicode-normalization = "0.1"
```

```rust
// anki_forge/src/deck/identity.rs
use blake3::Hasher;
use serde::Serialize;
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedIdentity {
    pub stable_id: String,
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

pub fn normalize_text(value: &str) -> String {
    let nfc: String = value.nfc().collect();
    let lf = nfc.replace("\r\n", "\n").replace('\r', "\n");
    lf.trim().to_string()
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
    let stable_id = format!("afid:v1:{}", hasher.finalize().to_hex());
    Ok((stable_id, canonical_payload))
}
```

```rust
// anki_forge/src/deck/builders.rs (assign_identity match block replacement)
match requested {
    Some("") => anyhow::bail!("stable_id must not be blank"),
    Some(stable_id) => {
        anyhow::ensure!(
            !deck.used_note_ids.contains(stable_id),
            "AFID.STABLE_ID_DUPLICATE: {}",
            stable_id,
        );
        note.assign_stable_id(stable_id.to_string());
        note.assign_provenance(crate::deck::model::IdentityProvenance::ExplicitStableId);
        deck.identity_payload_by_id
            .insert(stable_id.to_string(), format!("explicit:{stable_id}"));
    }
    None => {
        let resolved = crate::deck::identity::resolve_inferred_identity(deck, note)?;
        anyhow::ensure!(
            !deck.used_note_ids.contains(&resolved.stable_id),
            "AFID.IDENTITY_COLLISION: {}",
            resolved.stable_id,
        );
        note.assign_stable_id(resolved.stable_id.clone());
        note.assign_provenance(resolved.provenance);
        deck.identity_payload_by_id
            .insert(resolved.stable_id.clone(), resolved.canonical_payload);
    }
}
```

- [ ] **Step 4: Re-run identity tests**

Run: `cargo test -p anki_forge --test deck_identity_tests -v`  
Expected: PASS (`afid:v1:*` for inferred identities, explicit ids preserved).

- [ ] **Step 5: Commit**

```bash
git add anki_forge/Cargo.toml anki_forge/src/deck/identity.rs anki_forge/src/deck/mod.rs anki_forge/src/deck/builders.rs anki_forge/tests/deck_identity_tests.rs
git commit -m "feat: add afid v1 identity resolver and inferred id assignment"
```

### Task 3: Implement Notetype-Level `identity_from_fields` + Escape Hatch Enforcement

**Files:**
- Modify: `anki_forge/src/deck/identity.rs`
- Modify: `anki_forge/src/deck/builders.rs`
- Modify: `anki_forge/src/deck/model.rs`
- Modify: `anki_forge/tests/deck_identity_tests.rs`

- [ ] **Step 1: Add failing tests for notetype fields and override reason requirement**

```rust
// anki_forge/tests/deck_identity_tests.rs
use anki_forge::{BasicNote, Deck};

#[test]
fn notetype_identity_fields_change_basic_recipe() {
    let mut deck = Deck::builder("Spanish")
        .basic_identity_from_fields(["Front", "Back"])
        .build();

    deck.add(BasicNote::new("hola", "hello")).expect("add first");
    deck.add(BasicNote::new("hola", "hi"))
        .expect("add second with different back");

    assert_ne!(deck.notes()[0].id(), deck.notes()[1].id());
}

#[test]
fn note_level_identity_override_requires_reason_code() {
    let mut deck = Deck::new("Spanish");
    let err = deck
        .add(BasicNote::new("hola", "hello").identity_from_fields(["Front", "Back"]))
        .expect_err("missing reason code should fail");

    assert!(err
        .to_string()
        .contains("AFID.NOTE_LEVEL_IDENTITY_OVERRIDE_REASON_REQUIRED"));
}
```

- [ ] **Step 2: Run identity tests and verify expected failures**

Run: `cargo test -p anki_forge --test deck_identity_tests -v`  
Expected: FAIL because resolver ignores deck notetype policy and does not enforce reason code for note-level override.

- [ ] **Step 3: Wire field-selection resolution order and reason-code check**

```rust
// anki_forge/src/deck/identity.rs (selection order helper)
pub enum FieldSelectionSource<'a> {
    NoteOverride(&'a [String]),
    NotetypeDefault(&'a [String]),
    StockDefault,
}

fn resolve_basic_field_selection<'a>(
    deck: &'a crate::deck::model::Deck,
    note: &'a crate::deck::model::BasicNote,
) -> anyhow::Result<(FieldSelectionSource<'a>, bool)> {
    if let Some(override_cfg) = note.identity_override.as_ref() {
        let reason = normalize_text(&override_cfg.reason_code);
        anyhow::ensure!(
            !reason.is_empty(),
            "AFID.NOTE_LEVEL_IDENTITY_OVERRIDE_REASON_REQUIRED"
        );
        anyhow::ensure!(
            !override_cfg.fields.is_empty(),
            "AFID.IDENTITY_FIELDS_EMPTY: basic override fields"
        );
        return Ok((FieldSelectionSource::NoteOverride(&override_cfg.fields), true));
    }

    if let Some(fields) = deck.identity_policy.basic.as_ref() {
        anyhow::ensure!(!fields.is_empty(), "AFID.IDENTITY_FIELDS_EMPTY: basic policy");
        return Ok((FieldSelectionSource::NotetypeDefault(fields), false));
    }

    Ok((FieldSelectionSource::StockDefault, false))
}
```

```rust
// anki_forge/src/deck/identity.rs (field extraction for Basic)
#[derive(Debug, Serialize)]
struct BasicComponents {
    selected_fields: Vec<(String, String)>,
}

fn build_basic_components(
    note: &crate::deck::model::BasicNote,
    selection: FieldSelectionSource<'_>,
) -> anyhow::Result<BasicComponents> {
    let mut selected_fields = Vec::new();
    let names: Vec<String> = match selection {
        FieldSelectionSource::NoteOverride(values) => values.to_vec(),
        FieldSelectionSource::NotetypeDefault(values) => values.to_vec(),
        FieldSelectionSource::StockDefault => vec!["Front".to_string()],
    };

    for name in names {
        let value = match name.as_str() {
            "Front" => normalize_text(&note.front),
            "Back" => normalize_text(&note.back),
            other => anyhow::bail!("AFID.IDENTITY_FIELD_NOT_FOUND: Basic.{other}"),
        };
        selected_fields.push((name, value));
    }

    let any_non_empty = selected_fields.iter().any(|(_, value)| !value.is_empty());
    anyhow::ensure!(any_non_empty, "AFID.IDENTITY_COMPONENT_EMPTY: basic");
    Ok(BasicComponents { selected_fields })
}
```

- [ ] **Step 4: Re-run identity tests**

Run: `cargo test -p anki_forge --test deck_identity_tests -v`  
Expected: PASS for notetype-field selection and missing-reason rejection.

- [ ] **Step 5: Commit**

```bash
git add anki_forge/src/deck/identity.rs anki_forge/src/deck/builders.rs anki_forge/src/deck/model.rs anki_forge/tests/deck_identity_tests.rs
git commit -m "feat: enforce notetype identity fields and note override reason"
```

### Task 4: Implement Cloze Recipe (`base_text_skeleton` + `deletions`)

**Files:**
- Modify: `anki_forge/src/deck/identity.rs`
- Modify: `anki_forge/tests/deck_identity_tests.rs`

- [ ] **Step 1: Add failing Cloze recipe tests**

```rust
// anki_forge/tests/deck_identity_tests.rs
use anki_forge::{ClozeNote, Deck};

#[test]
fn cloze_hint_change_does_not_change_identity() {
    let mut deck_a = Deck::new("Geo");
    deck_a
        .add(ClozeNote::new("Capital of {{c1::France::country}} is {{c2::Paris::city}}"))
        .expect("add a");

    let mut deck_b = Deck::new("Geo");
    deck_b
        .add(ClozeNote::new("Capital of {{c1::France::nation}} is {{c2::Paris::place}}"))
        .expect("add b");

    assert_eq!(deck_a.notes()[0].id(), deck_b.notes()[0].id());
}

#[test]
fn cloze_ord_change_changes_identity() {
    let mut deck_a = Deck::new("Geo");
    deck_a
        .add(ClozeNote::new("Capital of {{c1::France}} is {{c2::Paris}}"))
        .expect("add a");

    let mut deck_b = Deck::new("Geo");
    deck_b
        .add(ClozeNote::new("Capital of {{c2::France}} is {{c1::Paris}}"))
        .expect("add b");

    assert_ne!(deck_a.notes()[0].id(), deck_b.notes()[0].id());
}
```

- [ ] **Step 2: Run cloze tests and confirm failure**

Run: `cargo test -p anki_forge --test deck_identity_tests cloze_ -v`  
Expected: FAIL because current cloze inference ignores cloze structure (`ord`, `slot`, `hint`).

- [ ] **Step 3: Implement cloze parser and structured components**

```rust
// anki_forge/src/deck/identity.rs
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct ClozeDeletion {
    ord: u32,
    text: String,
    slot: usize,
}

#[derive(Debug, Serialize)]
struct ClozeComponents {
    base_text_skeleton: String,
    deletions: Vec<ClozeDeletion>,
}

fn parse_cloze_components(input: &str) -> anyhow::Result<ClozeComponents> {
    let mut skeleton = String::new();
    let mut deletions = Vec::new();
    let mut cursor = 0usize;
    let mut slot = 0usize;

    while let Some(start_rel) = input[cursor..].find("{{c") {
        let start = cursor + start_rel;
        skeleton.push_str(&normalize_text(&input[cursor..start]));
        let end_rel = input[start..]
            .find("}}")
            .ok_or_else(|| anyhow::anyhow!("AFID.IDENTITY_COMPONENT_EMPTY: malformed cloze"))?;
        let end = start + end_rel + 2;
        let token = &input[start + 2..end - 2];

        let mut parts = token.splitn(2, "::");
        let ord_part = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("AFID.IDENTITY_COMPONENT_EMPTY: missing cloze ord"))?;
        let body_part = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("AFID.IDENTITY_COMPONENT_EMPTY: missing cloze body"))?;
        let ord = ord_part
            .strip_prefix('c')
            .ok_or_else(|| anyhow::anyhow!("AFID.IDENTITY_COMPONENT_EMPTY: invalid cloze ord"))?
            .parse::<u32>()?;

        let deleted_text = body_part
            .splitn(2, "::")
            .next()
            .map(normalize_text)
            .unwrap_or_default();

        skeleton.push_str("[CLOZE]");
        deletions.push(ClozeDeletion {
            ord,
            text: deleted_text,
            slot,
        });
        slot += 1;
        cursor = end;
    }

    skeleton.push_str(&normalize_text(&input[cursor..]));
    anyhow::ensure!(
        !deletions.is_empty(),
        "AFID.IDENTITY_COMPONENT_EMPTY: cloze deletions"
    );
    Ok(ClozeComponents {
        base_text_skeleton: skeleton,
        deletions,
    })
}
```

- [ ] **Step 4: Re-run cloze tests**

Run: `cargo test -p anki_forge --test deck_identity_tests cloze_ -v`  
Expected: PASS (`hint` changes ignored, `ord`/`slot` changes reflected).

- [ ] **Step 5: Commit**

```bash
git add anki_forge/src/deck/identity.rs anki_forge/tests/deck_identity_tests.rs
git commit -m "feat: add cloze identity recipe with skeleton and deletions"
```

### Task 5: Implement IO Recipe With Integer Quantized Masks

**Files:**
- Modify: `anki_forge/src/deck/identity.rs`
- Modify: `anki_forge/tests/deck_identity_tests.rs`

- [ ] **Step 1: Add failing IO quantization and notetype-key tests**

```rust
// anki_forge/tests/deck_identity_tests.rs
use anki_forge::{Deck, IoMode, MediaSource};

#[test]
fn io_mask_order_does_not_change_identity() {
    let mut deck_a = Deck::new("Anatomy");
    let image_a = deck_a
        .media()
        .add(MediaSource::from_bytes("heart.png", vec![1, 2, 3]))
        .expect("media a");
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
        .add(MediaSource::from_bytes("heart.png", vec![1, 2, 3]))
        .expect("media b");
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
fn same_front_text_in_basic_and_cloze_produces_different_ids() {
    let mut deck = Deck::new("Mixed");
    deck.add(anki_forge::BasicNote::new("Paris", "city"))
        .expect("basic");
    deck.add(anki_forge::ClozeNote::new("{{c1::Paris}} is a city"))
        .expect("cloze");

    assert_ne!(deck.notes()[0].id(), deck.notes()[1].id());
}
```

- [ ] **Step 2: Run IO tests and confirm failure**

Run: `cargo test -p anki_forge --test deck_identity_tests io_ -v`  
Expected: FAIL because masks are not canonicalized by quantized geometry and notetype key separation is not applied everywhere.

- [ ] **Step 3: Implement quantized IO mask components and include `notetype_key`**

```rust
// anki_forge/src/deck/identity.rs
#[derive(Debug, Serialize, PartialEq, Eq, Clone)]
struct IoMaskComponent {
    x_q: u32,
    y_q: u32,
    w_q: u32,
    h_q: u32,
}

#[derive(Debug, Serialize)]
struct IoComponents {
    image_anchor: String,
    occlusion_mode: String,
    normalized_masks: Vec<IoMaskComponent>,
}

fn quantize_ratio(numerator: u32, denominator: u32) -> u32 {
    let denom = denominator.max(1);
    let ratio = (numerator as f64 / denom as f64).clamp(0.0, 1.0);
    (ratio * 10000.0).round() as u32
}

fn derived_canvas_size(rects: &[crate::deck::model::IoRect]) -> (u32, u32) {
    let width = rects
        .iter()
        .map(|rect| rect.x.saturating_add(rect.width))
        .max()
        .unwrap_or(1)
        .max(1);
    let height = rects
        .iter()
        .map(|rect| rect.y.saturating_add(rect.height))
        .max()
        .unwrap_or(1)
        .max(1);
    (width, height)
}

fn build_io_components(
    deck: &crate::deck::model::Deck,
    note: &crate::deck::model::IoNote,
) -> anyhow::Result<IoComponents> {
    let media = deck
        .media
        .get(note.image.name())
        .ok_or_else(|| anyhow::anyhow!("AFID.IDENTITY_COMPONENT_EMPTY: missing io media"))?;
    let image_anchor = media.sha1_hex.clone();
    let (canvas_w, canvas_h) = derived_canvas_size(&note.rects);

    let mut normalized_masks: Vec<IoMaskComponent> = note
        .rects
        .iter()
        .map(|rect| IoMaskComponent {
            x_q: quantize_ratio(rect.x, canvas_w),
            y_q: quantize_ratio(rect.y, canvas_h),
            w_q: quantize_ratio(rect.width, canvas_w),
            h_q: quantize_ratio(rect.height, canvas_h),
        })
        .collect();

    normalized_masks.sort_by_key(|mask| (mask.x_q, mask.y_q, mask.w_q, mask.h_q));

    anyhow::ensure!(
        !normalized_masks.is_empty(),
        "AFID.IDENTITY_COMPONENT_EMPTY: io masks"
    );

    Ok(IoComponents {
        image_anchor,
        occlusion_mode: format!("{:?}", note.mode),
        normalized_masks,
    })
}
```

- [ ] **Step 4: Re-run IO/notetype-key tests**

Run: `cargo test -p anki_forge --test deck_identity_tests -v`  
Expected: PASS for mask-order stability and cross-notetype id separation.

- [ ] **Step 5: Commit**

```bash
git add anki_forge/src/deck/identity.rs anki_forge/tests/deck_identity_tests.rs
git commit -m "feat: add io quantized mask identity and notetype-key separation"
```

### Task 6: Enforce Blocking Duplicate/Collision Errors And AFID Diagnostics

**Files:**
- Modify: `anki_forge/src/deck/validation.rs`
- Modify: `anki_forge/src/deck/builders.rs`
- Modify: `anki_forge/tests/deck_validation_tests.rs`

- [ ] **Step 1: Add failing validation tests for duplicate/collision blocking behavior**

```rust
// anki_forge/tests/deck_validation_tests.rs
use anki_forge::{BasicNote, Deck, ValidationCode};

#[test]
fn explicit_stable_id_duplicate_is_error() {
    let mut deck = Deck::new("Spanish");
    deck.add(BasicNote::new("hola", "hello").stable_id("es-hola"))
        .expect("first");

    let err = deck
        .add(BasicNote::new("adios", "bye").stable_id("es-hola"))
        .expect_err("duplicate explicit stable id");
    assert!(err.to_string().contains("AFID.STABLE_ID_DUPLICATE"));
}

#[test]
fn inferred_duplicate_payload_is_error() {
    let mut deck = Deck::new("Spanish");
    deck.add(BasicNote::new("hola", "hello")).expect("first");

    let err = deck
        .add(BasicNote::new("hola", "hello"))
        .expect_err("duplicate payload");
    assert!(err.to_string().contains("AFID.IDENTITY_DUPLICATE_PAYLOAD"));
}

#[test]
fn note_level_override_emits_warning_diagnostic() {
    let mut deck = Deck::new("Spanish");
    deck.add(
        BasicNote::new("bank", "river")
            .identity_from_fields(["Front", "Back"])
            .identity_override_reason_code("sense-disambiguation"),
    )
    .expect("override note");

    let report = deck.validate_report().expect("report");
    assert!(report
        .diagnostics()
        .iter()
        .any(|item| item.code == ValidationCode::NoteLevelIdentityOverrideUsed));
}
```

- [ ] **Step 2: Run validation tests and confirm failure**

Run: `cargo test -p anki_forge --test deck_validation_tests -v`  
Expected: FAIL because duplicate inferred payload is not blocked and AFID warning code does not exist yet.

- [ ] **Step 3: Add AFID validation codes and collision classifier wiring**

```rust
// anki_forge/src/deck/validation.rs
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ValidationCode {
    NotetypeIdentityFieldsRequired,
    IdentityFieldNotFound,
    IdentityFieldsEmpty,
    IdentityComponentEmpty,
    NoteLevelIdentityOverrideReasonRequired,
    IdentityDuplicatePayload,
    IdentityCollision,
    StableIdDuplicate,
    NoteLevelIdentityOverrideUsed,
    EmptyIoMasks,
    UnknownMediaRef,
}
```

```rust
// anki_forge/src/deck/builders.rs (duplicate/collision section)
if let Some(existing_payload) = deck.identity_payload_by_id.get(&resolved.stable_id) {
    if existing_payload == &resolved.canonical_payload {
        anyhow::bail!("AFID.IDENTITY_DUPLICATE_PAYLOAD: {}", resolved.stable_id);
    } else {
        anyhow::bail!("AFID.IDENTITY_COLLISION: {}", resolved.stable_id);
    }
}
```

```rust
// anki_forge/src/deck/builders.rs (validate_report warning injection)
if note.uses_note_level_identity_override() {
    diagnostics.push(ValidationDiagnostic {
        code: ValidationCode::NoteLevelIdentityOverrideUsed,
        message: format!("note '{}' used note-level identity override", note.id()),
        severity: "warning".into(),
    });
}
```

- [ ] **Step 4: Re-run validation tests**

Run: `cargo test -p anki_forge --test deck_validation_tests -v`  
Expected: PASS with AFID duplicate/collision errors and override warning diagnostics.

- [ ] **Step 5: Commit**

```bash
git add anki_forge/src/deck/validation.rs anki_forge/src/deck/builders.rs anki_forge/tests/deck_validation_tests.rs
git commit -m "feat: enforce blocking afid duplicate and collision diagnostics"
```

### Task 7: Finalize Docs, Regression Coverage, And Full Verification

**Files:**
- Modify: `README.md`
- Modify: `anki_forge/src/deck/mod.rs`
- Test: `anki_forge/tests/deck_model_tests.rs`
- Test: `anki_forge/tests/deck_identity_tests.rs`
- Test: `anki_forge/tests/deck_validation_tests.rs`

- [ ] **Step 1: Update README identity section**

```markdown
<!-- README.md -->
### Stable note identity defaults

`Deck` note ids are explicit-first:

1. `stable_id("es-hola")` is used as-is.
2. If omitted, `anki_forge` infers deterministic `afid:v1:*` ids from note content.

Identity inference supports:

1. notetype-level defaults via `Deck::builder("Spanish").basic_identity_from_fields(["Front"])`, `cloze_identity_from_fields(["Text"])`, and `image_occlusion_identity_from_fields(["Image", "Rects", "Mode"])`
2. note-level overrides via `identity_from_fields(["Front", "Back"])` plus required `identity_override_reason_code("sense-disambiguation")`

Duplicate/collision behavior is blocking:

1. explicit duplicate stable id -> error
2. inferred same id + same payload -> error
3. inferred same id + different payload -> error
```

- [ ] **Step 2: Run focused deck tests**

Run: `cargo test -p anki_forge --test deck_model_tests --test deck_identity_tests --test deck_validation_tests -v`  
Expected: PASS for all deck identity, model, and validation behavior.

- [ ] **Step 3: Run full crate tests**

Run: `cargo test -p anki_forge -v`  
Expected: PASS with no regression in product/runtime tests.

- [ ] **Step 4: Run formatting and lints**

Run: `cargo fmt --all --check`  
Expected: PASS (no formatting drift).

Run: `cargo clippy -p anki_forge --tests -- -D warnings`  
Expected: PASS (no new warnings).

- [ ] **Step 5: Commit**

```bash
git add README.md anki_forge/src/deck/mod.rs anki_forge/tests/deck_model_tests.rs anki_forge/tests/deck_identity_tests.rs anki_forge/tests/deck_validation_tests.rs
git commit -m "docs: describe afid identity defaults and verify deck identity flow"
```

## Plan Self-Review

### 1. Spec coverage check

- Explicit-first resolver: Task 2.
- No generated fallback: Task 2 + Task 6.
- Notetype-level `identity_from_fields`: Task 1 + Task 3.
- Note-level escape hatch + reason code: Task 1 + Task 3 + Task 6.
- `notetype_key` in payload: Task 5.
- Blocking duplicate/collision semantics: Task 6.
- Text-only normalization: Task 2 + Task 4 + Task 5.
- IO integer-only quantized masks: Task 5.
- Diagnostics/provenance visibility: Task 2 + Task 6.

### 2. Placeholder scan

- No `TODO`, `TBD`, or deferred implementation placeholders.
- Every code-changing step includes concrete code snippets.
- Every verification step includes concrete commands and expected outcomes.

### 3. Type/signature consistency check

- `DeckIdentityPolicy`, `NoteIdentityOverride`, and `IdentityProvenance` introduced in Task 1 and reused consistently in Tasks 2-6.
- AFID validation codes introduced in Task 6 align with spec naming.
- Resolver output shape (`stable_id`, `canonical_payload`, `provenance`, `used_override`) stays consistent across builder wiring and validation.
