# Anki Forge Rust North-Star API Design

- Date: 2026-04-10
- Status: Approved in brainstorming, updated after review
- Scope: top-level Rust authoring API shape after Phase 5A
- Related specs:
  - `2026-03-27-anki-forge-platform-phasing-design.md`
  - `2026-04-07-phase-5a-product-authoring-features-design.md`

## 1. Purpose

This document records the decided Rust north-star API for `anki-forge`.

The API must make the default authoring journey feel close to a deck authoring
tool, while preserving the contract-first platform and the existing
`product -> lowering -> normalize -> build` pipeline.

The main default task is:

> create notes, put them in a deck, and export an `.apkg`

## 2. Confirmed API Facts

The following points are the decided facts for the Rust surface.

### 2.1 Default root types

The crate root should expose these default author-facing types:

1. `Deck`
2. `Package`
3. `BuildResult`
4. `ValidationReport`
5. `MediaSource`
6. `MediaRef`
7. `IoMode`

### 2.2 Default versus advanced entry points

The default author-facing root is:

```rust
anki_forge::Deck
```

The advanced roots remain:

1. `anki_forge::product`
2. `anki_forge::runtime`

The split is by entry point and documentation emphasis, not by names such as
`easy` and `pro`.

### 2.3 Deck constructors

`Deck::new(name)` is the shortest constructor.

`Deck::builder(name)` also exists for lightweight root configuration such as
stable identity, without forcing users into advanced modules.

### 2.4 Package exists from the start

`Package` is introduced now, even if the main tutorial continues to start from
`Deck`.

`Package` represents a single root-deck `.apkg` package.

It does not model multi-root `.apkg` composition.

Future collection-wide export is a separate concern and should be expressed
through a future `CollectionPackage` or `write_colpkg(...)` API, not by letting
`.apkg` drift into a multi-deck collection abstraction.

The simple path:

```rust
deck.write_apkg("spanish.apkg")?;
```

is semantically equivalent to:

```rust
Package::single(deck).write_apkg("spanish.apkg")?;
```

This keeps `Deck` and `Package` aligned without teaching an `.apkg` model that
conflicts with Anki's deck-package boundary.

## 3. Authoring Model Facts

### 3.1 Default deck workflow

The default deck workflow is:

1. create a `Deck`
2. optionally assign stable identity
3. add notes
4. optionally run explicit preflight validation
5. export bytes, writer output, or a filesystem artifact

### 3.2 Lane methods are sugar

`Deck` exposes explicit note-type lanes:

1. `deck.basic()`
2. `deck.cloze()`
3. `deck.image_occlusion()`

These lane methods are ergonomic sugar.

The lower-level authoring primitive is:

```rust
deck.add(note)
```

The model must support owned note DTOs so bindings and structured producers can
construct notes directly.

Planned owned note DTOs:

1. `BasicNote`
2. `ClozeNote`
3. `IoNote`
4. `CustomNote`

### 3.3 Stable identity is a default concern

Update-friendly identity is part of the default layer.

The default API should expose:

1. `stable_id(...)`

At minimum:

1. deck/package-level stable identity
2. note-level stable identity
3. optional lower-level `guid(...)` escape hatch for advanced cases

Repeated exports are expected to support stable evolution of authored notes, so
stable identity cannot be treated as an advanced-only concern.

Stable note updates are not the only invariant that matters during evolution.

If the advanced layer later exposes custom note-type merge and note-type
evolution, the implementation must also preserve stable field and template
identifiers for update-friendly notetype changes.

### 3.4 Deck is a façade, not the ownership truth

`Deck` is an author-facing entry point.

The internal model must continue to preserve distinct concepts for:

1. notes
2. note types
3. package composition

This avoids baking the incorrect assumption that note types are fundamentally
deck-owned.

### 3.5 Escape hatches to the advanced layer

The default layer and the advanced `product` layer must interoperate without
forcing users to rewrite their work.

The API should therefore support conversions such as:

1. `into_product_document()`
2. `from_product_document()`

## 4. Standard Note Type Facts

### 4.1 Basic

The initial `Basic` lane may start with the common single-card path.

However, the internal model must not permanently freeze `Basic` into:

- exactly two fields
- exactly one generated card

The design must leave room for standard basic-family variants such as reversed,
optional reversed, and type-answer variants.

### 4.2 Cloze

The default `Cloze` lane must expose `Extra` directly.

North-star shape:

```rust
deck.cloze()
    .note("La capital de Espana es {{c1::Madrid}}")
    .extra("Europe")
    .add()?;
```

### 4.3 Image Occlusion

The default `Image Occlusion` lane must expose the commonly-used visible and
behavioral fields directly.

Required default-level controls:

