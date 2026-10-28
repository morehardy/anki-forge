# Compatibility and releases

Canonical page: https://morehardy.github.io/anki-forge/docs/compatibility/
Language: en
Description: Published packages, verified environments and Anki client conditions.

## Packages and verified environments

Public Rust, npm and PyPI packages are `0.3.1` as checked on 2026-10-13. Start with [Rust](https://morehardy.github.io/anki-forge/docs/installation/), [Node / TypeScript](https://morehardy.github.io/anki-forge/docs/node-quickstart/) or [Python](https://morehardy.github.io/anki-forge/docs/python-quickstart/).

| Interface | Package | Runtime and verification |
| --- | --- | --- |
| Rust | [ankiforge 0.3.1](https://crates.io/crates/ankiforge) | Rust 1.92.0 and current stable; packaged consumers on all four supported platforms, plus a fresh public crates.io consumer |
| Node / TypeScript | [ankiforge 0.3.1](https://www.npmjs.com/package/ankiforge) | Node 22.13.0, current 22, 24 and 26; 16 public npm installation, ESM/CJS and TypeScript checks |
| Python | [ankiforge 0.3.1](https://pypi.org/project/ankiforge/) | Ordinary CPython 3.11–3.14; 16 isolated public PyPI wheel installation checks |

Published native packages cover macOS arm64/x64, Linux x64 GNU/glibc and Windows x64. Package availability alone is not a successful runtime test on every host. The [Node coverage](https://github.com/morehardy/anki-forge/blob/main/bindings/node/COVERAGE.md), [Python coverage](https://github.com/morehardy/anki-forge/blob/main/bindings/python/COVERAGE.md) and release evidence record the exact checks. The successful `0.3.1` release runs are [Rust](https://github.com/morehardy/anki-forge/actions/runs/38371221696), [npm](https://github.com/morehardy/anki-forge/actions/runs/38371222114) and [PyPI](https://github.com/morehardy/anki-forge/actions/runs/38371222047). Other Python implementations, free-threaded interpreters and subinterpreters need separate validation. Browser, Electron, Bun, Alpine/musl and ARM64 Linux/Windows are outside the recorded Node verification scope.

The [batch media](https://morehardy.github.io/anki-forge/docs/media/#batch-import-from-source) and [prepared publication](https://morehardy.github.io/anki-forge/docs/updates/#build-once-review-then-publish-from-source) guides are available in the public `0.3.1` packages.

## Anki import and playback

Normal export needs no Anki installation. Importing, rendering and playing media still depends on the target client’s version, codecs and import settings. The 2026-10-08 benchmark passed 40 Anki import/content/representative-render checks (20 per implementation), not every client or GUI/audio behavior.

For updates, retain the previous original distribution APKG. Anki re-exports do not retain the required identity evidence. Omitting a note from a package does not delete it from a learner’s collection. Field/template changes, sort fields, local edits and review history require client-specific testing.

Use the [manual client scenarios](https://github.com/morehardy/anki-forge/blob/main/docs/manual-validation/anki-desktop-v1/TEMPLATE.md) and each oracle’s exact Anki version and settings to evaluate your own distribution. Read [updates](https://morehardy.github.io/anki-forge/docs/updates/) and [build guarantees](https://morehardy.github.io/anki-forge/docs/build-guarantees/) for client conditions and per-file publication behavior.
