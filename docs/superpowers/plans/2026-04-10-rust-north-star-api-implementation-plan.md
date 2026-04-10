# Rust North-Star API Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver the approved crate-root Rust authoring facade for `anki_forge` with `Deck`, `Package`, `MediaSource`, opaque `MediaRef`, `ValidationReport`, and `BuildResult`, while preserving the existing `product -> lowering -> normalize -> build` pipeline behind the new author-facing surface.

**Architecture:** Add a new internal `anki_forge::deck` module that owns the author-facing root model, ordered note storage, identity assignment, media registry, validation, and export helpers. Lower the root facade one-way into the existing `product` layer for stock note-type shaping, then export through shared runtime-default helpers and writer-core artifact refs instead of hardcoded config strings or guessed filenames.

**Tech Stack:** Rust workspace (`cargo`, `serde`, `anyhow`), existing `anki_forge`, `authoring_core`, and `writer_core` crates, `tempfile`, `base64`, fixture-driven tests, Markdown docs/examples

---

## Scope Check

This plan covers one coherent subsystem: the Rust crate-root authoring facade approved in
`docs/superpowers/specs/2026-04-10-rust-north-star-api-design.md`.

The first pass intentionally includes:

1. `Deck`, `Package`, and ordered `Vec<DeckNote>` storage
2. `deck.add(note)` as the primitive plus `basic()/cloze()/image_occlusion()` sugar
3. `stable_id` assignment rules, generated ids, and structured validation diagnostics
4. opaque `MediaRef` plus filename-keyed media registry collision handling
5. one-way lowering into `product` while preserving note order and tags
6. `to_apkg_bytes()`, `write_to(...)`, `write_apkg(...)`, and `build(...)` backed by shared runtime defaults and real artifact refs

The first pass explicitly excludes:

1. `from_product_document()`
2. collection-wide `.colpkg` export
3. root-level custom notetype authoring
4. ellipse/polygon Image Occlusion geometry helpers

## Execution Prerequisite

Before running any implementation task below, create or switch to a dedicated worktree for this
plan, for example:

```bash
git worktree add ../anki-forge-north-star-api -b codex/rust-north-star-api
cd ../anki-forge-north-star-api
```

Execute the plan in that worktree, not on top of unrelated in-progress work.

## File Structure Map

### New crate-root facade

- Modify: `anki_forge/Cargo.toml` - add `base64` and `tempfile` for named media payload encoding and bytes-first export helpers
- Modify: `anki_forge/src/lib.rs` - re-export the new crate-root authoring types
- Create: `anki_forge/src/deck/mod.rs` - public exports for the root facade
- Create: `anki_forge/src/deck/model.rs` - `Deck`, `Package`, ordered `Vec<DeckNote>`, owned note DTOs, opaque `MediaRef`, package/build DTOs, and common accessors
- Create: `anki_forge/src/deck/builders.rs` - `Deck::builder()`, `deck.add(note)`, `add_basic(...)`, and lane builders for `basic()/cloze()/image_occlusion()`
- Create: `anki_forge/src/deck/media.rs` - `MediaSource`, filename-keyed registry, duplicate-name collision rules, and media lookup
- Create: `anki_forge/src/deck/lowering.rs` - one-way `Deck`/`Package` into `product::ProductDocument` and `AuthoringDocument`
- Create: `anki_forge/src/deck/validation.rs` - `ValidationReport`, `ValidationCode`, `ValidationDiagnostic`, local insertion checks, and full preflight validation
- Create: `anki_forge/src/deck/export.rs` - `to_apkg_bytes()`, `write_to(...)`, `write_apkg(...)`, `build(...)`, `BuildResult`, and `Deck`/`Package` symmetry

### Product/runtime/writer bridge changes

- Create: `anki_forge/src/product/stock.rs` - stock notetype ids plus shared stock helpers used by the root facade during one-way lowering
- Modify: `anki_forge/src/product/mod.rs` - re-export stock ids/helpers for root-facade lowering
- Modify: `anki_forge/src/product/model.rs` - add `tags` to stock note variants so root-facade tags survive lowering
- Modify: `anki_forge/src/product/builders.rs` - keep stock-note constructors aligned with the updated tag-bearing structs
- Modify: `anki_forge/src/product/lowering.rs` - pass stock-note tags through to `AuthoringNote.tags`
- Create: `anki_forge/src/runtime/defaults.rs` - shared helper that discovers the workspace runtime and loads default writer policy/build context from the current bundle
- Modify: `anki_forge/src/runtime/mod.rs` - export the new default-loading helper
- Modify: `writer_core/src/lib.rs` - re-export artifact-ref path resolution helper
- Modify: `writer_core/src/inspect.rs` - make artifact-ref-to-path resolution public

### Tests and docs

- Create: `anki_forge/tests/deck_model_tests.rs` - root DTO, ordered storage, `deck.add(note)`, and package identity tests
- Create: `anki_forge/tests/deck_validation_tests.rs` - generated-id diagnostics, duplicate/blank stable-id failures, lightweight insertion checks, and media-registry diagnostics
- Create: `anki_forge/tests/deck_lowering_tests.rs` - order-preserving `Deck` to `ProductDocument` / `AuthoringDocument` lowering tests for `Basic`, `Cloze`, `Image Occlusion`, tags, and named media
- Create: `anki_forge/tests/deck_export_tests.rs` - `to_apkg_bytes()`, `write_to(...)`, `write_apkg(...)`, `build(...)`, and `Package::single(...)` symmetry tests
- Modify: `anki_forge/tests/product_lowering_tests.rs` - cover stock-note tags flowing through `product` lowering
- Modify: `anki_forge/tests/product_model_tests.rs` - cover updated stock note structs with tags
- Create: `anki_forge/examples/deck_basic_flow.rs` - new north-star example from `Deck` to `.apkg`
- Modify: `README.md` - put `Deck` in the main Rust tutorial, document stable-id defaults, and explicitly point advanced users to `product` / `runtime`

## Implementation Notes

- Keep `.apkg` semantics aligned with Anki deck packages: one root `Deck`, no multi-root `.apkg` API, and any future collection-wide export belongs to a later `.colpkg` design.
- `Package` must stay symmetric with `Deck` for `to_apkg_bytes()`, `write_to(...)`, `write_apkg(...)`, and `build(...)`. `Deck` methods should delegate to `Package::single(self.clone())`.
- Internally, `Deck` must store notes in one ordered collection such as `Vec<DeckNote>`. Do not split the internal state into `basic_notes`, `cloze_notes`, and `io_notes` collections.
- `deck.add(note)` is the root primitive. `deck.basic()/cloze()/image_occlusion()` and `deck.add_basic(...)` are sugar over creating owned DTOs and calling `deck.add(note)`.
- Use `stable_id` as the only default-layer identity term. If a note is added without a user-supplied `stable_id`, allocate a unique generated id instead of lowering an empty string. Duplicate or blank explicit stable ids are hard errors. Generated ids must remain diagnosable so users can see they are on a non-update-friendly path.
- `Package` must also have optional package-level stable identity, with a default derived from the root deck when not explicitly set.
- `MediaRef` must be opaque. The registry must be keyed by exported filename, reuse same-name same-content registrations, and error on same-name different-content collisions.
- `from_product_document()` is intentionally out of scope for this first pass. Do not promise a reverse bridge until the project has a lossless contract for IO geometry/media round-tripping.
- Do not force `validate()` into the happy path. `add()` does lightweight local checks, `validate_report()` / `validate()` provide optional preflight, and every export/build path must run full validation internally.
- The root export layer must load the default writer policy and build context through shared runtime/default helpers. Do not duplicate config strings in the facade layer.
- `BuildResult` must wrap the underlying `PackageBuildResult` and store resolved concrete artifact paths from returned refs. Do not guess `package.apkg` or `staging/manifest.json` in the root facade.
- `Image Occlusion` first pass intentionally supports `rect(...)` only. Document that this is a narrowed initial geometry helper, not full parity with native ellipse/polygon editing.

