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
5. project stable id storage on `Project`

`build` owns orchestration:

1. choose update-safety mode
2. load lockfile
3. inspect previous APKG
4. construct current identity index
5. reconcile baselines
6. pass GUID guidance to writer
7. write lockfile
8. attach update-safety diagnostics and summary to `BuildReport`
9. define `BuildOptions` update-safety fields and `UpdateSafetyMode`

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
Disabled mode disables baseline analysis and strict proof, not artifact
self-description. Product builds still embed `notes.data.anki_forge_identity`
for notes with resolved stable identity when the carrier can be merged, so an
ordinary APKG can later serve as a `compare_to(...)` baseline. Notes that truly
lack resolved stable identity in a disabled build cannot receive identity
metadata and will not be recoverable by embedded metadata later.

`Strict` is activated when the caller supplies `compare_to(...)`,
`identity_lockfile(...)`, or `update_safety(UpdateSafetyMode::Strict)`.
Strict update safety does not blanket-promote existing Product validation
warnings. Phase 1 warnings, such as missing custom identity recipes or
auto-derived field keys, keep their existing severity unless they prevent a
Phase 3 proof. When they do prevent proof, the Phase 3 diagnostic is what
blocks the build, for example missing resolved note stable ids or field/template
config id drift. Phase 4 may add broader risk policy promotion.

`ReportOnly` is an explicit preview mode for users who want diagnostics from a
baseline without blocking output. Baseline-bearing options do not silently fall
back to report-only; callers must request it.

Mode selection is a single ordered algorithm:

1. Validate output options first. If `write_identity_lockfile(true)` is set
   without `identity_lockfile(...)`, fail before writer execution with
   `UPDATE.LOCKFILE_PATH_REQUIRED`.
2. If the caller explicitly sets `UpdateSafetyMode::Disabled`, the effective
   mode is disabled. Baseline inputs are ignored, and
   `UPDATE.BASELINE_IGNORED_DISABLED` records that choice. If lockfile writing
   is also requested, the build writes a current-only lockfile to the supplied
   `identity_lockfile(...)` path without reconciling ignored baselines.
3. If the caller explicitly sets `UpdateSafetyMode::ReportOnly`, baseline
   inputs are analyzed but do not block artifact output.
4. If the caller explicitly sets `UpdateSafetyMode::Strict`, baseline proof
   failures block output.
5. If no explicit mode is set and `identity_lockfile(...)` is present, the
   effective mode is `Strict`. This includes
   `identity_lockfile(...) + write_identity_lockfile(true)`, so the first
   lockfile-creation build validates current identity strictly while treating a
   missing lockfile as an empty baseline.
6. If no explicit mode is set and `compare_to(...)` is present, the effective
   mode is `Strict`.
7. If no explicit mode is set and no baseline input is present, the effective
   mode is `Disabled`.

`write_identity_lockfile(true)` without a baseline input does not activate
strict mode by itself, but it still needs an `identity_lockfile(...)` path so
the library knows where to write. A CLI may provide its own default path, such
as `anki-forge.lock.json` in the command working directory, before calling the
library API. When combined with `compare_to(...)`, the written lockfile
contains the reconciled selected GUIDs.

The second-build workflow may use only `identity_lockfile(...)` with no
previous APKG. That is a valid strict update-safe build, but it is weaker than
using both baselines because it cannot observe artifact drift outside the
lockfile.

If `identity_lockfile(path)` is supplied and the file does not exist, behavior
depends on write intent. With `write_identity_lockfile(true)`, the missing file
is treated as an empty baseline so a first build can create the initial
lockfile. Without `write_identity_lockfile(true)`, the missing file is an
unreadable required baseline in strict mode.

If current project, previous APKG metadata, and lockfile all carry
`project_stable_id`, they must match in strict mode. A mismatch emits
`UPDATE.PROJECT_STABLE_ID_MISMATCH` and blocks output because it usually means
the caller compared against the wrong project's baseline.

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
  writer_policy_ref
  project_stable_id
  notes[]
  notetypes[]
  limitations[]
```

`source_ref` is a stable logical source handle, not a filesystem path:

```text
current: "current"
previous_apkg: "baseline.previous_apkg.primary"
lockfile: "baseline.identity_lockfile.primary"
```

Diagnostics may carry display paths separately, but `source_ref` stays stable
across machines and is safe to snapshot. These values are permanent logical
identifiers, not method-name mirrors. If a later phase adds another APKG or
lockfile baseline, it must define a new `source_ref` value instead of reusing
the primary Phase 3 handle.

`NoteIdentityEntry`:

```text
stable_id
normalized_note_id?
anki_guid
current_guid_candidate
guid_derivation_version
note_type_id
recipe_id
canonical_payload_hash?
provenance
used_override
entry_lifecycle
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

`note_type_id` is the primary join key for `NotetypeIdentityEntry` across
current, lockfile, and previous APKG indexes. `anki_model_id` is observed
artifact identity and audit data, not the Product-side join key. Field and
template merge entries are then compared within the joined notetype by
`field_key` and `template_key`.

`current_guid_candidate` is the GUID that the current build would derive
without baseline preservation. `anki_guid` is the GUID observed or selected for
the entry source.

`normalized_note_id` is the `NormalizedNote.id` produced by lowering and
normalization. It is optional only for lockfile entries with
`entry_lifecycle: absent_from_current`. In strict Phase 3 Product builds, every
active current note entry must carry both `stable_id` and `normalized_note_id`.
For current Product entries using `guid.raw-stable-id.v1`,
`normalized_note_id == stable_id` is a strict requirement. If either value is
missing or they differ for an active current entry, update-safe build fails
before writer execution with `UPDATE.NORMALIZED_NOTE_ID_MISMATCH`.
For valid Phase 3 lowering this invariant should never fire. It guards against
pipeline defects where resolved Product identity and `NormalizedNote.id`
diverge, such as a future lowering regression that applies a note override to
one layer but not the other. Tests can cover it through deliberate fixture
corruption rather than a normal user workflow.

