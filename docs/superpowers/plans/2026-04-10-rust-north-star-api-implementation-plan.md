# Rust North-Star API Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver the approved crate-root Rust authoring facade for `anki_forge` with `Deck`, `Package`, `MediaSource`, `MediaRef`, `ValidationReport`, and `BuildResult`, while preserving the existing `product -> lowering -> normalize -> build` pipeline behind the new author-facing surface.

**Architecture:** Add a new internal `anki_forge::deck` module that owns the root author-facing DTOs, builders, media registry, validation, and export helpers. Lower the root facade into the existing `product` layer for stock note-type shaping, then merge root-only concerns such as tags, named media, validation diagnostics, and single-root package export behavior before calling `normalize()` and `writer_core::build()`.

**Tech Stack:** Rust workspace (`cargo`, `serde`, `anyhow`), existing `anki_forge`, `authoring_core`, and `writer_core` crates, `tempfile`, `base64`, fixture-driven tests, Markdown docs/examples

---

## Scope Check

This plan covers one coherent subsystem: the Rust crate-root authoring facade approved in
[2026-04-10-rust-north-star-api-design.md](/Users/hp/Desktop/2026/anki-forge/docs/superpowers/specs/2026-04-10-rust-north-star-api-design.md).

The work stays inside the existing `anki_forge` crate and one small extension of the current
`product` layer. It should not be split further unless the user explicitly wants to peel off one
of these slices:

1. root DTOs and builders only
2. validation/export only
3. docs/examples only

## Execution Prerequisite

Before running any implementation task below, create or switch to a dedicated worktree for this
plan, for example:

```bash
git worktree add ../anki-forge-north-star-api -b codex/rust-north-star-api
cd ../anki-forge-north-star-api
```

Execute the plan in that worktree, not on top of unrelated in-progress work.

## File Structure Map

### New crate-root authoring facade

- Modify: `anki_forge/Cargo.toml` - add `base64` and `tempfile` dependencies for named media encoding and bytes-first export helpers
- Modify: `anki_forge/src/lib.rs` - re-export `Deck`, `Package`, `BuildResult`, `ValidationReport`, `MediaSource`, `MediaRef`, `IoMode`, and note DTOs from the new facade module
- Create: `anki_forge/src/deck/mod.rs` - public exports for the root facade
- Create: `anki_forge/src/deck/model.rs` - `Deck`, `Package`, `BuildResult`, `ValidationReport`, note DTOs, mask geometry DTOs, opaque lowered-occlusion state, and stable-id state
- Create: `anki_forge/src/deck/builders.rs` - `Deck::builder()`, `deck.basic()/cloze()/image_occlusion()`, `.note()`, `.tags()`, `.stable_id()`, `.extra()`, `.mode()`, `.back_extra()`, `.comments()`, `.rect()`, `.add()`
- Create: `anki_forge/src/deck/media.rs` - `MediaSource`, `MediaRef`, named-media registry, `deck.media().add(...)`, and `deck.media().get(...)`
- Create: `anki_forge/src/deck/lowering.rs` - conversion from `Deck`/`Package` into `product::ProductDocument`, `AuthoringDocument`, and bridge helpers `into_product_document()` / `from_product_document()`
- Create: `anki_forge/src/deck/validation.rs` - `ValidationReport`, `ValidationDiagnostic`, `validate()`, and `validate_report()`
- Create: `anki_forge/src/deck/export.rs` - `to_apkg_bytes()`, `write_to(...)`, `write_apkg(...)`, `build(...)`, and `BuildResult`

### Product-layer bridge changes

- Modify: `anki_forge/src/product/model.rs` - add `tags` to stock note variants so root-facade tags survive lowering and bridge round-trips
- Modify: `anki_forge/src/product/builders.rs` - keep constructor helpers aligned with the updated stock note structs
- Modify: `anki_forge/src/product/lowering.rs` - pass stock-note tags through to `AuthoringNote.tags`

### Tests and docs

- Create: `anki_forge/tests/deck_model_tests.rs` - root DTO, builder, and root-export smoke tests
- Create: `anki_forge/tests/deck_lowering_tests.rs` - `Deck` to `ProductDocument` / `AuthoringDocument` lowering tests for `Basic`, `Cloze`, `Image Occlusion`, tags, and named media
- Create: `anki_forge/tests/deck_validation_tests.rs` - preflight validation and structured diagnostics tests
- Create: `anki_forge/tests/deck_export_tests.rs` - `to_apkg_bytes()`, `write_to(...)`, `write_apkg(...)`, `build(...)`, and `Package::single(...)` symmetry tests
- Modify: `anki_forge/tests/product_lowering_tests.rs` - cover stock-note tags flowing through `product` lowering
- Modify: `anki_forge/tests/product_model_tests.rs` - cover updated stock note structs with tags
- Create: `anki_forge/examples/deck_basic_flow.rs` - new north-star example from `Deck` to `.apkg`
- Modify: `README.md` - put `Deck` in the main Rust tutorial and move `product` / `runtime` to advanced sections

## Implementation Notes