### Task 1: Add the root model, ordered note storage, and `deck.add(note)`

**Files:**
- Modify: `anki_forge/Cargo.toml`
- Modify: `anki_forge/src/lib.rs`
- Create: `anki_forge/src/deck/mod.rs`
- Create: `anki_forge/src/deck/model.rs`
- Create: `anki_forge/src/deck/builders.rs`
- Test: `anki_forge/tests/deck_model_tests.rs`

- [ ] **Step 1: Write the failing ordered-storage and package-identity tests**

```rust
// anki_forge/tests/deck_model_tests.rs
use anki_forge::{BasicNote, ClozeNote, Deck, DeckNote, Package};

#[test]
fn deck_add_preserves_mixed_note_order() {
    let mut deck = Deck::builder("Mixed")
        .stable_id("mixed-v1")
        .build();

    deck.add(BasicNote::new("front 1", "back 1").stable_id("basic-1"))
        .expect("add first basic");
    deck.add(ClozeNote::new("A {{c1::cloze}} card").stable_id("cloze-1"))
        .expect("add cloze");
    deck.add(BasicNote::new("front 2", "back 2").stable_id("basic-2"))
        .expect("add second basic");

    assert_eq!(deck.notes().len(), 3);
    assert!(matches!(&deck.notes()[0], DeckNote::Basic(_)));
    assert!(matches!(&deck.notes()[1], DeckNote::Cloze(_)));
    assert!(matches!(&deck.notes()[2], DeckNote::Basic(_)));
    assert_eq!(deck.stable_id().as_deref(), Some("mixed-v1"));
}

#[test]
fn package_single_can_override_package_stable_id_without_changing_root_deck() {
    let deck = Deck::builder("Mixed")
        .stable_id("mixed-v1")
        .build();

    let package = Package::single(deck).with_stable_id("package-v1");

    assert_eq!(package.stable_id().as_deref(), Some("package-v1"));
    assert_eq!(package.root_deck().stable_id().as_deref(), Some("mixed-v1"));
}
```

- [ ] **Step 2: Run the model tests to verify they fail**

Run: `cargo test -p anki_forge --test deck_model_tests -v`
Expected: FAIL with unresolved imports for `Deck`, `DeckNote`, `BasicNote`, `ClozeNote`, missing `deck.add(...)`, or missing package identity accessors.

- [ ] **Step 3: Add the root model, crate-root exports, and generic `deck.add(note)` primitive**

```rust
// anki_forge/src/lib.rs
mod deck;
pub mod product;
pub mod runtime;

pub use deck::*;

// keep the existing authoring_core and writer_core re-exports below unchanged
```

```rust
// anki_forge/src/deck/mod.rs
pub mod builders;
pub mod model;

pub use model::{BasicNote, ClozeNote, Deck, DeckNote, IoMode, IoNote, MediaRef, Package};
```

```rust
// anki_forge/src/deck/model.rs
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deck {
    pub(crate) name: String,
    pub(crate) stable_id: Option<String>,
    pub(crate) notes: Vec<DeckNote>,
    pub(crate) next_generated_note_id: u64,
    pub(crate) media: BTreeMap<String, RegisteredMedia>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Package {
    pub(crate) root_deck: Deck,
    pub(crate) stable_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeckNote {
    Basic(BasicNote),
    Cloze(ClozeNote),
    ImageOcclusion(IoNote),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BasicNote {
    pub(crate) id: String,
    pub(crate) stable_id: Option<String>,
    pub(crate) front: String,
    pub(crate) back: String,
    pub(crate) tags: Vec<String>,
    pub(crate) generated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClozeNote {
    pub(crate) id: String,
    pub(crate) stable_id: Option<String>,
    pub(crate) text: String,
    pub(crate) extra: String,
    pub(crate) tags: Vec<String>,
    pub(crate) generated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IoRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IoMode {
    HideAllGuessOne,
    HideOneGuessOne,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaRef(String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisteredMedia {
    pub(crate) name: String,
    pub(crate) mime: String,
    pub(crate) data_base64: String,
    pub(crate) sha1_hex: String,
}

impl Deck {
    pub fn builder(name: impl Into<String>) -> crate::deck::builders::DeckBuilder {
        crate::deck::builders::DeckBuilder::new(name)
    }

    pub fn new(name: impl Into<String>) -> Self {
        crate::deck::builders::DeckBuilder::new(name).build()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn stable_id(&self) -> Option<&str> {
        self.stable_id.as_deref()
    }

    pub fn notes(&self) -> &[DeckNote] {
        &self.notes
    }
}

impl Package {
    pub fn single(root_deck: Deck) -> Self {
        let stable_id = root_deck.stable_id.clone();
        Self { root_deck, stable_id }
    }

    pub fn with_stable_id(mut self, stable_id: impl Into<String>) -> Self {
        self.stable_id = Some(stable_id.into());
        self
    }

    pub fn root_deck(&self) -> &Deck {
        &self.root_deck
    }

    pub fn stable_id(&self) -> Option<&str> {
        self.stable_id.as_deref()
    }
}

impl BasicNote {
    pub fn new(front: impl Into<String>, back: impl Into<String>) -> Self {
        Self {
            id: String::new(),
            stable_id: None,
            front: front.into(),
            back: back.into(),
            tags: Vec::new(),
            generated: false,
        }
    }

    pub fn stable_id(mut self, stable_id: impl Into<String>) -> Self {
        self.stable_id = Some(stable_id.into());
        self
    }
}

impl ClozeNote {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            id: String::new(),
            stable_id: None,
            text: text.into(),
            extra: String::new(),
            tags: Vec::new(),
            generated: false,
        }
    }

    pub fn stable_id(mut self, stable_id: impl Into<String>) -> Self {
        self.stable_id = Some(stable_id.into());
        self
    }
}

impl From<BasicNote> for DeckNote {
    fn from(note: BasicNote) -> Self {
        Self::Basic(note)
    }
}

impl From<ClozeNote> for DeckNote {
    fn from(note: ClozeNote) -> Self {
        Self::Cloze(note)
    }
}

impl From<IoNote> for DeckNote {
    fn from(note: IoNote) -> Self {
        Self::ImageOcclusion(note)
    }
}
```

