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

Smoke tests share only scenario ids and user-intent names with the full matrix. The full matrix and manual Desktop generation share the same scenario builders inside the ignored integration-test target; the manual script invokes those builders in export mode instead of reimplementing public API calls separately.

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

The inspection oracle is the existing `anki_forge::writer::inspect_apkg(path)` re-export, which returns the existing `InspectReport` shape from the writer layer. Matrix helpers should derive assertions from that report's observed note types, fields, templates, cards, media filenames, media references, deck names, and note identity metadata. They should not parse zip entries directly unless a future scenario needs a package property that `InspectReport` cannot expose; in that case, add a focused inspection projection before adding ad hoc zip assertions.

`inspect_apkg(path)` returns `anyhow::Result<InspectReport>`. In success scenarios, an `Err` from `inspect_apkg` is a harness/package-generation failure, and an `Ok(report)` with degraded observation status or missing required observed domains is a package observation failure. Success scenarios should assert that inspection succeeds and that the inspected observations contain the scenario's expected package surface.

Named diagnostic codes in this spec are existing stable registry codes under `contracts/errors/error-registry.yaml` or existing Rust diagnostics covered by the current test suite. The capability matrix should not invent new diagnostic strings. If implementation discovers a missing code, stop and update the registry/diagnostic design before pinning a new capability assertion.

Pre-audited diagnostic codes for this matrix:

| Code | Source |
| --- | --- |
| `AFID.IDENTITY_COMPONENT_EMPTY` | `contracts/errors/error-registry.yaml` |
| `AFID.STABLE_ID_DUPLICATE` | `contracts/errors/error-registry.yaml` |
| `DECK.BLANK_STABLE_ID` | `contracts/errors/error-registry.yaml` |
| `MEDIA.SOURCE_MISSING` | `contracts/errors/error-registry.yaml` |
| `MEDIA.MISSING_REFERENCE` | `contracts/errors/error-registry.yaml` |
| `MEDIA.UNUSED_BINDING` | `contracts/errors/error-registry.yaml` |
| `MEDIA.UNSAFE_FILENAME` | `contracts/errors/error-registry.yaml` |
| `MEDIA.UNSAFE_REFERENCE` | `contracts/errors/error-registry.yaml` |
| `MEDIA.DECLARED_MIME_MISMATCH` | `contracts/errors/error-registry.yaml` |
| `UPDATE.BASELINE_APKG_UNREADABLE` | `contracts/errors/error-registry.yaml` |
| `UPDATE.BASELINE_CONFLICT_GUID` | `contracts/errors/error-registry.yaml` |
| `UPDATE.GUID_PRESERVED_FROM_PREVIOUS` | `contracts/errors/error-registry.yaml` |
| `UPDATE.GUID_DERIVATION_DRIFT` | `contracts/errors/error-registry.yaml` |
| `UPDATE.FIELD_RENAMED` | `contracts/errors/error-registry.yaml` |
| `UPDATE.FIELD_MERGE_ID_CHANGED` | `contracts/errors/error-registry.yaml` |
| `UPDATE.FIELD_ORD_CHANGED` | `contracts/errors/error-registry.yaml` |
| `UPDATE.TEMPLATE_ORD_CHANGED` | `contracts/errors/error-registry.yaml` |
| `UPDATE.TEMPLATE_MERGE_ID_CHANGED` | `contracts/errors/error-registry.yaml` |
| `RISK.TEMPLATE_REORDER` | `anki_forge/src/risk/rules.rs` |

Dependency audit completed on 2026-06-05: every code in this table was found in the named source, `RISK.TEMPLATE_REORDER` was found in the risk rules, `ProjectMediaPolicy::Strict` was found in `anki_forge/src/build/options.rs`, the `update_preserves_guid` APKG rewrite helper shape was found in `anki_forge/tests/update_safety_build_tests.rs`, and the field config drift lockfile helper shape was found in `anki_forge/tests/phase4_product_build_tests.rs`.

The required update-safety and reporting surface already exists in the Rust API. The full matrix may use:

- `BuildOptions::new().output(path)` for explicit package output.
- `BuildOptions::compare_to(previous_apkg)` for previous-package update safety.
- `BuildOptions::fail_on(level)` when a scenario needs a risk threshold.
- `BuildOptions::update_safety(UpdateSafetyMode::...)` only when a scenario intentionally tests disabled, report-only, or strict behavior.
- `BuildReport` for successful, warning, report-only, or inspectable invalid results.
- `BuildError.report` for failed builds that still produce a structured user report. `BuildError` is not a subtype of `BuildReport`; it is the error wrapper that carries the report and failure cause.

The default media policy is `ProjectMediaPolicy::Strict` when no advanced media policy is supplied. The full matrix uses that default unless a scenario explicitly says otherwise. Strict policy is the validation profile; individual diagnostics still keep their configured severity, so an unused binding remains a non-blocking warning while a declared MIME mismatch remains an error.

The default update-safety effective mode is `Strict` when `compare_to(...)` or an identity lockfile is supplied, and `Disabled` when no update-safety evidence is supplied. Update-safety scenarios use either `compare_to(...)` or `identity_lockfile(...)` as named in the row, and therefore use default `Strict` mode unless a row explicitly names another mode.

Test helpers should normalize success and error paths with a small local pattern:

```text
Result<BuildReport, BuildError>
  -> BuildReport from Ok(report)
  -> BuildReport from Err(error).report, plus optional BuildFailureCause assertions
```

The current API exposes `BuildError { report: Box<BuildReport>, cause: BuildFailureCause }`. After normalization, helper assertions read diagnostics, status, counts, artifact, policy, media summary, and `update_safety.baseline_sources` from the same `BuildReport` type.

The report field paths used by this matrix are pre-audited against `anki_forge/src/build/report.rs` on 2026-06-05: `BuildReport.counts`, `BuildReport.media.objects`, `BuildReport.media.bindings`, `BuildReport.media.missing_references`, `BuildReport.media.unsafe_references`, `BuildReport.diagnostics`, `BuildReport.update_safety`, `BuildReport.risk`, and `BuildReport.status` already exist.

The full matrix should live in a dedicated ignored integration-test target that ordinary `cargo test --workspace` compiles but does not fully execute. The target shape is:

- Smoke tests: normal `#[test]` cases in `anki_forge/tests/rust_user_api_smoke_tests.rs`.
- Full matrix: ignored integration tests in `anki_forge/tests/rust_user_capability_matrix.rs`, invoked by `scripts/run_rust_user_capabilities.sh`.
- CI: `scripts/verify-ci.sh` calls the full matrix script after ordinary workspace tests.

The `rust_user_capability_matrix` test target must be compiled unconditionally by ordinary `cargo test --workspace`: no feature gate, no target-specific `cfg` that skips compilation, and no optional dependency that is absent during the normal workspace test command. Heavy scenario bodies are skipped by `#[ignore]`; helper modules still compile every time.

Each scenario id must also be the Rust ignored test function name. The script uses Cargo's test list as the source of truth instead of maintaining a separate scenario list. It discovers scenarios with:

```bash
cargo test -p anki_forge --test rust_user_capability_matrix -- --ignored --list
```

The first matrix defines at least 23 full-matrix scenario ids: 7 success scenarios, 10 diagnostic and warning scenarios, and 6 update-safety scenarios. The `rust_user_capability_matrix` target should reserve ignored root-level tests for capability scenarios, so the script can parse listed test names directly. For all-scenario runs, the script loops over the discovered ordered list and invokes each scenario separately; for named runs, it validates names against the discovered list and invokes only those names. Discovery is a script sanity check: if parsing finds zero scenarios, a name outside `[a-z0-9_]+`, or fewer than 23 scenarios before a spec update reduces the matrix, the script exits with infrastructure code `2` and prints the raw `cargo test --list` output location.

The script should call the exact target explicitly. A single-scenario automated invocation uses:

```bash
cargo test -p anki_forge --test rust_user_capability_matrix "$scenario" -- --ignored --exact --nocapture
```

The ordinary workspace test run still compiles the full matrix test target, which catches API signature breakage without executing every heavy scenario. Manual Desktop export uses the same scenario ids and builders, but the script sets `ANKI_FORGE_CAPABILITY_MODE=manual-desktop`; automated matrix runs set or default to `ANKI_FORGE_CAPABILITY_MODE=automated`.