- Keep `.apkg` semantics aligned with Anki deck packages: one root `Deck`, optional child decks only through future collection work, and no multi-root `.apkg` API.
- `Package` must stay symmetric with `Deck` for `to_apkg_bytes()`, `write_to(...)`, `write_apkg(...)`, and `build(...)`. `Deck` methods should delegate to `Package::single(self.clone())`.
- Use `stable_id` as the only default-layer identity term. `guid` may exist only as a lower-level override hook inside the root model and should not appear in the primary docs path.
- `MediaSource::from_bytes(name, bytes)` must require a stable exported filename. Named lookup belongs in `deck.media().get(name)`.
- Root-lane builders should materialize stock-note data into the existing `product` layer whenever possible, then merge root-only state such as dynamic note media and validation diagnostics at the `AuthoringDocument` boundary.
- Do not force `validate()` into the happy path. `add()` does lightweight checks, `validate_report()` / `validate()` provide optional preflight, and every export/build path must run full validation internally.
- `Image Occlusion` first pass intentionally supports `rect(...)` only. Document that this is a narrowed initial geometry helper, not full parity with native ellipse/polygon editing.
- Preserve update-friendly invariants in code comments and tests: note `stable_id` drives note identity, and note-type evolution must continue to preserve stable field/template ids in the underlying `product` / `authoring` pipeline.

### Task 1: Bootstrap the root facade types and exports

**Files:**
- Modify: `anki_forge/Cargo.toml`
- Modify: `anki_forge/src/lib.rs`
- Create: `anki_forge/src/deck/mod.rs`
- Create: `anki_forge/src/deck/model.rs`
- Test: `anki_forge/tests/deck_model_tests.rs`

- [ ] **Step 1: Write the failing root-export smoke test**

```rust
// anki_forge/tests/deck_model_tests.rs
use anki_forge::{BuildResult, Deck, IoMode, MediaRef, MediaSource, Package, ValidationReport};

#[test]
fn crate_root_exposes_north_star_types() {
    let deck = Deck::new("Spanish");
    let package = Package::single(deck.clone());
    let _mode = IoMode::HideAllGuessOne;
    let _source = MediaSource::from_bytes("heart.png", vec![0x89, 0x50, 0x4e, 0x47]);
    let _media_ref: Option<MediaRef> = None;
    let _build: Option<BuildResult> = None;
    let _report: Option<ValidationReport> = None;

    assert_eq!(deck.name(), "Spanish");
    assert_eq!(package.root_deck().name(), "Spanish");
}
```

- [ ] **Step 2: Run the smoke test to verify it fails**

Run: `cargo test -p anki_forge --test deck_model_tests crate_root_exposes_north_star_types -v`
Expected: FAIL with unresolved imports for `Deck`, `Package`, `MediaSource`, or the new facade module.

- [ ] **Step 3: Add the minimal module skeleton and root re-exports**

```rust
// anki_forge/Cargo.toml
[dependencies]
anyhow = "1"
authoring_core = { path = "../authoring_core" }
base64 = "0.22"
hex = "0.4"
jsonschema = { version = "0.18.3", default-features = false }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
sha1 = "0.10"
tempfile = "3"
url = "2.5.2"
writer_core = { path = "../writer_core" }
```

```rust
// anki_forge/src/lib.rs
mod deck;
pub mod product;
pub mod runtime;

pub use deck::{
    BasicNote, BuildResult, ClozeNote, Deck, IoMode, IoNote, MediaRef, MediaSource, Package,
    ValidationReport,
};

// keep the existing authoring_core and writer_core re-exports below unchanged
```

```rust
// anki_forge/src/deck/mod.rs
pub mod model;

pub use model::{
    BasicNote, BuildResult, ClozeNote, Deck, IoMode, IoNote, MediaRef, MediaSource, Package,
    ValidationReport,
};
```

```rust
// anki_forge/src/deck/model.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deck {
    name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Package {
    root_deck: Deck,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildResult;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationReport;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaRef {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaSource {
    File { path: String },
    Bytes { name: String, bytes: Vec<u8> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IoMode {
    HideAllGuessOne,
    HideOneGuessOne,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BasicNote;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClozeNote;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IoNote;

impl Deck {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Package {
    pub fn single(deck: Deck) -> Self {
        Self { root_deck: deck }
    }

    pub fn root_deck(&self) -> &Deck {
        &self.root_deck
    }
}

impl MediaSource {
    pub fn from_bytes(name: impl Into<String>, bytes: Vec<u8>) -> Self {
        Self::Bytes {
            name: name.into(),
            bytes,
        }
    }
}
```

- [ ] **Step 4: Run the smoke test to verify it passes**

Run: `cargo test -p anki_forge --test deck_model_tests crate_root_exposes_north_star_types -v`
Expected: PASS with `crate_root_exposes_north_star_types`.

- [ ] **Step 5: Commit**

```bash
git add anki_forge/Cargo.toml anki_forge/src/lib.rs anki_forge/src/deck/mod.rs anki_forge/src/deck/model.rs anki_forge/tests/deck_model_tests.rs
git commit -m "feat: bootstrap rust north-star root facade"
```