Phase 3 defines one GUID derivation version:

```text
guid_derivation_version = "guid.raw-stable-id.v1"
current_guid_candidate = stable_id
```

In strict update-safety mode, note stable ids used as GUID candidates must be
non-empty after trimming, must not have leading or trailing whitespace, and must
not contain ASCII control characters. A future change to encoding, truncation,
hashing, or Anki-specific GUID normalization must introduce a new
`guid_derivation_version`; it must not silently change `guid.raw-stable-id.v1`.
Phase 3 never truncates a stable id to fit Anki. If the selected writer policy
or current Anki GUID validator imposes a length or character limit and
`stable_id` violates it, the build emits `UPDATE.ANKI_GUID_INVALID` and blocks
writer execution.

In strict update-safety mode, every current output note requires a resolved
stable id, whether explicit or inferred from an identity recipe. This includes
new notes that are not present in any baseline; they still need stable Product
identity so future updates can prove continuity.
`UPDATE.STABLE_ID_MISSING_IN_STRICT_MODE` is therefore scoped to current output
notes, not to old baseline entries that are absent from the current project.
Report-only mode reports the condition without blocking; disabled mode keeps
the existing relaxed behavior.

`project_stable_id` is the explicit value set with `Project::stable_id(...)`.
It must be non-empty after trimming and must not contain ASCII control
characters. The recommended value is a stable human-controlled slug such as
`jp-core` or `org.example.jp-core`; no global namespace format is required in
Phase 3. Strict update-safety builds that read or write lockfiles require it.
If it is missing, the build emits `UPDATE.PROJECT_STABLE_ID_MISSING` and does
not write a lockfile. Compare-only strict builds may continue in degraded mode;
in that case the diagnostic is warning-level and the summary must state that no
project-level lockfile anchor was available.
In this degraded compare-only mode, GUID preservation from a recoverable
previous APKG still works; project-level mismatch checks and lockfile writes do
not run.

Compare-only means `compare_to(...)` is present, `identity_lockfile(...)` is
absent, and `write_identity_lockfile(false)` is effective. If
`write_identity_lockfile(true)` is set without `identity_lockfile(...)`, the
build fails with `UPDATE.LOCKFILE_PATH_REQUIRED` before entering degraded mode.
If `identity_lockfile(...)` is present, the build is lockfile-bearing and a
missing `project_stable_id` is blocking.

Degraded compare-only strict mode still blocks on current duplicate/missing
stable ids, duplicate selected GUIDs, unrecoverable expected previous APKG
entries, unreadable previous APKG, and field/template config id drift. Only
project-level lockfile-anchor checks are skipped.

`writer_policy_ref` is the loaded writer policy identity in `"{id}@{version}"`
form, for example `writer-policy.default@1.0.0`. The `id` and `version`
segments are the exact field values from `WriterPolicy { id, version }`, with
no case folding, trimming, or semantic-version parsing. Neither segment may
contain `@` or ASCII control characters; a policy violating that rule produces
`UPDATE.WRITER_POLICY_REF_INVALID` when update-safety analysis or lockfile
writing is requested. Lockfile consumers compare this value to the current
writer policy and warn when it changes. The value comes from the `WriterPolicy`
loaded by the runtime writer stack; this struct already exists in
`writer_core`. Changes to writer behavior that affect GUID assignment,
identity metadata embedding, or field/template merge semantics must update the
writer policy version. Phase 3 compares `writer_policy_ref` by exact string
equality; all mismatches are warning-level `UPDATE.WRITER_POLICY_MISMATCH`.

`canonical_payload_hash` is optional. When present, it is a BLAKE3 hash of the
canonical identity payload produced by the relevant identity recipe, matching
the canonical payload rules from the note stable id design. Explicit stable ids
omit it in Phase 3 because explicit identity intentionally wins over recipe
inference. The
`UPDATE.IDENTITY_PAYLOAD_CHANGED` diagnostic is emitted only when both baseline
and current entries have comparable payload hashes.
When omitted in JSON, the key is absent. It is not serialized as `null` or an
empty string.

Comparable payload hashes must use the same hash algorithm, the same
`recipe_id`, and the same canonical identity payload rules.
The serialized format is `blake3:<lowercase-hex>`. The prefix is required and
is part of compatibility validation; comparison then uses the full string.
Phase 3 reuses the existing BLAKE3 note stable id contract; it does not
introduce a separate hash family for update safety. The repository already uses
BLAKE3 for note identity and media content-addressing, so Phase 3 should call
the existing canonical payload/hash helpers instead of adding a new dependency
or hashing abstraction.
`recipe_id` is the payload-rule version boundary. If canonical payload
serialization rules change, the responsible identity recipe must receive a new
`recipe_id`; Phase 3 does not add a second
`canonical_payload_hash_version` field.

`provenance` reuses the existing resolved note identity provenance values for
current entries and fully recovered baseline entries:

```text
ExplicitStableId
InferredFromNoteFields
InferredFromNotetypeFields
InferredFromStockRecipe
```

For previous APKG entries recovered without embedded metadata, `provenance`
records the best available Product-side provenance when a lockfile entry
supplies it. If no Product provenance can be recovered, the baseline-only value
is `unknown_baseline` and the index carries
`unknown_baseline_provenance` in `limitations[]`. Current indexes must not emit
`unknown_baseline`.

`recovery_method` is one of:

```text
current_resolution
embedded_metadata
lockfile_join
guid_equals_stable_id
unrecoverable
```

`entry_lifecycle` is one of:

```text
active
absent_from_current
```

Current indexes only emit `active`. Lockfile indexes may carry
`absent_from_current` entries when a stable id was present in an older lockfile
but is not present in the current project.

`limitations[]` is a sorted, deduplicated list of limitation codes describing
degraded identity evidence. Initial codes:

