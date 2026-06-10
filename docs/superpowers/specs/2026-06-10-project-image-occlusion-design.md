# Project Image Occlusion Design

## Goal

Direct `Project` authoring must support the stock Anki Image Occlusion note type without forcing users to drop to `Deck` or hand-write ProductDocument internals. The implementation should connect the existing Product model and builder support through Rust `Project`, product-v2 lowering, and the Python product-v2 binding.

## Current State

The lower Product layer already has `ProductNoteType::ImageOcclusion`, `ProductNote::ImageOcclusion`, `ProductDocument::with_image_occlusion`, and `ProductDocument::add_image_occlusion_note_with_tags`. `Deck` already exposes `deck.image_occlusion()` and lowers through those ProductDocument helpers.

Direct `Project` authoring is incomplete:

- `Project::validate()` only treats `basic` and `cloze` as supported implicit stock note types.
- `Project::implicit_stock_notetype_ids()` only discovers `basic` and `cloze`.
- `Project::to_product_document()` only auto-declares and converts `basic` and `cloze` notes; other note type ids fall through as custom notes.
- product-v2 lowering only accepts stock ids `basic` and `cloze`.
- product-v2 stock note field mapping only knows `basic` and `cloze` keys.
- Python Project output is product-v2 and only recognizes `basic` and `cloze` stock ids and field keys.

## Scope

This design covers direct Rust `Project` Image Occlusion authoring, product-v2 Image Occlusion stock lowering, Python product-v2 serialization support, and focused integration tests.

This design does not change `Deck` Image Occlusion behavior. `Project::from(deck)` should continue using the existing Deck lowering path unchanged.

This design does not implement full Deck-parity inferred Image Occlusion identity for direct Project notes. That requires Product media raster metadata and shared IO identity extraction, which should be designed separately.

## Rust Public API

Add a structured Project note builder in `anki_forge/src/product/note.rs`:

```rust
pub struct ImageOcclusionNoteBuilder {
    image: crate::product::MediaRef,
    mode: crate::IoMode,
    rects: Vec<crate::IoRect>,
    stable_id: Option<String>,
    deck_name: Option<String>,
    header: String,
    back_extra: String,
    comments: String,
    tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProductNoteError {
    ImageOcclusionStableIdMissing,
    ImageOcclusionStableIdBlank,
    ImageOcclusionEmptyMasks,
    ImageOcclusionRectEmpty,
    ImageOcclusionRectDuplicate,
}

impl Note {
    pub fn image_occlusion(image: crate::product::MediaRef) -> ImageOcclusionNoteBuilder;
}

impl ImageOcclusionNoteBuilder {
    pub fn stable_id(self, stable_id: impl Into<String>) -> Self;
    pub fn deck(self, deck_name: impl Into<String>) -> Self;
    pub fn mode(self, mode: crate::IoMode) -> Self;
    pub fn rect(self, x: u32, y: u32, width: u32, height: u32) -> Self;
    pub fn rects<I>(self, rects: I) -> Self
    where
        I: IntoIterator<Item = crate::IoRect>;
    pub fn header(self, header: impl Into<String>) -> Self;
    pub fn back_extra(self, back_extra: impl Into<String>) -> Self;
    pub fn comments(self, comments: impl Into<String>) -> Self;
    pub fn tag(self, tag: impl Into<String>) -> Self;
    pub fn tags<T, I>(self, tags: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<String>;
    pub fn build(self) -> Result<Note, ProductNoteError>;
}
```

`ProductNoteError` implements `std::error::Error`, `Display`, and `crate::diagnostics::ErrorCodeExt`.

Error code mapping:

- `ProductNoteError::ImageOcclusionEmptyMasks` maps to `ErrorCode::ImageOcclusionEmptyMasks` (`DECK.EMPTY_IO_MASKS`).
- `ProductNoteError::ImageOcclusionStableIdMissing` maps to `ErrorCode::DeckMissingStableId` (`DECK.MISSING_STABLE_ID`).
- `ProductNoteError::ImageOcclusionStableIdBlank` maps to `ErrorCode::StableIdBlank` (`DECK.BLANK_STABLE_ID`).
- `ProductNoteError::ImageOcclusionRectEmpty` maps to `ErrorCode::ImageOcclusionRectEmpty` (`AFID.IO_RECT_EMPTY`).
- `ProductNoteError::ImageOcclusionRectDuplicate` maps to `ErrorCode::ImageOcclusionRectDuplicate` (`AFID.IO_RECT_DUPLICATE`).

