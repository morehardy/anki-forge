---
asset_refs:
  - schema/identity-index.schema.json
  - schema/identity-lockfile.schema.json
  - schema/update-safety-summary.schema.json
---

# Identity Update Safety Semantics

Phase 3 update safety is built around `identity-index-v1`, `identity-lockfile-v1`, and `identity-note-v1`.

The only Phase 3 GUID derivation version is `guid.raw-stable-id.v1`. It sets `current_guid_candidate` to the resolved Product `stable_id` with no truncation or hashing. Changing this rule requires a new `guid_derivation_version`.

`IdentityIndex.source_ref` uses stable logical values:

- `current`
- `baseline.previous_apkg.primary`
- `baseline.identity_lockfile.primary`

Lockfile JSON must use lexicographic object-key ordering by Unicode scalar value after JSON string decoding. Arrays with semantic order preserve that order. Identity entries are sorted by `stable_id`.

Limitations describe source evidence and diagnostics describe build events. Implementations must derive overlapping values from one internal classifier pass.

## Numeric notetype identity (bundle 0.5.0)

The logical `note_type_id` and numeric `anki_model_id` are distinct identities.
Model assignments are selected before writing, persisted in staging, written to
Anki's notetypes and note references, recovered by inspection, and recorded in
the identity lockfile. Selection precedence is previous APKG, identity lockfile,
then deterministic derivation for a new logical type. Conflicting known baseline
IDs produce `UPDATE.NOTETYPE_MODEL_ID_CONFLICT`; the APKG is artifact truth.

New IDs use BLAKE3 derive-key context `anki-forge.notetype-id.v1` over the UTF-8
document ID followed by the logical notetype ID. Each string is prefixed with its
byte length as an unsigned 64-bit big-endian integer. Interpret the first eight
digest bytes as big-endian, mask to 53 bits, and replace zero with one. Names and
declaration order are not inputs. Baseline IDs are preserved, not rehashed.
Assignments must be positive and unique across current and reserved absent types;
collisions block writing even in report-only mode. Rewritten lockfiles retain
absent notetype identities so removing and later reintroducing a type does not
allocate another ID. The note GUID derivation rule is unchanged.

Legacy lockfiles with `anki_model_id: null` remain readable. For a matching type,
strict mode requires a previous APKG to recover the actual ID, otherwise it stops
with `UPDATE.NOTETYPE_MODEL_ID_MISSING` before writing output or lockfile.
Report-only warns and may use a derived ID, but cannot claim identity preservation.
Old staging manifests without model assignments use their original positional
IDs; new manifests always carry an explicit assignment map.

Field membership is compared by field key even with a lockfile-only baseline.
`UPDATE.FIELD_REMOVED` blocks strict updates and yields high import risk;
`UPDATE.FIELD_ADDED` warns and yields medium risk. Report-only downgrades the
removal diagnostic to a warning without lowering its risk. Existing field rename,
order, and config-ID checks still apply.

When diagnostic and semantic-diff evidence identify the same removed field,
emit one `RISK.FIELD_REMOVED_OR_RENAMED` finding with both evidence references
and the higher risk level. Different field selectors remain separate findings;
diff-only field removal retains its existing medium-risk classification.

## Note content revision (bundle 0.5.0)

An identity entry may carry `revision: {content_hash, mtime_secs}`. This is distinct
from `canonical_payload_hash`, which may cover only the fields used for identity.
The full-content digest is `note-content.v1:blake3:` followed by the lowercase
64-character BLAKE3 digest of canonical JSON with keys `notetype_id`, `fields`,
and `tags`. Fields retain every normalized field name/value. Tags are split on
the writer's ASCII space separator, with empty parts removed, deduplicated and
sorted. GUID, source metadata, deck names, and modification time are not hashed.

APKG inspection recomputes revision evidence from actual stored note fields,
tags, logical notetype ID, and `notes.mod`. Embedded metadata is not revision
truth. Per stable identity, select previous APKG evidence before lockfile evidence.
Conflicting known revisions emit `UPDATE.NOTE_REVISION_CONFLICT`.

For equal content, preserve baseline `mtime_secs`. For changed content, use checked
`baseline.mtime_secs + 1`; even a content revert advances. A new note starts at the
deterministic initial value `1`. These logical revision times occupy Anki's time
field but do not represent wall-clock build time. No system clock is consulted.
The chosen revision is written to normalized staging, actual APKG notes, and the
identity lockfile. Temporarily absent notes retain revision history in lockfiles.

Legacy entries without revision remain readable. Strict updates of those notes
require the previous APKG; otherwise `UPDATE.NOTE_REVISION_MISSING` blocks writing.
Report-only downgrades the diagnostic to a warning but retains high risk
`RISK.NOTE_UPDATE_UNVERIFIED`. Overflow blocks both modes, and invalid evidence is
rejected by baseline readers. A non-authoritative report-only build cannot promise
that Anki will apply the update.

Same current content and same baseline produce the same selected times. Consumers
must use their latest distributed APKG, or persist the updated identity lockfile
after each release (`write_identity_lockfile(true)`). Baseline-free/disabled builds
remain deterministic first-release exports, not update-safe replacements. Anki's
own import conditions still govern newer local edits in the receiving collection.

## Report-only persistence and rejected baselines

Report-only permission to emit a best-effort APKG is not permission to turn
unverified baseline evidence into a trusted lockfile. If a requested APKG or
lockfile baseline is unreadable, or an existing note/type still lacks revision
or model-ID evidence, skip the entire requested lockfile write. Preserve any
existing lockfile byte-for-byte, emit `UPDATE.LOCKFILE_WRITE_SKIPPED_UNVERIFIED`,
and leave `update_safety.lockfile_written` false. A later strict build must still
require recovery. A readable previous APKG can supply missing legacy evidence;
normal first-release lockfile creation and verified report-only updates still write.

`UPDATE.BASELINE_LOCKFILE_UNREADABLE` and `UPDATE.BASELINE_APKG_UNREADABLE`
always contribute high risk
`RISK.BASELINE_UNAVAILABLE`, even when report-only downgrades its diagnostic to
a warning. This includes invalid model IDs/revisions and parse/read failures.
Identity rejection remains high risk even if raw artifact comparison is complete.
The finding retains the baseline source and diagnostic evidence. An APKG rejection
and an unavailable comparison describe one APKG baseline risk, not two findings.
`fail_on(High)`
blocks publication without changing existing output or lockfile bytes.