```text
project_stable_id_missing
writer_policy_mismatch
partial_apkg_inspection
identity_metadata_missing
identity_metadata_schema_unsupported
identity_metadata_malformed
lockfile_join_used
guid_equals_stable_id_compat
unrecoverable_apkg_entries
project_stable_id_mismatch
unknown_baseline_provenance
```

Limitations are recomputed for each index. They are not append-only; if a later
build embeds metadata and no longer needs `guid_equals_stable_id_compat`, the
limitation disappears from the new index.

`limitations[]` and `UPDATE.*` diagnostics serve different layers. Limitations
are serialized evidence properties of a specific `IdentityIndex` and travel
with lockfiles and inspected APKG baselines. Diagnostics are build events with
severity, source locations, remediation text, and mode-specific blocking
behavior. Implementations should derive overlapping limitation and diagnostic
values from shared internal classifiers to avoid divergence.
The shared classifier model is a small internal registry: each classifier
returns a stable limitation code when the condition describes source evidence,
and may also return one or more `UPDATE.*` diagnostic codes when the condition
matters to the current build mode. The implementation plan should define this
registry before adding lockfile or report serialization so both taxonomies are
generated from the same condition evaluation.

Supported schema versions are exact-match in Phase 3:

```text
IdentityIndex.schema_version = "identity-index-v1"
IdentityLockfile.schema_version = "identity-lockfile-v1"
embedded note metadata schema_version = "identity-note-v1"
```

These versions are independent. A newer or unknown baseline schema version is
`UPDATE.BASELINE_SCHEMA_UNSUPPORTED`. Strict mode treats it as blocking.
Report-only mode reports it as a warning and ignores that baseline.
Unknown enum values inside a lockfile, including `guid_derivation_version`,
`guid_source`, or `recovery_method`, are lockfile schema failures reported as
`UPDATE.BASELINE_LOCKFILE_UNREADABLE` with `error_kind: schema`. Unknown enum
values inside embedded APKG note metadata make only that note's metadata
malformed; the inspector records `identity_metadata_malformed` and attempts the
other recovery paths.

Future schema changes must provide an explicit migration path or instruct users
to regenerate the lockfile from a trusted previous APKG. Phase 3 performs no
silent lockfile migration. If a future implementation supports
`identity-lockfile-v2`, it must either read and migrate `identity-lockfile-v1`
deterministically or fail with `UPDATE.BASELINE_SCHEMA_UNSUPPORTED` and a
clear remediation message.

### 8.2 IdentityLockfile

The lockfile is a git-committable baseline, not the only source of truth.

Recommended default name:

```text
anki-forge.lock.json
```

When `write_identity_lockfile(true)` is used, `identity_lockfile(...)` must
provide the path. The same path is used for read and write. CLI wrappers may
supply the default path on behalf of the user.

Lockfile writes must be target-preserving. The implementation writes canonical
JSON to a temporary file in the same directory, validates the staged JSON
against the lockfile schema, and then replaces the target with a
platform-supported atomic replace primitive, such as POSIX same-filesystem
`rename` or Windows `ReplaceFileW`/equivalent. If the platform cannot provide a
target-preserving replace, the build fails with `UPDATE.LOCKFILE_WRITE_FAILED`
before modifying the target. If any write, validation, sync, or replace step
fails, the previous target lockfile must remain readable and unchanged.
Phase 3 assumes filesystem lockfile writes on macOS, Linux, and Windows.
Targets without a target-preserving file replacement primitive may support APKG
building but must report lockfile writing as unsupported rather than weakening
this guarantee.

Required fields:

```text
IdentityLockfile
  schema_version
  project_stable_id
  writer_policy_ref
  identity_index
  generated_by
```

`generated_by` is deterministic tool provenance, not an audit timestamp:

```text
generated_by
  tool: "anki-forge"
  tool_version: "<crate/package version>"
  writer_policy_ref: "<id>@<version>"
```

`tool_version` is the exact package version that produced the lockfile. The
field intentionally excludes wall-clock time, user name, host name, cwd, and
absolute paths so lockfile diffs stay reviewable and reproducible.

The lockfile stores only update-safety evidence. It does not store note content
as a replacement for source files, and it does not attempt to become an editable
project format.

When writing a new lockfile, entries for stable ids that are absent from the
current build are carried forward with `entry_lifecycle: absent_from_current`.
They do not participate in the current APKG write, but they remain visible for
audit and can preserve a GUID if the same stable id is reintroduced later.
When a stable id is reintroduced, the existing lockfile entry is updated back
to `entry_lifecycle: active`; the lockfile never contains two entries with the
same stable id. Phase 3 does not auto-prune absent entries. Users may manually
remove them from the lockfile when they intentionally want to forget an old
Anki GUID; manual removal means deleting those entries from the lockfile JSON
and letting schema validation confirm the edited file remains valid. A
dedicated pruning command can be a later follow-up.
Long-running projects may accumulate these entries; this is an accepted Phase 3
limitation and should be called out in docs until a pruning command exists.
If a lockfile contains more than 10,000 `absent_from_current` entries, the build
emits `UPDATE.LOCKFILE_ABSENT_ENTRIES_HIGH` at info level.
Phase 3's intended comfortable scale is up to roughly 100,000 total lockfile
note entries or a 25 MB lockfile on a normal developer machine. The
implementation plan must benchmark parse/reconcile/write time at that scale. If
the benchmark shows multi-second latency or excessive memory use, the pruning
command moves from future follow-up to Phase 3 exit work and the diagnostic
threshold should be lowered.

Notetype entries are carried forward when referenced by an active note. An
`absent_from_current` note may also carry forward the last known notetype
snapshot from the existing lockfile for GUID audit and future reintroduction.
That carried-forward notetype is not compared for field/template config drift
unless a current notetype with the same stable key exists. Unreferenced removed
notetypes that are not needed by active or retained absent notes are dropped
from the new lockfile.

