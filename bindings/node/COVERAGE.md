# Node public API verification

`test/public-api.test.mjs` exercises the sole Project authoring path against a rebuilt native library. `native/examples/sdk_parity.rs` independently produces a matching project through Rust's default API and inspects actual APKG collections/media with ZIP, zstd and SQLite; it does not use internal-tools.

Coverage includes explicit identity and text/HTML parity, immutable model reuse and atomic conflicts, media file snapshots and >64 KiB bytes, budgets/MIME/naming errors, bundle-v2 ownership, both IO modes and stable mask validation, compare/update policy and identity preservation, structured error causes and source metadata, error template locations and budgets, observations retained after baseline success/candidate failure, successful warning outcomes, lossless u64 budgets, artifact clone/persist/close, concurrent build snapshots, and worker teardown. `scripts/installed-smoke.mjs` tests installed ESM/CJS packages, README examples, TypeScript .mts/.cts consumers, and read-only node_modules.

These tests do not substitute for the repository's real Anki import oracle, four-platform native rebuilds, or release validation. `npm run test:parity` runs the same public contract suite, which always includes Rust parity.