```rust
// anki_forge/src/deck/builders.rs
use crate::deck::model::{BasicNote, ClozeNote, Deck, DeckNote};

pub struct DeckBuilder {
    name: String,
    stable_id: Option<String>,
}

impl DeckBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            stable_id: None,
        }
    }

    pub fn stable_id(mut self, stable_id: impl Into<String>) -> Self {
        self.stable_id = Some(stable_id.into());
        self
    }

    pub fn build(self) -> Deck {
        Deck {
            name: self.name,
            stable_id: self.stable_id,
            notes: Vec::new(),
            next_generated_note_id: 1,
            media: Default::default(),
        }
    }
}

impl Deck {
    pub fn add(&mut self, note: impl Into<DeckNote>) -> anyhow::Result<()> {
        let mut note = note.into();
        assign_identity(self, &mut note)?;
        self.notes.push(note);
        Ok(())
    }
}

fn assign_identity(deck: &mut Deck, note: &mut DeckNote) -> anyhow::Result<()> {
    let requested = note.requested_stable_id().map(str::trim);
    match requested {
        Some("") => anyhow::bail!("stable_id must not be blank"),
        Some(stable_id) => {
            anyhow::ensure!(
                deck.notes.iter().all(|existing| existing.id() != stable_id),
                "duplicate stable_id: {}",
                stable_id,
            );
            note.assign_stable_id(stable_id.to_string());
        }
        None => {
            let generated = format!("generated:{}:{}", deck.name(), deck.next_generated_note_id);
            deck.next_generated_note_id += 1;
            note.assign_generated_id(generated);
        }
    }
    Ok(())
}

impl DeckNote {
    pub fn id(&self) -> &str {
        match self {
            Self::Basic(note) => &note.id,
            Self::Cloze(note) => &note.id,
            Self::ImageOcclusion(note) => &note.id,
        }
    }

    pub fn requested_stable_id(&self) -> Option<&str> {
        match self {
            Self::Basic(note) => note.stable_id.as_deref(),
            Self::Cloze(note) => note.stable_id.as_deref(),
            Self::ImageOcclusion(note) => note.stable_id.as_deref(),
        }
    }

    pub fn assign_stable_id(&mut self, stable_id: String) {
        match self {
            Self::Basic(note) => {
                note.id = stable_id.clone();
                note.stable_id = Some(stable_id);
                note.generated = false;
            }
            Self::Cloze(note) => {
                note.id = stable_id.clone();
                note.stable_id = Some(stable_id);
                note.generated = false;
            }
            Self::ImageOcclusion(note) => {
                note.id = stable_id.clone();
                note.stable_id = Some(stable_id);
                note.generated = false;
            }
        }
    }

    pub fn assign_generated_id(&mut self, id: String) {
        match self {
            Self::Basic(note) => {
                note.id = id;
                note.generated = true;
            }
            Self::Cloze(note) => {
                note.id = id;
                note.generated = true;
            }
            Self::ImageOcclusion(note) => {
                note.id = id;
                note.generated = true;
            }
        }
    }

    pub fn generated(&self) -> bool {
        match self {
            Self::Basic(note) => note.generated,
            Self::Cloze(note) => note.generated,
            Self::ImageOcclusion(note) => note.generated,
        }
    }
}
```

- [ ] **Step 4: Run the model tests to verify they pass**

Run: `cargo test -p anki_forge --test deck_model_tests -v`
Expected: PASS with `deck_add_preserves_mixed_note_order` and `package_single_can_override_package_stable_id_without_changing_root_deck`.

- [ ] **Step 5: Commit**

```bash
git add anki_forge/Cargo.toml anki_forge/src/lib.rs anki_forge/src/deck/mod.rs anki_forge/src/deck/model.rs anki_forge/src/deck/builders.rs anki_forge/tests/deck_model_tests.rs
git commit -m "feat: add ordered deck root model"
```

### Task 2: Add lane sugar, lightweight insertion checks, and structured validation diagnostics

**Files:**
- Modify: `anki_forge/src/deck/model.rs`
- Modify: `anki_forge/src/deck/builders.rs`
- Create: `anki_forge/src/deck/validation.rs`
- Modify: `anki_forge/src/deck/mod.rs`
- Test: `anki_forge/tests/deck_validation_tests.rs`

- [ ] **Step 1: Write the failing identity and validation tests**

```rust
// anki_forge/tests/deck_validation_tests.rs
use anki_forge::{Deck, ValidationCode};

#[test]
fn add_basic_generates_non_empty_id_and_validate_report_warns() {
    let mut deck = Deck::new("Spanish");
    deck.add_basic("hola", "hello").expect("add basic note");

    assert!(deck.notes()[0].id().starts_with("generated:"));

    let report = deck.validate_report().expect("validation report");
    assert!(report
        .diagnostics()
        .iter()
        .any(|item| item.code == ValidationCode::MissingStableId));
}

#[test]
fn blank_explicit_stable_id_fails_at_add_time() {
    let mut deck = Deck::new("Spanish");

    let err = deck
        .basic()
        .note("hola", "hello")
        .stable_id("   ")
        .add()
        .expect_err("blank stable id must fail");

    assert!(err.to_string().contains("stable_id"));
}
```

- [ ] **Step 2: Run the validation tests to verify they fail**

Run: `cargo test -p anki_forge --test deck_validation_tests -v`
Expected: FAIL because lane builders, `add_basic(...)`, and `validate_report()` do not exist yet.

- [ ] **Step 3: Implement lane builders, lightweight checks, and typed validation diagnostics**

```rust
// anki_forge/src/deck/validation.rs
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ValidationCode {
    MissingStableId,
    DuplicateStableId,
    BlankStableId,
    EmptyIoMasks,
    UnknownMediaRef,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ValidationDiagnostic {
    pub code: ValidationCode,
    pub message: String,
    pub severity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub struct ValidationReport {
    diagnostics: Vec<ValidationDiagnostic>,
}

impl ValidationReport {
    pub fn diagnostics(&self) -> &[ValidationDiagnostic] {
        &self.diagnostics
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|item| item.severity == "error")
    }
}
```

```rust
// anki_forge/src/deck/mod.rs
pub mod validation;

pub use validation::{ValidationCode, ValidationDiagnostic, ValidationReport};
```