Lockfile JSON uses lexicographic object keys by Unicode scalar value after JSON
string decoding. Phase 3 schema keys are ASCII, but the ordering rule is
explicit for future extension. Arrays whose order is semantic, such as notetype
templates by `ord`, preserve semantic order. Arrays whose order is not
semantic, such as identity entries in lockfiles, are sorted by stable key
before serialization. Active and `absent_from_current` identity entries are
interleaved in that single stable-key order, not grouped by lifecycle.

Lockfiles enforce two uniqueness invariants:

1. `stable_id` is unique across note entries.
2. `anki_guid` is unique across note entries.

Duplicate stable ids or GUIDs make the lockfile invalid. Strict mode reports
them as blocking diagnostics.

A manually removed `absent_from_current` entry means "forget this old GUID."
If the same stable id is later reintroduced and neither previous APKG nor
lockfile can recover its old GUID, the note is treated as new and receives the
current derived GUID.

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
2. Else if lockfile has an entry for `stable_id`, including an
   `absent_from_current` entry reintroduced by the current project, preserve
   that GUID.
3. Else use current derivation for a new note.
4. If previous APKG and lockfile disagree, preserve previous APKG and emit a
   warning.
5. If current identity data is missing or duplicated in strict mode, fail.
6. Lockfile entries absent from the current build produce
   `UPDATE.STABLE_ID_REMOVED_FROM_CURRENT` at info level in Phase 3. They are
   not blocking because Anki import does not delete missing notes by default.
7. If two different stable ids would receive the same selected GUID during
   reconcile, emit `UPDATE.GUID_DUPLICATE_AT_RECONCILE` and fail in strict
   mode. Phase 3 does not auto-resolve GUID collisions. The check runs after
   GUID selection across all sources, so it catches duplicate previous APKG
   GUIDs, duplicate lockfile GUIDs, `guid == stable_id` compatibility
   collisions, cross-source cases where an APKG GUID recovered for one stable
   id collides with a different lockfile stable id, and current-derivation
   collisions.
   The diagnostic payload must name both stable ids and both GUID sources.

Reconciliation joins by stable id after baseline recovery. Recovery itself may
use embedded stable id metadata, lockfile GUID joins, or `guid == stable_id`
compatibility matching to assign a stable id to a previous APKG entry. Once a
previous entry has a recovered stable id, the reconcile step treats it the same
as any other stable-id keyed baseline entry.

If a reintroduced `absent_from_current` lockfile entry and previous APKG both
provide a GUID for the same stable id, previous APKG still wins and
`UPDATE.BASELINE_CONFLICT_GUID` records the conflict.

If strict reconcile produces any `fail` action, writer execution does not
start and no `WriterGuidPlan` is passed to writer. Report-only mode may still
write an artifact; in that case it passes a full plan using current derivation
for entries that could not safely preserve a baseline GUID, and records the
failed preservation as diagnostics.

### 8.4 Writer GUID Plan

Reconcile passes writer guidance as a sorted list, not as an unordered map.

```text
WriterGuidPlan
  assignments[]

WriterGuidAssignment
  normalized_note_id
  stable_id
  selected_anki_guid
  guid_derivation_version
  source: previous_apkg | lockfile | current_derivation
```

`normalized_note_id` is the `NormalizedNote.id` produced before GUID
preservation. The writer uses `normalized_note_id` to find the note and writes
`selected_anki_guid` to Anki `notes.guid`.

The selected writer API change is an optional `guid_plan` parameter on the
writer build entrypoint. `BuildContext` remains runtime/build configuration and
does not carry per-note GUID assignments. The plan stays sorted and
schema-governed so writer tests can snapshot it. `WriterGuidPlan.assignments`
is sorted by `normalized_note_id` ascending, with `stable_id` as a deterministic
tie-breaker that should never be needed for valid current input.

When a `WriterGuidPlan` is supplied, writer validates it before writing. The
set of `normalized_note_id` values in the plan must exactly equal the set of
`NormalizedNote.id` values in the normalized IR. Missing assignments, extra
assignments, or duplicate assignments produce `UPDATE.WRITER_GUID_PLAN_MISMATCH`
and block writer execution. Writer must not silently skip, panic, or partially
apply a mismatched plan.

## 9. APKG Identity Recovery

Previous APKG is the artifact truth source only when identity can be recovered.
Phase 3 must make newly produced APKGs self-describing enough for future
update-safe builds.

The writer embeds minimal anki-forge identity metadata in the artifact. The
preferred storage is a stable JSON payload in note-level artifact metadata,
such as `notes.data`, plus notetype metadata already used by writer config.
The payload is namespaced under `anki_forge_identity` and must not overwrite
other JSON keys that Anki or add-ons may place in `notes.data`.

`notes.data` merge strategy:

1. Empty or missing note data is treated as `{}`.
2. Existing valid JSON object data is preserved, and only the
   `anki_forge_identity` key is inserted or replaced.
3. Existing invalid JSON, non-object JSON, or an unmergeable data payload emits
   `UPDATE.NOTE_DATA_METADATA_UNMERGEABLE` and blocks writer execution.
4. Writer never replaces the entire `notes.data` payload to force metadata in.

The unmergeable-data failure is intentionally whole-build, not per-note skip.
Phase 3 APKG output represents the current `Project`; omitting one note would
silently change the deck, and writing only some identity metadata would make
future baseline recovery misleading. Phase 3 has no skip-with-diagnostic option
for this case.

Minimum `notes.data` shape:

```json
{
  "anki_forge_identity": {
    "schema_version": "identity-note-v1",
    "stable_id": "jp-vocab:taberu",
    "recipe_id": "custom.fields.v1",
    "canonical_payload_hash": "blake3:...",
    "current_guid_candidate": "...",
    "selected_anki_guid": "...",
    "guid_derivation_version": "guid.raw-stable-id.v1",
    "guid_source": "previous_apkg",
    "recovery_method": "embedded_metadata"
  }
}
```

`guid_source` is one of:

```text
previous_apkg
lockfile
current_derivation
```

