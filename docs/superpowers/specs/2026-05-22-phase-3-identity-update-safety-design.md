# Phase 3 Identity and Update Safety Design

- Date: 2026-05-22
- Status: Approved in brainstorming
- Scope: `docs/api-design.md` Phase 3, `Identity / update safety`
- Related specs:
  - `2026-05-16-phase-1-user-facing-rust-mvp-design.md`
  - `2026-05-20-phase-2-media-diagnostics-productization-design.md`
  - `2026-04-13-note-stable-id-design.md`
  - `2026-04-04-phase-3-anki-compatibility-inspection-writer-design.md`

## 1. Purpose

Phase 3 makes anki-forge's long-term update behavior safer than a
genanki-style package writer.

The goal is not to introduce `FieldKey`, `TemplateKey`, or stable config id
derivation for the first time. Phase 1 already owns those. Phase 3 connects
existing product identity, field/template merge metadata, previous APKG
inspection, identity snapshots, lockfiles, and Anki import/update evidence into
one update-safety loop.

Phase 3 succeeds when an update-safe build can answer these questions with
structured evidence:

1. Which current note stable ids map to existing Anki note GUIDs?
2. Which GUIDs were preserved from a previous APKG or lockfile?
3. Which notes are new and safely derive a new GUID?
4. Which field/template merge ids are stable across updates?
5. Which changes are risky enough to block update-safe output?
6. Which parts are proven by CI automation, and which are covered by real Anki
   import/update oracle gates?

## 2. Confirmed Decisions

The brainstorming session fixed these decisions:

1. Implement the complete end-to-end Phase 3 design, from API through lockfile,
   previous APKG, diagnostics, and oracle strategy.
2. Use layered oracle coverage:
   - mainline CI uses APKG inspection, diff, SQLite/package observations, and
     lockfile fixtures
   - release or nightly gates include real Anki import/update oracle coverage
3. Keep ordinary builds relaxed. They may report update-safety risks, but they
   do not become strict update-safety builds by default.
4. Make update-safe builds strict. Passing `compare_to(...)`,
   `identity_lockfile(...)`, or explicitly enabling update safety activates the
   stricter path unless the caller chooses an explicit report-only mode.
5. Use two baselines:
   - identity lockfile as a fast, reviewable, git-committable source baseline
   - previous APKG as the artifact truth source
6. When lockfile and previous APKG conflict for the same stable id, prefer the
   previous APKG and report the conflict.
7. If stable id is unchanged but the current GUID derivation differs from the
   previous GUID, update-safe mode preserves the previous GUID. Ordinary builds
   only report the drift when they have enough baseline data.

## 3. Non-Goals

Phase 3 does not implement the full Phase 4 diff/risk product:

1. no complete artifact diff API
2. no complete semantic diff API
3. no `fail_on(RiskLevel::...)` policy enforcement
4. no machine-readable full CI report beyond the update-safety summary
5. no APKG import back into a full editable `Project`
6. no Python package release

Phase 3 may expose update-safety observations that Phase 4 later consumes, but
Phase 4 remains responsible for the broader diff/risk/CI product.

## 4. Current Context

The repository already has several Phase 1 and Phase 2 foundations:

1. Product `Project`, `NoteType`, `Field`, `Template`, and `Note` APIs.
2. `FieldKey` and `TemplateKey` with stable config id derivation:
   `stable_config_id(namespace, note_type_id, key)`.
3. Custom note type lowering that writes field and template `config_id`.
4. Product validation warnings for missing custom identity recipes and
   auto-derived field keys.
5. Deck-side resolved identity snapshots for stock Basic, Cloze, and Image
   Occlusion paths.
6. `BuildReport` with artifact, counts, diagnostics, media summary, metrics,
   and inspect summary.
7. `writer_core` APKG writing and APKG inspection of notes, notetypes, fields,
   templates, media, and config ids.
8. Writer-level inspect/diff contracts and tests.

The main Phase 3 gaps are:

1. Product `Project` custom note identity recipes do not yet produce a complete
   resolved identity snapshot for every note.
2. Product build does not yet produce a first-class identity index.
3. Writer currently uses `NormalizedNote.id` as `notes.guid`; it does not accept
   a separate GUID preservation plan.