## Capability Matrix

Each scenario records:

- Scenario id and name.
- Public API entry point.
- User intent.
- Expected package artifact.
- Expected `BuildReport` or `BuildError.report` behavior.
- Expected APKG inspection observations.
- Whether manual Desktop validation is useful.

Scenario ids use `<domain>_<action>_<outcome>` where possible. Examples: `deck_basic_apkg`, `missing_media_source`, `field_config_id_drift_blocks`.

### Success Scenarios

| Scenario | User intent | Public API | Core assertions |
| --- | --- | --- | --- |
| `deck_basic_apkg` | Build a minimal Basic deck | `Deck::new`, `deck.basic()`, `write_apkg` | Success report, one note, one card, stock Basic note type |
| `deck_cloze_apkg` | Build a Cloze deck with cloze markers | `deck.cloze()`, `write_apkg` | Success report, cloze note type, expected generated card |
| `deck_image_occlusion_apkg` | Build one image occlusion note from one image and one rectangular mask | `deck.image_occlusion()`, media API, `write_apkg` | Success report, image media packaged, one generated IO card |
| `deck_bytes_export` | Produce package bytes without choosing a permanent output path | `Deck::new`, `deck.basic()`, `to_apkg_bytes` | Minimal Basic deck yields non-empty APKG bytes that can be inspected after writing to a temp path |
| `project_stock_notes_apkg` | Build Basic and Cloze notes through `Project` | `Project::new`, `Note::basic`, `Note::cloze`, `write_apkg` | Success report, stock note types, stable counts |
| `project_custom_notetype_apkg` | Build a custom note type with fields/templates/rules | `NoteType::custom`, `Field`, `Template`, `GenerationRule` | Success report, custom fields/templates visible in APKG |
| `project_media_references_apkg` | Use one audio file and one image file in fields, template HTML, and CSS | `Project.media_mut`, `Note.sound`, `Note.image`, template/CSS references | Two media bindings packaged, references resolved, media summary has two objects/bindings and zero missing/unsafe references |

Full-matrix success scenarios use this assertion floor:

| Scenario | Minimum full-matrix assertions |
| --- | --- |
| `deck_basic_apkg` | `BuildReport.counts` is exactly one note, one card, zero media; `inspect_apkg` succeeds; inspected package contains the stock Basic note type, `Front` and `Back` fields, one note, and one card |
| `deck_cloze_apkg` | `BuildReport.counts` is exactly one note and one card; `inspect_apkg` succeeds; inspected package contains the stock Cloze note type/template and one card generated from the cloze marker |
| `deck_image_occlusion_apkg` | `BuildReport.counts` is exactly one note, one card, and one media item; `inspect_apkg` succeeds; inspected package contains the image media filename, the stock image-occlusion note type/template, and one generated image-occlusion card |
| `deck_bytes_export` | Minimal Basic deck APKG bytes are non-empty; writing them to a file inside the scenario artifact directory lets `inspect_apkg` succeed; inspected package contains exactly one note and one card |
| `project_stock_notes_apkg` | One Basic note and one Cloze note produce exactly two notes and two cards; inspected package contains both stock note types and expected deck names |
| `project_custom_notetype_apkg` | One custom note produces exactly one note and one card; inspected package contains the custom note type, declared fields, declared template, and expected deck name |
| `project_media_references_apkg` | One custom note produces exactly one note, one card, and two media bindings; `BuildReport.media.objects == 2`, `bindings == 2`, `missing_references == 0`, `unsafe_references == 0`; inspected package contains both media filenames and resolved references from field/template/CSS usage |

Where a row names both `BuildReport.counts` and `inspect_apkg`, assert both. `BuildReport` proves the user-facing report is correct; `inspect_apkg` proves the generated APKG carries the same observable result.

`deck_image_occlusion_apkg` is an API/package plumbing scenario in the first matrix. It uses one deterministic rectangular mask, for example a 100x100 PNG with a single rectangle at `left=0`, `top=0`, `width=50`, `height=50`, and asserts package/card/media outcomes plus the stock image-occlusion note type/template. Mask geometry validation, mask count edge cases, and IO-specific rendering metadata remain covered by lower-level tests until the inspection oracle exposes additional IO-specific public observations.

