# RFC 0004: Deterministic baseline-driven note revisions

Status: implemented; PR review follow-up included.

## Problem and evidence

Product normalization leaves note `mtime_secs` unset, so writing uses `1` for
every export. Anki's IfNewer condition requires a greater time; even Always
skips equal times. On Anki 25.9.2, an answer-only change is logged as a duplicate.
Changing only the target note's time in a temporary collection makes Always
apply the update. Both merge-notetypes settings reproduce this result.

Four new Project-level regressions reproduce unchanged output times, absent
full-content revision evidence in lockfiles, and missing strict/report-only
handling for legacy revision evidence.

## Decision

Keep note identity separate from note content revision. Add optional `revision`
evidence to each identity entry: `{content_hash, mtime_secs}`. The versioned
`note-content.v1:blake3:` hash covers logical notetype ID, all normalized field
names and values, and a sorted, deduplicated tag set. It does not reuse the
identity-recipe hash, which may omit answer fields. It excludes GUID, time,
display deck, and source/evidence metadata.

One internal Module calculates content evidence from normalized notes for both
current builds and inspection. APKG evidence is recomputed from actual stored
fields, tags, and `notes.mod`, not trusted from embedded metadata or the lockfile.

For each stable note identity, select the same source precedence as GUIDs:
previous APKG, then lockfile. With a baseline, retain its time if content is equal;
otherwise use checked `previous_time + 1`. A revert is another revision and must
advance. New notes retain the writer's deterministic initial time `1`. These are
logical revision times in Anki's timestamp slot, not a claim of wall-clock export
time. Existing real baseline timestamps are preserved and advanced numerically.
The selected time is placed into normalized IR before staging/writing and into
the lockfile. Preserve revision history for temporarily absent notes.

An old lockfile without revision evidence is readable but insufficient for a
strict update of that note: require its previous APKG. Report-only warns at high
risk and can continue without an update guarantee. Invalid revision evidence or
time overflow must not silently wrap or produce a falsely safe update. Conflicting
known APKG/lockfile revisions are diagnosed; actual APKG evidence takes precedence.

No public build options or clocks are added. Baseline-free and explicitly disabled
builds remain deterministic first-release exports; updating existing notes requires
`compare_to` or a maintained identity lockfile. Users must supply the latest
distributed baseline. Local Anki note edits newer than that baseline remain
subject to Anki's import policy; IfNewer is not a forced-overwrite promise.

## Alternatives rejected

- Wall clock: makes repeated builds different and still collides within one second.
- Content hash as time: not monotonic, so IfNewer loses updates and reverts.
- Identity-recipe hash: not full content; answer-only and tag-only edits are missed.
- Unconditional increments: rewrites unchanged notes and makes repeat builds drift.

## Compatibility and validation

Advance the bundle to 0.5.0; keep identity wire versions and make revision optional
for legacy reads. Document strict migration, register diagnostics, update inspect
goldens, regenerate the embedded bundle, and keep the Rust supported facade intact.

Verify answer/tag edits, unchanged notes, reverts, same-baseline reproducibility,
legacy migration, missing/conflicting/invalid evidence, overflow, absent-note
history, and inspection of actual APKG values. Re-run the original real Anki
import with IfNewer and Always, with/without notetype merging, and require updated
content alongside preserved GUIDs, note/card IDs, and review state.

## Verification evidence (2026-09-05)

- `cargo test --workspace --all-features --no-fail-fast` passed; the final added
  actual-storage and schema cases also passed in focused reruns. Ordinary ignored
  tests were not enabled, except the explicitly invoked Anki oracle below.
- `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets
  --all-features -- -D warnings`, and `git diff --check` passed. No commit, tag,
  publication, or change to a user Anki collection was performed.
- The initial four regressions failed twice with the expected symptoms before the
  repair: changed note time remained `1`, lockfile content evidence was absent,
  and missing evidence was neither blocked nor reported.
- `update_safety_note_revision_tests`: ten deterministic tests passed, including
  actual APKG field/time recovery, tag normalization, time overflow, stale lockfile
  precedence, strict migration, report-only risk, and absent-note history.
- The schema regression accepts legacy entries and supported revision evidence,
  and rejects malformed/missing hashes or invalid times.
- The opt-in real Anki test passed on installed Anki 25.9.2, in isolated temporary
  collections, for all four combinations of IfNewer/Always and notetype merging
  off/on. Exactly one changed note updated and one unchanged note was skipped;
  reimport applied zero further updates. GUIDs, model/note/card IDs and review
  state remained unchanged. No real user collection was opened.
- The original RFC 0003 reproduction was also rerun without modifying the target
  collection's note times: `[alpha, beta]` to `[gamma, beta, alpha]`. Both merge
  modes updated the two changed answers, added exactly one note/type, and retained
  both original identities and review states with zero conflicts.
- Contract `verify`, `summary`, and `package` passed for 0.5.0. The generated
  archive matches the embedded archive byte-for-byte; the Rust crate payload
  inventory check passed.

To rerun the real import oracle, set `ANKI_FORGE_ANKI_PYTHON` to the Python
executable of an installed Anki environment, then run:

```sh
cargo test -p anki_forge --features internal-tools \
  --test update_safety_note_revision_tests \
  real_anki_applies_content_updates_without_changing_identity_or_review_state \
  -- --ignored --nocapture
```

The oracle lives in `anki_forge/tests/support/note_revision_import_oracle.py` and
is intentionally opt-in: ordinary Rust CI does not require an Anki installation.

## PR 36 review follow-up

Five failing-first Rust cases reproduced the review findings: falsely complete
cross-version report comparison, two report-only rewrites that manufactured
missing baseline evidence, and two rejected-lockfile cases with no import risk.
Report-only now suppresses unverified lockfile writes and leaves the original
bytes intact; strict builds still require recovery. Invalid requested lockfiles
remain high risk, so policy rejection preserves published files and lockfiles.
Valid APKG migration and normal report-only creation/updates remain supported.
The observation model advances to v2, with legacy/mixed-version binding coverage;
see RFC 0003's review follow-up for the version boundary.