4. APKG output is not yet self-describing enough for future builds to recover
   Product stable ids when preserved Anki GUIDs differ from current derivation.
5. There is no identity lockfile contract or API.
6. `BuildOptions` has no previous APKG or update-safety configuration.
7. `BuildReport` has no update-safety summary.
8. There are no import/update semantic tests for GUID preservation.

## 5. Approach

The selected approach is a layered update-safety spine.

The spine runs beside the existing build pipeline:

```text
Project
  -> validate/lower/normalize
  -> current IdentityIndex
  -> previous APKG IdentityIndex
  -> lockfile IdentityIndex
  -> reconcile
  -> writer GUID guidance
  -> APKG
  -> new lockfile and BuildReport summary
```

This approach keeps ordinary builds fast and familiar while making
update-safe builds strict and evidence-driven.

Rejected alternatives:

1. Lockfile-first. This is faster to implement and pleasant for CI, but it can
   drift away from the actual artifact imported into Anki.
2. APKG-first. This trusts the real artifact, but it is slower, harder to review
   in source control, and brittle for older APKGs that do not embed enough
   anki-forge identity metadata.

The selected design uses both sources and gives previous APKG priority when it
can recover the relevant identity.

## 6. Ownership Boundaries

`product` owns user intent:

1. note stable ids
2. custom notetype identity recipes
3. note-level identity overrides
4. source paths for diagnostics
5. update-safety API shape on `BuildOptions`

`build` owns orchestration:

1. choose update-safety mode
2. load lockfile
3. inspect previous APKG
4. construct current identity index
5. reconcile baselines
6. pass GUID guidance to writer
7. write lockfile
8. attach update-safety diagnostics and summary to `BuildReport`

`writer_core` owns artifact construction:

1. apply resolved GUID guidance
2. write Anki note GUIDs
3. preserve field/template config ids
4. embed enough anki-forge identity metadata for future inspection
5. report writer diagnostics without deciding gate policy

`inspect` owns artifact observation:

1. read previous APKG note GUIDs
2. read embedded anki-forge identity metadata when present
3. inspect notetype field/template config ids and ords
4. expose degradation when identity cannot be recovered

`contracts` owns stable interchange shapes:

1. identity index schema
2. identity lockfile schema
3. update-safety report or summary schema
4. fixture catalog entries for update-safety cases

## 7. Update-Safety Modes

Phase 3 recognizes three modes:

```text
Disabled
ReportOnly
Strict
```

`Disabled` is the default when no baseline or update-safety option is supplied.
The build follows existing behavior.

`Strict` is activated when the caller supplies `compare_to(...)`,
`identity_lockfile(...)`, or `update_safety(true)`.

`ReportOnly` is an explicit preview mode for users who want diagnostics from a
baseline without blocking output. Baseline-bearing options do not silently fall
back to report-only; callers must request it.

Recommended API shape:

```rust
project.build(
    BuildOptions::new()
        .output("jp-core.apkg")
        .compare_to("previous/jp-core.apkg")
        .identity_lockfile("anki-forge.lock.json")
        .write_identity_lockfile(true)
)?;
```

Optional explicit preview shape:

```rust
project.build(
    BuildOptions::new()
        .output("jp-core.apkg")
        .compare_to("previous/jp-core.apkg")
        .update_safety(UpdateSafetyMode::ReportOnly)
)?;
```

Strict mode changes severity. It does not create a different build pipeline.
The same lower/normalize/write stages run, but identity proof failures become
blocking diagnostics.

## 8. Core Data Models

### 8.1 IdentityIndex

`IdentityIndex` is the stable evidence model for update safety. It is produced
from current build input, previous APKG inspection, and lockfile loading.

Required fields:

```text
IdentityIndex
  schema_version
  source_kind: current | previous_apkg | lockfile
  source_ref
  observation_fingerprint
  project_stable_id
  notes[]
  notetypes[]
  limitations[]
```

`NoteIdentityEntry`:

```text
stable_id
anki_guid
current_guid_candidate
note_type_id
recipe_id
canonical_payload_hash
provenance
used_override
source_path
recovery_method
```

