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

## Supported 0.1 interface

The supported consumer interface is intentionally small: import
`anki_forge::prelude::*`, or use the root `Deck`, `Project`, and `Severity`
exports. `facade_api_version()` and `embedded_contract_version()` expose the
two compatibility axes. Normal consumers do not need contract loading,
normalization IR, writer, inspection, or persistence modules.

The `internal-tools` Cargo feature exists only for this repository's
unpublished contract tool and deep conformance tests. Its hidden modules are
not covered by the 0.1 compatibility promise and must not be enabled by
downstream applications.

## Errors and concurrency

High-level `Deck` and `Project` operations return structured errors and build
reports; callers should inspect stable diagnostic codes instead of matching
human-readable messages. File-writing operations are synchronous and may leave
diagnostic evidence in a requested report path when a build fails.

`compare_to(...)` baselines are read-only: output, report, and writable lockfile
paths must not alias them, including through symlinks or hard links. Comparison
and risk checks use a snapshot captured before building and run before APKG or
lockfile publication. A policy-blocked build preserves existing outputs and
lockfiles and reports diff/risk evidence with no artifact path. Publication is
atomic per file, not transactional across all requested files.
New destinations are rechecked after creation, so a late path-collision error
may leave a valid published APKG but cannot replace it with lockfile/report JSON.
Private candidates follow the artifact workspace's filesystem and are cleaned
up after building. Lockfile publication uses an exclusively reserved temporary
file beside its target, without reusing predictable names that may alias inputs
or outputs.

Values are ordinary owned Rust values and may be moved between threads when
their fields permit it. A single builder or project is not designed for
concurrent mutation; coordinate shared mutation in the calling application.
The embedded read-only contract runtime is initialized once and can be loaded
concurrently.

See the [repository documentation](https://github.com/morehardy/anki-forge)
for custom note types, media, update-safe builds, compatibility policy, and
release operations.

Licensed under MIT.