### Task 2: Add note DTOs, lane builders, named media, and lowering to the existing product pipeline

**Files:**
- Create: `anki_forge/src/deck/builders.rs`
- Create: `anki_forge/src/deck/media.rs`
- Create: `anki_forge/src/deck/lowering.rs`
- Modify: `anki_forge/src/deck/mod.rs`
- Modify: `anki_forge/src/deck/model.rs`
- Modify: `anki_forge/src/product/model.rs`
- Modify: `anki_forge/src/product/builders.rs`
- Modify: `anki_forge/src/product/lowering.rs`
- Test: `anki_forge/tests/deck_lowering_tests.rs`
- Test: `anki_forge/tests/product_lowering_tests.rs`
- Test: `anki_forge/tests/product_model_tests.rs`

- [ ] **Step 1: Write the failing lowering tests for stock lanes, tags, and named media**

```rust
// anki_forge/tests/deck_lowering_tests.rs
use anki_forge::{Deck, IoMode, MediaSource};
use anki_forge::product::ProductNote;

#[test]
fn deck_lowers_stock_lanes_into_product_and_authoring_documents() {
    let mut deck = Deck::builder("Spanish")
        .stable_id("spanish-v1")
        .build();

    let heart = deck
        .media()
        .add(MediaSource::from_bytes("heart.png", vec![0x89, 0x50]))
        .expect("register media");

    deck.basic()
        .note("hola", "hello")
        .stable_id("es-hola")
        .tags(["vocab", "a1"])
        .add()
        .expect("add basic note");

    deck.cloze()
        .note("La capital es {{c1::Madrid}}")
        .extra("Europe")
        .stable_id("geo-es-capital")
        .add()
        .expect("add cloze note");

    deck.image_occlusion()
        .note(heart)
        .mode(IoMode::HideAllGuessOne)
        .rect(10, 20, 80, 40)
        .header("Heart")
        .back_extra("Identify the chamber")
        .comments("Left ventricle")
        .stable_id("anatomy-heart-1")
        .add()
        .expect("add io note");

    let product = deck.clone().into_product_document().expect("product bridge");
    let lowered = deck.lower_authoring().expect("authoring lowering");

    assert_eq!(product.document_id(), "spanish-v1");
    assert_eq!(lowered.metadata_document_id, "spanish-v1");
    assert_eq!(lowered.media.len(), 1);

    match &product.notes()[0] {
        ProductNote::Basic(note) => assert_eq!(note.tags, vec!["vocab", "a1"]),
        other => panic!("expected basic note, got {other:?}"),
    }

    assert!(lowered
        .notes
        .iter()
        .any(|note| note.id == "geo-es-capital" && note.fields.contains_key("Back Extra")));
    assert!(lowered
        .notes
        .iter()
        .any(|note| note.id == "anatomy-heart-1" && note.fields.contains_key("Comments")));
    assert_eq!(lowered.media[0].filename, "heart.png");
}
```

- [ ] **Step 2: Run the lowering tests to verify they fail**

Run: `cargo test -p anki_forge --test deck_lowering_tests deck_lowers_stock_lanes_into_product_and_authoring_documents -v`
Expected: FAIL because `Deck::builder()`, lane builders, media registry, `into_product_document()`, or `lower_authoring()` do not exist yet.

- [ ] **Step 3: Implement the root model, builders, media registry, and product bridge**

```rust
// anki_forge/src/deck/model.rs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deck {
    pub(crate) name: String,
    pub(crate) stable_id: Option<String>,
    pub(crate) basic_notes: Vec<BasicNote>,
    pub(crate) cloze_notes: Vec<ClozeNote>,
    pub(crate) io_notes: Vec<IoNote>,
    pub(crate) media: Vec<RegisteredMedia>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BasicNote {
    pub stable_id: String,
    pub front: String,
    pub back: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClozeNote {
    pub stable_id: String,
    pub text: String,
    pub extra: String,
    pub tags: Vec<String>,
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
    pub stable_id: String,
    pub image: MediaRef,
    pub mode: IoMode,
    pub rects: Vec<IoRect>,
    pub raw_occlusion: Option<String>,
    pub header: String,
    pub back_extra: String,
    pub comments: String,
    pub tags: Vec<String>,
}
```