`NotetypeIdentityEntry`:

```text
note_type_id
anki_model_id
name
fields[]
templates[]
```

`FieldMergeEntry`:

```text
field_key
field_name
ord
config_id
tag
```

`TemplateMergeEntry`:

```text
template_key
template_name
ord
config_id
```

`current_guid_candidate` is the GUID that the current build would derive
without baseline preservation. `anki_guid` is the GUID observed or selected for
the entry source.

### 8.2 IdentityLockfile

The lockfile is a git-committable baseline, not the only source of truth.

Recommended default name:

```text
anki-forge.lock.json
```

Required fields:

```text
IdentityLockfile
  schema_version
  project_stable_id
  writer_policy_ref
  observation_fingerprint
  identity_index
  generated_by
```

The lockfile stores only update-safety evidence. It does not store note content
as a replacement for source files, and it does not attempt to become an editable
project format.

### 8.3 GuidResolution

Reconcile produces one `GuidResolution` per current note.

Required fields:

```text
GuidResolution
  stable_id
  selected_anki_guid
  current_guid_candidate
  source: previous_apkg | lockfile | current_derivation
  action: preserve | derive | fail
  diagnostics[]
```

Rules:

1. If previous APKG has a recoverable entry for `stable_id`, preserve that GUID.
2. Else if lockfile has an entry for `stable_id`, preserve that GUID.
3. Else use current derivation for a new note.
4. If previous APKG and lockfile disagree, preserve previous APKG and emit a
   warning.
5. If current identity data is missing or duplicated in strict mode, fail.

## 9. APKG Identity Recovery

Previous APKG is the artifact truth source only when identity can be recovered.
Phase 3 must make newly produced APKGs self-describing enough for future
update-safe builds.

The writer should embed minimal anki-forge identity metadata in the artifact.
The preferred storage is a stable JSON payload in note-level artifact metadata,
such as `notes.data`, plus notetype metadata already used by writer config.

Minimum note metadata:

```text
anki_forge_identity:
  stable_id
  recipe_id
  canonical_payload_hash
  current_guid_candidate
```

Inspection recovery order:

1. Use embedded anki-forge note identity metadata when present.
2. For older anki-forge APKGs without embedded note metadata, allow a
   compatibility recovery path when `notes.guid` exactly matches a current or
   lockfile stable id.
3. If neither path applies, mark the baseline entry unrecoverable.

This rule prevents a false claim that arbitrary APKGs can always be mapped back
to Product stable ids. In strict update-safe mode, unrecoverable identity for an
expected existing note is blocking.

## 10. Build Flow

Phase 3 build flow:

1. Run existing product validation.
2. Lower to authoring document.
3. Normalize to `NormalizedIr`.
4. Build current `IdentityIndex`.
5. If update-safety mode is disabled, continue with normal writer behavior.
6. If update-safety mode is report-only or strict, load baseline indexes:
   - inspect previous APKG if `compare_to(...)` is present
   - read lockfile if `identity_lockfile(...)` is present
7. Reconcile current index with baseline indexes.
8. Convert `GuidResolution` entries into writer GUID guidance.
9. Build APKG.
10. Inspect the produced APKG when requested.
11. Optionally write a new lockfile from the final selected identity index.
12. Attach update-safety summary and diagnostics to `BuildReport`.

`compare_to(...)` in Phase 3 means "use this previous APKG for identity and
update safety." It is not the complete Phase 4 diff API.

## 11. GUID Preservation Semantics

`stable_id` is Product identity. `anki_guid` is Anki import/update identity.
They are related, but not identical.

Current behavior writes normalized note ids as `notes.guid`. Phase 3 should
split the concepts:

```text
Product stable_id
  -> current_guid_candidate
  -> selected_anki_guid
  -> writer notes.guid
```

`current_guid_candidate` is deterministic from current resolved identity. The
exact derivation must be versioned and snapshot-tested. Existing packages may
use `stable_id` as the candidate; if Phase 3 changes the derivation, the change
must be represented in `current_guid_candidate` and not silently overwrite old
GUIDs in update-safe mode.

Strict update-safe rule:

1. Same stable id plus previous GUID means preserve previous GUID.
2. Same stable id plus changed current GUID candidate is warning-level drift,
   not a reason to create a duplicate note.
3. New stable id means derive a new GUID.
4. Duplicate stable ids are blocking.
5. Missing stable ids for notes that need update-safety proof are blocking.

Ordinary build rule:

1. Existing relaxed diagnostics remain warnings unless they are already errors.
2. If a baseline is not supplied, ordinary build cannot prove drift.
3. If report-only baseline analysis is requested, drift is reported but output
   is not blocked.

## 12. Field and Template Merge Safety

Phase 3 must prove that field and template merge metadata remains stable.

Current custom lowering already writes:

```text
field.config_id = stable_config_id("field", note_type_id, field.key)
template.config_id = stable_config_id("template", note_type_id, template.key)
```

Phase 3 extends this with baseline comparison:

1. Compare current notetype field key/name/ord/config id against previous APKG
   and lockfile snapshots.
2. Compare current template key/name/ord/config id against previous APKG and
   lockfile snapshots.
3. Treat config id drift for the same field/template key as blocking in strict
   mode.
4. Report field or template rename as non-blocking when key and config id stay
   stable.
5. Report template ord changes as warning in Phase 3, because cards are linked
   by `nid + ord` and scheduling risk belongs to the Phase 4 risk model.
6. Preserve source paths back to product fields and templates.

Strict Phase 3 does not need to fully decide scheduling risk. It must surface
template ord changes clearly so Phase 4 can promote them into risk policy.

## 13. Diagnostics

Phase 3 adds the `UPDATE.*` diagnostic family.

Recommended codes:

```text
UPDATE.BASELINE_APKG_UNREADABLE
UPDATE.BASELINE_LOCKFILE_UNREADABLE
UPDATE.BASELINE_CONFLICT_GUID
UPDATE.STABLE_ID_MISSING_IN_STRICT_MODE
UPDATE.STABLE_ID_DUPLICATE_ACROSS_BASELINES
UPDATE.GUID_PRESERVED_FROM_PREVIOUS
UPDATE.GUID_PRESERVED_FROM_LOCKFILE
UPDATE.GUID_DERIVED_FOR_NEW_NOTE
UPDATE.IDENTITY_PAYLOAD_CHANGED
UPDATE.FIELD_MERGE_ID_CHANGED
UPDATE.TEMPLATE_MERGE_ID_CHANGED
UPDATE.TEMPLATE_ORD_CHANGED
UPDATE.BASELINE_IDENTITY_UNRECOVERABLE
UPDATE.LOCKFILE_WRITTEN
```

Severity policy:

1. Errors in strict mode:
   - unreadable required previous APKG
   - unreadable or invalid required lockfile
   - duplicate stable ids
   - missing stable id for a note requiring update-safety proof
   - baseline identity unrecoverable for an expected existing note
   - field/template config id drift for the same key
2. Warnings:
   - previous APKG and lockfile conflict, with previous APKG selected
   - previous GUID preserved while current derivation differs
   - template ord changed
   - identity payload changed under the same explicit stable id
3. Info:
   - GUID preserved from previous APKG
   - GUID preserved from lockfile
   - GUID derived for a new note
   - lockfile written

Warnings and info should be summarized in `BuildReport`, but info-level
diagnostics may be omitted from pretty output unless requested.

## 14. BuildReport Extension

`BuildReport` gains an optional update-safety summary.

Recommended shape:

```text
UpdateSafetySummary
  mode: disabled | report_only | strict
  baseline_sources[]
  notes_preserved
  notes_derived
  notes_failed
  baseline_conflicts
  blocking_diagnostics
  lockfile_written
```

The existing `BuildError` shape remains valid. If strict update-safety fails,
`BuildError { cause: Diagnostics, report }` carries the full evidence.

## 15. Contract Assets

Phase 3 should add these contract assets:

```text
contracts/schema/identity-index.schema.json
contracts/schema/identity-lockfile.schema.json
contracts/schema/update-safety-summary.schema.json
contracts/semantics/identity-update-safety.md
```

Fixture catalog additions should cover:

1. current index generation
2. lockfile roundtrip
3. previous APKG priority over lockfile
4. GUID preservation
5. new note GUID derivation
6. baseline identity unrecoverable
7. field config id preservation
8. template config id preservation
9. template ord warning

## 16. Testing Strategy

### 16.1 Mainline CI Automation

Mainline CI should avoid dependence on a desktop Anki runtime.

It should test:

1. Product update-safe build writes APKG and lockfile.
2. Second build with same stable ids preserves previous GUIDs.
3. New note derives a new GUID.
4. Previous APKG and lockfile conflict chooses previous APKG.
5. Missing stable id blocks strict update-safe build.
6. Duplicate stable id blocks strict update-safe build.
7. Field config id drift blocks strict update-safe build.
8. Template config id drift blocks strict update-safe build.
9. Template ord change emits warning.
10. Baseline identity recovery degrades when metadata is absent and GUID cannot
    be matched to stable id.

These tests can use `writer_core::inspect_apkg`, SQLite observation, and
contract fixtures.

### 16.2 Contract and Golden Layer

Contract tests should validate:

1. identity index schema
2. lockfile schema
3. update-safety summary schema
4. fixture catalog integrity
5. golden JSON stability
6. compatibility recovery behavior for old APKGs where `notes.guid` equals
   stable id

### 16.3 Real Anki Oracle Layer

Release or nightly gates should run real Anki import/update scenarios.

Minimum scenarios:

1. First import then update with same stable ids updates existing notes instead
   of creating duplicates.
2. Adding a new note inserts only the new note.
3. Field rename with stable field key/config id remains update-safe.
4. Field config id drift is caught before import.
5. Template reorder is observed as a scheduling risk signal.

These gates may use manual desktop scenarios at first, then move to an Anki
rslib or desktop automation harness when available.

## 17. API Surface

Minimum public API additions:

```rust
pub enum UpdateSafetyMode {
    Disabled,
    ReportOnly,
    Strict,
}

impl BuildOptions {
    pub fn compare_to(self, path: impl Into<PathBuf>) -> Self;
    pub fn identity_lockfile(self, path: impl Into<PathBuf>) -> Self;
    pub fn write_identity_lockfile(self, write: bool) -> Self;
    pub fn update_safety(self, mode: UpdateSafetyMode) -> Self;
}
```

The default is:

```text
compare_to: None
identity_lockfile: None
write_identity_lockfile: false
update_safety: Disabled
```

Supplying `compare_to` or `identity_lockfile` with no explicit mode upgrades
the effective mode to `Strict`.

## 18. Rollout Order

Recommended implementation sequence:

1. Add contract schemas and semantics docs for identity index and lockfile.
2. Add current `IdentityIndex` generation from normalized Product builds.
3. Embed note identity metadata into newly produced APKGs.
4. Extend APKG inspect to recover identity metadata and compatibility
   `guid == stable_id` cases.
5. Add lockfile read/write.
6. Add reconcile and `GuidResolution`.
7. Add writer GUID guidance and selected GUID writing.
8. Add `BuildOptions` update-safety API.
9. Add `BuildReport` update-safety summary.
10. Add mainline CI tests.
11. Add release/nightly real Anki oracle scenarios.

The order intentionally makes artifacts self-describing before relying on APKG
baselines for future update-safe builds.

## 19. Acceptance Criteria

Phase 3 is complete when all are true:

1. `Project::build(compare_to(...))` can preserve previous GUIDs for unchanged
   stable ids.
2. `Project::build(identity_lockfile(...))` can preserve GUIDs from a lockfile.
3. When both baselines are present and conflict, previous APKG wins and a
   warning is reported.
4. New notes derive new GUIDs deterministically.
5. Strict update-safe mode blocks missing stable ids, duplicate stable ids,
   unrecoverable expected baseline identities, and field/template config id
   drift.
6. Template ord changes are visible in diagnostics and summary.
7. Build output can write an updated identity lockfile.
8. Newly produced APKGs embed enough identity metadata for future recovery.
9. Mainline CI proves the update-safety loop without a desktop Anki dependency.
10. Release or nightly oracle coverage proves at least one happy-path update
    and one dangerous-change scenario against real Anki behavior.