If `anki_forge_identity` is present on a note but has an unknown
`schema_version`, invalid JSON shape, missing required fields, or values of the
wrong type, only that note's embedded metadata is treated as unrecoverable. The
inspector records `identity_metadata_schema_unsupported` or
`identity_metadata_malformed` and then attempts lockfile join or
`guid == stable_id` compatibility recovery. The entire APKG baseline is not
rejected unless required note/notetype/merge-id domains are unreadable.

The embedded metadata records both the pre-reconciliation
`current_guid_candidate` and the post-reconciliation `selected_anki_guid`.
Future APKG inspection uses `selected_anki_guid` as the actual Anki import
identity and uses `current_guid_candidate` only for drift/audit diagnostics.

The `notes.data` carrier is guaranteed only for APKGs directly produced by
anki-forge until real Anki oracle evidence proves import/export preservation.
If a user supplies an APKG re-exported by Anki and `notes.data` was stripped or
rewritten, recovery degrades to lockfile join or `guid == stable_id`
compatibility matching. Phase 3 exit evidence must record whether supported
Anki versions preserve this metadata; if they do not, embedded identity
metadata from re-exported APKGs is treated as unavailable.

The Phase 3 contingency if Anki strips `notes.data.anki_forge_identity` is to
treat re-exported APKG embedded metadata as unavailable, not to add a late
schema-changing writer carrier. Re-exported APKG baselines then require either
a valid lockfile GUID join or `guid == stable_id` compatibility recovery.
Hidden user-visible note fields, synthetic templates, or other carrier changes
are explicitly out of Phase 3 scope because they alter the user's notetype
surface. A later phase may introduce a proven alternative carrier after a
separate design review.

Inspection recovery order:

1. Use embedded anki-forge note identity metadata when present.
2. If a lockfile is supplied, join previous APKG `notes.guid` to lockfile
   `anki_guid` entries and inherit the corresponding stable id.
3. For older anki-forge APKGs without embedded note metadata, allow a
   compatibility recovery path when `notes.guid` exactly matches a current or
   lockfile stable id.
4. If none of these paths apply, mark the baseline entry unrecoverable.

The `guid == stable_id` compatibility path is an exact string match against a
stable id in the current `IdentityIndex` or a stable id stored in the lockfile.
It does not match against `current_guid_candidate`, and it does not trim or
normalize the APKG GUID during matching.
Lockfile `absent_from_current` entries are included in this match. Stable ids
are project-global identities, not notetype-scoped identities; if a current
note uses a stable id that exists only as an absent lockfile entry, Phase 3
treats that as an intentional reintroduction and preserves the old GUID when
possible. Users who intend the same string to mean a new note must manually
remove the absent lockfile entry before rebuilding.

If lockfile GUID uniqueness validation fails, lockfile join is not attempted.
The whole lockfile is rejected for GUID-to-stable-id join purposes, not only
the duplicate entries. In strict mode the invalid lockfile already blocks the
build. In report-only mode, APKG inspection may still use embedded metadata and
`guid == stable_id` compatibility recovery, but it must not use any GUID join
from that invalid lockfile.

This rule prevents a false claim that arbitrary APKGs can always be mapped back
to Product stable ids.

Previous APKG notes never become output notes by themselves. The current
`Project` controls the output APKG contents. Unrecoverable previous APKG notes
that are not represented by current project notes are excluded from the output
artifact and reported only as baseline limitations.

An "expected existing note" means a current stable id that is present in a
supplied lockfile or in previous APKG embedded metadata. Strict mode emits
`UPDATE.BASELINE_IDENTITY_UNRECOVERABLE` only when that expected stable id
cannot be matched to a previous APKG note by embedded metadata, lockfile GUID
join, or `guid == stable_id` compatibility recovery. Unrecoverable APKG notes
that cannot be associated with any current or lockfile stable id degrade
baseline coverage, but they do not block strict mode by themselves.

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

"When requested" in step 10 means the existing `BuildOptions::inspect` flag is
true. Phase 3 does not add another inspect toggle; strict and report-only modes
use the same flag and default behavior as ordinary builds.

In strict mode, an unreadable required previous APKG or lockfile prevents
writer execution and APKG output. The build still returns a `BuildError` with a
partial `BuildReport` containing validation and baseline-loading diagnostics.
Report-only mode records baseline-loading failures as warning diagnostics and
may continue to write the artifact when the current project itself is otherwise
valid. Current-project validation errors remain errors in every mode.

Partially corrupt previous APKGs may produce a partial identity index with
`partial_apkg_inspection` when the inspector can still read the required note,
notetype, and merge-id domains. If those domains are unavailable, the APKG is
treated as unreadable for update-safety purposes.

A valid readable APKG with zero notes or zero notetypes is a normal empty
baseline. It blocks only if strict mode expected existing notes from a lockfile
or embedded metadata and cannot match them.

If `write_identity_lockfile(true)` is requested and lockfile writing fails,
the build returns `BuildError` with an IO/write-failure cause and an
`UPDATE.LOCKFILE_WRITE_FAILED` diagnostic. This is an error in every mode
because the caller explicitly requested the lockfile as an output. The APKG may
already exist on disk; the report must state whether the artifact was written.
Because lockfile writes are atomic, the target lockfile remains at its previous
valid contents when the write fails, though a temporary file may remain for
debugging or cleanup.
If `write_identity_lockfile(true)` is requested without an
`identity_lockfile(...)` path, the build fails before writer execution with
`UPDATE.LOCKFILE_PATH_REQUIRED`.

Explicit `UpdateSafetyMode::Disabled` ignores baseline inputs but does not
disable requested outputs. If `write_identity_lockfile(true)` and
`identity_lockfile(...)` are supplied in Disabled mode, the build writes a
current-only lockfile using current derivation and emits
`UPDATE.BASELINE_IGNORED_DISABLED` for ignored baseline inputs.