```rust
// append to anki_forge/src/deck/builders.rs
use crate::deck::model::{IoMode, IoNote, IoRect, MediaRef};
use crate::deck::validation::{ValidationCode, ValidationDiagnostic, ValidationReport};

pub struct BasicLane<'a> {
    deck: &'a mut Deck,
}

pub struct ClozeLane<'a> {
    deck: &'a mut Deck,
}

pub struct IoLane<'a> {
    deck: &'a mut Deck,
}

pub struct BasicDraft<'a> {
    deck: &'a mut Deck,
    note: BasicNote,
}

pub struct ClozeDraft<'a> {
    deck: &'a mut Deck,
    note: ClozeNote,
}

pub struct IoDraft<'a> {
    deck: &'a mut Deck,
    note: IoNote,
}

impl BasicNote {
    pub fn tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.tags = tags.into_iter().map(Into::into).collect();
        self
    }
}

impl ClozeNote {
    pub fn extra(mut self, extra: impl Into<String>) -> Self {
        self.extra = extra.into();
        self
    }

    pub fn tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.tags = tags.into_iter().map(Into::into).collect();
        self
    }
}

impl IoNote {
    pub fn new(image: MediaRef) -> Self {
        Self {
            id: String::new(),
            stable_id: None,
            image,
            mode: IoMode::HideAllGuessOne,
            rects: Vec::new(),
            header: String::new(),
            back_extra: String::new(),
            comments: String::new(),
            tags: Vec::new(),
            generated: false,
        }
    }

    pub fn stable_id(mut self, stable_id: impl Into<String>) -> Self {
        self.stable_id = Some(stable_id.into());
        self
    }
}

impl Deck {
    pub fn add_basic(
        &mut self,
        front: impl Into<String>,
        back: impl Into<String>,
    ) -> anyhow::Result<()> {
        self.add(BasicNote::new(front, back))
    }

    pub fn basic(&mut self) -> BasicLane<'_> {
        BasicLane { deck: self }
    }

    pub fn cloze(&mut self) -> ClozeLane<'_> {
        ClozeLane { deck: self }
    }

    pub fn image_occlusion(&mut self) -> IoLane<'_> {
        IoLane { deck: self }
    }

    pub fn validate_report(&self) -> anyhow::Result<ValidationReport> {
        let mut diagnostics = Vec::new();
        let mut seen_stable_ids = std::collections::BTreeSet::new();
        for note in &self.notes {
            match note.requested_stable_id().map(str::trim) {
                Some("") => diagnostics.push(ValidationDiagnostic {
                    code: ValidationCode::BlankStableId,
                    message: format!("note '{}' has a blank explicit stable_id", note.id()),
                    severity: "error".into(),
                }),
                Some(stable_id) => {
                    if !seen_stable_ids.insert(stable_id.to_string()) {
                        diagnostics.push(ValidationDiagnostic {
                            code: ValidationCode::DuplicateStableId,
                            message: format!("stable_id '{}' is duplicated", stable_id),
                            severity: "error".into(),
                        });
                    }
                }
                None if note.generated() => diagnostics.push(ValidationDiagnostic {
                    code: ValidationCode::MissingStableId,
                    message: format!("note '{}' was assigned a generated id", note.id()),
                    severity: "warning".into(),
                }),
                None => {}
            }
            if let DeckNote::ImageOcclusion(io) = note {
                if io.rects.is_empty() {
                    diagnostics.push(ValidationDiagnostic {
                        code: ValidationCode::EmptyIoMasks,
                        message: format!("image occlusion note '{}' requires at least one rect", io.id),
                        severity: "error".into(),
                    });
                }
            }
        }
        Ok(ValidationReport { diagnostics })
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        let report = self.validate_report()?;
        anyhow::ensure!(!report.has_errors(), "deck validation failed");
        Ok(())
    }
}

impl<'a> BasicLane<'a> {
    pub fn note(self, front: impl Into<String>, back: impl Into<String>) -> BasicDraft<'a> {
        BasicDraft {
            deck: self.deck,
            note: BasicNote::new(front, back),
        }
    }
}

impl<'a> BasicDraft<'a> {
    pub fn stable_id(mut self, stable_id: impl Into<String>) -> Self {
        self.note = self.note.stable_id(stable_id);
        self
    }

    pub fn tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.note = self.note.tags(tags);
        self
    }

    pub fn add(self) -> anyhow::Result<()> {
        self.deck.add(self.note)
    }
}

impl<'a> ClozeLane<'a> {
    pub fn note(self, text: impl Into<String>) -> ClozeDraft<'a> {
        ClozeDraft {
            deck: self.deck,
            note: ClozeNote::new(text),
        }
    }
}

impl<'a> ClozeDraft<'a> {
    pub fn stable_id(mut self, stable_id: impl Into<String>) -> Self {
        self.note = self.note.stable_id(stable_id);
        self
    }

    pub fn extra(mut self, extra: impl Into<String>) -> Self {
        self.note = self.note.extra(extra);
        self
    }

    pub fn add(self) -> anyhow::Result<()> {
        self.deck.add(self.note)
    }
}

impl<'a> IoLane<'a> {
    pub fn note(self, image: MediaRef) -> IoDraft<'a> {
        IoDraft {
            deck: self.deck,
            note: IoNote::new(image),
        }
    }
}

impl<'a> IoDraft<'a> {
    pub fn stable_id(mut self, stable_id: impl Into<String>) -> Self {
        self.note = self.note.stable_id(stable_id);
        self
    }

    pub fn mode(mut self, mode: IoMode) -> Self {
        self.note.mode = mode;
        self
    }

    pub fn rect(mut self, x: u32, y: u32, width: u32, height: u32) -> Self {
        self.note.rects.push(IoRect { x, y, width, height });
        self
    }

    pub fn header(mut self, header: impl Into<String>) -> Self {
        self.note.header = header.into();
        self
    }

    pub fn back_extra(mut self, back_extra: impl Into<String>) -> Self {
        self.note.back_extra = back_extra.into();
        self
    }

    pub fn comments(mut self, comments: impl Into<String>) -> Self {
        self.note.comments = comments.into();
        self
    }

    pub fn add(self) -> anyhow::Result<()> {
        anyhow::ensure!(!self.note.rects.is_empty(), "image occlusion note requires at least one rect");
        self.deck.add(self.note)
    }
}

fn validate_note_shape_before_insert(note: &DeckNote) -> anyhow::Result<()> {
    if let DeckNote::ImageOcclusion(io) = note {
        anyhow::ensure!(!io.rects.is_empty(), "image occlusion note requires at least one rect");
    }
    Ok(())
}
```

```rust
// tighten the add() path in anki_forge/src/deck/builders.rs
impl Deck {
    pub fn add(&mut self, note: impl Into<DeckNote>) -> anyhow::Result<()> {
        let mut note = note.into();
        assign_identity(self, &mut note)?;
        validate_note_shape_before_insert(&note)?;
        self.notes.push(note);
        Ok(())
    }
}
```

- [ ] **Step 4: Run the model and validation tests**

Run: `cargo test -p anki_forge --test deck_model_tests --test deck_validation_tests -v`
Expected: PASS with generated-id warnings and insertion-time failures for blank stable ids and empty IO geometry.

- [ ] **Step 5: Commit**

```bash
git add anki_forge/src/deck/model.rs anki_forge/src/deck/builders.rs anki_forge/src/deck/validation.rs anki_forge/src/deck/mod.rs anki_forge/tests/deck_validation_tests.rs
git commit -m "feat: add deck validation and lane builders"
```

### Task 3: Add opaque media references and registry collision rules

