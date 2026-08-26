# anki_forge

`anki_forge` is a typed Rust library for building Anki decks. The crate ships
its default contract resources, so normal use does not require a source
checkout, a particular working directory, or a separate runtime installation.

```rust,no_run
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

The crate version is `0.1.0`; it embeds contract bundle `0.3.0`. These are
independent compatibility axes. Rust 1.92.0 is the minimum supported compiler
for the 0.1.x line.

## Errors and concurrency

High-level `Deck` and `Project` operations return structured errors and build
reports; callers should inspect stable diagnostic codes instead of matching
human-readable messages. File-writing operations are synchronous and may leave
diagnostic evidence in a requested report path when a build fails.

Values are ordinary owned Rust values and may be moved between threads when
their fields permit it. A single builder or project is not designed for
concurrent mutation; coordinate shared mutation in the calling application.
The embedded read-only contract runtime is initialized once and can be loaded
concurrently.

See the [repository documentation](https://github.com/morehardy/anki-forge)
for custom note types, media, update-safe builds, compatibility policy, and
release operations.

Licensed under MIT.
