# anki-forge

`anki_forge` provides a minimal Rust API for creating Anki decks and exporting `.apkg` files.

If you just want to build decks, add notes/media, and export packages, use the `Deck` API first.
You do not need to touch normalized IR, writer policies, or build artifacts for common workflows.

## Minimal API (Recommended)

### What you can do

- Build a deck with a stable identity
- Add `basic`, `cloze`, and `image occlusion` notes
- Register media from file or bytes
- Validate deck shape and export `.apkg`

### End-to-end example

```rust
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
        .tags(["geography"])
        .add()?;

    let heart = deck.media().add(MediaSource::from_bytes(
        "heart.png",
        vec![0x89, 0x50, 0x4E, 0x47],
    ))?;

    deck.image_occlusion()
        .note(heart)
        .mode(IoMode::HideAllGuessOne)
        .rect(10, 20, 80, 40)
        .header("Heart")
        .back_extra("Identify the chamber")
        .comments("Left ventricle")
        .stable_id("anatomy-heart-1")
        .tags(["anatomy"])
        .add()?;

    deck.validate()?;
    deck.write_apkg("spanish.apkg")?;

    Ok(())
}
```

### Identity guidance

- Prefer `stable_id(...)` on notes and deck/package when you want import-friendly updates.
- `add_basic(front, back)` is available for the shortest path, but it generates a non-stable note id.

## What Happens Under the Hood

The minimal API is a high-level facade over the existing core pipeline:

1. `Deck` model and validation
2. Lower to product/authoring document
3. Normalize (`authoring_core`)
4. Build package (`writer_core`)
5. Return artifact paths or `.apkg` bytes

This gives you a simple authoring surface while preserving the same core execution path used by lower-level APIs.

## Advanced Surfaces

Use these only if you need tighter control than the minimal API:

- `anki_forge::product`: explicit product/notetype-level authoring
- `anki_forge::runtime`: file-driven normalize/build/inspect/diff workflows
- direct re-exports from `authoring_core` and `writer_core`

## Repository Verification (Maintainers)

The repository remains contract-first:

- `contracts/` is the normative source of truth
- `contract_tools/` is internal verification tooling

Run from repository root:

```bash
cargo run -p contract_tools -- verify --manifest "$(pwd)/contracts/manifest.yaml"
cargo run -p contract_tools -- summary --manifest "$(pwd)/contracts/manifest.yaml"
cargo run -p contract_tools -- package --manifest "$(pwd)/contracts/manifest.yaml" --out-dir dist
```

Before phase exits or release-related merges, capture evidence in:

- `docs/superpowers/checklists/phase-1-exit-evidence.md`
- `docs/superpowers/checklists/phase-2-exit-evidence.md`
- `docs/superpowers/checklists/phase-3-exit-evidence.md`
- `docs/superpowers/checklists/phase-5a-exit-evidence.md`