**Files:**
- Create: `anki_forge/src/deck/media.rs`
- Modify: `anki_forge/src/deck/model.rs`
- Modify: `anki_forge/src/deck/builders.rs`
- Test: `anki_forge/tests/deck_validation_tests.rs`

- [ ] **Step 1: Write the failing media collision test**

```rust
// append to anki_forge/tests/deck_validation_tests.rs
use anki_forge::{Deck, MediaSource};

#[test]
fn media_registry_reuses_same_name_same_content_and_rejects_conflicts() {
    let mut deck = Deck::new("Anatomy");

    let first = deck
        .media()
        .add(MediaSource::from_bytes("heart.png", vec![1, 2, 3]))
        .expect("first registration");
    let second = deck
        .media()
        .add(MediaSource::from_bytes("heart.png", vec![1, 2, 3]))
        .expect("same-bytes registration");

    assert_eq!(first, second);

    let err = deck
        .media()
        .add(MediaSource::from_bytes("heart.png", vec![9, 9, 9]))
        .expect_err("different bytes must fail");

    assert!(err.to_string().contains("heart.png"));
}

#[test]
fn image_occlusion_without_rects_fails_at_add_time() {
    let mut deck = Deck::new("Anatomy");
    let image = deck
        .media()
        .add(MediaSource::from_bytes("heart.png", vec![1, 2, 3]))
        .expect("register media");

    let err = deck
        .image_occlusion()
        .note(image)
        .stable_id("io-1")
        .add()
        .expect_err("io note without rects must fail");

    assert!(err.to_string().contains("rect"));
}
```

- [ ] **Step 2: Run the media test to verify it fails**

Run: `cargo test -p anki_forge --test deck_validation_tests media_registry_reuses_same_name_same_content_and_rejects_conflicts -v`
Expected: FAIL because the media registry, opaque refs, and collision handling do not exist yet.

- [ ] **Step 3: Implement filename-keyed media registration and opaque refs**

```rust
// anki_forge/src/deck/media.rs
use base64::Engine as _;
use sha1::{Digest, Sha1};

use crate::deck::model::{Deck, MediaRef, RegisteredMedia};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MediaSource {
    File { path: std::path::PathBuf },
    Bytes { name: String, bytes: Vec<u8> },
}

pub struct MediaRegistry<'a> {
    deck: &'a mut Deck,
}

impl MediaSource {
    pub fn from_file(path: impl Into<std::path::PathBuf>) -> Self {
        Self::File { path: path.into() }
    }

    pub fn from_bytes(name: impl Into<String>, bytes: Vec<u8>) -> Self {
        Self::Bytes {
            name: name.into(),
            bytes,
        }
    }
}

impl MediaRef {
    pub(crate) fn new(name: String) -> Self {
        Self(name)
    }

    pub fn name(&self) -> &str {
        &self.0
    }
}

impl Deck {
    pub fn media(&mut self) -> MediaRegistry<'_> {
        MediaRegistry { deck: self }
    }
}

impl<'a> MediaRegistry<'a> {
    pub fn add(&mut self, source: MediaSource) -> anyhow::Result<MediaRef> {
        let registered = RegisteredMedia::from_source(source)?;
        if let Some(existing) = self.deck.media.get(&registered.name) {
            if existing.sha1_hex == registered.sha1_hex {
                return Ok(MediaRef::new(existing.name.clone()));
            }
            anyhow::bail!("conflicting media payload for {}", registered.name);
        }

        let reference = MediaRef::new(registered.name.clone());
        self.deck.media.insert(registered.name.clone(), registered);
        Ok(reference)
    }

    pub fn get(&self, name: &str) -> Option<MediaRef> {
        self.deck
            .media
            .get(name)
            .map(|registered| MediaRef::new(registered.name.clone()))
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

        let sha1_hex = hex::encode(Sha1::digest(&bytes));
        Ok(Self {
            name: name.clone(),
            mime: mime_from_name(&name),
            data_base64: base64::engine::general_purpose::STANDARD.encode(bytes),
            sha1_hex,
        })
    }
}

fn mime_from_name(name: &str) -> String {
    match name.rsplit('.').next().map(|ext| ext.to_ascii_lowercase()) {
        Some(ext) if ext == "png" => "image/png".into(),
        Some(ext) if ext == "jpg" || ext == "jpeg" => "image/jpeg".into(),
        Some(ext) if ext == "svg" => "image/svg+xml".into(),
        Some(ext) if ext == "mp3" => "audio/mpeg".into(),
        Some(ext) if ext == "wav" => "audio/wav".into(),
        _ => "application/octet-stream".into(),
    }
}
```

```rust
// anki_forge/src/deck/mod.rs
pub mod media;

pub use media::MediaSource;
```

- [ ] **Step 4: Run the validation tests again**

Run: `cargo test -p anki_forge --test deck_validation_tests -v`
Expected: PASS with same-name reuse and conflicting-payload rejection.

- [ ] **Step 5: Commit**

```bash
git add anki_forge/src/deck/model.rs anki_forge/src/deck/builders.rs anki_forge/src/deck/media.rs anki_forge/tests/deck_validation_tests.rs
git commit -m "feat: add opaque media refs and collision checks"
```

### Task 4: Implement one-way lowering into `product` while preserving order and tags

**Files:**
- Create: `anki_forge/src/deck/lowering.rs`
- Create: `anki_forge/src/product/stock.rs`
- Modify: `anki_forge/src/deck/mod.rs`
- Modify: `anki_forge/src/product/mod.rs`
- Modify: `anki_forge/src/product/model.rs`
- Modify: `anki_forge/src/product/builders.rs`
- Modify: `anki_forge/src/product/lowering.rs`
- Test: `anki_forge/tests/deck_lowering_tests.rs`
- Test: `anki_forge/tests/product_lowering_tests.rs`
- Test: `anki_forge/tests/product_model_tests.rs`

- [ ] **Step 1: Write the failing mixed-order lowering test**

```rust
// anki_forge/tests/deck_lowering_tests.rs
use anki_forge::{Deck, IoMode, MediaSource};
use anki_forge::product::ProductNote;

#[test]
fn deck_lowers_notes_in_original_mixed_order() {
    let mut deck = Deck::builder("Mixed")
        .stable_id("mixed-v1")
        .build();

    let heart = deck
        .media()
        .add(MediaSource::from_bytes("heart.png", vec![0x89, 0x50]))
        .expect("register media");

    deck.cloze()
        .note("A {{c1::cloze}} card")
        .stable_id("cloze-1")
        .extra("extra")
        .add()
        .expect("add cloze");
    deck.basic()
        .note("front", "back")
        .stable_id("basic-1")
        .tags(["demo"])
        .add()
        .expect("add basic");
    deck.image_occlusion()
        .note(heart)
        .mode(IoMode::HideAllGuessOne)
        .rect(10, 20, 80, 40)
        .header("Heart")
        .back_extra("Identify the chamber")
        .comments("Left ventricle")
        .stable_id("io-1")
        .add()
        .expect("add io");

    let product = deck.clone().into_product_document().expect("product bridge");
    let lowered = deck.lower_authoring().expect("authoring lowering");

    assert!(matches!(&product.notes()[0], ProductNote::Cloze(_)));
    assert!(matches!(&product.notes()[1], ProductNote::Basic(_)));
    assert!(matches!(&product.notes()[2], ProductNote::ImageOcclusion(_)));

    assert_eq!(lowered.notes[0].id, "cloze-1");
    assert_eq!(lowered.notes[1].id, "basic-1");
    assert_eq!(lowered.notes[2].id, "io-1");
    assert_eq!(lowered.notes[1].tags, vec!["demo"]);
    assert_eq!(lowered.media.len(), 1);
}
```