### Diagnostic And Warning Scenarios

These scenarios pin the expected severity boundary. Shared assertion helpers are fine, but each scenario remains separate because each represents a different user mistake and a different stable diagnostic code.

| Scenario | User-visible behavior | Expected result |
| --- | --- | --- |
| `duplicate_stable_id` | User sees a stable duplicate diagnostic | Blocking: `Project.build(...)` returns `BuildError.report` with `AFID.STABLE_ID_DUPLICATE`, source, and error severity; `ensure_success()` fails |
| `blank_stable_id` | User sees an invalid stable id diagnostic | Blocking at public add/validate boundary: `let mut deck = Deck::new(...); deck.basic().note("front", "back").stable_id(" ").add()` returns `DECK.BLANK_STABLE_ID`; no package is written |
| `cloze_inferred_identity_requires_marker` | Cloze note without marker does not silently build a useless package when relying on inferred identity | Blocking at add time: `Deck::cloze().note("plain text").add()` without explicit stable id returns `AFID.IDENTITY_COMPONENT_EMPTY` because cloze deletions are required identity input; no package is written |
| `missing_media_source` | A registered file source cannot be read | Blocking: `MEDIA.SOURCE_MISSING` identifies the missing source file, `ensure_success()` fails |
| `missing_media_reference` | Note/template/CSS content references an unregistered package filename | Blocking: `MEDIA.MISSING_REFERENCE` identifies the unresolved media reference, `ensure_success()` fails |
| `unused_media_binding` | Extra registered media is reported under strict media policy | Non-blocking warning: strict policy emits `MEDIA.UNUSED_BINDING`; package writes; `ensure_success()` passes |
| `unsafe_media_reference` | Note/template/CSS content uses an unsafe packaged-media reference | Blocking: `Project.build(...)` returns `BuildError.report` with `MEDIA.UNSAFE_REFERENCE`, source, and error severity; `ensure_success()` fails. The first matrix trigger is an HTML field reference such as `<img src="bad%2Fname.png">`, which encodes a path separator and is not a bare packaged-media filename. |
| `unsafe_media_export_filename` | User chooses an unsafe export filename while registering media | Blocking at public media-registration boundary: `Project.media_mut().add_bytes(...).export_as("../chart.png")` or names containing characters outside `[A-Za-z0-9._-]` return an error whose stable code maps to `MEDIA.UNSAFE_FILENAME`; no media binding is added and no package is written. The filename boundary follows the existing public validation in `authoring_core::validate_authoring_media_filename` and Product media `export_as` validation: non-empty bare filenames only, no absolute paths, no parent components, no path separators, and only ASCII alphanumeric, `.`, `_`, and `-` characters. |
| `mime_mismatch` | Declared/exported MIME mismatch is visible | Blocking under strict media policy: register PNG bytes with `Project.media_mut().add_bytes(...).export_as("chart.mp3")`, reference it through `Note::basic(...).sound(...)`, then `Project.build(...)` returns `BuildError.report` with `MEDIA.DECLARED_MIME_MISMATCH`, error severity, and media source; `ensure_success()` fails |
| `baseline_apkg_unreadable` | User passes a missing previous package path to `compare_to` | Blocking: `Project.build(BuildOptions::new().compare_to(missing_previous_path))` returns `BuildError.report` with `UPDATE.BASELINE_APKG_UNREADABLE`, no output package is written, and `ensure_success()` fails. Corrupt files and valid non-APKG files are follow-up variants, not first-matrix rows. |

`missing_media_source` and `missing_media_reference` are intentionally distinct. The first registers a media item whose source file or bytes cannot be loaded. The second leaves content pointing at a package filename that was never registered.

`unused_media_binding` should use the default strict media policy. Advanced media-policy variants are out of scope for the first full matrix, so the warning/error boundary is not configurable inside the scenario. This row intentionally verifies that strict policy does not promote all media diagnostics to errors.

