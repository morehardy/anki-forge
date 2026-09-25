# Native release checks

Build the native addon for macOS arm64/x64, Linux x64 glibc and Windows x64 using `node scripts/build.mjs --release --target <triple>`. The SDK and each optional native package must share a version. Protocol 3 uses only Rust's default public API; previous native binaries are rejected.

Before packaging run `npm run check`, `npm test`, `npm run test:installed`, and `npm run check:release` after all platform binaries are available. Tests cover the new Project/NoteType/Media model and real APKG contents. Verify the repository's full Rust packaged-consumer and Anki import oracle gates separately. `npm run pack:release` creates artifacts without publishing.

This API intentionally removes the old Deck, registry, identity recipe and legacy runtime exports. Do not ship stale dist output: the build clears dist before generating CommonJS, ESM and declarations. Optional packages require no install hook, Rust compiler, Cargo, or CLI in the consumer environment.