Partial reports from strict baseline-loading failure include the effective
update-safety mode, current identity index when it was successfully generated,
baseline-loading diagnostics, no artifact, and an update-safety summary with
zero preserved/derived counts and `blocking_diagnostics` populated.
Reconcile failures follow the same report rule: if the current `IdentityIndex`
was generated, the error report includes it along with reconciliation
diagnostics, selected counts computed before the failure when meaningful, no
artifact, and populated `blocking_diagnostics`.

`compare_to(...)` in Phase 3 means "use this previous APKG for identity and
update safety." It is not the complete Phase 4 diff API.

If neither current project nor previous APKG nor lockfile carries
`project_stable_id`, an update-safe build cannot prove the previous APKG belongs
to the intended project. In that case the build relies on note-level stable id
matching and diagnostics must make the degraded project-level proof explicit;
users are responsible for supplying the correct previous APKG.

If the current project has `project_stable_id` but an older previous APKG lacks
one, strict mode does not treat that as a mismatch. It records a limitation for
the missing baseline project id and continues with note-level proof. If a
lockfile is also present and its `project_stable_id` disagrees with current,
the mismatch remains blocking.

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
Phase 3 derivation is exactly `guid.raw-stable-id.v1`: the current stable id is
used as the candidate GUID. Existing packages that already wrote stable ids as
Anki GUIDs therefore have no artificial drift. If a later phase changes the
derivation, the change must use a new `guid_derivation_version` and must not
silently overwrite old GUIDs in update-safe mode.
`UPDATE.GUID_PRESERVED_FROM_PREVIOUS` and
`UPDATE.GUID_PRESERVED_FROM_LOCKFILE` are info diagnostics emitted whenever a
baseline GUID is selected. `UPDATE.GUID_DERIVATION_DRIFT` is the warning
emitted only when `selected_anki_guid != current_guid_candidate` for the same
stable id. That can happen in Phase 3 for older baselines whose Anki GUIDs were
not raw stable ids, and it becomes more common if a later phase introduces a
new `guid_derivation_version`.

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
5. Report field and template ord changes as warnings in Phase 3. Field order is
   part of the import merge surface; template order matters because cards are
   linked by `nid + ord` and scheduling risk belongs to the Phase 4 risk model.
6. Preserve source paths back to product fields and templates.

Field ord comparison matches fields by `field_key`, not by position. If the
same key has a different `ord` between baseline and current, Phase 3 emits
`UPDATE.FIELD_ORD_CHANGED`. A swap of two fields emits two warnings.

Template ord comparison matches templates by `template_key`, not by position.
If the same key has a different `ord` between baseline and current, Phase 3
emits `UPDATE.TEMPLATE_ORD_CHANGED`. A swap of two templates therefore emits
two warnings. Added or removed template keys are reported as template set
changes with `UPDATE.TEMPLATE_SET_CHANGED`, not as ord changes for unrelated
templates.
If a template removal causes remaining templates to shift ord, Phase 3 reports
both facts: one `UPDATE.TEMPLATE_SET_CHANGED` for the removed key and
`UPDATE.TEMPLATE_ORD_CHANGED` for each remaining key whose ord changed. The
diagnostics are intentionally not collapsed because Phase 4 risk policy may
treat set changes and ord shifts differently.

Field and template renames with stable keys and config ids emit
`UPDATE.FIELD_RENAMED` or `UPDATE.TEMPLATE_RENAMED` as warnings. The merge-id
changed diagnostics are reserved for config id drift.
`UPDATE.TEMPLATE_SET_CHANGED` includes `change_kind: added | removed`.
Rename diagnostics are valid only when the corresponding merge/config id stayed
stable; if config id drift is present for the same key, the drift diagnostic is
the blocking signal and the rename warning is secondary or omitted.
If a notetype's `name` changes while `note_type_id` stays the same, Phase 3
emits `UPDATE.NOTETYPE_RENAMED` as a warning. If a notetype id is added or
removed, Phase 3 reports `UPDATE.NOTETYPE_SET_CHANGED` with
`change_kind: added | removed`.

Strict Phase 3 does not need to fully decide scheduling or field-order risk. It
must surface field and template ord changes clearly so Phase 4 can promote them
into risk policy.

## 13. Diagnostics

Phase 3 adds the `UPDATE.*` diagnostic family.

Recommended codes:

```text
UPDATE.BASELINE_APKG_UNREADABLE
UPDATE.BASELINE_LOCKFILE_UNREADABLE
UPDATE.BASELINE_SCHEMA_UNSUPPORTED
UPDATE.BASELINE_CONFLICT_GUID
UPDATE.BASELINE_IGNORED_DISABLED
UPDATE.PROJECT_STABLE_ID_MISSING
UPDATE.PROJECT_STABLE_ID_MISMATCH
UPDATE.WRITER_POLICY_MISMATCH
UPDATE.WRITER_POLICY_REF_INVALID
UPDATE.WRITER_GUID_PLAN_MISMATCH
UPDATE.LOCKFILE_PATH_REQUIRED
UPDATE.LOCKFILE_ABSENT_ENTRIES_HIGH
UPDATE.NORMALIZED_NOTE_ID_MISMATCH
UPDATE.ANKI_GUID_INVALID
UPDATE.STABLE_ID_MISSING_IN_STRICT_MODE
UPDATE.STABLE_ID_DUPLICATE_IN_BASELINE
UPDATE.GUID_DUPLICATE_IN_BASELINE
UPDATE.GUID_DUPLICATE_AT_RECONCILE
UPDATE.STABLE_ID_REMOVED_FROM_CURRENT
UPDATE.GUID_PRESERVED_FROM_PREVIOUS
UPDATE.GUID_PRESERVED_FROM_LOCKFILE
UPDATE.GUID_DERIVATION_DRIFT
UPDATE.GUID_DERIVED_FOR_NEW_NOTE
UPDATE.IDENTITY_PAYLOAD_CHANGED
UPDATE.IDENTITY_PAYLOAD_HASH_DROPPED
UPDATE.IDENTITY_PAYLOAD_HASH_ADDED
UPDATE.NOTETYPE_SET_CHANGED
UPDATE.NOTETYPE_RENAMED
UPDATE.FIELD_MERGE_ID_CHANGED
UPDATE.FIELD_RENAMED
UPDATE.FIELD_ORD_CHANGED
UPDATE.TEMPLATE_SET_CHANGED
UPDATE.TEMPLATE_MERGE_ID_CHANGED
UPDATE.TEMPLATE_RENAMED
UPDATE.TEMPLATE_ORD_CHANGED
UPDATE.BASELINE_IDENTITY_UNRECOVERABLE
UPDATE.LOCKFILE_WRITTEN
UPDATE.LOCKFILE_WRITE_FAILED
UPDATE.NOTE_DATA_METADATA_UNMERGEABLE
```