The `cloze_inferred_identity_requires_marker` scenario is intentionally scoped to the current Rust inferred-identity path. It does not claim that `AFID.IDENTITY_COMPONENT_EMPTY` is the general "missing cloze marker" diagnostic. The first matrix does not assert a Rust `PRODUCT.CLOZE_MARKER_MISSING` diagnostic, because that code currently belongs to the Python public API. An explicit-stable-id Rust Cloze note without cloze markers is a separate generated-card diagnostic gap; track it in a follow-up diagnostic-design spec before expanding this matrix to cover it.

The diagnostic matrix intentionally includes both add-time and build-time public API boundaries. `blank_stable_id` and `cloze_inferred_identity_requires_marker` assert the earliest public boundary exposed to a Rust user, because those invalid identity inputs are rejected by `Deck` builders before package construction. The build-time rows assert errors that require project validation, media normalization, package writing, or update-safety baseline loading.

### Update-Safety Scenarios

| Scenario | User intent | Mode | Core assertions |
| --- | --- | --- | --- |
| `update_preserves_guid` | Rebuild the same stable notes with `compare_to` | Default Strict via `compare_to` | `Project.build(...)` returns `BuildReport`; `ensure_success()` passes; report includes `UPDATE.GUID_PRESERVED_FROM_PREVIOUS`, marks one preserved note, and the updated APKG keeps the previous Anki GUID |
| `update_adds_new_note` | Add one new note while preserving existing notes | Default Strict via `compare_to` | `Project.build(...)` returns `BuildReport`; `ensure_success()` passes; existing note identity stays stable and counts increase by one |
| `field_rename_stable_key_safe` | Rename display field while keeping stable key/config identity | Default Strict via `compare_to` | `Project.build(...)` returns `BuildReport`; `ensure_success()` passes; report includes warning `UPDATE.FIELD_RENAMED`, does not include `UPDATE.FIELD_MERGE_ID_CHANGED`, and APKG remains inspectable |
| `field_config_id_drift_blocks` | Detect unsafe field identity drift before import | Default Strict via `identity_lockfile`, no `fail_on` threshold | `Project.build(...)` returns `BuildError.report`; report status is invalid, artifact is absent, `ensure_success()` fails, and diagnostics include `UPDATE.FIELD_MERGE_ID_CHANGED` with error severity and field source |
| `template_reorder_risk` | Surface scheduling/card risk when templates reorder | Default Strict via `compare_to`, no `fail_on` threshold | `Project.build(...)` returns `BuildReport`; report status is success, artifact is inspectable, `ensure_success()` passes, diagnostics include warning `UPDATE.TEMPLATE_ORD_CHANGED`, and risk contains high finding `RISK.TEMPLATE_REORDER` |
| `template_config_id_drift_blocks` | Detect unsafe template identity drift before import | Default Strict via `identity_lockfile`, no `fail_on` threshold | `Project.build(...)` returns `BuildError.report`; report status is invalid, artifact is absent, `ensure_success()` fails, and diagnostics include `UPDATE.TEMPLATE_MERGE_ID_CHANGED` with error severity and template source |

Update-safety scenarios construct their baseline and update packages with public Rust API calls:

- `update_preserves_guid`: build a baseline `Project` with `Project::stable_id` and one stable note, rewrite only the previous APKG's Anki GUID inside the temp directory to simulate an existing imported deck, then build an updated `Project` with the same stable id and `compare_to(previous)`. The rewrite helper follows the existing test-helper shape already present in `anki_forge/tests/update_safety_build_tests.rs`: open the APKG zip, read and zstd-decode `collection.anki21b`, update the single `notes.guid` row and its `notes.data.anki_forge_identity.selected_anki_guid`, zstd-encode the collection, and repackage the same zip entries. This helper is mandatory before the `update_preserves_guid` scenario ships; the simpler "compare two freshly generated packages" path does not prove preservation of an externally assigned previous Anki GUID. This is test input setup only; the scenario action remains the public `build(...compare_to(previous))` call.
- `update_adds_new_note`: build a baseline `Project` with one stable note, then build an updated `Project` with the same note plus one new stable note and `compare_to(previous)`.
- `field_rename_stable_key_safe`: baseline custom note type uses `Field::new("Expression").key("expr")` and a note with `.text("expr", ...)`; update uses `Field::new("Prompt").key("expr")` with the same note type id, note stable id, and note field key.
- `field_config_id_drift_blocks`: current custom note type uses `Field::new("Expression").key("expr")` and a note with `.text("expr", ...)`; the test prepares an identity lockfile baseline for the same project/note type/field key but with a different field `config_id`, following the existing `write_field_config_drift_lockfile` helper shape in `anki_forge/tests/phase4_product_build_tests.rs`. The scenario action is `Project.build(BuildOptions::new().output(...).identity_lockfile(lockfile))`.
- `template_reorder_risk`: baseline custom note type declares two templates in order `recognition`, then `production`; update declares the same template keys in the opposite order.
- `template_config_id_drift_blocks`: current custom note type declares a stable template key such as `recognition`; the test prepares an identity lockfile baseline for the same project/note type/template key but with a different template `config_id`. Creating this template drift lockfile helper is an explicit update-safety slice subtask, based on the existing field drift lockfile helper shape. The scenario action is `Project.build(BuildOptions::new().output(...).identity_lockfile(lockfile))`.

