# Rust User API Capability Testing Design

- Date: 2026-06-03
- Status: Approved design draft
- Scope: Rust `Deck` and `Project` public API capability tests that produce real Anki `.apkg` packages

## Confirmed Decisions

1. This phase covers only the Rust user-facing API: `Deck`, `Project`, and their public package-writing surfaces.
2. Tests are based on user API behavior and observable results, not internal lowering, IR, or reconciliation functions.
3. Automatic package validation writes real `.apkg` files and inspects them with the existing APKG inspection oracle.
4. Manual Anki Desktop coverage remains a generated-package and checklist flow. The test chain does not automate Anki Desktop import.
5. The first capability matrix covers success paths, diagnostic and warning paths, and update-safety paths.
6. Automation is layered: a small smoke set runs in ordinary `cargo test`; the full capability matrix runs through `make verify-ci` or a dedicated script; manual Desktop scenarios remain outside ordinary CI.

## Goal

The test chain should prove that a Rust user can build useful Anki packages through the public API and receive stable, understandable results when something goes wrong.

Unit tests continue to validate small rules and implementation details. Capability tests validate end-to-end user outcomes:

- A user calls `Deck` or `Project` APIs.
- The library produces a real `.apkg` package or a structured report explaining why it cannot.
- The package can be inspected as an APKG and contains the expected notes, cards, note types, templates, decks, media, references, and identity metadata.
- Update-safety reports distinguish safe updates, risks, and blocking conflicts.

## Non-Goals

- Python, Node, or low-level CLI user capability coverage in this phase.
- Automated Anki Desktop UI import or GUI testing.
- A required upstream Anki `rslib` roundtrip oracle. That remains optional and separate because it needs local `docs/source/anki` and `protoc`.
- Replacing existing unit, contract, writer, or binding tests.
- Testing internal IR shape as the main assertion target for this suite.
- Keeping generated `.apkg` files in the repository.

## Recommended Approach

Use a three-layer test chain:

1. `user-api-smoke`: ordinary tests that run with `cargo test --workspace`.
2. `rust-user-capability-matrix`: a full Rust user API matrix invoked by `make verify-ci` and by a dedicated local script.
3. `manual-desktop-rust-api-v1`: generated APKGs, inspect reports, and human checklist files for Anki Desktop import validation.

The layers share scenario definitions where practical, but they have different speed and evidence goals.

## Alternatives Considered

### Put The Full Matrix In Ordinary Cargo Tests

This gives the simplest command surface, but it makes everyday test runs slower and mixes broad package acceptance checks with narrow unit failures. It also makes it harder to add media-heavy and update-safety scenarios without punishing every local edit.

### Script-Only Acceptance Suite

A script can manage artifacts and reports cleanly, but script-only tests are easier to skip accidentally and do not integrate as well with Rust test failure names. This is useful for manual Desktop generation, not as the only automated capability layer.

### Layered Matrix

The layered approach keeps the public API smoke signal close to ordinary development while reserving the heavier matrix for PR and release confidence. This matches the repository's existing split between `make verify-fast`, `make verify-ci`, manual Desktop scripts, and optional roundtrip oracle scripts.

## Architecture

The capability tests use public Rust API calls as the entry point:

```text
Rust user API
  -> Deck.write_apkg / Project.write_apkg / build / to_apkg_bytes
  -> BuildReport or BuildError.report
  -> generated .apkg
  -> inspect_apkg observation
  -> user-visible assertions
```

The inspection step is an APKG observation oracle. It is allowed to inspect the generated package, but tests should not assert on private lowering decisions unless the user-visible package or report requires them.

The full matrix should live in a dedicated integration-test target or runner that ordinary `cargo test --workspace` compiles but does not fully execute. The preferred shape is:

- Smoke tests: normal `#[test]` cases in the existing Rust test suite.
- Full matrix: ignored integration tests or a dedicated Rust runner invoked by `scripts/run_rust_user_capabilities.sh`.
- CI: `scripts/verify-ci.sh` calls the full matrix script after ordinary workspace tests.

If ignored integration tests are used, the script should call the exact target explicitly, for example:

```bash
cargo test -p anki_forge --test rust_user_capability_matrix -- --ignored --nocapture
```

The ordinary workspace test run still compiles the full matrix test target, which catches API signature breakage without executing every heavy scenario.

## Capability Matrix

Each scenario records:

- Scenario id and name.
- Public API entry point.
- User intent.
- Expected package artifact.
- Expected `BuildReport` or `BuildError.report` behavior.
- Expected APKG inspection observations.
- Whether manual Desktop validation is useful.

### Success Scenarios

| Scenario | User intent | Public API | Core assertions |
| --- | --- | --- | --- |
| `deck_basic_apkg` | Build a minimal Basic deck | `Deck::new`, `deck.basic()`, `write_apkg` | Success report, one note, one card, stock Basic note type |
| `deck_cloze_apkg` | Build a Cloze deck with cloze markers | `deck.cloze()`, `write_apkg` | Success report, cloze note type, expected generated card |
| `deck_image_occlusion_apkg` | Build image occlusion notes from media | `deck.image_occlusion()`, media API, `write_apkg` | Success report, image media packaged, expected IO card count |
| `deck_bytes_export` | Produce package bytes without choosing a permanent output path | `to_apkg_bytes` | Non-empty APKG bytes that can be inspected after writing to a temp path |
| `project_stock_notes_apkg` | Build Basic and Cloze notes through `Project` | `Project::new`, `Note::basic`, `Note::cloze`, `write_apkg` | Success report, stock note types, stable counts |
| `project_custom_notetype_apkg` | Build a custom note type with fields/templates/rules | `NoteType::custom`, `Field`, `Template`, `GenerationRule` | Success report, custom fields/templates visible in APKG |
| `project_media_references_apkg` | Use media in fields, templates, and CSS | `Project.media_mut`, `Note.sound`, `Note.image`, template/CSS references | Media bindings packaged, references resolved, report media summary matches |

### Diagnostic And Warning Scenarios

| Scenario | User-visible behavior | Core assertions |
| --- | --- | --- |
| `duplicate_stable_id` | User sees a stable duplicate diagnostic | Report or build error includes stable diagnostic code, source, severity |
| `blank_stable_id` | User sees an invalid stable id diagnostic | Diagnostic source points at the user note path |
| `cloze_marker_missing` | Cloze note without marker does not silently build a useless package | Report status invalid or error, cloze diagnostic code present |
| `missing_media_source` | Missing source file produces structured media diagnostics | Diagnostic code identifies missing media source, `ensure_success()` fails |
| `missing_media_reference` | Referenced media not registered is visible to the user | Diagnostic code identifies unresolved media reference |
| `unused_media_binding` | Extra registered media is reported as a warning | Build can succeed or warn according to policy; warning code is present |
| `unsafe_media_filename` | Unsafe export name is rejected or diagnosed | Diagnostic identifies unsafe media filename |
| `mime_mismatch` | Declared/exported MIME mismatch is visible | Diagnostic code and message identify mismatch |

### Update-Safety Scenarios

| Scenario | User intent | Core assertions |
| --- | --- | --- |
| `update_preserves_guid` | Rebuild the same stable notes with `compare_to` | Report indicates GUID preservation from previous APKG |
| `update_adds_new_note` | Add one new note while preserving existing notes | Existing note identity stays stable; counts increase by one |
| `field_rename_stable_key_safe` | Rename display field while keeping stable key/config identity | Report does not block update; APKG remains inspectable |
| `field_config_id_drift_blocks` | Detect unsafe field identity drift before import | Report status blocks or fails with update-safety diagnostic |
| `template_reorder_risk` | Surface scheduling/card risk when templates reorder | Report contains risk signal without conflating it with package corruption |
| `custom_merge_id_change` | Detect changed field/template merge ids | Merge-safety diagnostic appears with source and risk severity |

## Assertion Rules

Capability tests should assert on user-visible outcomes:

- Package exists and is non-empty.
- `BuildReport.status`, `counts`, `artifact`, `diagnostics`, `media`, `policy`, and `baseline` fields match the user scenario.
- `report.ensure_success()` passes for clean success and fails for invalid, blocked, or error reports.
- APKG inspection sees the expected note types, fields, templates, card counts, media filenames, media references, deck names, and note identity metadata.
- Diagnostics use stable codes and useful source paths.

