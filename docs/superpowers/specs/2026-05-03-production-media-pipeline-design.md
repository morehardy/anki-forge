# Production Media Pipeline Design

## Goal

Upgrade Anki Forge media handling from inline JSON payloads to a production
media pipeline that is stable, reproducible, memory-conscious, and suitable for
large media libraries.

The design replaces normalized `data_base64` media with a two-layer model:

- authoring input may reference local paths or small inline bytes
- normalized output references content-addressed media objects and export
  filename bindings

The writer consumes normalized metadata plus a content-addressed media store. It
does not perform high-level media semantics.

## Current Context

The current contract has `AuthoringMedia` and `NormalizedMedia` shaped as
`filename`, `mime`, and `data_base64`. Staging decodes base64 payloads into
`staging/media`, and APKG materialization reads those staged files, computes
SHA-1, zstd-compresses payloads, and writes numbered APKG media entries.

Existing behavior already includes:

- basic deck-layer registration from file or bytes
- same-name same-content reuse in the deck API
- same-name different-content rejection in the deck API
- field media reference scanning for `[sound:]`, HTML `src`, and HTML `data`
- missing media diagnostics controlled by unresolved asset behavior
- APKG v3 media map and zstd media payload writing

The production gap is that the normalized IR still carries media as inline
base64 payloads, so large media is expensive to serialize, duplicate filenames
can conflict in the writer lane, and reports cannot fully describe unused,
missing, unsafe, deduplicated, or policy-controlled media states.

## Chosen Approach

Use a dual-layer media model.

Authoring input accepts friendly source forms. Normalization ingests those
sources into a content-addressed store. Normalized IR only contains stable media
metadata:

- `MediaObject` records content identity
- `MediaBinding` records APKG export filename binding
- `MediaReference` records static references discovered in notes/templates

The writer reads original bytes from the content-addressed store, checks object
integrity and normalized invariants, materializes staging artifacts, and writes
APKG output.

This approach was chosen over direct local-path normalized IR because normalized
artifacts need to be reproducible across machines, CI, and repeated builds. It
was chosen over a broad asset compiler because the first production need is
correct media ingestion, validation, dedupe, and APKG output rather than remote
fetching, transforms, or automatic rewrite machinery.

## Scope

Included:

- Replace normalized `data_base64` media with content-addressed object metadata.
- Add authoring media sources for relative paths and small inline bytes.
- Add Rust ingest helpers for file, bytes, and reader-style inputs.
- Add explicit normalization options for path base and media store location.
- Add object/binding/reference media diagnostics and report data.
- Make writer read media from CAS as its source of truth.
- Keep `staging/media` as a reproducible inspect artifact.
- Update schema, semantic docs, fixtures, Rust model, normalize, writer,
  inspect, and focused high-level Rust APIs.

Excluded:

- Backward compatibility for normalized or writer-facing `data_base64`.
- Node/Python stream APIs in this pass.
- Automatic filename rewriting for conflicts.
- Remote URL downloading.
- Media transcoding.
- APKG payload-level reuse for repeated content.
- A full asset compiler for fonts/templates/assets.

Test helpers may keep convenience constructors for small in-memory media, but
normalized IR and writer code must not retain a `data_base64` compatibility
surface.

## Authoring Input

Authoring media becomes a declaration of source plus desired export filename:

```json
{
  "id": "media:heart",
  "desired_filename": "heart.png",
  "source": {
    "kind": "path",
    "path": "assets/heart.png"
  },
  "declared_mime": "image/png"
}
```

`desired_filename` must be a bare filename:

- non-empty
- not absolute
- no `/` or `\`
- no `.` or `..` path components

`source.path` is resolved against normalization options, not against an implicit
process working directory. JSON paths must be relative. The normalizer resolves
`base_dir.join(source.path)`, canonicalizes the result, and rejects any path
that escapes `base_dir`, including symlink escapes.

Normalization options:

```rust
NormalizeOptions {
    base_dir: PathBuf,
    media_store_dir: PathBuf,
    inline_bytes_max: usize,
    max_media_object_bytes: Option<u64>,
    max_total_media_bytes: Option<u64>
}
```

CLI and fixture flows may default `base_dir` to the input file's parent
directory. Library callers should pass it explicitly.

`inline_bytes` may exist in authoring input for small payloads. In JSON
authoring documents, the byte payload is encoded as base64 under an
authoring-only `inline_bytes` source variant because JSON has no native byte
type. It is capped by `inline_bytes_max`. Normalization must ingest inline bytes
into CAS. The normalized IR must never contain `inline_bytes` or `data_base64`.

Source information may be retained in diagnostics or a debug/provenance sidecar.
It is not part of normalized build semantics. Once media is ingested into CAS,
the writer must not care whether the original input came from a path, bytes, or
a reader.

## Media Store

The media store is content-addressed by BLAKE3 and stores original bytes only.
It does not store APKG-compressed or zstd-compressed bytes.

Recommended object path:

```text
objects/blake3/<first-two-hex>/<next-two-hex>/<full-blake3-hex>
```

During ingestion, normalization computes:

- BLAKE3 for CAS identity
- SHA-1 for APKG media map semantics
- byte length
- effective MIME

The CAS layer deduplicates identical bytes regardless of filename. Multiple
bindings may point at the same object.

## Normalized Media Model

Normalized IR has three media arrays.

`media_objects` describe content:

```json
{
  "id": "obj:blake3:<hash>",
  "object_ref": "media://blake3/<hash>",
  "blake3": "<hash>",
  "sha1": "<sha1>",
  "size_bytes": 12345,
  "mime": "image/png"
}
```

Invariants:

- `id` is exactly `obj:blake3:<hash>`.
- `object_ref` is exactly `media://blake3/<hash>`.
- `blake3` is exactly `<hash>`.
- `sha1` is the SHA-1 hex digest of the original bytes.
- `size_bytes` is the original byte length.
- `mime` is the effective MIME after normalization, not the declared MIME.