These preparations may inspect or locally edit the previous APKG artifact or identity lockfile as test input setup, but they must not call private lowering, reconciliation, or merge-safety functions as the scenario action.

All update-safety rows use the default `fail_on(None)` threshold unless a row explicitly says otherwise. Warning diagnostics and risk findings below an active threshold do not make `ensure_success()` fail. Error-severity update diagnostics in default Strict mode produce `BuildError.report` before an artifact is written.

## Assertion Rules

Capability tests should assert on user-visible outcomes:

- Package exists and is non-empty.
- `BuildReport.status`, `counts`, `artifact`, `diagnostics`, `media`, `policy`, and `update_safety.baseline_sources` match the user scenario.
- `report.ensure_success()` passes for clean success and fails for invalid, blocked, or error reports.
- APKG inspection sees the expected note types, fields, templates, card counts, media filenames, media references, deck names, and note identity metadata.
- Diagnostics use stable codes and useful source paths.

Capability tests should avoid:

- Asserting normalized IR field order unless the APKG or report exposes the behavior.
- Calling private reconciliation/lowering helpers as the main test action.
- Depending on test order or artifacts left by previous tests.

## Data And Artifact Flow

Each automated scenario gets an artifact directory created by `scripts/run_rust_user_capabilities.sh`, not by an auto-deleting `tempfile` owned by the Rust test. The script passes that path through `ANKI_FORGE_CAPABILITY_ARTIFACT_DIR`. Baseline and update packages are created inside that directory when needed.

Update-safety baselines are generated inline by the scenario through the Rust user API, then passed to the update build with `BuildOptions::compare_to(previous_apkg)`. The full matrix does not use committed APKG fixtures. If a scenario needs a legacy or drifted previous package, the test may prepare that previous APKG as local test input inside the scenario temp directory, but the scenario action under test remains the public build with `compare_to`.

The test helper layer should provide only test support:

- Validate and prepare the scenario artifact directory supplied by the script.
- Write a package through the public API.
- Inspect the generated package.
- Load or compute package SHA-256 when a checklist needs it.
- Format failure messages around the scenario id and user intent.

If `ANKI_FORGE_CAPABILITY_ARTIFACT_DIR` is unset or empty for an ignored matrix scenario, the Rust test fails immediately with a clear message directing the user to `scripts/run_rust_user_capabilities.sh`. Ordinary smoke tests are separate and may use auto-cleaning temp directories.

Generated artifacts are not committed. Manual Desktop runs write under `tmp/manual-desktop-rust-api-v1/`, matching the existing manual scenario style.

Automated matrix runs write under `target/tmp/rust-user-capabilities/<run-id>/<scenario>/`. The run id is generated once per script invocation as `YYYYMMDDTHHMMSSZ-<pid>` using UTC time and the script process id, for example `20260605T094233Z-12345`. The script removes a scenario directory only after that scenario exits successfully, unless `--keep-artifacts` is supplied. On failure, the script preserves the directory and prints it in the `fail` line. This avoids relying on panic/unwind behavior from temporary-directory destructors. Manual Desktop scripts write to the stable `tmp/manual-desktop-rust-api-v1/<scenario>/` tree for human inspection; those scripts remove only the targeted Rust API scenario output directory before regenerating it and must not delete the existing Authoring IR manual Desktop outputs.

