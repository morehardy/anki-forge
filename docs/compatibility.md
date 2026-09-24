# Compatibility and release status

These guides describe this source checkout and its new public API. Source
versions, supported platforms, completed verification and registry availability
are separate facts; this page does not announce a release.

## Language status

| Interface | Source version | Intended environment | Evidence |
| --- | --- | --- | --- |
| Rust | 0.1.0 | Rust 1.92+ | [Release audit](rust-crate-release-readiness.md) and current packaged-consumer gates |
| Node / TypeScript | 0.2.0 | Node 22.13+; macOS arm64/x64, Linux x64 GNU, Windows x64 | [Node coverage](../bindings/node/COVERAGE.md) |
| Python | 0.2.0 | Ordinary CPython 3.11/3.12; macOS arm64/x64, Linux x64, Windows x64 | [Python coverage](../bindings/python/COVERAGE.md) |

Do not extend a local verification result to every platform in the intended
matrix. Other Python versions, free-threaded interpreters and subinterpreters
need separate validation. The Node matrix does not claim browser, Electron, Bun,
Alpine/musl or ARM64 Linux/Windows support.

Start with [Rust](installation.md), [Node](node/quick-start.md), or
[Python](python/quick-start.md). The API is pre-1.0 and this redesign is
intentionally breaking; regenerate integrations using the current authoring
model instead of mixing old facades with new native binaries.

## Shared semantics

- `Project` is the sole authoring container. A namespace and every note key are
  explicit. Models are immutable values carried by their notes.
- Ordinary strings are Text everywhere; HTML requires explicit Content.
- Media imports own snapshots and collect typed dependencies automatically.
  Raw HTML/CSS/script assets need explicit declarations.
- Image Occlusion supports both hide-all-guess-one and hide-one-guess-one, keyed
  masks, complete supported-image decoding and stable update ordinals.
- Bundle loading accepts `template-bundle-v2` and returns a model with owned
  assets; templates reference field keys.
- Build success includes a guaranteed owned artifact. Reports and JSON data do
  not own temporary files.
- Updates require complete evidence in the previous original distribution APKG.
  Risk policy cannot bypass hard evidence, schema or resource errors.

## Packages and real Anki clients

Public API tests inspect actual packages, identities, cards and media. Those
checks do not by themselves validate every Anki rendering engine, codec or
import setting. Field/template changes, sort-field changes, omissions and
learner-local edits need client-specific import checks. Stable identities do not
promise deletion synchronization or automatic recovery of review history.

Use the repository's [manual client scenarios](manual-validation/anki-desktop-v1/TEMPLATE.md)
and each recorded oracle's exact Anki version and settings when evaluating a
release. [Build publication](build-guarantees.md) is atomic per APKG file; a
separate report write is a separate operation.