```rust
// anki_forge/src/deck/builders.rs
pub struct BasicLane<'a> {
    deck: &'a mut Deck,
}

pub struct ClozeLane<'a> {
    deck: &'a mut Deck,
}

pub struct IoLane<'a> {
    deck: &'a mut Deck,
}

pub struct BasicNoteBuilder<'a> {
    deck: &'a mut Deck,
    stable_id: String,
    front: String,
    back: String,
    tags: Vec<String>,
}

pub struct ClozeNoteBuilder<'a> {
    deck: &'a mut Deck,
    stable_id: String,
    text: String,
    extra: String,
    tags: Vec<String>,
}

pub struct IoNoteBuilder<'a> {
    deck: &'a mut Deck,
    stable_id: String,
    image: MediaRef,
    mode: IoMode,
    rects: Vec<IoRect>,
    raw_occlusion: Option<String>,
    header: String,
    back_extra: String,
    comments: String,
    tags: Vec<String>,
}

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
            basic_notes: Vec::new(),
            cloze_notes: Vec::new(),
            io_notes: Vec::new(),
            media: Vec::new(),
        }
    }
}

impl Deck {
    pub fn builder(name: impl Into<String>) -> DeckBuilder {
        DeckBuilder::new(name)
    }

    pub fn add_basic(&mut self, front: impl Into<String>, back: impl Into<String>) -> anyhow::Result<()> {
        self.basic().note(front, back).add()
    }

    pub fn basic(&mut self) -> BasicLane<'_> { BasicLane::new(self) }
    pub fn cloze(&mut self) -> ClozeLane<'_> { ClozeLane::new(self) }
    pub fn image_occlusion(&mut self) -> IoLane<'_> { IoLane::new(self) }
}

impl<'a> BasicLane<'a> {
    fn new(deck: &'a mut Deck) -> Self {
        Self { deck }
    }

    pub fn note(self, front: impl Into<String>, back: impl Into<String>) -> BasicNoteBuilder<'a> {
        BasicNoteBuilder::new(self.deck, front.into(), back.into())
    }
}

impl<'a> ClozeNoteBuilder<'a> {
    fn new(deck: &'a mut Deck, text: String) -> Self {
        Self {
            deck,
            stable_id: String::new(),
            text,
            extra: String::new(),
            tags: Vec::new(),
        }
    }

    pub fn extra(mut self, extra: impl Into<String>) -> Self {
        self.extra = extra.into();
        self
    }
}

impl<'a> IoNoteBuilder<'a> {
    fn new(deck: &'a mut Deck, image: MediaRef) -> Self {
        Self {
            deck,
            stable_id: String::new(),
            image,
            mode: IoMode::HideAllGuessOne,
            rects: Vec::new(),
            raw_occlusion: None,
            header: String::new(),
            back_extra: String::new(),
            comments: String::new(),
            tags: Vec::new(),
        }
    }

    pub fn mode(mut self, mode: IoMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn rect(mut self, x: u32, y: u32, width: u32, height: u32) -> Self {
        self.rects.push(IoRect { x, y, width, height });
        self
    }

    pub fn raw_occlusion(mut self, occlusion: impl Into<String>) -> Self {
        self.raw_occlusion = Some(occlusion.into());
        self
    }
}

impl<'a> ClozeLane<'a> {
    fn new(deck: &'a mut Deck) -> Self {
        Self { deck }
    }

    pub fn note(self, text: impl Into<String>) -> ClozeNoteBuilder<'a> {
        ClozeNoteBuilder::new(self.deck, text.into())
    }
}

impl<'a> IoLane<'a> {
    fn new(deck: &'a mut Deck) -> Self {
        Self { deck }
    }

    pub fn note(self, image: MediaRef) -> IoNoteBuilder<'a> {
        IoNoteBuilder::new(self.deck, image)
    }
}

impl<'a> BasicNoteBuilder<'a> {
    fn new(deck: &'a mut Deck, front: String, back: String) -> Self {
        Self {
            deck,
            stable_id: String::new(),
            front,
            back,
            tags: Vec::new(),
        }
    }

    pub fn stable_id(mut self, stable_id: impl Into<String>) -> Self {
        self.stable_id = stable_id.into();
        self
    }

    pub fn tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.tags = tags.into_iter().map(Into::into).collect();
        self
    }

    pub fn add(self) -> anyhow::Result<()> {
        self.deck.basic_notes.push(BasicNote {
            stable_id: self.stable_id,
            front: self.front,
            back: self.back,
            tags: self.tags,
        });
        Ok(())
    }
}

impl<'a> ClozeNoteBuilder<'a> {
    pub fn stable_id(mut self, stable_id: impl Into<String>) -> Self {
        self.stable_id = stable_id.into();
        self
    }

    pub fn tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.tags = tags.into_iter().map(Into::into).collect();
        self
    }

    pub fn add(self) -> anyhow::Result<()> {
        self.deck.cloze_notes.push(ClozeNote {
            stable_id: self.stable_id,
            text: self.text,
            extra: self.extra,
            tags: self.tags,
        });
        Ok(())
    }
}

impl<'a> IoNoteBuilder<'a> {
    pub fn stable_id(mut self, stable_id: impl Into<String>) -> Self {
        self.stable_id = stable_id.into();
        self
    }

    pub fn header(mut self, header: impl Into<String>) -> Self {
        self.header = header.into();
        self
    }

    pub fn back_extra(mut self, back_extra: impl Into<String>) -> Self {
        self.back_extra = back_extra.into();
        self
    }

    pub fn comments(mut self, comments: impl Into<String>) -> Self {
        self.comments = comments.into();
        self
    }

    pub fn tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.tags = tags.into_iter().map(Into::into).collect();
        self
    }

    pub fn add(self) -> anyhow::Result<()> {
        self.deck.io_notes.push(IoNote {
            stable_id: self.stable_id,
            image: self.image,
            mode: self.mode,
            rects: self.rects,
            raw_occlusion: self.raw_occlusion,
            header: self.header,
            back_extra: self.back_extra,
            comments: self.comments,
            tags: self.tags,
        });
        Ok(())
    }
}
```