`AFID.*` is an existing Anki Forge identity and deterministic-id diagnostic namespace in `crate::diagnostics`. This design reuses existing AFID IO geometry codes and does not introduce a new diagnostic domain.

The builder should be re-exported from `crate::product` as `ImageOcclusionNoteBuilder`. `IoMode` and `IoRect` are already available from the crate root through the Deck exports; `crate::prelude` should also export `IoMode` for Project examples.

## Builder Semantics

`Note::image_occlusion(image)` starts a stock Image Occlusion note with no rects and default mode `IoMode::HideAllGuessOne`.

`stable_id(...)` is mandatory for this builder. `build()` returns `ProductNoteError::ImageOcclusionStableIdMissing` when `stable_id(...)` was not called and `ProductNoteError::ImageOcclusionStableIdBlank` when the provided stable id is blank after trimming. The builder stores the trimmed stable id in the returned note.

The current `IoMode` enum variants are `IoMode::HideAllGuessOne` and `IoMode::HideOneGuessOne`. Both are valid for the builder. `render_image_occlusion_cloze(...)` already matches these variants; adding a future `IoMode` variant requires updating that helper, this builder's tests, and the Python mode mapping in the same change.

Repeated `mode(...)` calls use last-write-wins semantics. Repeated `stable_id(...)`, `deck(...)`, `header(...)`, `back_extra(...)`, and `comments(...)` calls also use last-write-wins semantics.

`rect(x, y, width, height)` appends one rectangle. Chaining accumulates masks:

```rust
Note::image_occlusion(image)
    .rect(10, 20, 30, 40)
    .rect(100, 20, 30, 40)
    .build()?;
```

`rects(...)` appends all provided rectangles after any existing rectangles.

`build()` validates only geometry that does not require image metadata:

- no rectangles returns `ProductNoteError::ImageOcclusionEmptyMasks`;
- any rectangle with `width == 0` or `height == 0` returns `ProductNoteError::ImageOcclusionRectEmpty`;
- duplicate `(x, y, width, height)` rectangles return `ProductNoteError::ImageOcclusionRectDuplicate`.

Negative coordinates are impossible because the API uses `u32`. Out-of-bounds checks are deferred because Product `MediaRef` does not expose image dimensions. The separate identity-parity follow-up should add Product media raster metadata and reuse or extract Deck IO geometry validation.

The builder does not prove that a `MediaRef` belongs to the same `Project`. Product `MediaRef` is currently an export-filename handle and has no registry ownership token. Missing or cross-project media should surface through existing media reference validation during normalize/build. The builder documentation must state that callers should create the image via `project.media_mut().add_file(...).export_as(...)` or `add_bytes(...).export_as(...)` before building the note.

`deck(...)` follows existing `product::Note::deck(...)` behavior. When set, it overrides the Project default deck for the note. When left unset, `Project::to_product_document()` uses the Project default deck or Project name, as it already does for `basic`, `cloze`, and custom notes. The builder does not add deck name validation beyond existing Project behavior.

## Built Note Shape

`build()` returns a regular `product::Note`:

- `note_type_id = STOCK_IMAGE_OCCLUSION_ID`;
- trimmed `stable_id` copied from the builder setter;
- `deck_name` copied from the builder setter when present;
- `Occlusion` set to the exact string produced by `render_image_occlusion_cloze(mode, &rects)` after builder validation has rejected empty rect collections;
- `Image` set to the Product media image HTML, using the export filename from `MediaRef`;
- `Header`, `Back Extra`, and `Comments` always present, defaulting to empty strings;
- tags copied from the builder.

The rendered `Image` field must use the Product media export filename, not the original source filename. `MediaRef::filename()` currently represents the export filename, and `MediaRef::image()` renders `<img src="...">`; tests should lock this behavior.

The existing `render_image_occlusion_cloze` function lives in `anki_forge/src/product/stock.rs`, is public, and is already re-exported from `crate::product`. The builder should call that Product stock helper rather than introducing a separate renderer.

The helper's doc comment should state that callers must pass at least one rect. The Image Occlusion builder enforces that precondition before calling it.

## Rust Project Plumbing

