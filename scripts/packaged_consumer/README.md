# Independent consumer inputs

`check_rust_crate_package.sh` copies this directory into a temporary directory outside the repository and points its generated Cargo manifest only at Cargo's unpacked crate. Runtime fixtures live in that copied directory. The PNG, WAV and WOFF inputs are original repository fixtures (MIT), copied from `anki_forge/tests/fixtures/public-api`; their generation recipes are documented there. The template bundle is the current custom-cloze example.

The consumer uses the default public API, with default features disabled. It observes real ZIP/zstd/protobuf/SQLite contents independently of AnkiForge's inspection implementation. Tests cover content escaping, custom key compilation, ownership after source deletion, media/name conflicts, large bytes, explicit assets, bundle closure, both IO modes, strict comparison/update policy, stable GUIDs, snapshots, budgets and artifact lifetime. Snapshot checks include lossless Unix byte paths and Windows unpaired UTF-16 units on their respective CI platforms.

The website executor reuses only the fixtures and independent observation helper. It copies and runs every complete public Rust documentation program in its own working directory; it does not rewrite example bodies or bypass missing assets.