```rust
// anki_forge/src/deck/media.rs
use base64::Engine as _;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisteredMedia {
    pub name: String,
    pub mime: String,
    pub data_base64: String,
}

impl MediaSource {
    pub fn from_file(path: impl Into<String>) -> Self {
        Self::File { path: path.into() }
    }
}

impl RegisteredMedia {
    pub fn from_source(source: MediaSource) -> anyhow::Result<Self> {
        match source {
            MediaSource::File { path } => {
                let bytes = std::fs::read(&path)?;
                let name = std::path::Path::new(&path)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .ok_or_else(|| anyhow::anyhow!("media path must end in a valid filename"))?
                    .to_string();
                Ok(Self {
                    mime: infer_mime(&name).into(),
                    name,
                    data_base64: base64::engine::general_purpose::STANDARD.encode(bytes),
                })
            }
            MediaSource::Bytes { name, bytes } => Ok(Self {
                mime: infer_mime(&name).into(),
                name,
                data_base64: base64::engine::general_purpose::STANDARD.encode(bytes),
            }),
        }
    }
}

fn infer_mime(name: &str) -> &'static str {
    match std::path::Path::new(name)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "svg" => "image/svg+xml",
        "mp3" => "audio/mpeg",
        "ogg" => "audio/ogg",
        _ => "application/octet-stream",
    }
}

pub struct MediaRegistry<'a> {
    deck: &'a mut Deck,
}

impl Deck {
    pub fn media(&mut self) -> MediaRegistry<'_> {
        MediaRegistry { deck: self }
    }
}

impl<'a> MediaRegistry<'a> {
    pub fn add(&mut self, source: MediaSource) -> anyhow::Result<MediaRef> {
        let registered = RegisteredMedia::from_source(source)?;
        let media_ref = MediaRef { name: registered.name.clone() };
        self.deck.media.push(registered);
        Ok(media_ref)
    }

    pub fn get(&self, name: &str) -> Option<MediaRef> {
        self.deck.media.iter().any(|media| media.name == name).then(|| MediaRef {
            name: name.to_string(),
        })
    }
}
```

```rust
// anki_forge/src/deck/lowering.rs
fn render_io_svg(mode: &IoMode, rects: &[IoRect]) -> String {
    let group_mode = match mode {
        IoMode::HideAllGuessOne => "hide_all_guess_one",
        IoMode::HideOneGuessOne => "hide_one_guess_one",
    };
    let shapes = rects
        .iter()
        .map(|rect| {
            format!(
                "<rect data-mode=\"{group_mode}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" />",
                rect.x, rect.y, rect.width, rect.height
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!("<svg data-anki-forge-io=\"{group_mode}\">{shapes}</svg>")
}

impl Deck {
    pub fn into_product_document(self) -> anyhow::Result<crate::product::ProductDocument> {
        let document_id = self.stable_id.clone().unwrap_or_else(|| self.name.clone());
        let mut product = crate::product::ProductDocument::new(document_id)
            .with_default_deck(self.name.clone())
            .with_basic("basic")
            .with_cloze("cloze")
            .with_image_occlusion("image_occlusion");

        for note in self.basic_notes {
            product = product.add_basic_note("basic", note.stable_id, self.name.clone(), note.front, note.back, note.tags);
        }
        for note in self.cloze_notes {
            product = product.add_cloze_note("cloze", note.stable_id, self.name.clone(), note.text, note.extra, note.tags);
        }
        for note in self.io_notes {
            product = product.add_image_occlusion_note(
                "image_occlusion",
                note.stable_id,
                self.name.clone(),
                note.raw_occlusion
                    .clone()
                    .unwrap_or_else(|| render_io_svg(&note.mode, &note.rects)),
                format!("<img src=\"{}\">", note.image.name),
                note.header,
                note.back_extra,
                note.comments,
                note.tags,
            );
        }

        Ok(product)
    }

    pub fn lower_authoring(&self) -> anyhow::Result<crate::AuthoringDocument> {
        let product = self.clone().into_product_document()?;
        let mut lowered = product.lower()?.authoring_document;
        lowered.media.extend(self.media.iter().map(|media| crate::AuthoringMedia {
            filename: media.name.clone(),
            mime: media.mime.clone(),
            data_base64: media.data_base64.clone(),
        }));
        Ok(lowered)
    }
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
    tags: Vec<String>,
) -> Self {
    self.notes.push(ProductNote::Basic(BasicNote {
        id: id.into(),
        note_type_id: note_type_id.into(),
        deck_name: deck_name.into(),
        front: front.into(),
        back: back.into(),
        tags,
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
    tags: Vec<String>,
) -> Self {
    self.notes.push(ProductNote::Cloze(ClozeNote {
        id: id.into(),
        note_type_id: note_type_id.into(),
        deck_name: deck_name.into(),
        text: text.into(),
        back_extra: back_extra.into(),
        tags,
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
    tags: Vec<String>,
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
        tags,
    }));
    self
}
```