1. `mode(...)`
2. `header(...)`
3. `back_extra(...)`
4. `comments(...)`

`comments()` must not exist without `back_extra()`, because `Back Extra` is a
standard visible field in the default card path.

The first pass intentionally narrows author-friendly geometry input to
rectangles through `rect(...)`.

This is a deliberate first-pass boundary, not a statement that native Image
Occlusion only supports rectangles.

Future expansion may add ellipse and polygon helpers while keeping the same
lane-level authoring shape.

## 5. Media Model Facts

The default API separates:

1. registering media
2. referencing media from notes

The core abstractions are:

1. `MediaSource`
2. `MediaRef`

Minimum source constructors:

1. `MediaSource::from_file(...)`
2. `MediaSource::from_bytes(name, bytes)`

Existing named-media lookup belongs on the media registry side, for example:

```rust
deck.media().get("heart.png")
```

The note side should reference `MediaRef`, not raw paths.

The default API should not encourage template-driven filename assembly for
dynamic note media. Notes should reference registered media explicitly through
`MediaRef`.

This avoids making filesystem paths the core abstraction and keeps the design
portable to bindings, services, and memory-only environments.

## 6. Validation and Export Facts

### 6.1 Validation

Validation is part of the default layer.

`ValidationReport` is the structured diagnostics form for bindings, services,
and IDE-facing integrations.

The explicit preflight entry points are:

```rust
deck.validate()?;
```

```rust
let report = deck.validate_report()?;
```

Validation should occur at three levels:

1. lightweight structural checks during `add_*()` or builder `.add()`
2. explicit optional preflight validation via `validate()` or
   `validate_report()`
3. full validation during `build()` and export methods

The main happy path should not require calling `validate()` before export.

`validate()` is a preflight checkpoint, not a mandatory gate users must call by
hand before `write_apkg()`, `to_apkg_bytes()`, or `build()`.

### 6.2 Export outputs

The default export surface must support:

1. `to_apkg_bytes()`
2. `write_to<W: Write>(...)`
3. `write_apkg(path)`
4. `build(dir)`

`write_apkg(path)` is path sugar, not the only export route.

This keeps Rust services, future bindings, and WASM-style environments on the
main path instead of making temp files mandatory.

`Deck` and `Package` should expose the same export and build surface.

`Deck` methods are sugar over `Package::single(deck)`.

### 6.3 BuildResult

`BuildResult` represents the advanced build result for a single root-deck
package run.

It should provide:

1. `.apkg` path access
2. staging manifest path access
3. underlying build metadata access
4. convenience inspection helpers

## 7. Detailed Examples

### 7.1 Shortest Basic happy path

```rust
use anki_forge::Deck;

let mut deck = Deck::new("Spanish");
deck.add_basic("hola", "hello")?;
deck.write_apkg("spanish.apkg")?;
```

### 7.2 Basic notes with stable identity and tags

```rust
use anki_forge::Deck;

let mut deck = Deck::builder("Spanish")
    .stable_id("spanish-v1")
    .build();

deck.basic()
    .note("adios", "goodbye")
    .stable_id("es-adios")
    .tags(["vocab", "a1"])
    .add()?;

let bytes = deck.to_apkg_bytes()?;
std::fs::write("spanish.apkg", bytes)?;
```

### 7.3 Cloze with `Extra`

```rust
use anki_forge::Deck;

let mut deck = Deck::new("Geography");

deck.cloze()
    .note("La capital de Espana es {{c1::Madrid}}")
    .extra("Europe")
    .stable_id("geo-es-capital")
    .add()?;

deck.write_apkg("geography.apkg")?;
```

### 7.4 Image Occlusion with explicit media registration

```rust
use anki_forge::{Deck, IoMode, MediaSource};

let mut deck = Deck::new("Anatomy");

let heart = deck.media().add(MediaSource::from_file("heart.png"))?;

deck.image_occlusion()
    .note(heart)
    .mode(IoMode::HideAllGuessOne)
    .rect(10, 20, 80, 40)
    .rect(120, 60, 70, 35)
    .header("Heart")
    .back_extra("Identify the chamber")
    .comments("Left ventricle")
    .stable_id("anatomy-heart-1")
    .add()?;

deck.write_apkg("anatomy.apkg")?;
```

### 7.5 Memory-only export

```rust
use anki_forge::Deck;

let mut deck = Deck::new("Service Output");
deck.add_basic("q", "a")?;

let bytes = deck.to_apkg_bytes()?;
assert!(!bytes.is_empty());
```

### 7.6 Stream-oriented export

```rust
use std::io::Cursor;

use anki_forge::Deck;

let mut deck = Deck::new("Service Output");
deck.add_basic("q", "a")?;

let mut sink = Cursor::new(Vec::new());
deck.write_to(&mut sink)?;
let bytes = sink.into_inner();
assert!(!bytes.is_empty());
```