Capability tests should avoid:

- Asserting normalized IR field order unless the APKG or report exposes the behavior.
- Calling private reconciliation/lowering helpers as the main test action.
- Depending on test order or artifacts left by previous tests.

## Data And Artifact Flow

Each automated scenario creates its own temporary directory. Baseline and update packages are created inside that directory when needed.

The test helper layer should provide only test support:

- Create a scenario temp directory.
- Write a package through the public API.
- Inspect the generated package.
- Load or compute package SHA-256 when a checklist needs it.
- Format failure messages around the scenario id and user intent.

Generated artifacts are not committed. Manual Desktop runs write under a temp or `tmp/` output tree, matching the existing manual scenario style.

## Test Entrances

### Ordinary Cargo Tests

Ordinary tests keep a small smoke set:

- One `Deck.write_apkg` Basic scenario.
- One `Project.write_apkg` stock or custom scenario.
- Optionally one `to_apkg_bytes` smoke if it is not already covered by existing tests.

These run during `cargo test --workspace` and catch severe API/package regressions quickly.

### Full Rust Capability Matrix

Add a dedicated local entry point such as:

```bash
./scripts/run_rust_user_capabilities.sh
./scripts/run_rust_user_capabilities.sh deck_basic_apkg update_preserves_guid
```

The script should support:

- Running all scenarios.
- Running named scenarios.
- Printing where artifacts and inspect reports were written.
- Exiting non-zero on failed assertions.

`scripts/verify-ci.sh --ci` should call this script. `--fast` may skip the full matrix unless later performance proves it is cheap enough.

### Manual Desktop Packages

The manual layer should produce:

- `tmp/manual-desktop-rust-api-v1/<scenario>/package.apkg`
- `tmp/manual-desktop-rust-api-v1/<scenario>/apkg.inspect.json`
- `tmp/manual-desktop-rust-api-v1/<scenario>/manual-checklist.md`

The checklist records:

- Date.
- Platform.
- Anki version.
- anki-forge commit.
- Package SHA-256.
- Import before/after note and card counts.
- GUID/update behavior when relevant.
- Duplicate note outcome when relevant.
- Media rendering notes.
- Relevant diagnostics.

This layer can coexist with the existing Authoring IR manual Desktop scenarios. The Rust API manual scenarios should be named clearly so users know which public API behavior they validate.

## Error Handling

The suite distinguishes clean success, warnings, invalid input, blocked updates, and infrastructure failures.

- Clean success: `ensure_success()` passes, APKG exists, APKG inspection is complete.
- Warning: the report exposes warning diagnostics and policy state; the test documents whether `ensure_success()` should pass under the selected policy.
- Invalid input: the user receives a parseable `BuildReport` or `BuildError.report` with stable diagnostics.
- Blocked update: the update-safety report identifies why importing the package would be unsafe.
- Infrastructure failure: missing temp paths, inability to write artifacts, or broken inspection fails the test as a harness problem, not as a user diagnostic expectation.

Optional dependencies such as upstream Anki source, `protoc`, or Anki Desktop are not required for this phase.

## Success Criteria

The design is implemented when:

1. Ordinary workspace tests include a small Rust user API package-writing smoke set.
2. A dedicated full matrix entry point exercises success, diagnostics, warnings, and update safety through Rust `Deck` and `Project` APIs.
3. Full matrix scenarios write real `.apkg` files and validate them through APKG inspection.
4. `make verify-ci` runs the full matrix.
5. Manual Desktop generation emits APKGs, inspect reports, and checklists without requiring Anki Desktop automation.
6. Test names, scenario ids, and assertions describe user behavior and results rather than internal implementation phases.

## Open Implementation Notes

The implementation plan should decide whether the full matrix uses ignored integration tests or a dedicated Rust runner. The design preference is ignored integration tests when they give clear per-scenario failure names and can still be invoked cleanly from a script.

Existing tests already cover pieces of this behavior. The implementation should reuse or relocate them only when that improves the user-facing capability story. It should not remove lower-level tests that still protect contracts or internals.
