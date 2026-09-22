# Compatibility and release status

These guides describe this source checkout. Package versions, supported APIs,
platform verification and public registry availability are separate facts.
Use the linked release evidence before choosing a published dependency.

## Language status

| Interface | Source version | Environment | Distribution evidence |
| --- | --- | --- | --- |
| Rust | 0.1.0 | Rust 1.92+ | [Release audit](rust-crate-release-readiness.md); use the source setup until publication is confirmed |
| Node / TypeScript | 0.2.0 | Node 22.13+; candidate macOS arm64/x64, Linux x64 GNU, Windows x64 | [Coverage](../bindings/node/COVERAGE.md) records local verification and remaining release gates |
| Python | 0.2.0 | Ordinary CPython 3.11/3.12; macOS arm64/x64, Linux x64, Windows x64 | [Coverage](../bindings/python/COVERAGE.md) records four-platform wheel and source-distribution verification; it is not a PyPI publication notice |

Python's metadata allows Python 3.11+, but the declared verified matrix covers
3.11 and 3.12. Other versions, free-threaded interpreters and subinterpreters are
not claimed. Node's candidate matrix does not cover browser execution, Electron,
Bun, Alpine/musl or ARM64 Linux/Windows.

Start with [Rust](installation.md), [Node](node/quick-start.md), or
[Python](python/quick-start.md). For an existing Python 0.1 integration, read
[the 0.2 migration guide](../bindings/python/MIGRATION.md) before upgrading.

## Shared behavior and limits

- Rust's public consumer API is pre-1.0. Import `anki_forge::prelude::*`;
  `internal-tools` is reserved for repository tooling.
- [Image Occlusion](image-occlusion.md) supports hide-all-guess-one. The shared
  core currently rejects hide-one-guess-one grouped cloze output.
- [Text and HTML](concepts.md#text-and-html) differ between Project content
  setters and Deck conveniences. Choose escaping deliberately.
- Templates are checked for supported Anki semantics. Validation does not prove
  browser HTML/CSS/JavaScript correctness or execute third-party add-on filters.
- [Update baselines](updates.md) provide identity and revision evidence. Import
  settings, newer local edits and review scheduling remain controlled by Anki.
- [Output publication](build-guarantees.md) is atomic per file, not a transaction
  spanning APKG, lockfile and report JSON.

## Package checks and Anki client checks

The runnable guides check exported packages and reports. A package that passes
those checks still needs client validation for the behavior you plan to promise:
rendering, actual audio/video playback, upgrade imports and review state.

The repository keeps [manual validation scenarios](manual-validation/anki-desktop-v1/TEMPLATE.md)
and [recorded showcase verification](assets/readme/README.md). Each record's
Anki version, scenarios and scope apply to that record; they are not a blanket
claim for all clients or future versions.