Severity policy:

1. Errors:
   - unreadable required previous APKG
   - unreadable or invalid required lockfile
   - unsupported required baseline schema version
   - project stable id mismatch across supplied baselines
   - invalid writer policy ref when update-safety analysis or lockfile writing
     is requested
   - requested lockfile write has no lockfile path
   - requested lockfile write failed
   - writer GUID plan does not exactly match normalized notes
   - note metadata cannot be merged into `notes.data` without overwriting
     unrelated data
   - active current note has missing or mismatched normalized note id
   - selected Anki GUID violates the current writer/Anki GUID validator
   - duplicate stable ids in a lockfile or recovered previous APKG identity index
   - duplicate GUIDs in a lockfile or recovered previous APKG identity index
   - duplicate selected GUIDs during reconcile
   - current output note has no resolved stable id in strict mode
   - missing project stable id when reading or writing a lockfile
   - baseline identity unrecoverable for an expected existing note
   - field/template config id drift for the same key
2. Warnings:
   - `UPDATE.BASELINE_CONFLICT_GUID`: previous APKG and lockfile conflict, with
     previous APKG selected
   - project stable id missing in strict compare-only mode, leaving the build
     without project-level baseline proof
   - writer policy mismatch between baseline and current build
   - selected baseline GUID differs from current derivation for the same stable
     id
   - field ord changed
   - template ord changed
   - notetype set changed
   - template set changed
   - notetype renamed while stable notetype id stayed unchanged
   - identity payload changed for the same stable id when both entries have
     comparable inferred payload hashes
   - identity payload hash was present in the baseline but omitted in current
     output because the note moved to explicit stable id identity
   - identity payload hash was absent in the baseline but present in current
     output because the note moved from explicit to inferred identity
   - field or template renamed while stable key/config id stayed unchanged
3. Info:
   - baseline inputs ignored because explicit mode is disabled
   - lockfile entry absent from current build
   - lockfile has more than 10,000 absent entries
   - GUID preserved from previous APKG
   - GUID preserved from lockfile
   - GUID derived for a new note
   - lockfile written

Warnings and info should be summarized in `BuildReport`, but info-level
diagnostics may be omitted from pretty output unless requested.
Machine-readable reports may retain per-note diagnostic occurrences. Human
summaries must aggregate high-volume update diagnostics by code and include at
least the count and up to a small sample of affected stable ids. This applies
to diagnostics such as `UPDATE.IDENTITY_PAYLOAD_CHANGED`,
`UPDATE.GUID_DERIVATION_DRIFT`, and ord-change warnings.

Writer/output invariants block in every mode where that path is exercised.
Examples are requested lockfile path/write failures, invalid writer policy refs,
writer GUID plan mismatch, unmergeable `notes.data`, and invalid selected Anki
GUIDs. Baseline proof errors block only in strict mode; report-only records
them and may still write when the current project and writer output are valid.

`UPDATE.BASELINE_APKG_UNREADABLE` and `UPDATE.BASELINE_LOCKFILE_UNREADABLE`
carry a machine-readable `error_kind`, such as `not_found`, `io`, `parse`, or
`schema`. Schema-version mismatches use `UPDATE.BASELINE_SCHEMA_UNSUPPORTED`
instead. This keeps remediation clear without multiplying top-level diagnostic
codes for every parser and filesystem case.

Strict-mode validation and reconcile should collect all blocking diagnostics
that can be discovered before writer execution, then return one `BuildError`
with the accumulated report. Immediate abort is reserved for unrecoverable IO
or parser failures that prevent further safe inspection.

`UPDATE.IDENTITY_PAYLOAD_HASH_DROPPED` is detected when the same stable id has a
baseline entry with `canonical_payload_hash` and `recipe_id`, but the current
entry has the same stable id with explicit identity provenance and no
`canonical_payload_hash`.
The reverse case emits `UPDATE.IDENTITY_PAYLOAD_HASH_ADDED`: the baseline entry
for the same stable id lacked `canonical_payload_hash`, but the current entry
has comparable inferred identity provenance and a hash. `UPDATE.IDENTITY_PAYLOAD_CHANGED`
is emitted only when both baseline and current entries already have comparable
payload hashes and the full `blake3:<hex>` strings differ.

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

`baseline_sources[]` contains one entry for each requested baseline input and
for each ignored baseline input in disabled mode:

```text
BaselineSourceSummary
  source_kind: previous_apkg | lockfile
  source_ref
  display_path?
  status: loaded | partial | unreadable | ignored_disabled | schema_unsupported
  used_for_reconcile: bool
  limitations[]
  diagnostic_codes[]
```

`display_path` is for humans and may be absolute or relative depending on the
caller. It is not used for stable snapshots. `diagnostic_codes[]` lists the
`UPDATE.*` diagnostics attached to that baseline source; full diagnostic
payloads remain in the report's diagnostics collection.

The existing `BuildError` shape remains valid. If strict update-safety fails,
`BuildError { cause: Diagnostics, report }` carries the full evidence.

## 15. Contract Assets

Phase 3 adds these contract assets:

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
10. `absent_from_current` entry reintroduced and preserving its old GUID
11. deliberate `normalized_note_id` versus `stable_id` corruption rejected by
    `UPDATE.NORMALIZED_NOTE_ID_MISMATCH`
