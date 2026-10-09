# Install and run with Rust

Use Rust 1.92 or later and the [public ankiforge 0.3.0 crate](https://crates.io/crates/ankiforge).

```sh
cargo new anki-deck
cd anki-deck
cargo add ankiforge@0.3.0
```

## Export your first deck

Save this as `src/main.rs`:

<!-- source: anki_forge/examples/target_api_basic.rs -->
```rust
use ankiforge::{BuildOptions, Note, Project};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut project = Project::new("spanish")?.default_deck("Spanish");
    project.add("es:hola", Note::basic("hola", "hello"))?;
    let output = project.build(BuildOptions::to("spanish.apkg"))?;
    println!("{}", output.artifact().path().display());
    Ok(())
}
```
<!-- /source -->

```sh
cargo run
```

Open the persistent `spanish.apkg` in Anki and study **hola → hello** in **Spanish**. Exporting does not require Anki to be installed.
The namespace `spanish` and note key `es:hola` stay stable when you edit the card.

## Continue

- [Basic and Cloze cards](cards.md)
- [Images and audio](media.md)
- [Custom note types](custom-notetypes.md) and [template bundles](template-bundles.md)
- [Image Occlusion](image-occlusion.md)
- [Compare and update](updates.md)
- [Rust API](rust-api.md) and [verified environments](compatibility.md)

For repository examples and source dependencies, use [source builds](source-builds.md).
