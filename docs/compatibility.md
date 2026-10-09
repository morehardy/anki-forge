# Compatibility

## Packages and verified environments

Public Rust, npm and PyPI packages are `0.3.0` as checked on 2026-10-09.
Start with [Rust](installation.md), [Node / TypeScript](node/quick-start.md) or [Python](python/quick-start.md).

| Interface | Package | Runtime and verification |
| --- | --- | --- |
| Rust | [ankiforge 0.3.0](https://crates.io/crates/ankiforge) | Rust 1.92+; packaged consumers and the Rust source checks |
| Node / TypeScript | [ankiforge 0.3.0](https://www.npmjs.com/package/ankiforge) | Node 22.13+; installed ESM/CJS and TypeScript checks, plus the macOS arm64 consumer in this documentation validation |
| Python | [ankiforge 0.3.0](https://pypi.org/project/ankiforge/) | Ordinary CPython 3.11/3.12 in recorded wheel checks; macOS arm64 public consumer in this documentation validation |

Published native packages cover macOS arm64/x64, Linux x64 GNU/glibc and Windows x64.
Package availability alone is not a successful runtime test on every host.
The [Node coverage](../bindings/node/COVERAGE.md), [Python coverage](../bindings/python/COVERAGE.md)
and release evidence record the exact checks. Workflow matrices describe intended tests;
they do not establish that a particular run passed. Other Python versions, free-threaded
interpreters and subinterpreters need separate validation. Browser, Electron, Bun,
Alpine/musl and ARM64 Linux/Windows are outside the recorded Node verification scope.

The [batch media](media.md#batch-import-from-source) and [prepared publication](updates.md#build-once-review-then-publish-from-source)
guides are available in the public `0.3.0` packages.

## Anki import and playback

Normal export needs no Anki installation. Importing, rendering and playing media
still depends on the target client's version, codecs and import settings.
The 2026-10-08 benchmark passed 40 Anki import/content/representative-render checks
(20 per implementation), not every client or GUI/audio behavior.

For updates, retain the previous original distribution APKG. Anki re-exports do
not retain the required identity evidence. Omitting a note from a package does
not delete it from a learner's collection. Field/template changes, sort fields,
local edits and review history require client-specific testing.

Use the [manual client scenarios](manual-validation/anki-desktop-v1/TEMPLATE.md)
and each oracle's exact Anki version and settings to evaluate your own distribution.
Read [updates](updates.md) and [build guarantees](build-guarantees.md) for client
conditions and per-file publication behavior.