`media_bindings` describe export filenames:

```json
{
  "id": "media:heart",
  "export_filename": "heart.png",
  "object_id": "obj:blake3:<hash>"
}
```

Rules:

- `export_filename` must be a bare filename.
- A package may bind the same object to multiple filenames.
- The same `export_filename` must not bind to different objects.
- First version behavior for same-filename different-object conflict is an
  error. It does not automatically rename or rewrite card content.

`media_references` describe static references discovered during normalization:

```json
{
  "owner_kind": "note",
  "owner_id": "note:001",
  "location_kind": "field",
  "location_name": "Front",
  "raw_ref": "heart.png",
  "ref_kind": "html_src",
  "media_id": "media:heart"
}
```

Reference records are an index and report surface. They let the writer validate
precomputed invariants without redoing semantic media scanning.

Stable ordering:

- `media_objects` are sorted by `id`.
- `media_bindings` are sorted by `(export_filename, id)`.
- `media_references` are sorted by `(owner_kind, owner_id, location_name,
  raw_ref, ref_kind)`, where `location_name` is the field name, template name,
  CSS owner name, or another explicit owner-local slot name.

The APKG media entry order follows `media_bindings` sorted by
`(export_filename, id)`.

## MIME Rules

`declared_mime` is authoring input and diagnostic evidence only. It does not
become the normalized semantic MIME.

The normalizer sniffs content and writes `MediaObject.mime` as the effective
MIME. If the sniff result is high-confidence and conflicts with
`declared_mime`, normalization emits `MEDIA.DECLARED_MIME_MISMATCH` as an error
by default. Low-confidence or unknown MIME outcomes are policy-controlled.

## Reference Scanning

Normalization scans note fields and relevant template/CSS text for local media
references.

Supported static forms include:

- `[sound:filename]`
- HTML `src`
- HTML `data`
- CSS `url(...)` for already-supported media/font asset cases

Missing-reference diagnostics exclude:

- `data:` URIs
- `http://` and `https://` URLs
- protocol-relative URLs
- other external protocol URLs
- dynamic template expressions that cannot be statically resolved

Dynamic or ignored references may be recorded in a skipped/ignored report bucket
but are not treated as missing media.

## Diagnostics

Normalization is the main media diagnostic phase.

Default errors:

- `MEDIA.UNSAFE_FILENAME`: `desired_filename` or `export_filename` is not a bare
  filename.
- `MEDIA.UNSAFE_SOURCE_PATH`: `source.path` is absolute, traverses upward, or
  canonicalizes outside `base_dir`.
- `MEDIA.SOURCE_MISSING`: a path source does not exist or cannot be read.
- `MEDIA.INLINE_TOO_LARGE`: inline authoring bytes exceed `inline_bytes_max`.
- `MEDIA.DUPLICATE_FILENAME_CONFLICT`: one export filename maps to different
  objects.
- `MEDIA.MISSING_REFERENCE`: a statically resolved local reference has no
  matching binding.
- `MEDIA.DECLARED_MIME_MISMATCH`: high-confidence sniffed MIME conflicts with
  declared MIME.
- `MEDIA.SIZE_LIMIT_EXCEEDED`: single-object or total media size exceeds policy.

Policy-controlled diagnostics:

- `MEDIA.UNUSED_BINDING`: binding is declared but not referenced.
- `MEDIA.UNKNOWN_MIME`: MIME cannot be determined confidently.

Informational diagnostics:

- `MEDIA.DUPLICATE_OBJECT`: multiple bindings point to the same object and
  dedupe is active.

Reports should include:

- objects with hash, size, effective MIME, and reference count
- bindings with export filename, object id, and reference count
- references with owner, location kind/name, raw ref, ref kind, and resolved
  media id
- skipped references with reason where useful
- diagnostics with path, owner selector, stage, and operation

