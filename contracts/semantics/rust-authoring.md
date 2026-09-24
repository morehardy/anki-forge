---
asset_refs:
  - schema/template-bundle.schema.json
  - schema/project-input.schema.json
  - schema/build-report.schema.json
  - schema/comparison-report.schema.json
  - schema/identity-evidence.schema.json
  - errors/error-registry.yaml
---

# Native authoring values and observable outcomes

Project is the sole authoring container. A validated immutable NoteType owns its
fields, templates and assets; Note owns its model and structured Content. Adding
a note automatically collects its model and media. Add and add_asset either succeed
completely or leave the Project unchanged. Reusing a model key requires an identical
definition. Notes and assets can be reused across independent projects.

Field and template keys are distinct Rust types. Keys retain exact author input;
the library does not slug display names, trim, normalize Unicode, or silently
accept names as keys. Completing a schema rejects blank/control keys, duplicate
keys/names, invalid field-expression symbols, ambiguous field key/name collisions,
unknown field references and invalid generation rules. Field symbols cannot use
Anki special expressions or template syntax. Model/template identities are not
field expressions and do not inherit their syntax restrictions.

Plain strings are Text on Basic, Cloze and custom notes. Text is HTML escaped at
output; Cloze delimiters retain their deletion semantics. Explicit Html is emitted
as author markup. Image and Sound content retain owned Media values. Sequence
retains child nodes. Rendering and dependency collection happen within the one
native pipeline; adapters do not render and rewrap content as raw HTML.

Media imports own immutable snapshots. File import streams bytes and calculates a
digest; data beyond the internal 1 MiB memory threshold is kept in owned temporary
storage. Clones and identical byte content share storage. The last owner removes
temporary storage. Import failure cleans up partial storage. The default per-asset
budget is 256 MiB; configurable before reading. Byte inputs take ownership and
require a syntactically valid concrete MIME type; an identified conflicting format
is rejected. There is no transport-driven 64 KiB authoring limit.

Default export filenames derive from content digest and MIME essence. Fixed names
must be portable and contain no paths, controls or reserved filesystem names.
Conflict keys use Unicode NFC and default case folding. Different original names
with equal conflict keys fail even for equal bytes; exact name/content pairs are
idempotent. CSS, font, script and handwritten HTML assets are explicitly declared
and remain exported even without a recognized static reference.

IO decodes PNG/JPEG/GIF/WebP/BMP completely with an independent 256 MiB pixel budget.
EXIF orientation is applied before coordinate checks; transformed images become
owned PNG snapshots. Rectangles must be finite, positive and within displayed
bounds. Mask keys are explicit and unique. HideAllGuessOne is the default; the
alternative HideOneGuessOne is explicit. Generated image/occlusion fields cannot
be overwritten; header/back_extra/comments use ordinary content assignment.

BuildOptions chooses a persistent destination or an owned temporary artifact.
Successful builds return BuildOutput with a guaranteed ApkgArtifact. BuildReport
contains observations only; its clone/JSON snapshot cannot retain a file or
represent success. BuildOutput and BuildError supply the actual result snapshot.
Persisting copies atomically beside the destination before replacement and reports
whether publication happened and durability was confirmed. A late sync failure
may return failure with a published path; it must never claim nothing was written.
JSON report saving is separate and never changes an already-completed build result.

Inspection budgets apply independently to baseline and candidate: archive bytes,
entry count, central directory, individual/total ZIP expansion, metadata, media
map, decoded collection, individual media, aggregate decoded content, zstd window
and identity evidence. The identity entry defaults to 64 MiB. Resource errors retain
budget, entry, limit and observed values and cannot be ignored by update policy.
Public errors expose stable kind/code and real source chains; binding adapters
preserve these facts, observation snapshots and publication stages.