Add a private stock helper near Project/Product lowering code:

```rust
fn is_supported_stock_notetype_id(id: &str) -> bool {
    matches!(
        id,
        STOCK_BASIC_ID | STOCK_CLOZE_ID | STOCK_IMAGE_OCCLUSION_ID
    )
}
```

Use it in the Project validation and product-v2 stock whitelist paths so supported stock ids do not drift again.

`Project::validate()` must treat `image_occlusion` as supported when no custom note type exists. A direct Image Occlusion note must not produce `PROJECT.UNSUPPORTED_NOTE_TYPE`.

`Project::implicit_stock_notetype_ids()` must include `STOCK_IMAGE_OCCLUSION_ID` when any direct Project note uses that id. The order should stay deterministic as `basic`, `cloze`, `image_occlusion` because existing tests already reason about implicit stock declarations before custom note types.

If a custom `NoteType::custom("image_occlusion")` exists while an implicit stock Image Occlusion note exists, validation should produce `NOTETYPE.ID_DUPLICATE` with the same implicit-stock duplicate message pattern used for `basic` and `cloze`.

`Project::to_product_document()` must:

- call `with_image_occlusion(STOCK_IMAGE_OCCLUSION_ID)` when direct notes use `image_occlusion`;
- convert direct Image Occlusion notes with `add_image_occlusion_note_with_tags(...)`;
- read rendered fields by visible stock field name: `Occlusion`, `Image`, `Header`, `Back Extra`, `Comments`;
- default absent rendered fields to empty strings, matching existing `basic` and `cloze` conversion behavior.

## product-v2 Lowering

product-v2 stock note type declarations should accept `image_occlusion` in addition to `basic` and `cloze`.

`lower_product_v2_stock_note()` should use an explicit stock field map helper:

```rust
fn stock_field_map(note_type_id: &str) -> &'static [(&'static str, &'static str)] {
    match note_type_id {
        "basic" => &[("front", "Front"), ("back", "Back")],
        "cloze" => &[("text", "Text"), ("back_extra", "Back Extra")],
        "image_occlusion" => &[
            ("occlusion", "Occlusion"),
            ("image", "Image"),
            ("header", "Header"),
            ("back_extra", "Back Extra"),
            ("comments", "Comments"),
        ],
        _ => &[],
    }
}
```

The `image` field may be text, html, or typed image content. Typed image content must continue through `render_v2_content(...)`, so `{"kind":"image","media_id":"media:heart"}` renders to `<img src="heart.png">` based on the product-v2 media declaration.

Required field behavior comes from the product-v2 stock declaration. For Python-generated IO declarations, `occlusion`, `image`, `header`, and `back_extra` are required and `comments` is optional, matching authoring stock defaults.

Unknown fields on stock IO notes should produce `PRODUCT.FIELD_UNKNOWN` with source paths like `project.notes[0].fields["extra"]`, consistent with existing basic/cloze behavior.

If hand-written product-v2 input declares the Image Occlusion stock note type but all IO notes are skipped because of lowering errors such as `PRODUCT.IDENTITY_MISSING`, the lowering plan may still contain the stock note type and zero notes. This is valid intermediate lowering state because the accompanying product diagnostic makes build fail before producing a successful APKG. The implementation should not suppress the stock note type declaration as an error recovery mechanism.

## product-v2 Identity

When a product-v2 stock Image Occlusion note has `stable_id`, lowering uses that explicit stable id.

When a hand-written product-v2 stock Image Occlusion note lacks `stable_id`, `lower_product_v2_stock_note()` should emit `PRODUCT.IDENTITY_MISSING` at the note source path and skip lowering that note. This uses the existing diagnostic code already emitted by `lower_product_v2_stock_note()` when basic/cloze stock identity derivation fails.

The implementation must not derive IO identity from `Occlusion`, `Image`, or cloze text fields. `Note::image_occlusion(...).build()` prevents builder-created notes without explicit stable ids. Raw low-level `Note::new("image_occlusion")` values without `stable_id` may continue the existing direct Project generated fallback, because `Project::to_product_document()` resolves a note id before producing ProductDocument notes. That fallback is not the recommended IO API and should be covered only by compatibility tests.

## Python Binding

Python should support Image Occlusion in the same capability slice because Python Project serializes product-v2 and otherwise remains blocked by the v2 lowering changes.