## Writer Responsibilities

The writer must not redo high-level semantic media diagnosis. It only reports:

- CAS object missing
- CAS object hash or size mismatch
- normalized media object/binding/reference invariant violation
- staging IO failure
- APKG zip/protobuf/SQLite write failure

The writer's source of truth for media payloads is CAS. `staging/media` is a
derived, reproducible artifact for inspection and debugging.

## Staging

Staging manifest contains normalized metadata, not media payloads.

Materializing staging:

- writes the manifest deterministically
- reads original bytes from CAS
- copies or hardlinks each binding to `staging/media/<export_filename>`
- verifies object integrity while doing so

`staging/media` can be deleted and regenerated from the manifest plus CAS. It is
not the writer's primary input.

## APKG Output

The APKG writer reads original bytes from CAS and writes media according to the
target APKG schema.

Rules:

- APKG media entries use binding order sorted by `(export_filename, id)`.
- Zip payload entry names remain `0`, `1`, ...
- `MediaEntries.entries[].name` is `export_filename`.
- `size` and `sha1` originate from `MediaObject`, after writer integrity
  checks.
- Final field encoding, compression, and map shape follow the target APKG
  schema.
- Latest APKG output zstd-compresses media payloads as required by APKG v3.
- If multiple filenames bind to the same object, v1 writes multiple APKG payload
  entries and reports the dedupe as info. It does not try to reuse zip payload
  entries.

## Inspect

Staging inspection reads the manifest and derived `staging/media` files. It may
report Forge media object and binding metadata because those are declared in the
staging manifest.

APKG inspection reports only observable APKG facts:

- filename
- payload presence
- size
- SHA-1
- media map shape
- collection references where observable

APKG inspection must not infer Forge-only metadata such as `media_id`,
`object_id`, or CAS object refs.

## Testing Strategy

Schema and contract invariant tests:

- authoring schema accepts path and inline bytes media sources
- normalized schema requires `media_objects`, `media_bindings`, and
  `media_references`
- normalized schema rejects `data_base64` and `inline_bytes`
- invariant tests verify `id`, `object_ref`, and `blake3` consistency
- writer/build result schemas expose media diagnostics without payload fields

Normalization tests:

- relative path resolution uses `NormalizeOptions.base_dir`
- absolute source paths fail
- `..` traversal fails
- symlink and canonical path escapes fail
- bare filename validation rejects path-like desired filenames
- inline bytes size limit is enforced
- CAS dedupes identical bytes
- same export filename with different object fails
- same object with multiple filenames emits info
- high-confidence declared MIME mismatch fails
- unknown MIME behavior follows policy
- unused binding behavior follows policy
- external URLs and data URIs are excluded from missing-reference diagnostics
- dynamic template references are skipped rather than treated as missing

Writer tests:

- writer reads media from CAS, not `staging/media`
- missing CAS object fails with writer error
- BLAKE3 mismatch fails with writer error
- SHA-1 mismatch fails with writer error
- size mismatch fails with writer error
- malformed object/binding/reference invariants fail with writer error
- writer does not emit semantic diagnostics such as unused binding, missing
  reference, or MIME mismatch when given a normalized IR fixture
- staging media is regenerated from CAS
- APKG media map follows `(export_filename, id)` order

Golden tests:

- deterministic normalized manifest ordering
- deterministic staging manifest bytes for equivalent input
- deterministic APKG media map order
- fixture snapshots for image, audio, video, and font-as-media/CSS-url cases

Inspect tests:

- staging inspect reports manifest-visible object and binding metadata
- APKG inspect reports only observable media facts
- APKG inspect does not include Forge-only object or binding ids

End-to-end tests:

- Deck/Product lowering through normalize ingest, staging, APKG, and inspect
- image, audio, video, and font-as-ordinary-media flows
- no full asset compiler behavior is introduced for fonts

## Migration Plan

Because backward compatibility is explicitly out of scope, contract and fixture
updates should be direct:

- replace `AuthoringMedia { filename, mime, data_base64 }` with authoring source
  declarations
- replace `NormalizedMedia { filename, mime, data_base64 }` with
  `media_objects`, `media_bindings`, and `media_references`
- delete normalized/writer `data_base64` compatibility paths
- rewrite existing media fixtures to use path sources and fixture-local
  `base_dir`
- keep small test helpers only outside the normalized/writer contract surface

## Implementation Notes

The implementation plan can refine exact Rust module boundaries, but the design
expects these responsibilities:

- `authoring_core`: media source model, normalized media model, normalize
  options, media validation, reference indexing
- `writer_core`: CAS-backed staging and APKG writer integrity checks
- `anki_forge`: ergonomic file/bytes/reader helpers that produce authoring media
  declarations or invoke ingestion before build
- `contracts`: schemas, semantics, fixtures, and policy defaults for media
  diagnostics

The next step after this design is approved is a task-by-task implementation
plan.