- [ ] **Step 2: Run the lowering tests to verify they fail**

Run: `cargo test -p anki_forge --test deck_lowering_tests -v`
Expected: FAIL because the root facade does not yet lower through `product` with order preservation, tags, and stock helpers.

- [ ] **Step 3: Implement stock helpers and one-way lowering through `product`**

```rust
// anki_forge/src/product/stock.rs
use crate::IoMode;
use crate::deck::model::IoRect;

pub const STOCK_BASIC_ID: &str = "basic";
pub const STOCK_CLOZE_ID: &str = "cloze";
pub const STOCK_IMAGE_OCCLUSION_ID: &str = "image_occlusion";

pub fn render_image_occlusion_cloze(mode: IoMode, rects: &[IoRect]) -> anyhow::Result<String> {
    anyhow::ensure!(!rects.is_empty(), "image occlusion note requires at least one rect");

    let prefix = match mode {
        IoMode::HideAllGuessOne => "c1",
        IoMode::HideOneGuessOne => "c1,2",
    };

    let mut rendered = String::new();
    for rect in rects {
        rendered.push_str(&format!(
            "{{{{{}::image-occlusion:rect:left={}:top={}:width={}:height={}}}}}<br>",
            prefix, rect.x, rect.y, rect.width, rect.height,
        ));
    }
    Ok(rendered)
}
```

```rust
// anki_forge/src/product/mod.rs
pub mod stock;
pub use stock::{
    render_image_occlusion_cloze, STOCK_BASIC_ID, STOCK_CLOZE_ID, STOCK_IMAGE_OCCLUSION_ID,
};
```

```rust
// anki_forge/src/product/model.rs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BasicNote {
    pub id: String,
    pub note_type_id: String,
    pub deck_name: String,
    pub front: String,
    pub back: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClozeNote {
    pub id: String,
    pub note_type_id: String,
    pub deck_name: String,
    pub text: String,
    pub back_extra: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageOcclusionNote {
    pub id: String,
    pub note_type_id: String,
    pub deck_name: String,
    pub occlusion: String,
    pub image: String,
    pub header: String,
    pub back_extra: String,
    pub comments: String,
    #[serde(default)]
    pub tags: Vec<String>,
}
```

```rust
// anki_forge/src/product/builders.rs
pub fn add_basic_note(
    mut self,
    note_type_id: impl Into<String>,
    id: impl Into<String>,
    deck_name: impl Into<String>,
    front: impl Into<String>,
    back: impl Into<String>,
    tags: impl IntoIterator<Item = impl Into<String>>,
) -> Self {
    self.notes.push(ProductNote::Basic(BasicNote {
        id: id.into(),
        note_type_id: note_type_id.into(),
        deck_name: deck_name.into(),
        front: front.into(),
        back: back.into(),
        tags: tags.into_iter().map(Into::into).collect(),
    }));
    self
}

pub fn add_cloze_note(
    mut self,
    note_type_id: impl Into<String>,
    id: impl Into<String>,
    deck_name: impl Into<String>,
    text: impl Into<String>,
    back_extra: impl Into<String>,
    tags: impl IntoIterator<Item = impl Into<String>>,
) -> Self {
    self.notes.push(ProductNote::Cloze(ClozeNote {
        id: id.into(),
        note_type_id: note_type_id.into(),
        deck_name: deck_name.into(),
        text: text.into(),
        back_extra: back_extra.into(),
        tags: tags.into_iter().map(Into::into).collect(),
    }));
    self
}

pub fn add_image_occlusion_note(
    mut self,
    note_type_id: impl Into<String>,
    id: impl Into<String>,
    deck_name: impl Into<String>,
    occlusion: impl Into<String>,
    image: impl Into<String>,
    header: impl Into<String>,
    back_extra: impl Into<String>,
    comments: impl Into<String>,
    tags: impl IntoIterator<Item = impl Into<String>>,
) -> Self {
    self.notes.push(ProductNote::ImageOcclusion(ImageOcclusionNote {
        id: id.into(),
        note_type_id: note_type_id.into(),
        deck_name: deck_name.into(),
        occlusion: occlusion.into(),
        image: image.into(),
        header: header.into(),
        back_extra: back_extra.into(),
        comments: comments.into(),
        tags: tags.into_iter().map(Into::into).collect(),
    }));
    self
}
```

```rust
// anki_forge/src/product/lowering.rs
notes.push(AuthoringNote {
    id: basic.id.clone(),
    notetype_id: basic.note_type_id.clone(),
    deck_name: deck_name.clone(),
    fields,
    tags: basic.tags.clone(),
});

notes.push(AuthoringNote {
    id: cloze.id.clone(),
    notetype_id: cloze.note_type_id.clone(),
    deck_name: deck_name.clone(),
    fields,
    tags: cloze.tags.clone(),
});

notes.push(AuthoringNote {
    id: io.id.clone(),
    notetype_id: io.note_type_id.clone(),
    deck_name: deck_name.clone(),
    fields,
    tags: io.tags.clone(),
});
```

```rust
// anki_forge/src/deck/lowering.rs
use crate::deck::model::{Deck, DeckNote};
use crate::product::{
    render_image_occlusion_cloze, ProductDocument, STOCK_BASIC_ID, STOCK_CLOZE_ID,
    STOCK_IMAGE_OCCLUSION_ID,
};

impl Deck {
    pub fn into_product_document(self) -> anyhow::Result<ProductDocument> {
        let document_id = self
            .stable_id
            .clone()
            .unwrap_or_else(|| self.name.clone());
        let deck_name = self.name.clone();
        let mut product = ProductDocument::new(document_id)
            .with_default_deck(deck_name.clone())
            .with_basic(STOCK_BASIC_ID)
            .with_cloze(STOCK_CLOZE_ID)
            .with_image_occlusion(STOCK_IMAGE_OCCLUSION_ID);

        for note in self.notes {
            product = match note {
                DeckNote::Basic(note) => product.add_basic_note(
                    STOCK_BASIC_ID,
                    note.id,
                    deck_name.clone(),
                    note.front,
                    note.back,
                    note.tags,
                ),
                DeckNote::Cloze(note) => product.add_cloze_note(
                    STOCK_CLOZE_ID,
                    note.id,
                    deck_name.clone(),
                    note.text,
                    note.extra,
                    note.tags,
                ),
                DeckNote::ImageOcclusion(note) => product.add_image_occlusion_note(
                    STOCK_IMAGE_OCCLUSION_ID,
                    note.id,
                    deck_name.clone(),
                    render_image_occlusion_cloze(note.mode, &note.rects)?,
                    format!("<img src=\"{}\">", note.image.name()),
                    note.header,
                    note.back_extra,
                    note.comments,
                    note.tags,
                ),
            };
        }

        Ok(product)
    }

    pub fn lower_authoring(&self) -> anyhow::Result<crate::AuthoringDocument> {
        let product = self.clone().into_product_document()?;
        let mut lowered = product.lower()?.authoring_document;
        lowered.media.extend(self.media.values().map(|media| crate::AuthoringMedia {
            filename: media.name.clone(),
            mime: media.mime.clone(),
            data_base64: media.data_base64.clone(),
        }));
        Ok(lowered)
    }
}
```

