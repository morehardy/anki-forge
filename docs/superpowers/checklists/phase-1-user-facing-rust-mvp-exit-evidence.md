# Phase 1 User-Facing Rust MVP Exit Evidence

Prepared against the current worktree on 2026-05-16. Task 8 records the PASS
evidence for each command.

## Required Evidence

- [ ] `cargo test -p anki_forge --test project_api_tests -v` passes.
- [ ] `cargo test -p anki_forge --test deck_project_facade_tests -v` passes.
- [ ] `cargo test -p anki_forge --test custom_merge_id_snapshot_tests -v` passes.
- [ ] `cargo test -p anki_forge --test project_media_api_tests -v` passes.
- [ ] `cargo run -q -p anki_forge --example target_api_basic` writes `spanish.apkg`.
- [ ] `cargo run -q -p anki_forge --example target_api_custom_notetype` writes `jp-core.apkg`.
- [ ] `cargo run -q -p anki_forge --example target_api_media` writes `spanish-media.apkg`.
- [ ] README first screen teaches `Deck`; second screen teaches `Project`.
- [ ] `bindings/python/examples/target_api_custom.py` documents Python Product API shape.
- [ ] Existing manual scenarios `S01_basic_text_minimal`, `S02_cloze_minimal`, `S04_basic_image`, and `S05_basic_audio` are referenced as Phase 1 oracle evidence.

## Oracle References

- Basic: `docs/manual-validation/anki-desktop-v1/S01_basic_text_minimal.md`
- Cloze: `docs/manual-validation/anki-desktop-v1/S02_cloze_minimal.md`
- Image media: `docs/manual-validation/anki-desktop-v1/S04_basic_image.md`
- Audio media: `docs/manual-validation/anki-desktop-v1/S05_basic_audio.md`
- Field/template merge id snapshot: `anki_forge/tests/custom_merge_id_snapshot_tests.rs`