```rust
// anki_forge/src/product/model.rs
pub struct BasicNote {
    pub id: String,
    pub note_type_id: String,
    pub deck_name: String,
    pub front: String,
    pub back: String,
    pub tags: Vec<String>,
}

pub struct ClozeNote {
    pub id: String,
    pub note_type_id: String,
    pub deck_name: String,
    pub text: String,
    pub back_extra: String,
    pub tags: Vec<String>,
}

pub struct ImageOcclusionNote {
    pub id: String,
    pub note_type_id: String,
    pub deck_name: String,
    pub occlusion: String,
    pub image: String,
    pub header: String,
    pub back_extra: String,
    pub comments: String,
    pub tags: Vec<String>,
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
```

- [ ] **Step 4: Run the lowering tests and updated product tests**

Run: `cargo test -p anki_forge --test deck_lowering_tests --test product_lowering_tests --test product_model_tests -v`
Expected: PASS with the new root-lowering test and updated stock-note tag assertions.

- [ ] **Step 5: Commit**

```bash
git add anki_forge/src/deck/mod.rs anki_forge/src/deck/model.rs anki_forge/src/deck/builders.rs anki_forge/src/deck/media.rs anki_forge/src/deck/lowering.rs anki_forge/src/product/model.rs anki_forge/src/product/builders.rs anki_forge/src/product/lowering.rs anki_forge/tests/deck_lowering_tests.rs anki_forge/tests/product_lowering_tests.rs anki_forge/tests/product_model_tests.rs
git commit -m "feat: add deck builders and lowering bridge"
```

### Task 3: Add structured validation, bytes-first export, and `BuildResult` symmetry for `Deck` and `Package`

**Files:**
- Create: `anki_forge/src/deck/validation.rs`
- Create: `anki_forge/src/deck/export.rs`
- Modify: `anki_forge/src/deck/mod.rs`
- Modify: `anki_forge/src/deck/model.rs`
- Test: `anki_forge/tests/deck_validation_tests.rs`
- Test: `anki_forge/tests/deck_export_tests.rs`

- [ ] **Step 1: Write the failing validation and export tests**

```rust
// anki_forge/tests/deck_validation_tests.rs
use anki_forge::{Deck, IoMode, MediaSource};

#[test]
fn validate_report_returns_structured_diagnostics_without_forcing_export() {
    let mut deck = Deck::new("Validation Demo");
    deck.basic()
        .note("", "back")
        .stable_id("blank-front")
        .add()
        .expect("builder-level insertion remains lightweight");

    let report = deck.validate_report().expect("validation report");

    assert!(!report.is_ok());
    assert!(report
        .diagnostics()
        .iter()
        .any(|item| item.code == "AF.DECK.BLANK_BASIC_FRONT"));
}

#[test]
fn export_surfaces_are_bytes_first_and_package_symmetric() {
    let mut deck = Deck::new("Spanish");
    deck.add_basic("hola", "hello").expect("add basic note");

    let bytes = deck.to_apkg_bytes().expect("deck bytes");
    assert!(!bytes.is_empty());

    let mut sink = std::io::Cursor::new(Vec::new());
    deck.write_to(&mut sink).expect("stream export");
    assert!(!sink.into_inner().is_empty());

    let package_bytes = anki_forge::Package::single(deck)
        .to_apkg_bytes()
        .expect("package bytes");
    assert!(!package_bytes.is_empty());
}
```

- [ ] **Step 2: Run the validation/export tests to verify they fail**

Run: `cargo test -p anki_forge --test deck_validation_tests --test deck_export_tests -v`
Expected: FAIL because `validate_report()`, `to_apkg_bytes()`, `write_to(...)`, `write_apkg(...)`, `build(...)`, and `BuildResult` do not exist yet.

- [ ] **Step 3: Implement validation diagnostics and export helpers**

```rust
// anki_forge/src/deck/validation.rs
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ValidationDiagnostic {
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ValidationReport {
    diagnostics: Vec<ValidationDiagnostic>,
}

impl ValidationReport {
    pub fn is_ok(&self) -> bool {
        self.diagnostics.is_empty()
    }

    pub fn diagnostics(&self) -> &[ValidationDiagnostic] {
        &self.diagnostics
    }
}

impl Deck {
    pub fn validate_report(&self) -> anyhow::Result<ValidationReport> {
        let mut diagnostics = Vec::new();

        for note in &self.basic_notes {
            if note.front.trim().is_empty() {
                diagnostics.push(ValidationDiagnostic {
                    code: "AF.DECK.BLANK_BASIC_FRONT",
                    message: format!("basic note '{}' has a blank front field", note.stable_id),
                });
            }
        }

        for note in &self.io_notes {
            if note.rects.is_empty() {
                diagnostics.push(ValidationDiagnostic {
                    code: "AF.DECK.EMPTY_IO_MASKS",
                    message: format!("image occlusion note '{}' requires at least one rect()", note.stable_id),
                });
            }
            if self.media.iter().all(|media| media.name != note.image.name) {
                diagnostics.push(ValidationDiagnostic {
                    code: "AF.DECK.UNKNOWN_MEDIA_REF",
                    message: format!("image occlusion note '{}' references missing media '{}'", note.stable_id, note.image.name),
                });
            }
        }

        Ok(ValidationReport { diagnostics })
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        let report = self.validate_report()?;
        anyhow::ensure!(report.is_ok(), "deck validation failed");
        Ok(())
    }
}
```

