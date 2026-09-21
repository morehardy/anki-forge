# anki-forge

Build Anki flashcards in code and export them as `.apkg` files. anki-forge is a
Rust-first library for generating decks from your own data, with Node.js and
Python APIs for integrating deck generation into other applications.

- **Author cards:** Basic, Cloze, and custom note types with HTML/CSS templates.
- **Package media:** images, audio, and video alongside your notes.
- **Maintain decks:** stable identities, baseline comparisons, and identity
  lockfiles for rebuilding distributed decks.
- **Check exports:** validation, structured diagnostics, and build reports with
  note, card, and media counts.

[Quick start](#quick-start) · [Rust guide](docs/rust-guide.md) ·
[Node SDK](bindings/node/README.md) · [Python guide](bindings/python/README.md)

## Quick start

Use Rust **1.92.0** for this checkout. Run the Basic example from source:

```sh
git clone https://github.com/morehardy/anki-forge.git
cd anki-forge
cargo run -q -p anki_forge --example target_api_basic
```

This creates `spanish.apkg` in the current directory. Import it into Anki Desktop
to get a Spanish deck containing one card: **hola → hello**.

The [complete example](anki_forge/examples/target_api_basic.rs) is:

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

To use this checkout in your own Rust application, add a path dependency and
the `anyhow` dependency used by the example. Adjust the path to your checkout:

```sh
cargo add anki_forge --path ../anki-forge/anki_forge
cargo add anyhow
```

The Rust library embeds its default contract resources; normal deck generation
needs no separate contract files or Anki installation.

## Choose an API

| Interface | Use it for | Requirements and entry point |
| --- | --- | --- |
| Rust `Deck` | A short path from notes to one APKG | Rust 1.92+; [quick start](#quick-start) |
| Rust `Project` | Custom note types, media, validation, and repeated updates | Rust 1.92+; [authoring guide](docs/rust-guide.md) |
| Node.js / TypeScript | Native Rust `Deck` and `Project` APIs in Node applications | Node 22.13+; [SDK setup and status](bindings/node/README.md) |
| Python | `Project`, `Note`, custom note types, and media through the Rust runtime | Python 3.11+; [source setup](bindings/python/README.md#from-a-source-checkout) |

**Release status:** this checkout declares Rust `0.1.0`, Node `0.2.0`, and
Python `0.1.0`. The [Rust release audit](docs/rust-crate-release-readiness.md)
records outstanding publication gates, and the Node SDK documents a local
candidate with npm publication and platform verification pending. The setup
instructions here use source builds. For a published Rust release, the registry
dependency is `anki_forge`; follow the release documentation before relying on
registry availability.

## Build a project you can maintain

Use `Project` when you need explicit project identity, custom note types, or
validation before export:

```rust
use anki_forge::prelude::*;

fn main() -> anyhow::Result<()> {
    let mut project = Project::new("Japanese Core")
        .stable_id("jp-core")
        .default_deck("Japanese::Core");
    project.add_note(Note::basic("食べる", "to eat").stable_id("jp:taberu"))?;
    project.validate().ensure_success()?;
    project.write_apkg("jp-core.apkg")?.ensure_success()?;
    Ok(())
}
```

Stable IDs preserve note identity. For updates to an already distributed deck,
also use a previous APKG with `compare_to(...)` or a maintained identity lockfile
with `update_safe(...)`. A plain `write_apkg()` export does not guarantee that
Anki will apply later content edits. See
[updating distributed decks](docs/rust-guide.md#updating-distributed-decks)
for the first-build and subsequent-build workflow.

| Next step | Guide or runnable example |
| --- | --- |
| Custom fields and card templates | [Custom note type example](anki_forge/examples/target_api_custom_notetype.rs) |
| Images, audio, and template media | [Media example](anki_forge/examples/target_api_media.rs) · [troubleshooting](docs/rust-guide.md#media-troubleshooting) |
| Reusable templates, CSS, and assets | [Template bundles](docs/template-bundles.md) |
| Build reports, temporary files, and persistence | [Artifact ownership](anki_forge/README.md#artifact-ownership) |
| Moving from genanki to the Python API | [Migration guide](docs/python/genanki-migration.md) |

## Compatibility and limitations

- The Rust API is pre-1.0. The supported interface is available through
  `anki_forge::prelude`; the `internal-tools` feature is reserved for repository
  tooling. See the [crate guide](anki_forge/README.md#supported-01-interface).
- Image Occlusion supports the `hide-all-guess-one` export path. The
  `hide-one-guess-one` renderer has a known core limitation that rejects its
  grouped cloze markup; see the [recorded limitation](bindings/node/README.md#deck-and-image-occlusion).
- Basic text is escaped as text. Cloze text preserves HTML and raw
  `{{cN::...}}` markers; pass trusted HTML when using that path.
- Anki's import settings and newer local edits affect how deck updates are
  applied. Keep the previous distributed APKG or identity lockfile as update
  evidence.

## Performance evidence

The benchmark suite compares the native Rust `Deck` API with genanki across
text, image, audio, and mixed-media workloads. It records export time, peak
memory, package size, and content/import checks. Node and Python bindings are
outside that comparison.

<details>
<summary>Current comparison: 100–1,000 notes, measured on 2026-09-21</summary>

In one complete session on an Apple M1 Pro, 1,000 text-only notes took
53.9 ms versus 115.5 ms (53.3% less time); unique images took
238.6 ms versus 365.5 ms (34.7% less time). The image workload used
40.25 MiB versus 35.77 MiB peak RSS. Rust and genanki were both measured
again, with 10 timings and 5 separate RSS samples per implementation/cell.
All 840 exports and 40 Anki import/content/render checks passed.

![Current export timings across five workloads and four note counts](benchmarks/results/20260921-readme-genanki/time-heatmap.svg)

These results describe the current uncommitted source snapshot on this machine,
with uncontrolled desktop background load and filesystem cache. Default APKG
formats differ between the libraries. See the [full report and raw evidence](benchmarks/results/20260921-readme-genanki/report.md)
for versions, source hashes, sample spread, memory results, and reproduction.

</details>

For reproduction and separate optimization reports, use the
[benchmark guide](benchmarks/README.md).

## Contributing

See the [development guide](docs/development.md) for prerequisites, verification
commands, contract tools, and manual Anki checks. It also links the architecture
decisions and release procedures.

Report bugs and request features through
[GitHub issues](https://github.com/morehardy/anki-forge/issues). For security
reports, follow the [security policy](SECURITY.md).

## License

Project-owned code is licensed under [MIT](LICENSE). Mirrored and other
third-party source retains its own license.
