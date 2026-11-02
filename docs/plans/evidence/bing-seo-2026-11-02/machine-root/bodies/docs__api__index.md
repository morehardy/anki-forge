# Public API overview

Canonical page: https://ankiforge.dev/docs/api/
Language: en
Description: Explore Anki Forge APIs for Rust, Python and Node.js: create notes, package images and audio, diagnose failures, and compare deck updates before distribution.

Anki Forge is an open-source Anki APKG generation library with a Rust core and Rust, Python and Node.js interfaces. Use the public `ankiforge` package to author notes, package media and inspect updates before distributing a new deck. MIT licensed.

| Task | Rust | Python | Node / TypeScript |
| --- | --- | --- | --- |
| Author notes and models | [Rust API](https://ankiforge.dev/docs/rust-api/) | [Python API](https://ankiforge.dev/docs/python-api/) | [Node API](https://ankiforge.dev/docs/node-api/) |
| Images, audio and occlusion | [Media](https://ankiforge.dev/docs/media/) | [Owned media](https://ankiforge.dev/docs/python-api/#media-and-occlusion) | [Owned media](https://ankiforge.dev/docs/node-api/#media-and-image-occlusion) |
| Diagnose failures | [Troubleshooting](https://ankiforge.dev/docs/troubleshooting/) | [Diagnostics](https://ankiforge.dev/docs/python-diagnostics/) | [Errors](https://ankiforge.dev/docs/node-api/#errors) |
| Compare and publish | [Update workflow](https://ankiforge.dev/docs/updates/) | [Build and compare](https://ankiforge.dev/docs/python-api/) | [Build and compare](https://ankiforge.dev/docs/node-api/#build-compare-and-output) |

## Stable note identity

Keep a stable project namespace and note key for each logical note. Updating an existing release also requires its original distribution APKG. An Anki re-export is not a substitute for the complete identity evidence in that original package.

## Start with a task

[Choose your language](https://ankiforge.dev/docs/languages/) or follow the complete [JSON generation and update recipe](https://ankiforge.dev/docs/json-to-anki/). See [compatibility](https://ankiforge.dev/docs/compatibility/) for published runtimes and [output guarantees](https://ankiforge.dev/docs/build-guarantees/) for file ownership.