```rust
// anki_forge/src/deck/export.rs
use std::{fs, io::Write, path::{Path, PathBuf}};

pub struct BuildResult {
    inner: crate::PackageBuildResult,
    apkg_path: PathBuf,
    staging_manifest_path: PathBuf,
}

impl BuildResult {
    pub fn apkg_path(&self) -> &Path {
        &self.apkg_path
    }

    pub fn staging_manifest_path(&self) -> &Path {
        &self.staging_manifest_path
    }

    pub fn inspect_staging(&self) -> anyhow::Result<crate::InspectReport> {
        crate::inspect_staging(&self.staging_manifest_path)
    }

    pub fn inspect_apkg(&self) -> anyhow::Result<crate::InspectReport> {
        crate::inspect_apkg(&self.apkg_path)
    }
}

impl Package {
    pub fn build(&self, artifacts_dir: impl AsRef<Path>) -> anyhow::Result<BuildResult> {
        self.root_deck.validate()?;
        let lowered = self.root_deck.lower_authoring()?;
        let normalized = crate::normalize(crate::NormalizationRequest::new(lowered));
        let normalized_ir = normalized.normalized_ir.expect("validated facade must normalize");
        let target = crate::BuildArtifactTarget::new(artifacts_dir.as_ref().to_path_buf(), "artifacts");
        let writer_policy = crate::WriterPolicy {
            id: "writer-policy.default".into(),
            version: "1.0.0".into(),
            compatibility_target: "latest-only".into(),
            stock_notetype_mode: "source-grounded".into(),
            media_entry_mode: "inline".into(),
            apkg_version: "latest".into(),
        };
        let build_context = crate::BuildContext {
            id: "build-context.default".into(),
            version: "1.0.0".into(),
            emit_apkg: true,
            materialize_staging: true,
            media_resolution_mode: "inline-only".into(),
            unresolved_asset_behavior: "fail".into(),
            fingerprint_mode: "canonical".into(),
        };
        let inner = crate::build(&normalized_ir, &writer_policy, &build_context, &target)?;
        Ok(BuildResult {
            inner,
            apkg_path: artifacts_dir.as_ref().join("package.apkg"),
            staging_manifest_path: artifacts_dir.as_ref().join("staging/manifest.json"),
        })
    }

    pub fn to_apkg_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let temp = tempfile::tempdir()?;
        let build = self.build(temp.path())?;
        Ok(fs::read(build.apkg_path())?)
    }

    pub fn write_to<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        writer.write_all(&self.to_apkg_bytes()?)?;
        Ok(())
    }

    pub fn write_apkg(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        fs::write(path, self.to_apkg_bytes()?)?;
        Ok(())
    }
}

impl Deck {
    pub fn build(&self, artifacts_dir: impl AsRef<Path>) -> anyhow::Result<BuildResult> {
        Package::single(self.clone()).build(artifacts_dir)
    }

    pub fn to_apkg_bytes(&self) -> anyhow::Result<Vec<u8>> {
        Package::single(self.clone()).to_apkg_bytes()
    }

    pub fn write_to<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        Package::single(self.clone()).write_to(writer)
    }

    pub fn write_apkg(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        Package::single(self.clone()).write_apkg(path)
    }
}
```

- [ ] **Step 4: Run the validation/export tests and one end-to-end package build**

Run: `cargo test -p anki_forge --test deck_validation_tests --test deck_export_tests -v`
Expected: PASS with structured diagnostics and bytes-first exports.

Run: `cargo test -p anki_forge export_surfaces_are_bytes_first_and_package_symmetric -v`
Expected: PASS with a non-empty `.apkg` byte buffer from both `Deck` and `Package`.

- [ ] **Step 5: Commit**

```bash
git add anki_forge/src/deck/mod.rs anki_forge/src/deck/model.rs anki_forge/src/deck/validation.rs anki_forge/src/deck/export.rs anki_forge/tests/deck_validation_tests.rs anki_forge/tests/deck_export_tests.rs
git commit -m "feat: add deck validation and bytes-first export"
```

### Task 4: Finish the advanced bridge, docs, and examples

**Files:**
- Modify: `anki_forge/src/deck/lowering.rs`
- Modify: `anki_forge/src/deck/model.rs`
- Create: `anki_forge/examples/deck_basic_flow.rs`
- Modify: `README.md`
- Test: `anki_forge/tests/deck_lowering_tests.rs`