12. field ord warning

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
9. Field and template ord changes emit warnings.
10. Baseline identity recovery degrades when metadata is absent and GUID cannot
    be matched to stable id.
11. `absent_from_current` lockfile entry reintroduced by the current project
    preserves the old GUID.

These tests can use `writer_core::inspect_apkg`, SQLite observation, and
contract fixtures.
SQLite checks should inspect at least `notes.guid`, notetype/model metadata,
field/template config metadata, and `notes.data` for the embedded identity
payload. Inspect-report checks can assert the higher-level contract shape, but
SQLite observation is the backstop that the APKG actually contains the expected
Anki-facing values.

### 16.2 Contract and Golden Layer

Contract tests should validate:

1. identity index schema
2. lockfile schema
3. update-safety summary schema
4. fixture catalog integrity
5. golden JSON stability with canonical key ordering
6. compatibility recovery behavior for old APKGs where `notes.guid` equals
   stable id
7. `guid_derivation_version` remains the exact literal
   `guid.raw-stable-id.v1` for the Phase 3 derivation

Canonical JSON ordering uses lexicographic object keys by Unicode scalar value
after JSON string decoding. Arrays whose order is semantic, such as notetype
templates by `ord`, preserve semantic order. Arrays whose order is not
semantic, such as identity entries in lockfiles, are sorted by stable key
before serialization. Active and `absent_from_current` identity entries are
interleaved in that single stable-key order, not grouped by lifecycle.

The implementation plan should include a diagnostic coverage matrix mapping
each `UPDATE.*` code to at least one unit, contract, CI, or oracle test.

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

Manual desktop evidence is sufficient for Phase 3 sign-off if the automation
harness is not ready. The evidence must be documented in the phase exit
checklist with exact Anki version, scenario inputs, observed note/card counts,
APKG hashes, GUID comparison, and whether duplicates were created. A fully
automated real-Anki harness can be a parallel follow-up.
The oracle evidence must also record whether supported Anki versions preserve
the embedded `notes.data.anki_forge_identity` metadata through import and
export. If they do not, Phase 3 cannot treat `notes.data` as the reliable
embedded metadata carrier for re-exported APKG baselines.
Manual oracle notes should include the Anki platform as well as the Anki
version, because import/export behavior can vary across packaged builds.

## 17. API Surface

Minimum public API surface:

```rust
impl Project {
    pub fn stable_id(self, stable_id: impl Into<String>) -> Self;
}

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

`Project::stable_id(...)` already exists as the Product-level project identity
setter. Phase 3 treats that stored value as `project_stable_id`; it is not
written to Anki as a deck id or note guid. The build reads it before lowering,
threads it into `IdentityIndex`, lockfile validation, and update-safety reports,
and validates it with the rules in Section 8.1 whenever update-safety analysis
or lockfile writing is requested. Repeated calls use the normal builder
last-call-wins behavior. Missing project identity remains allowed for ordinary
builds and degraded compare-only strict builds, but not for lockfile-bearing
strict builds.

`IdentityIndex` and `IdentityLockfile` are public contract JSON artifacts in
Phase 3. They do not need to be stable public Rust API structs yet; library
consumers can read them through the lockfile, inspect/report JSON, or future
contract helpers. Promoting typed Rust accessors is a Phase 4/API ergonomics
decision.

Repeated calls to single-value builder methods use last-call-wins semantics.
For example, `.compare_to(a).compare_to(b)` uses `b`. Phase 3 supports one
previous APKG baseline and one lockfile baseline per build.
The library builder does not emit diagnostics for overwritten builder values
because it does not retain the earlier value. CLI wrappers should reject or
warn on duplicate `--compare-to` or `--identity-lockfile` flags before building
`BuildOptions`.

The default is:

```text
compare_to: None
identity_lockfile: None
write_identity_lockfile: false
update_safety: unset
```

Supplying `compare_to` or `identity_lockfile` with no explicit mode upgrades
the effective mode to `Strict`.

## 18. Rollout Order

Recommended implementation sequence:

1. Finalize contract schemas and semantics docs for identity index, lockfile,
   and embedded note metadata before implementing writer or inspector changes.
2. Complete custom notetype identity recipe resolution so strict builds can
   produce resolved stable ids for all current output notes before update-safety
   proof is enabled.
3. Run an early lightweight real-Anki probe for whether
   `notes.data.anki_forge_identity` survives import/export. If it is stripped,
   keep the documented re-export fallback behavior and treat embedded metadata
   as reliable only for APKGs directly produced by anki-forge.
4. Add current `IdentityIndex` generation from normalized Product builds.
5. Add early validation for note metadata carrier mergeability, then embed note
   identity metadata into newly produced APKGs.
6. Extend APKG inspect to recover identity metadata and compatibility
   `guid == stable_id` cases.
7. Add lockfile read/write.
8. Add reconcile and `GuidResolution`.
9. Add writer GUID guidance and selected GUID writing.
10. Add `BuildOptions` update-safety API.
11. Add `BuildReport` update-safety summary.
12. Add mainline CI tests.
13. Add release/nightly real Anki oracle scenarios.

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
6. Field and template ord changes are visible in diagnostics and summary.
7. Build output can write an updated identity lockfile.
8. Newly produced APKGs embed enough identity metadata for future recovery.
9. Mainline CI proves the update-safety loop without a desktop Anki dependency.
10. Release or nightly oracle coverage, manual or automated, proves at least
    one happy-path update and one dangerous-change scenario against real Anki
    behavior.
11. Phase exit evidence states whether real Anki import/export preserves the
    embedded `notes.data.anki_forge_identity` payload. If it does not, Phase 3
    documents that re-exported APKG baselines rely only on lockfile join or
    `guid == stable_id` compatibility until a later design introduces a proven
    carrier replacement.
12. CLI/API docs explain how to audit and manually prune
    `absent_from_current` lockfile entries until a dedicated pruning command
    exists.