## Test Entrances

### Ordinary Cargo Tests

Ordinary tests keep exactly three smoke assertions, reusing existing tests when they already satisfy this shape:

- `deck_basic_write_apkg_smoke`: one `Deck.write_apkg` Basic scenario.
- `project_stock_write_apkg_smoke`: one `Project.write_apkg` stock Basic or stock Basic+Cloze scenario.
- `deck_to_apkg_bytes_smoke`: one `to_apkg_bytes` scenario that writes the bytes to a temporary file and proves the APKG is inspectable.

These run during `cargo test --workspace` and catch severe API/package regressions quickly.

Each smoke assertion checks at least: successful report or successful bytes export, non-empty package artifact or byte buffer, `inspect_apkg` succeeds, and the inspected APKG contains at least one note and one card.

The smoke `deck_to_apkg_bytes_smoke` creates its own auto-cleaning temp directory for the inspection file. It does not depend on `ANKI_FORGE_CAPABILITY_ARTIFACT_DIR`, which is only for the ignored full-matrix target.

### Full Rust Capability Matrix

Add a dedicated local entry point such as:

```bash
./scripts/run_rust_user_capabilities.sh
./scripts/run_rust_user_capabilities.sh deck_basic_apkg update_preserves_guid
```

The script should support:

- Running all scenarios.
- Running named scenarios.
- Printing where artifacts and inspect reports were written, and whether the directory was kept or cleaned.
- Exiting non-zero on failed assertions.
- `--keep-artifacts` for preserving successful automated scenario directories during local debugging.
- `--manual-desktop <scenario>` for export-only generation of one manual Desktop package/checklist scenario.

The script should be written for Bash (`#!/usr/bin/env bash`) to match the repository's shell-script style and simplify strict-mode handling.

In automated mode, the script runs each selected scenario with:

- `ANKI_FORGE_CAPABILITY_MODE=automated`
- `ANKI_FORGE_CAPABILITY_ARTIFACT_DIR=target/tmp/rust-user-capabilities/<run-id>/<scenario>`

In manual Desktop mode, the script runs the selected scenario with:

- `ANKI_FORGE_CAPABILITY_MODE=manual-desktop`
- `ANKI_FORGE_CAPABILITY_ARTIFACT_DIR=tmp/manual-desktop-rust-api-v1/<scenario>`

The Rust test harness reads these environment variables. Scenario builders generate the same public API package in both modes; automated mode asserts and cleans artifacts on success, while manual Desktop mode keeps the APKG, inspect JSON, and checklist and exits non-zero only if export generation or APKG inspection fails.

Script infrastructure failures use exit code `2`; scenario assertion failures use exit code `1`. Infrastructure failures include inability to create the run root or scenario directory, missing system SHA-256 tool in manual Desktop mode, `cargo test --list` failing or discovering zero scenarios, and invalid scenario names supplied by the user. The script prints any completed `ok`/`fail` scenario lines before exiting. It does not delete previous run-id directories at startup; each run writes under a unique run id, and mid-run interruption leaves the current run directory in place for inspection. The first implementation does not impose a separate per-scenario timeout beyond the spawned Cargo process; if a timeout wrapper is later added, timeout is reported as `fail <scenario-id> <artifact-dir> kept`.

The canonical CI entry point is `make verify-ci`. That target delegates to `scripts/verify-ci.sh --ci`, and the script calls `scripts/run_rust_user_capabilities.sh`. `--fast` may skip the full matrix unless later performance proves it is cheap enough.

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
- Package SHA-256, computed by the script with a system hashing command such as `shasum -a 256` or `openssl dgst -sha256` so no Rust dependency is added only for manual metadata.
- Import before/after note and card counts.
- GUID/update behavior when relevant, or `N/A` for non-update scenarios.
- Duplicate note outcome when relevant.
- Media rendering notes.
- Media files verified in Anki browser/editor when relevant.
- Relevant diagnostics.

Checklist files should use this template. Every field remains present; when a field does not apply to the scenario, the generated checklist writes `N/A` rather than leaving it blank or omitting it.