```rust
// anki_forge/src/deck/mod.rs
pub mod lowering;
```

- [ ] **Step 4: Run the lowering and updated product tests**

Run: `cargo test -p anki_forge --test deck_lowering_tests --test product_lowering_tests --test product_model_tests -v`
Expected: PASS with mixed order preserved from `Deck` through `ProductDocument` and into `AuthoringDocument`.

- [ ] **Step 5: Commit**

```bash
git add anki_forge/src/deck/lowering.rs anki_forge/src/deck/mod.rs anki_forge/src/product/stock.rs anki_forge/src/product/mod.rs anki_forge/src/product/model.rs anki_forge/src/product/builders.rs anki_forge/src/product/lowering.rs anki_forge/tests/deck_lowering_tests.rs anki_forge/tests/product_lowering_tests.rs anki_forge/tests/product_model_tests.rs
git commit -m "feat: add one-way deck lowering into product"
```

### Task 5: Add bytes-first export, package symmetry, and runtime-backed build results

**Files:**
- Create: `anki_forge/src/deck/export.rs`
- Create: `anki_forge/src/runtime/defaults.rs`
- Modify: `anki_forge/src/runtime/mod.rs`
- Modify: `writer_core/src/lib.rs`
- Modify: `writer_core/src/inspect.rs`
- Modify: `anki_forge/src/deck/mod.rs`
- Test: `anki_forge/tests/deck_export_tests.rs`

- [ ] **Step 1: Write the failing export-symmetry tests**

```rust
// anki_forge/tests/deck_export_tests.rs
use anki_forge::{Deck, Package};

#[test]
fn deck_export_surfaces_use_runtime_defaults_and_real_artifact_paths() {
    let mut deck = Deck::builder("Spanish")
        .stable_id("spanish-v1")
        .build();
    deck.basic()
        .note("hola", "hello")
        .stable_id("es-hola")
        .add()
        .expect("add basic note");

    let temp = tempfile::tempdir().expect("tempdir");
    let build = deck.build(temp.path()).expect("build facade");

    assert!(build.apkg_path().exists());
    assert!(build.staging_manifest_path().exists());
    assert_eq!(build.package_build_result().result_status, "success");

    let bytes = deck.to_apkg_bytes().expect("apkg bytes");
    assert!(!bytes.is_empty());
}

#[test]
fn package_single_matches_deck_export_surface() {
    let mut deck = Deck::builder("Spanish")
        .stable_id("spanish-v1")
        .build();
    deck.basic()
        .note("hola", "hello")
        .stable_id("es-hola")
        .add()
        .expect("add basic note");

    let package = Package::single(deck.clone()).with_stable_id("package-v1");

    assert_eq!(
        deck.to_apkg_bytes().expect("deck bytes"),
        package.to_apkg_bytes().expect("package bytes"),
    );
}
```

- [ ] **Step 2: Run the export tests to verify they fail**

Run: `cargo test -p anki_forge --test deck_export_tests -v`
Expected: FAIL because runtime-default loading, bytes-first export, `Package` symmetry, and resolved artifact paths do not exist yet.

- [ ] **Step 3: Implement runtime-backed export and `BuildResult` without panics**

```rust
// anki_forge/src/runtime/defaults.rs
use std::path::Path;

use writer_core::{BuildContext, WriterPolicy};

pub fn load_default_writer_stack(
    start: impl AsRef<Path>,
) -> anyhow::Result<(super::ResolvedRuntime, WriterPolicy, BuildContext)> {
    let runtime = super::discover_workspace_runtime(start)?;
    let bundle = super::load_bundle_from_manifest(&runtime.manifest_path)?;
    let writer_policy = super::load_writer_policy(&bundle, "default")?;
    let build_context = super::load_build_context(&bundle, "default")?;
    Ok((runtime, writer_policy, build_context))
}
```

```rust
// anki_forge/src/runtime/mod.rs
pub mod defaults;

pub use defaults::load_default_writer_stack;
```

```rust
// writer_core/src/lib.rs
pub use inspect::artifact_path_from_ref;
```

```rust
// writer_core/src/inspect.rs
pub fn artifact_path_from_ref(target: &BuildArtifactTarget, reference: &str) -> PathBuf {
    let prefix = target.stable_ref_prefix.trim_end_matches('/');
    let trimmed = reference
        .strip_prefix(prefix)
        .unwrap_or(reference)
        .trim_start_matches('/');
    if trimmed.is_empty() {
        target.root_dir.clone()
    } else {
        target.root_dir.join(trimmed)
    }
}
```

