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
Adding a note rejects the raw U+001F Anki field separator in any Text or Html
node, including nested sequences and Cloze/IO fields, with InvalidContent and
NOTE.FIELD_CONTENT_INVALID before changing the Project. Content construction
remains infallible. Other text controls, HTML character references and literal
backslash escapes are not silently rewritten or rejected by this field-boundary
check.

Media imports own immutable snapshots. File import streams bytes and calculates a
digest; data beyond the internal 1 MiB memory threshold is kept in owned temporary
storage. Clones and identical byte content share storage. The last owner removes
temporary storage. Import failure cleans up partial storage. The default per-asset
budget is 256 MiB; configurable before reading. Byte inputs take ownership and
require a syntactically valid concrete MIME type; an identified conflicting format
is rejected. There is no transport-driven 64 KiB authoring limit.

Default export filenames derive from content digest and MIME essence. Fixed names
must be portable and contain no paths, controls or reserved filesystem names.
The runtime also enforces a maximum of 255 UTF-8 bytes per filename. Draft-07
input schemas check filename syntax and at most 255 Unicode characters;
multi-byte names still require the runtime byte-budget check. Schema validation
is the structural stage, not a guarantee that media import or project loading
will succeed. Valid Unicode filenames within the byte budget remain supported.
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
Snapshot artifact/publication paths serialize as Unicode strings when possible.
Other native paths use `{ "encoding": "unix_bytes", "bytes": [...] }` on Unix
or `{ "encoding": "windows_wide", "units": [...] }` on Windows. Each byte is an
integer from 0 to 255; each UTF-16 unit is an integer from 0 to 65535. The governed
build-report schema accepts both native encodings on all platforms; reconstructing
a filesystem path requires the matching platform. No lossy replacement or artifact
ownership is implied by a path snapshot.

Inspection budgets apply independently to baseline and candidate: archive bytes,
entry count, central directory, individual/total ZIP expansion, metadata, media
map, decoded collection, individual media, aggregate decoded content, zstd window
and identity evidence. The identity entry defaults to 64 MiB. Resource errors retain
budget, entry, limit and observed values and cannot be ignored by update policy.
Public errors expose stable kind/code and real source chains; binding adapters
preserve these facts, observation snapshots and publication stages.

Project has no display title. Its namespace is stable identity; default_deck and
per-note deck select destinations. Model, field and template display names remain.
Tool recipes require ankiforge-project-v2 and reject the top-level name member,
including null. Recipe v1 is rejected, even without a name member. Package identity
remains ankiforge-identity-v1 and build/report/comparison DTO versions are unchanged.

Image and Sound constructors remain infallible. Project::add validates the retained
import MIME: Image requires image/*; Sound accepts audio/* or video/* (Anki's sound
reference also plays video). Other categories, including application/octet-stream,
fail with InvalidMediaUsage / NOTE.MEDIA_USAGE_INVALID. A syntactically valid MIME
declaration for unidentified bytes establishes a category, not decodability or
playability. Explicit assets and raw HTML do not acquire typed-content checks.
Export names do not change retained MIME. Build-time staging still sniffs content
and checks extension-derived declarations, so PNG named wrong.mp3 passes image
addition but fails build with MEDIA.DECLARED_MIME_MISMATCH.

AddError exposes owned AddContext and optional AddDetail. Note additions always
retain the attempted note key (even invalid) and model key; standalone assets and
project default-deck build checks have neither. Targets distinguish note keys,
whole notes, models, fields, original tag positions, explicit/inherited decks,
model assets, occlusion images, explicit assets and project default decks.
Field paths are null for the field itself, [] for its root content, and zero-based
indices into the original nested sequences otherwise. Separator errors carry a
half-open UTF-8 byte range in the original Text/Html leaf. Conflict details retain
model keys/names or original media names and collision kind; media misuse retains
requested usage, filename and import MIME. Errors retain no media or artifact owner.
Adapters preserve these facts in AddError.details and nested source details, using
camelCase in Node and snake_case in Python, with explicit snake_case discriminants.
Node details are deeply frozen. New Node wrappers require native protocol 6;
Python wrappers require embedded contract 2.1.0. BUILD.NAME_INVALID is deprecated.

## Prepared publication

Project.prepare_publication (Node preparePublication) accepts BuildOptions and
returns a single-use owner of a fully inspected private candidate. Preparation
runs the same bounded baseline/candidate inspection, identity verification and
complete comparison as build/compare. A blocked policy is readable in the
BuildReport; publish consumes the owner and enforces that stored decision.
Reports and JSON snapshots own no files and cannot reconstruct a candidate.
Prepare never publishes the destination. Drop or SDK close removes unpublished
storage. Publish consumes the owner on success, policy refusal or I/O failure;
SDK repeats raise PreparedPublicationStateError / BUILD.PREPARED_UNAVAILABLE with
reason closed or consumed before scheduling work. Close is idempotent and does
not cancel a publication that has taken ownership. Python inherited owners may
not publish or close in a forked child. Node cleanup uses its existing workers.

Paths retain their invocation-time working-directory meaning, including native
symlink/parent traversal. Publication rejects aliases of the original baseline
identity, its resolved location and its current anchored path. The comparison
remains preparation-time evidence; baseline mutation is unsupported. Publish
retains atomic replacement, source error chains and truthful publication and
durability facts. Report duration includes preparation plus publication work,
excluding review time. Existing single-call build duration includes all work.

Node tasks and clones share immutable Project versions. add, addAsset and
defaultDeck detach before mutation when another owner is live; failed additions
remain atomic. This defers copying to the first edit while a snapshot is live,
not a guarantee of faster authoring, lower worst latency or lower RSS. Mutable
JavaScript bytes are still copied before task submission. Python authoring
ownership is unchanged. Node protocol 6 and Python binding protocol 1 are
required; old same-version native extensions are rejected.

Native APKG generation validates staging data and media but omits the unused
manifest JSON, SHA-1 and file. Internal Tools Interface staging calls keep their
real manifest bytes, fingerprints and references. Omitted manifest references
are absent, not fabricated. Manifest-only serialization/write failures therefore
no longer occur in native builds. All required I/O, integrity and final APKG
inspection remain; embedded identity evidence is unaffected.

Large bytes imports validate each call's budget and MIME before hashing and
looking up live immutable storage. A hit obtains a strong owner before leaving
the cache lock and before creating a redundant file; metadata remains per Media.
Miss registration still arbitrates concurrent duplicates. Weak ownership, PID
and inherited-lock protections, memory thresholds/budgets and last-owner cleanup
are unchanged. Hit-only redundant I/O failures no longer occur. No permanent
cache, shared mutable bytes, in-flight waiting or physical-I/O claim is added.