- [ ] **Step 1: Write the failing bridge/doc-path test**

```rust
// append to anki_forge/tests/deck_lowering_tests.rs
#[test]
fn deck_can_round_trip_through_product_document_for_stock_notes() {
    let mut deck = Deck::builder("Spanish")
        .stable_id("spanish-v1")
        .build();
    deck.basic()
        .note("hola", "hello")
        .tags(["vocab"])
        .stable_id("es-hola")
        .add()
        .expect("add basic note");

    let product = deck.clone().into_product_document().expect("bridge to product");
    let rebuilt = Deck::from_product_document(product).expect("bridge from product");

    assert_eq!(rebuilt.name(), "Spanish");
    let lowered = rebuilt.lower_authoring().expect("lower rebuilt deck");
    assert!(lowered.notes.iter().any(|note| note.id == "es-hola"));
    assert!(lowered.notes.iter().any(|note| note.tags == vec!["vocab"]));
}
```

- [ ] **Step 2: Run the bridge test to verify it fails**

Run: `cargo test -p anki_forge --test deck_lowering_tests deck_can_round_trip_through_product_document_for_stock_notes -v`
Expected: FAIL because `Deck::from_product_document(...)` does not exist yet.

- [ ] **Step 3: Implement the reverse bridge and update the docs/examples**

```rust
// anki_forge/src/deck/lowering.rs
fn extract_img_src_name(image_html: &str) -> &str {
    image_html
        .split("src=\"")
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .unwrap_or(image_html)
}

impl Deck {
    pub fn from_product_document(product: crate::product::ProductDocument) -> anyhow::Result<Self> {
        let mut deck = Deck::builder(product.default_deck_name().unwrap_or("Default"))
            .stable_id(product.document_id())
            .build();

        for note in product.notes() {
            match note {
                crate::product::ProductNote::Basic(note) => {
                    deck.basic()
                        .note(note.front.clone(), note.back.clone())
                        .stable_id(note.id.clone())
                        .tags(note.tags.clone())
                        .add()?;
                }
                crate::product::ProductNote::Cloze(note) => {
                    deck.cloze()
                        .note(note.text.clone())
                        .extra(note.back_extra.clone())
                        .stable_id(note.id.clone())
                        .tags(note.tags.clone())
                        .add()?;
                }
                crate::product::ProductNote::ImageOcclusion(note) => {
                    let media_ref = deck.media().get(extract_img_src_name(&note.image))
                        .unwrap_or(MediaRef { name: extract_img_src_name(&note.image).to_string() });
                    deck.image_occlusion()
                        .note(media_ref)
                        .stable_id(note.id.clone())
                        .header(note.header.clone())
                        .back_extra(note.back_extra.clone())
                        .comments(note.comments.clone())
                        .raw_occlusion(note.occlusion.clone())
                        .tags(note.tags.clone())
                        .add()?;
                }
                crate::product::ProductNote::Custom(_) => anyhow::bail!("custom product notes require product-layer editing"),
            }
        }

        Ok(deck)
    }
}
```

```rust
// anki_forge/examples/deck_basic_flow.rs
use anki_forge::Deck;

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

    deck.write_apkg("spanish.apkg")?;
    Ok(())
}
```

```md
<!-- README.md -->
## Rust Quick Start

```rust
use anki_forge::Deck;

let mut deck = Deck::new("Spanish");
deck.add_basic("hola", "hello")?;
deck.write_apkg("spanish.apkg")?;
```

For advanced authoring, use `anki_forge::product`.
For file-driven pipeline work, use `anki_forge::runtime`.
```

- [ ] **Step 4: Run the bridge test, the new example, and the full `anki_forge` test suite**

Run: `cargo test -p anki_forge --test deck_lowering_tests -v`
Expected: PASS with the round-trip bridge test.

Run: `cargo run -p anki_forge --example deck_basic_flow`
Expected: PASS and write a valid `spanish.apkg` in the current working directory.

Run: `cargo test -p anki_forge -v`
Expected: PASS with the new root-facade tests and existing product/runtime suites still green.

- [ ] **Step 5: Commit**

```bash
git add anki_forge/src/deck/lowering.rs anki_forge/examples/deck_basic_flow.rs README.md anki_forge/tests/deck_lowering_tests.rs
git commit -m "docs: publish rust north-star deck api"
```

## Verification Checklist

Run these commands before claiming the plan is fully executed:

```bash
cargo fmt --all
cargo test -p anki_forge --test deck_model_tests --test deck_lowering_tests --test deck_validation_tests --test deck_export_tests -v
cargo test -p anki_forge -v
cargo run -p anki_forge --example deck_basic_flow
cargo run -p anki_forge --example product_basic_flow
```

Expected outcomes:

1. All new root-facade tests pass.
2. Existing `product` tests still pass with stock-note tags flowing through lowering.
3. `deck_basic_flow` writes a valid `.apkg`.
4. `product_basic_flow` still demonstrates the advanced layer unchanged.