### 7.7 Optional explicit preflight validation

```rust
use anki_forge::Deck;

let mut deck = Deck::new("Validation Demo");
deck.add_basic("front", "back")?;

let report = deck.validate_report()?;
assert!(report.is_ok());

deck.validate()?;
deck.write_apkg("validation-demo.apkg")?;
```

### 7.8 Build-oriented advanced output

```rust
use anki_forge::Deck;

let mut deck = Deck::new("Debug Build");
deck.add_basic("front", "back")?;

let build = deck.build("out/debug-build")?;

println!("{}", build.apkg_path().display());
println!("{}", build.staging_manifest_path().display());

let staging = build.inspect_staging()?;
let apkg = build.inspect_apkg()?;
let diff = build.diff_artifacts()?;

assert_eq!(staging.observation_status, "complete");
assert_eq!(apkg.observation_status, "complete");
assert_eq!(diff.comparison_status, "complete");
```

### 7.9 Package symmetry for bytes and writer output

```rust
use anki_forge::{Deck, Package};

let mut spanish = Deck::new("Spanish");
spanish.add_basic("hola", "hello")?;

let package = Package::single(spanish);
let bytes = package.to_apkg_bytes()?;
assert!(!bytes.is_empty());
```

### 7.10 Package build symmetry

```rust
use anki_forge::{Deck, Package};

let mut spanish = Deck::new("Spanish");
spanish.add_basic("hola", "hello")?;

let build = Package::single(spanish).build("out/spanish-package")?;
println!("{}", build.apkg_path().display());
```

### 7.11 Media from named bytes

```rust
use anki_forge::{Deck, IoMode, MediaSource};

let mut deck = Deck::new("Anatomy");

let heart_bytes = std::fs::read("heart.png")?;
let heart = deck
    .media()
    .add(MediaSource::from_bytes("heart.png", heart_bytes))?;

deck.image_occlusion()
    .note(heart)
    .mode(IoMode::HideAllGuessOne)
    .rect(10, 20, 80, 40)
    .back_extra("Identify the chamber")
    .add()?;
```

### 7.12 Escape hatch to advanced authoring

```rust
use anki_forge::Deck;

let mut deck = Deck::new("Spanish");
deck.add_basic("hola", "hello")?;

let product = deck.into_product_document()?;

// advanced product-layer edits happen here

let deck = Deck::from_product_document(product)?;
deck.write_apkg("spanish.apkg")?;
```

## 8. First Implementation Scope

The first implementation pass of the crate-root API should cover:

1. `Deck::new(...)`
2. `Deck::builder(...).stable_id(...).build()`
3. `deck.add_basic(...)`
4. `deck.basic()` lane with builder `.note(...).tags(...).stable_id(...).add()?`
5. `deck.cloze()` lane with `.extra(...)`
6. `deck.image_occlusion()` lane with:
   - `mode(...)`
   - `header(...)`
   - `back_extra(...)`
   - `comments(...)`
   - `rect(...)` as the intentionally narrowed first-pass geometry helper
7. `MediaSource`, `MediaRef`, `deck.media().add(...)`, and `deck.media().get(...)`
8. `validate()`
9. `validate_report()`
10. `to_apkg_bytes()`
11. `write_to(...)`
12. `write_apkg(...)`
13. `build(...)`
14. `BuildResult`
15. `Package::single(...)`
16. `into_product_document()` and `from_product_document()`

The following remain advanced-path capabilities in the first pass:

1. helper declarations
2. inline asset bundling beyond the default media API
3. font binding
4. field metadata
5. browser appearance overrides
6. template target deck overrides
7. custom notetype authoring
8. collection-wide `.colpkg` export

## 9. Documentation Shape

The main crate documentation should teach this order:

1. `Deck::new(...)`
2. `deck.basic()/cloze()/image_occlusion()`
3. `deck.write_apkg(...)`
4. optional preflight `deck.validate()` / `deck.validate_report()`
5. optional `deck.build(...)`

Advanced documentation should then cover:

1. `Package`
2. `anki_forge::product`
3. `anki_forge::runtime`
4. contract tooling and release verification

## 10. Summary

The default Rust API is centered on:

1. `anki_forge::Deck`
2. stable identity as a baseline concept
3. explicit `Basic`, `Cloze`, and `Image Occlusion` authoring lanes
4. explicit media registration through `MediaSource` and `MediaRef`
5. optional explicit preflight validation with structured diagnostics
6. bytes-first and writer-based export paths
7. `Package` as the single-root deck package abstraction for `.apkg`

The advanced layers remain available, but the main tutorial and crate root
should feel like a direct authoring tool instead of a pipeline toolkit.
