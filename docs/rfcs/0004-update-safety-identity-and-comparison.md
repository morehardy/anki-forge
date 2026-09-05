# RFC 0004: Preserve model identity and close update comparison gaps

Status: implemented; PR review follow-up included.

## Problem and reproduced evidence

APKG model IDs were assigned from declaration positions and omitted from both
baseline recovery and lockfiles. Reordering or inserting a note type changed an
existing model ID. Lockfile-only strict builds also accepted field removal, and
diff ignored field metadata, browser templates, and template target decks while
claiming complete comparison.

The regression seam is a real `Project::build` followed by SQLite inspection of
the APKG, plus writer inspect/diff reports. The initial regression run reproduced
all seven failures in `update_safety_blind_spots_tests` and `writer_core_diff_tests`.

## Design

1. One internal writer identity Module derives new model IDs from a framed pair
   of normalized document ID and logical note type ID, using the versioned BLAKE3
   context `anki-forge.notetype-id.v1`. IDs are positive and fit in 53 bits. They
   do not depend on declaration order, field content, or display name.
2. Before writing, update safety selects the previous APKG's actual model ID,
   then the lockfile's ID, then the new deterministic ID. It validates positive
   IDs and uniqueness, preserves mappings for absent note types in rewritten
   lockfiles, and passes the selected map through staging to the writer.
3. Legacy lockfiles with null model IDs remain readable. Strict mode requires a
   previous APKG to recover missing IDs; it must not claim to preserve an identity
   it cannot observe. Report-only mode diagnoses the limitation and derives a new
   ID. A readable APKG takes precedence over a stale lockfile mapping.
4. Field matching remains based on logical field keys, with config-ID continuity
   checked separately. Added fields emit a warning;
   removed fields emit an error in strict mode and a high import-risk finding.
   Report-only mode downgrades the diagnostic, not the risk evidence.
5. Inspect owns the complete observation domain list. Diff consumes that list,
   includes all nine domains, and carries every missing domain into
   `uncompared_domains`. Incomplete comparisons cannot report an unqualified
   no-change result. Actual model IDs are part of notetype observations.

The supported Rust facade and note GUID derivation do not change. Numeric deck
ID allocation, output path collision handling, and archive resource limits are
separate work items.

## Compatibility and rollout

The integrated bundle advances to 0.5.0 as a pre-1.0 compatibility change. New unbaselined
builds use deterministic model IDs. Existing installations must use `compare_to`
or a lockfile with populated model IDs on the first build after upgrading. The
selected IDs are retained in subsequent lockfiles. Old APKG numeric IDs are
preserved rather than rewritten to the new derivation.

Identity schemas keep their existing wire versions: `anki_model_id` was already a nullable
identity-index field, and the three observation arrays already existed. New
diagnostics are registered, normative semantics are updated, and fixture
fingerprints plus the embedded bundle are regenerated from the changed writer.

## Verification

- Order/insertion invariance, APKG and lockfile model ID recovery, legacy ID
  preservation, missing/duplicate/conflicting identity evidence, and reintroduced
  note types.
- Lockfile-only field add/remove behavior in strict and report-only modes.
- Added/modified/removed observations in all extended domains, partial evidence,
  and staging/APKG equivalence.
- Workspace tests, contract verify/summary/package, embedded-bundle byte equality,
  formatting and clippy. Real Anki import verification where the local oracle is
  available; its outcome is recorded separately from Rust verification.

## Local verification evidence (2026-09-05)

- `cargo test --workspace --all-features --no-fail-fast`: passed (existing ignored
  tests were not enabled).
- `cargo test -p anki_forge --features internal-tools --test
  update_safety_blind_spots_tests --test writer_core_diff_tests`: 17 passed,
  including 14 new regressions. This focused rerun also covers the final staging
  validation change: legacy missing assignments recover positional IDs, while an
  explicit empty assignment map for a nonempty model set is rejected.
- `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets
  --all-features -- -D warnings`: passed.
- `contract_tools verify`, `summary`, and `package` with `contracts/manifest.yaml`:
  passed for bundle 0.4.0. The generated archive is byte-identical to the embedded
  archive. `ANKI_FORGE_ALLOW_DIRTY_PACKAGE=1 scripts/check_rust_crate_payload.sh`:
  passed. No commit, tag, or publication was performed.

### Real Anki identity verification and a separate remaining defect

Anki 25.9.2 was exercised through its installed Python backend, using isolated
temporary collections only. First import: `[alpha, beta]`, two notes and cards.
Second package: `[gamma, beta, alpha]`, with changed answer text for the original
notes and one new note. The original cards were assigned a known review state
before reimport. Both `merge_notetypes=false` and `true` were tested.

In both modes, the two original model IDs, note GUIDs, note row IDs, card row IDs,
intervals, repetitions, due dates, ease factors, lapses, card types, and queues
were preserved. Exactly one note/card and one notetype were added, with no
conflicting notes. This verifies the identity repair against real import behavior.

The same experiment uncovered an independent content-update problem:
`writer_core/apkg.rs::note_storage_values` defaults missing note `mtime_secs` to
`1`, while Product normalization does not populate it. Both original notes were
therefore logged as duplicates and their answer text was not updated, even with
Anki's Always update option. As a control, setting only the temporary collection's
existing note modification times to `2` made both notes update successfully while
all identity and review-state checks still passed, in both merge modes.

This work does **not** claim general content-update safety. The modification-time
defect is left unchanged and needs a separate policy for persisted note revision
times, unchanged-note handling, and deterministic builds. It should be the next
update-safety repair before claiming end-to-end update readiness.

Follow-up: RFC 0005 implements baseline-driven note revisions and records the
successful replay of this content-update reproduction. The limitation above
describes the result of RFC 0004 in isolation, before that follow-up repair.

### PR 36 review follow-up

The new numeric model IDs and note revision evidence must not share the old
observation model version. Bundle 0.5.0 now emits `phase3-inspect-v2`. A saved v1
report compared with freshly inspected evidence is partial, with an explicit
version-mismatch limitation. Node/Python accept v1, v2, and mixed-version diff
reports while rejecting unknown versions. A path-based runtime regression first
reproduced the falsely complete comparison, then passed with the new version.

Report-only fallback identities must not become authoritative through lockfile
rewrites. Missing model IDs/revisions and unreadable requested baselines suppress
the entire lockfile write, retaining the original bytes and emitting an explicit
skipped-write warning. Missing or invalid evidence therefore still requires
recovery on a subsequent strict build. A rejected requested lockfile also yields
high import risk, preserving its source and diagnostic evidence for `fail_on`.
Regression coverage includes both missing-evidence cases, invalid IDs/revisions,
policy-blocked output preservation, unreadable APKGs, valid APKG migration, and
normal first-release/verified report-only writes.