```rust
// anki_forge/src/deck/export.rs
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::Context;

pub struct BuildResult {
    inner: crate::PackageBuildResult,
    apkg_path: PathBuf,
    staging_manifest_path: PathBuf,
}

impl BuildResult {
    pub fn package_build_result(&self) -> &crate::PackageBuildResult {
        &self.inner
    }

    pub fn apkg_path(&self) -> &Path {
        &self.apkg_path
    }

    pub fn staging_manifest_path(&self) -> &Path {
        &self.staging_manifest_path
    }

    pub fn inspect_staging(&self) -> anyhow::Result<crate::InspectReport> {
        crate::inspect_staging(self.staging_manifest_path())
    }

    pub fn inspect_apkg(&self) -> anyhow::Result<crate::InspectReport> {
        crate::inspect_apkg(self.apkg_path())
    }
}

impl Package {
    pub fn build(&self, artifacts_dir: impl AsRef<Path>) -> anyhow::Result<BuildResult> {
        self.root_deck.validate()?;
        let lowered = self.root_deck.lower_authoring()?;
        let normalized = crate::normalize(crate::NormalizationRequest::new(lowered));
        let normalized_ir = normalized.normalized_ir.ok_or_else(|| {
            anyhow::anyhow!(
                "normalization failed: {} diagnostics",
                normalized.diagnostics.items.len()
            )
        })?;

        let (_, writer_policy, build_context) =
            crate::runtime::load_default_writer_stack(std::env::current_dir()?)?;
        let artifact_target = crate::BuildArtifactTarget::new(
            artifacts_dir.as_ref().to_path_buf(),
            "artifacts",
        );
        let inner = crate::build(&normalized_ir, &writer_policy, &build_context, &artifact_target)?;
        anyhow::ensure!(inner.result_status == "success", "build failed with status {}", inner.result_status);

        let apkg_ref = inner
            .apkg_ref
            .as_deref()
            .context("successful build must include apkg_ref")?;
        let staging_ref = inner
            .staging_ref
            .as_deref()
            .context("successful build must include staging_ref")?;

        Ok(BuildResult {
            apkg_path: writer_core::artifact_path_from_ref(&artifact_target, apkg_ref),
            staging_manifest_path: writer_core::artifact_path_from_ref(&artifact_target, staging_ref),
            inner,
        })
    }

    pub fn to_apkg_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let temp = tempfile::tempdir()?;
        let build = self.build(temp.path())?;
        std::fs::read(build.apkg_path())
            .with_context(|| format!("read apkg bytes: {}", build.apkg_path().display()))
    }

    pub fn write_to<W: Write>(&self, mut writer: W) -> anyhow::Result<()> {
        writer.write_all(&self.to_apkg_bytes()?)?;
        Ok(())
    }

    pub fn write_apkg(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        std::fs::write(path.as_ref(), self.to_apkg_bytes()?)
            .with_context(|| format!("write apkg: {}", path.as_ref().display()))
    }
}

impl Deck {
    pub fn build(&self, artifacts_dir: impl AsRef<Path>) -> anyhow::Result<BuildResult> {
        Package::single(self.clone()).build(artifacts_dir)
    }

    pub fn to_apkg_bytes(&self) -> anyhow::Result<Vec<u8>> {
        Package::single(self.clone()).to_apkg_bytes()
    }

    pub fn write_to<W: Write>(&self, writer: W) -> anyhow::Result<()> {
        Package::single(self.clone()).write_to(writer)
    }

    pub fn write_apkg(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        Package::single(self.clone()).write_apkg(path)
    }
}
```

```rust
// anki_forge/src/deck/mod.rs
pub mod export;

pub use export::BuildResult;
```

- [ ] **Step 4: Run the export tests and a focused end-to-end check**

Run: `cargo test -p anki_forge --test deck_export_tests -v`
Expected: PASS with resolved artifact paths, bytes-first export, and `Package::single(...)` symmetry.

Run: `cargo test -p anki_forge deck_export_surfaces_use_runtime_defaults_and_real_artifact_paths -v`
Expected: PASS with a successful build and non-empty `.apkg` bytes.

- [ ] **Step 5: Commit**

```bash
git add anki_forge/src/deck/export.rs anki_forge/src/deck/mod.rs anki_forge/src/runtime/defaults.rs anki_forge/src/runtime/mod.rs writer_core/src/lib.rs writer_core/src/inspect.rs anki_forge/tests/deck_export_tests.rs
git commit -m "feat: add runtime-backed deck export"
```

### Task 6: Publish the happy-path docs and examples

**Files:**
- Create: `anki_forge/examples/deck_basic_flow.rs`
- Modify: `README.md`

- [ ] **Step 1: Write the public-shape smoke test**

```rust
// append to anki_forge/tests/deck_export_tests.rs
#[test]
fn deck_basic_flow_example_shape_matches_the_public_happy_path() {
    let mut deck = Deck::builder("Spanish")
        .stable_id("spanish-v1")
        .build();

    deck.basic()
        .note("hola", "hello")
        .stable_id("es-hola")
        .add()
        .expect("add basic");

    assert_eq!(deck.notes().len(), 1);
    assert_eq!(deck.notes()[0].id(), "es-hola");
}
```

- [ ] **Step 2: Run the smoke test before editing docs**

Run: `cargo test -p anki_forge --test deck_export_tests deck_basic_flow_example_shape_matches_the_public_happy_path -v`
Expected: PASS once the root API is complete.

- [ ] **Step 3: Update the example and README**

```rust
// anki_forge/examples/deck_basic_flow.rs
use anki_forge::{Deck, IoMode, MediaSource};

fn main() -> anyhow::Result<()> {
    let mut deck = Deck::builder("Spanish")
        .stable_id("spanish-v1")
        .build();

    deck.basic()
        .note("hola", "hello")
        .stable_id("es-hola")
        .tags(["vocab", "a1"])
        .add()?;

    deck.cloze()
        .note("La capital de Espana es {{c1::Madrid}}")
        .extra("Europe")
        .stable_id("geo-es-capital")
        .add()?;

    let heart = deck
        .media()
        .add(MediaSource::from_bytes("heart.png", std::fs::read("heart.png")?))?;

    deck.image_occlusion()
        .note(heart)
        .mode(IoMode::HideAllGuessOne)
        .rect(10, 20, 80, 40)
        .header("Heart")
        .back_extra("Identify the chamber")
        .comments("Left ventricle")
        .stable_id("anatomy-heart-1")
        .add()?;

    deck.write_apkg("spanish.apkg")?;
    Ok(())
}
```

```md
<!-- README.md -->
## Rust Quick Start

```rust
use anki_forge::Deck;

let mut deck = Deck::builder("Spanish")
    .stable_id("spanish-v1")
    .build();

deck.basic()
    .note("hola", "hello")
    .stable_id("es-hola")
    .add()?;

deck.write_apkg("spanish.apkg")?;
```

`add_basic(...)` remains available for the shortest path, but it generates a non-stable note id.
Use `stable_id(...)` when you want import-friendly updates.

For advanced authoring, use `anki_forge::product`.
For file-driven pipeline work, use `anki_forge::runtime`.
The reverse bridge `from_product_document()` is intentionally deferred until IO/media round-tripping is lossless.
```

- [ ] **Step 4: Run the example and the full `anki_forge` test suite**

Run: `cargo run -p anki_forge --example deck_basic_flow`
Expected: PASS and write a valid `spanish.apkg` in the current working directory.

Run: `cargo test -p anki_forge -v`
Expected: PASS with the new root-facade tests and existing product/runtime suites still green.

- [ ] **Step 5: Commit**

```bash
git add anki_forge/examples/deck_basic_flow.rs README.md anki_forge/tests/deck_export_tests.rs
git commit -m "docs: publish deck happy path"
```

## Verification Checklist

Run these commands before claiming the plan is fully executed:

```bash
cargo fmt --all
cargo test -p anki_forge --test deck_model_tests --test deck_validation_tests --test deck_lowering_tests --test deck_export_tests -v
cargo test -p anki_forge -v
cargo run -p anki_forge --example deck_basic_flow
cargo run -p anki_forge --example product_basic_flow
```

Expected outcomes:

1. The root model preserves mixed-note insertion order in one `Vec<DeckNote>`.
2. Missing explicit `stable_id` produces generated ids plus validation diagnostics instead of blank lowered ids.
3. Blank or duplicate explicit `stable_id` values fail before export.
4. The media registry reuses same-name same-content assets and rejects conflicting payloads.
5. The root export path uses shared runtime defaults and resolves actual artifact paths from returned refs.
6. Existing `product` tests still pass with stock-note tags flowing through lowering.
7. Public docs keep `.apkg` scoped to a single root deck and do not promise `from_product_document()` in the first pass.