```markdown
# Manual Desktop Check: <scenario>

- Date:
- Platform:
- Anki version:
- anki-forge commit:
- Package path:
- Package SHA-256:
- Import action: file_import | double_click_apkg
- Notes before import:
- Notes after import:
- Cards before import:
- Cards after import:
- GUID/update result:
- Duplicate note result:
- Media rendering result:
- Media files verified:
- Relevant diagnostics:
- Pass/fail:
- Notes:
```

This layer can coexist with the existing Authoring IR manual Desktop scenarios. The Rust API manual scenarios should be named clearly so users know which public API behavior they validate.

A human reviewer marks `Pass/fail: Pass` when the imported APKG matches the inspect report's note, card, deck, note type, and media expectations, and no unexpected Anki import warnings or rendering failures appear.

## Error Handling

The suite distinguishes clean success, warnings, invalid input, blocked updates, and infrastructure failures.

- Clean success: `ensure_success()` passes, APKG exists, APKG inspection is complete.
- Warning: the report exposes warning diagnostics and policy state; the test documents whether `ensure_success()` should pass under the selected policy.
- Invalid input: the user receives a parseable `BuildReport` or `BuildError.report` with stable diagnostics.
- Blocked update: the update-safety report identifies why importing the package would be unsafe.
- Infrastructure failure: missing temp paths, inability to write artifacts, or broken inspection fails the test as a harness problem, not as a user diagnostic expectation.
- Generated APKG corruption or unparseable APKG inspection is a harness/package-generation failure for success scenarios. It should not be reclassified as an expected user diagnostic unless the scenario deliberately produced an invalid package.

Optional dependencies such as upstream Anki source, `protoc`, or Anki Desktop are not required for this phase.

## Success Criteria

The design is implemented when:

1. Ordinary workspace tests include a small Rust user API package-writing smoke set.
2. A dedicated full matrix entry point exercises success, diagnostics, warnings, and update safety through Rust `Deck` and `Project` APIs.
3. Full matrix scenarios write real `.apkg` files and validate them through APKG inspection.
4. `make verify-ci` runs the full matrix.
5. Manual Desktop generation emits APKGs, inspect reports, and checklists without requiring Anki Desktop automation.
6. Test names, scenario ids, and assertions describe user behavior and results rather than internal implementation phases.

## Implementation Anchor

The full matrix uses ignored Cargo integration tests. This gives clear per-scenario failure names, keeps ordinary `cargo test --workspace` fast, and lets `scripts/run_rust_user_capabilities.sh` invoke the matrix explicitly. The first implementation slice creates `anki_forge/tests/rust_user_capability_matrix.rs` and `scripts/run_rust_user_capabilities.sh` even if it initially registers only the smoke/success scenarios; the CI and artifact-management shape should not be postponed to the final slice.

The script output format is plain text:

- `ok <scenario-id> <artifact-dir> cleaned` for passing scenarios whose automated artifacts were removed.
- `ok <scenario-id> <artifact-dir> kept` for passing scenarios run with `--keep-artifacts` or manual Desktop export mode.
- `fail <scenario-id> <artifact-dir> kept` for failing scenarios before exiting non-zero.
- A final summary line with total, passed, failed, and skipped counts.

On success, automated artifact directories are removed by the script unless `--keep-artifacts` is supplied. On failure, the full matrix preserves the failing scenario artifact directory and prints it in the `fail` line so the generated APKG and inspect report remain available for debugging. Smoke tests can use ordinary auto-cleaning temp directories.

Manual Desktop package generation is an export mode of the same full-matrix scenario builders, for example `scripts/run_rust_user_capabilities.sh --manual-desktop <scenario>`. Export mode writes APKGs, inspect JSON, and checklist files under `tmp/manual-desktop-rust-api-v1/`.

Implementation should land in ordered slices while preserving this single design scope:

1. Smoke tests and success scenarios.
2. Diagnostic and warning scenarios.
3. Update-safety scenarios.
4. Manual Desktop package and checklist generation.

Existing tests already cover pieces of this behavior. The implementation should reuse or relocate them only when that improves the user-facing capability story. It should not remove lower-level tests that still protect contracts or internals.
