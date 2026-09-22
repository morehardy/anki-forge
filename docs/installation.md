# Add Anki Forge to your application

Create a Rust application that writes `spanish.apkg`. These instructions use the
current source checkout; they do not assume a package has been published.
For other languages, use the [Node quickstart](node/quick-start.md) or
[Python quickstart](python/quick-start.md).

## Requirements

Use Git and Rust 1.92 or later. Anki is only needed to import and study the result;
the library can generate packages without an Anki installation.

## Create your application

Run these commands in the directory where you keep your projects. The checkout
and your application will be siblings. Skip cloning if you already have a checkout,
and adjust the dependency path to its location.

```sh
git clone https://github.com/morehardy/anki-forge.git
cargo new my-decks
cd my-decks
cargo add anki_forge --path ../anki-forge/anki_forge
cargo add anyhow
```

Replace `src/main.rs` with this complete program:

<!-- source: anki_forge/examples/target_api_basic.rs -->
```rust
use anki_forge::prelude::*;

fn main() -> anyhow::Result<()> {
    let mut deck = Deck::new("Spanish");
    deck.basic()
        .note("hola", "hello")
        .stable_id("es:hola")
        .add()?;
    deck.write_apkg("spanish.apkg")?.ensure_success()?;
    Ok(())
}
```
<!-- /source -->

Run it from `my-decks`:

```sh
cargo run
```

The first run compiles dependencies. It writes `my-decks/spanish.apkg`, containing
one note and one card. Open the file with Anki and import it into your collection.
The **Spanish** deck asks **hola** and reveals **hello**.

The crate includes its default resources. You do not need `contract_tools`,
external contract files, or the `internal-tools` Cargo feature.

## Keep your project reproducible

Commit your application source and `Cargo.lock`. A path dependency uses the local
checkout, so record the Anki Forge commit you tested when sharing your application.
Follow the [release status](compatibility.md) before replacing it with a registry dependency.

If compilation fails, check the dependency path and `rustc --version`; see
[installation troubleshooting](troubleshooting.md#installation).

Continue with [core concepts](concepts.md), [Basic and Cloze cards](cards.md),
or [custom note types](custom-notetypes.md). Before distributing an updated deck,
read [updates and baselines](updates.md).
