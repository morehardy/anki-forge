# Public API overview

Anki Forge is an open-source Anki APKG generation library with a Rust core and
Rust, Python and Node.js interfaces. Use the public `ankiforge` package to author
notes, package media and inspect updates before distributing a new deck. MIT licensed.

| Task | Rust | Python | Node / TypeScript |
| --- | --- | --- | --- |
| Author notes and models | [Rust API](rust-api.md) | [Python API](python/api.md) | [Node API](node/api.md) |
| Images, audio and occlusion | [Media](media.md) | [Owned media](python/api.md#media-and-occlusion) | [Owned media](node/api.md#media-and-image-occlusion) |
| Diagnose failures | [Troubleshooting](troubleshooting.md) | [Diagnostics](python/diagnostics.md) | [Errors](node/api.md#errors) |
| Compare and publish | [Update workflow](updates.md) | [Build and compare](python/api.md) | [Build and compare](node/api.md#build-compare-and-output) |

## Stable note identity

Keep a stable project namespace and note key for each logical note. Updating an
existing release also requires its original distribution APKG. An Anki re-export
is not a substitute for the complete identity evidence in that original package.

## Start with a task

[Choose your language](https://ankiforge.dev/docs/languages/) or follow the complete
[JSON generation and update recipe](json-to-anki.md). See [compatibility](compatibility.md)
for published runtimes and [output guarantees](build-guarantees.md) for file ownership.
