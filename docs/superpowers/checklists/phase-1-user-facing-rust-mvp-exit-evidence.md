# Phase 1 User-Facing Rust MVP Exit Evidence

Recorded against the current worktree on 2026-05-16.

## Required Evidence

- [x] `cargo test -p anki_forge --test project_api_tests -v` passes.
  - PASS: 9 tests passed.
- [x] `cargo test -p anki_forge --test deck_project_facade_tests -v` passes.
  - PASS: 3 tests passed.
- [x] `cargo test -p anki_forge --test custom_merge_id_snapshot_tests -v` passes.
  - PASS: 8 tests passed.
- [x] `cargo test -p anki_forge --test project_media_api_tests -v` passes.
  - PASS: 5 tests passed.
- [x] `cargo run -q -p anki_forge --example target_api_basic` writes `spanish.apkg`.
  - PASS: run from `tmp/task8-examples`; produced `tmp/task8-examples/spanish.apkg`.
- [x] `cargo run -q -p anki_forge --example target_api_custom_notetype` writes `jp-core.apkg`.
  - PASS: run from `tmp/task8-examples`; produced `tmp/task8-examples/jp-core.apkg`.
- [x] `cargo run -q -p anki_forge --example target_api_media` writes `spanish-media.apkg`.
  - PASS: run from `tmp/task8-examples`; produced `tmp/task8-examples/spanish-media.apkg`.
- [x] README first screen teaches `Deck`; second screen teaches `Project`.
  - PASS: `README.md` sections 2 and 3 teach `Deck` first and `Project` second.
- [x] `bindings/python/examples/target_api_custom.py` documents Python Product API shape.
  - PASS: file documents the target future Product API shape and states it is not currently runnable.
- [x] Existing manual scenarios `S01_basic_text_minimal`, `S02_cloze_minimal`, `S04_basic_image`, and `S05_basic_audio` are referenced as Phase 1 oracle evidence.
  - PASS: oracle references are listed below.

## Full Verification

- [x] `cargo fmt --all -- --check`
  - PASS.
- [x] `cargo test -p anki_forge --test public_api_boundary_tests -v`
  - PASS: red before `authoring` namespace existed, then 3 tests passed after implementation.
- [x] `cargo test -p anki_forge -v`
  - PASS: all package tests and doc-tests passed.
- [x] `cargo test -p authoring_core -v`
  - PASS: all package tests and doc-tests passed.
- [x] `cargo test -p writer_core -v`
  - PASS: all package tests and doc-tests passed.

## Oracle References

- Basic: `docs/manual-validation/anki-desktop-v1/S01_basic_text_minimal.md`
- Cloze: `docs/manual-validation/anki-desktop-v1/S02_cloze_minimal.md`
- Image media: `docs/manual-validation/anki-desktop-v1/S04_basic_image.md`
- Audio media: `docs/manual-validation/anki-desktop-v1/S05_basic_audio.md`
- Field/template merge id snapshot: `anki_forge/tests/custom_merge_id_snapshot_tests.rs`
