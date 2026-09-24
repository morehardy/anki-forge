---
asset_refs:
  - schema/identity-evidence.schema.json
  - schema/comparison-report.schema.json
  - schema/build-report.schema.json
  - errors/error-registry.yaml
---

# Native publication identity and update semantics

Native Rust, Node, Python and product CLI authoring use one Project pipeline.
A project requires an explicit namespace; every note requires an explicit stable
key. Body text, display names and source filenames do not supply identity.
The update baseline is the original distributed APKG, including its complete
`ankiforge-identity.json` evidence entry. An Anki re-export does not retain this
entry and cannot serve as a baseline. There is no lockfile alternative, identity
inference mode, report-only evidence bypass, or legacy input migration fallback.

## Evidence and verification

The `ankiforge-identity-v1` envelope contains the namespace; every active and
retired note/model; model IDs; field/template config IDs, historical slots and
current physical ordinals; note GUIDs; content revision hashes and modification
times; card key/ordinal mappings; and IO mask keys, retirement flags and ordinal
high water. Active normal field/template physical ordinals are contiguous in
historical slot order. Removing a symbol does not free its permanent slot.
Restoring a key reuses its identity and compares against its retained history.

The envelope binds the actual decoded collection using BLAKE3 and each actual
decoded media file using length and SHA-1. Its identity checksum is BLAKE3 of
compact JSON with recursively sorted object keys. Reading a baseline or candidate
checks format, checksums, namespace, actual SQLite IDs/kinds/ordinals/revisions,
active cardinalities, semantic card mappings, masks and media descriptors.
Missing, malformed, inconsistent or incomplete evidence is a hard failure.
Duplicate ZIP entry names are rejected before any member is trusted. Checksums
establish consistency, not authenticity or authorization of the publisher.

New IDs use BLAKE3 derive-key context `ankiforge.package-identity.v1` and UTF-8
components prefixed with unsigned 64-bit big-endian byte lengths. A new note's
GUID takes the first 32 hexadecimal characters of the namespace/note/key digest.
Numeric model/field/template IDs use namespace/kind/model-key/symbol-key; the
first 13 hexadecimal characters produce a positive 52-bit integer, replacing
zero with one. Existing baseline identities take precedence. Colliding assigned
IDs fail validation. A note key cannot change its model, and a model key cannot
change between normal and Cloze kinds.

Card mappings use `template:<key>`, `cloze:<number>` and `mask:<key>`. Cloze
numbers must be 1–500. IO masks allocate zero-based ordinals from a persistent
high water of at most 500. Movement and declaration order do not change a mask's
ordinal. Retired ordinals are never assigned to a different mask key; restoring
the original key reuses its ordinal. Exhaustion requires splitting the note.

## Revisions and actual Anki imports

Initial model/note modification times use current Unix seconds. Unchanged
normalized content preserves its baseline hash and time; changed content advances
to `max(now, baseline + 1)`, with checked overflow. Note content hashes cover
normalized model reference, deck, fields, tags and card mapping. Model hashes
cover normalized model content with established IDs/ordinals. These revisions
must be written into the actual Anki SQLite rows, not only the envelope.

Anki finds notes by GUID and existing cards by its target note ID and template
ordinal. Scheduling preservation therefore requires actual import verification;
matching generated package IDs alone is insufficient. Field/template config IDs
support model merging. Anki's default import rejects incompatible schemas; merge
mode unions schemas and may retain target-only fields and templates. APKG import
does not delete omitted learner notes, cards or media. A structural or sort-field
change can advance target note times before content import; publisher timestamps
cannot guarantee overwriting newer local edits under `IfNewer` settings.
Comparison findings must expose these limitations, even when a risk is accepted.

## Comparison and publication policy

`compare` returns a complete comparison with original findings, evidence and a
separate policy decision. Blocking risk is a successful comparison. Hard failures
return structured errors with completed observations retained. A verified baseline's
counts survive subsequent candidate generation or inspection failure; an absent
count observation is distinct from zero observed entities.

`build` applies the same comparison before publication. High and Critical findings
block by default. Typed allowances accept specific registered risk categories and
never remove findings or downgrade their original severity. Unmatched allowances
produce `UPDATE.UNMATCHED_ALLOWANCE`. Unknown codes, wildcard allowances and hard
error codes are invalid. Explicit policy on Create fails regardless of setter order.

Field/template additions and removals, sort-field changes and omitted notes/cards
are High risks. IO additions are Medium, removals High. Ordinary content changes
are Low; model rendering changes and replacement media bytes are Medium. Exact
before/after evidence accompanies each finding. Restored entities compare with
retained history, so omitting then restoring an entity cannot bypass structural
risk checks. Equal total card counts do not hide card replacement.
