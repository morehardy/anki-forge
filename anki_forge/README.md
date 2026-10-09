# Anki Forge for Rust

Build Basic, Cloze, Image Occlusion and custom cards, package media, and check updates before distributing a new deck.

## Install

Rust 1.92+. See [verified environments](https://ankiforge.dev/docs/compatibility/).
The public package is [`ankiforge` 0.3.0](https://crates.io/crates/ankiforge).

```sh
cargo new anki-deck
cd anki-deck
cargo add ankiforge@0.3.0
```

## Your first deck

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

Open the persistent `spanish.apkg` in Anki. It contains one Basic card in **Spanish**, with **hola → hello**. Keep `spanish` and `es:hola` stable when editing this publication.

## Continue

- [Complete quickstart](https://ankiforge.dev/docs/quickstart/)
- [Images and audio](https://ankiforge.dev/docs/media/)
- [Custom note types](https://ankiforge.dev/docs/custom-notetypes/) and [template bundles](https://ankiforge.dev/docs/templates/)
- [Image Occlusion](https://ankiforge.dev/docs/image-occlusion/)
- [Compare and update](https://ankiforge.dev/docs/updates/)
- [Rust API](https://ankiforge.dev/docs/rust-api/)
- [Build SDKs from source](https://ankiforge.dev/docs/development/)