Add:

- `STOCK_NOTE_TYPE_IDS = {"basic", "cloze", "image_occlusion"}`;
- stock field keys for `image_occlusion`: `{"occlusion", "image", "header", "back_extra", "comments"}`;
- `image_occlusion_stock_notetype_json()` in `bindings/python/src/anki_forge/product_json.py`;
- a Python `ImageOcclusionNoteBuilder` in `bindings/python/src/anki_forge/note.py`;
- `Note.image_occlusion(image: MediaRef, *, stable_id: str | None = None, deck_name: str | None = None) -> ImageOcclusionNoteBuilder`.

Python builder methods:

```python
builder.mode("hide_all_guess_one" | "hide_one_guess_one") -> ImageOcclusionNoteBuilder
builder.rect(x: int, y: int, width: int, height: int) -> ImageOcclusionNoteBuilder
builder.rects(rects: Iterable[tuple[int, int, int, int]]) -> ImageOcclusionNoteBuilder
builder.header(value: str) -> ImageOcclusionNoteBuilder
builder.back_extra(value: str) -> ImageOcclusionNoteBuilder
builder.comments(value: str) -> ImageOcclusionNoteBuilder
builder.tag(value: str) -> ImageOcclusionNoteBuilder
builder.tags(values: Iterable[str]) -> ImageOcclusionNoteBuilder
builder.build() -> Note
```

Python `rect()` accumulates masks. Python `rects(...)` appends all provided rectangles after any existing rectangles. `build()` rejects a missing stable id, blank stable id, no rects, non-integer or negative coordinates, zero width, zero height, and duplicate rectangles with `ValidationError`. Python cannot check image bounds for the same reason Rust cannot.

Python `mode(...)` uses last-write-wins semantics and validates the value immediately against `hide_all_guess_one` and `hide_one_guess_one`. The Python builder renders the occlusion string directly from this validated mode string; Rust `IoMode` is not involved in Python serialization.

Python should render `occlusion` with the same string format as Rust:

- hide-all-guess-one prefix: `c1`;
- hide-one-guess-one prefix: `c1,2`;
- each rect renders `{{<prefix>::image-occlusion:rect:left=<x>:top=<y>:width=<w>:height=<h>}}}<br>`.

Python `ImageOcclusionNoteBuilder.build()` should create the returned `Note` by calling `Note("image_occlusion", stable_id=..., deck_name=...).html("occlusion", rendered_occlusion).image("image", image_ref).text("header", header).text("back_extra", back_extra).text("comments", comments)`, so product-v2 serialization writes `{"kind":"image","media_id":"..."}` for the `image` field and Rust lowering resolves the export filename.

`Project.to_product_document()` should include `image_occlusion_stock_notetype_json()` whenever any note uses the `image_occlusion` stock id.

Rust and Python intentionally store the `Image` field at different layers. The Rust builder returns a `product::Note`, whose fields are already rendered HTML, so the `Image` field stores `<img src="...">`. The Python builder returns a Python `Note` that serializes to product-v2 typed content, so the `image` field stores the media id until Rust product-v2 lowering renders `<img src="...">`.

## Tests

Rust Project API tests:

- `note_image_occlusion_builder_accumulates_rects_and_renders_fields`: builds a note with two `.rect(...)` calls and asserts `note_type_id`, `Occlusion`, `Image`, `Header`, `Back Extra`, `Comments`, and tags.
- `note_image_occlusion_builder_requires_stable_id`: `build()` without `stable_id(...)` returns `ProductNoteError::ImageOcclusionStableIdMissing`.
- `note_image_occlusion_builder_rejects_blank_stable_id`: blank stable id returns `ProductNoteError::ImageOcclusionStableIdBlank`.
- `note_image_occlusion_builder_rejects_empty_masks`: `build()` returns `ProductNoteError::ImageOcclusionEmptyMasks`.
- `note_image_occlusion_builder_rejects_empty_rect`: zero width or height returns `ProductNoteError::ImageOcclusionRectEmpty`.
- `note_image_occlusion_builder_rejects_duplicate_rect`: duplicate geometry returns `ProductNoteError::ImageOcclusionRectDuplicate`.
- `project_validate_accepts_stock_image_occlusion`: direct `Note::new("image_occlusion").stable_id(...).html("Occlusion", ...).image("Image", image)` does not produce `PROJECT.UNSUPPORTED_NOTE_TYPE`.
- `project_validate_reports_custom_image_occlusion_collision_with_implicit_stock`: collision with `NoteType::custom("image_occlusion")` reports `NOTETYPE.ID_DUPLICATE` using the implicit-stock message.
- `project_image_occlusion_build_writes_apkg`: full media registration to builder to `write_apkg`, expecting success, one note, one card, and one media item.
- `project_image_occlusion_builder_missing_stable_id_fails_before_project_add_note`: `Note::image_occlusion(image).rect(...).build()` fails before a Project build can silently discard the note.
- `project_raw_image_occlusion_without_stable_id_keeps_generated_fallback`: raw `Note::new("image_occlusion")` without stable id lowers through the existing direct Project generated-id behavior and does not use the builder path.
- `project_image_occlusion_cross_project_media_reports_missing_reference`: an IO builder note created with a `MediaRef` not registered in the target Project should fail through the existing media-reference diagnostics during normalize/build.
- `project_image_occlusion_lower_matches_deck_product_shape`: build equivalent Deck and Project IO notes with the same explicit stable id, compare the resulting ProductDocument note type id and fields after lowering to ProductDocument-level `ProductNote::ImageOcclusion`.

product-v2 lowering tests:

- `product_v2_stock_image_occlusion_lowers_to_authoring_fields`: stock IO declaration and note lower to stock `original_stock_kind = "image_occlusion"` with all five fields.
- `product_v2_stock_image_occlusion_typed_image_resolves_media`: typed image field resolves to `<img src="heart.png">`.
- `product_v2_stock_image_occlusion_unknown_field_source_path`: unknown key reports `PRODUCT.FIELD_UNKNOWN` at the field source path.
- `product_v2_stock_image_occlusion_missing_required_field`: missing `image` or `occlusion` reports `PRODUCT.REQUIRED_FIELD_MISSING`.
- `product_v2_stock_image_occlusion_without_stable_id_is_identity_missing`: missing stable id reports `PRODUCT.IDENTITY_MISSING` and skips the note.

Python tests:

- `test_note_image_occlusion_builder_outputs_product_v2_stock_note`: Python builder serializes a stock IO note and includes typed image content.
- `test_image_occlusion_renderer_matches_rust_expected_strings`: Python renderer output for both modes matches the exact strings produced by Rust `render_image_occlusion_cloze(...)` for the same fixed rect fixtures.
- `test_project_declares_image_occlusion_stock_notetype`: Python Project includes IO stock declaration only when used.
- `test_image_occlusion_builder_rejects_missing_or_blank_stable_id`: missing and blank stable ids raise `ValidationError`.
- `test_image_occlusion_builder_rejects_bad_rects`: no masks, zero dimensions, negative coordinates, duplicate rectangles raise `ValidationError`.
- `test_python_image_occlusion_runtime_build`: Python runtime path writes product-v2 and Rust runtime builds an APKG with one note, one card, and one media item.

Regression tests:

- Existing basic/cloze/custom Project tests continue passing.
- Existing Deck IO tests continue passing.
- Existing `Project::from(deck)` IO facade test remains unchanged.

## Documentation

Add a Rust example near the Project media docs:

```rust
let mut project = Project::new("Anatomy")
    .stable_id("anatomy")
    .default_deck("Anatomy");
let image = project
    .media_mut()
    .add_file("heart.png")?
    .export_as("heart.png")?;

project.add_note(
    Note::image_occlusion(image)
        .stable_id("heart:io:1")
        .mode(IoMode::HideAllGuessOne)
        .rect(10, 20, 80, 40)
        .header("Heart")
        .back_extra("Identify the structure")
        .build()?,
)?;
```

The docs must state that direct Project IO currently validates mask presence, non-zero rect dimensions, and duplicate rects, but not image bounds. The docs must also state that `Note::image_occlusion(...).build()` requires an explicit stable id, and raw low-level `Note::new("image_occlusion")` usage should provide an explicit stable id until Product media identity parity is implemented.

## Follow-Up

A separate design should add Product media raster metadata and shared IO identity logic so direct Project IO can infer the same `io.core.v2` stable ids as Deck IO. That follow-up should decide whether Product `MediaRef` needs registry ownership, image dimensions, content hash exposure, or a new internal identity context.
