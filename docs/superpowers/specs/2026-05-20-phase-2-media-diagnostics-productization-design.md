# Phase 2 Media And Diagnostics Productization Design

## Context

`docs/api-design.md` defines Phase 2 as Media + Diagnostics productization. Phase
1 has already delivered the user-facing Rust MVP: `Project`, the `Deck` facade,
custom note types, minimal `MediaRegistry` helpers, `MediaRef::sound()`,
`MediaRef::image()`, `BuildReport`, examples, and a Python API shape sketch.

The lower media pipeline is also farther along than the Product API surface:
`authoring_core` already supports CAS-backed media objects, export filename
bindings, media reference records, source path validation, MIME diagnostics,
unused binding diagnostics, and field-level reference scanning.

Phase 2 should therefore not rebuild the media pipeline. It should make those
capabilities reliable and visible at the Product API boundary.

## Goal

Turn Phase 1 media helpers into production-grade media authoring and reporting:
users can register media through `Project`, use `MediaRef` from notes, templates,
and CSS, and receive actionable media diagnostics and summaries in
`BuildReport`.

The implementation should proceed as end-to-end thin slices. Each slice should
carry behavior from Product API input through normalization, writer output,
reporting, examples, and tests.

## Non-Goals

- No automatic filename rename or content rewrite.
- No remote URL fetching.
- No media transforms or asset compiler.
- No formal Python Product API binding.
- No full diff/risk/CI policy implementation.
- No second writer path or Product-specific media writer.
- No writer-side re-interpretation of high-level media semantics.
- No `MediaRef::html_img(attrs)` helper in Phase 2; keep `image()` as the
  stable image helper and defer richer HTML generation.
- No custom media validation callbacks.

## Confirmed Decisions

- Use an end-to-end thin-slice implementation strategy.
- Keep strict media behavior as the stable default.
- Allow advanced media policy plumbing, but keep it out of the primary `prelude`
  experience.
- Diagnose filename collisions and unsafe/missing references; do not rewrite
  names or references automatically.
- Scan note fields, custom template front/back, custom browser templates, and
  notetype CSS.
- Python work is limited to target shape review, examples, and README notes.
- Source paths in diagnostics are opaque human-readable selector strings, not a
  new public Product navigation API.

## Architecture

All public build paths keep one pipeline:

```text
Project / Deck facade
-> ProductDocument + Product media declarations
-> Product lowering in anki_forge::product
   (AuthoringDocument + lowering mappings + Product source provenance)
-> AuthoringDocument
-> normalize_with_options(media policy + media store)
-> NormalizedIr(media_objects, media_bindings, media_references)
-> writer_core
-> BuildReport
```

Responsibilities remain layered:

- `anki_forge::product` owns user intent: media registration, Product source
  provenance, note/template/CSS attachment points, and ergonomic helpers.
- Product lowering is part of `anki_forge::product`. It is the
  `Project::lower()` / `ProductDocument::lower()` pass that finalizes all
  Product helper output, media declarations, lowering mappings, and source
  provenance before normalization begins. It is not a separate writer path and
  it is not owned by `anki_forge::build`.
- `authoring_core` owns normalized media semantics: CAS ingest, media objects,
  bindings, references, static reference scanning, and media diagnostics.
- `writer_core` owns artifact materialization: CAS integrity, normalized media
  invariants, staging, and APKG output.
- `anki_forge::build` owns Product-facing report aggregation, media summaries,
  pretty output, and build success rules.
- `anki_forge::diagnostics` owns stable diagnostic types, source paths, severity,
  and user-facing help text.

## Product API

The stable Product API should stay small and strict:

```rust
let audio = project.media_mut()
    .add_file("media/taberu.mp3")?
    .export_as("taberu.mp3")?;

let image = project.media_mut()
    .add_bytes("chart.png", bytes)?
    .export_as("chart.png")?;

project.add_note(
    Note::basic("taberu", "")
        .stable_id("jp:taberu")
        .sound("Back", audio)
        .image("BackImage", image)
)?;
```

Phase 2 should refine the existing behavior:

- `add_file` records source path provenance and keeps large files path-backed
  until normalization.
- `add_bytes` records a logical source label and remains suitable for tests and
  small payloads.
- `add_bytes` is capped by the strict inline media limit: 64 KiB. Phase 2 rejects
  oversized byte payloads immediately from `add_bytes(...)` with
  `MEDIA.INLINE_TOO_LARGE`; no pending guard is created. Large payloads should
  use `add_file(...)`.
- if `export_as(...)` fails for an `add_bytes(...)` payload, the registry must
  not retain a partial entry.
- `export_as` validates a bare, helper-safe filename.
- Same export filename with different content is an error.
- Same export filename with same content returns the existing reference.
- Same content under different export filenames is allowed and uses CAS dedupe.
- `MediaRef::sound()` and `MediaRef::image()` keep returning Anki-compatible
  `Content::Html`.
- Registry introspection should support reporting, but Phase 2 should avoid
  exposing the full registry schema as a stable public contract.

Advanced media policy is configured through `ProjectNormalizeOptions`, passed to
`BuildOptions::normalize_options(...)`. Stable examples and `prelude` should use
strict defaults.

Product registry collision checks are eager:

- `export_as(...)` compares against existing registry entries immediately.
- same filename with different observed content returns an error at registration
  time, before build;
- normalization keeps defensive duplicate filename checks for lower-level
  authoring input and non-Product callers.

When the same export filename is registered with the same content, the registry
returns a fresh `MediaRef` value for the existing logical binding. `MediaRef` is
a small value handle keyed by export filename, not a shared object identity.
When the same export filename is registered with different content, reuse
`MEDIA.DUPLICATE_FILENAME_CONFLICT`.

`add_file(...)` and `add_bytes(...)` are two-phase registrations. They return a
pending media guard that owns the source metadata and observed fingerprint, but
the registry is not mutated until `export_as(...)` succeeds. If `export_as(...)`
returns an error, dropping the pending guard leaves the registry unchanged.
`export_as(...)` consumes the pending guard, whether it succeeds or fails; the
same guard cannot be exported more than once.

The first argument to `add_bytes(source_label, bytes)` is diagnostic metadata,
not an export filename. It should be non-empty and free of control characters,
and validation failures are returned immediately from `add_bytes(...)`. It does
not need to follow helper-safe filename rules. After `export_as(...)` succeeds,
Product source paths use the export filename:
`project.media["chart.png"]`. Before an export filename exists, registration
errors may mention the source label in the message.

Source-label validation failures use `MEDIA.INVALID_SOURCE_LABEL`.

The Product registration signatures are fallible:

```rust
pub fn add_file(&mut self, path: impl AsRef<Path>) -> Result<PendingMedia<'_>>;
pub fn add_bytes(
    &mut self,
    source_label: impl Into<String>,
    bytes: Vec<u8>,
) -> Result<PendingMedia<'_>>;
```

Zero-byte file or byte media is rejected at registration with
`MEDIA.EMPTY_SOURCE`. Phase 2 does not package empty media objects.

For file-backed media, `add_file(...)` must stream the source file at
registration time to compute a fingerprint. "Path-backed until normalization"
means bytes are not stored inline in Product or normalized IR; it does not mean
the file is never read before normalization. Product media records must keep the
observed fingerprint and source metadata.

The fingerprint is BLAKE3 plus byte length, matching the CAS content hash
algorithm used by normalized media objects. This fingerprint is Product-internal
diagnostic state, not a public serialized contract.

Registration-time source failures use media source error codes:

- missing file: `MEDIA.SOURCE_MISSING`;
- non-regular file: `MEDIA.SOURCE_NOT_REGULAR_FILE`;
- unreadable or interrupted read: `MEDIA.SOURCE_READ_FAILED`.

These are returned as Product API registration errors. During normalization, the
same codes are used if the path becomes missing or unreadable. If the file no
longer matches the registered fingerprint, report `MEDIA.SOURCE_CHANGED` rather
than silently packaging different bytes.

If a file is deleted between registration and normalization, the diagnostic is
`MEDIA.SOURCE_MISSING`, not `MEDIA.SOURCE_CHANGED`.

The same source path may be registered more than once with different export
filenames. Source path is provenance, not identity. If the observed fingerprints
are the same, CAS dedupe handles the shared bytes. If the file changes after one
or more registrations, each affected binding reports `MEDIA.SOURCE_CHANGED` or
`MEDIA.SOURCE_MISSING` during normalization according to the current filesystem
state.

Phase 2 accepts the registration-time I/O cost to get deterministic collision
errors near the registration site. Implementations should hash by buffered
streaming and must not retain full file bytes in Product state. A future phase
can add an explicitly deferred registration API if large-project construction
cost becomes a proven problem.

Phase 2 media bindings are append-only after a successful `export_as(...)`.
There is no stable deregister or rebind API. If a user registers the wrong file
or wants to reuse an export filename for different content, they should create a
new `Project` or registry state. A future ergonomic API can add explicit
replacement semantics if a real workflow needs it.

Helper-safe export filenames use the same rules everywhere in the Product API:

- non-empty after trimming,
- one bare filename component,
- not absolute,
- no `/`, `\`, `.`, or `..` path components,
- ASCII alphanumeric plus `.`, `_`, and `-` only.

These strict rules intentionally reject whitespace, quotes, control characters,
brackets, angle brackets, percent signs, and platform-specific path syntax. They
make generated `[sound:...]` and `<img src="...">` helpers safe without adding
HTML escaping decisions to every call site.

Advanced media policy is limited in Phase 2 to severity controls for existing
diagnostics:

- unused binding behavior: ignore, info, warning, or error;
- unknown MIME behavior: ignore, info, warning, or error;
- declared MIME mismatch behavior: warning or error.

Missing references, unsafe references, filename collisions, unreadable sources,
source changes, inline size violations, Product helper wiring failures, and CAS
integrity failures remain errors. Advanced policy does not support automatic
renaming, automatic rewriting, remote fetching, transforms, or custom callbacks.

CAS media-store failures during normalization must remain media-specific
diagnostics, not collapse into generic normalization errors:

- media store write, fsync, rename, or finalize failure:
  `MEDIA.CAS_WRITE_FAILED`;
- existing CAS object integrity conflict:
  `MEDIA.CAS_OBJECT_INTEGRITY_CONFLICT`.

Both are strict-default errors and should keep source context when available.
`MEDIA.CAS_OBJECT_INTEGRITY_CONFLICT` means a content-addressed object already
exists in the media store at the expected path but its bytes, hash, or size no
longer match the address-derived invariant.

The policy owner is `ProjectNormalizeOptions`, passed through
`BuildOptions::normalize_options(...)`. `ProjectMediaPolicy::strict()` remains
the default. Phase 2 does not add a separate policy root.

## Diagnostics

Phase 2 diagnostics should be actionable at the Product layer. Users should see
selectors such as:

```text
project.media["taberu.mp3"]
project.notes["jp:taberu"].fields["Audio"]
project.note_types["jp-vocab"].templates["Recognition"].front
project.note_types["jp-vocab"].templates["Recognition"].browser_back
project.note_types["jp-vocab"].css
```

These selectors are opaque diagnostic strings. They do not imply a new
`project.media[...]` or `project.note_types[...]` public indexer API.

Notetype source paths use the Product notetype id, such as
`NoteType::custom("jp-vocab")`, not the display name. If duplicate notetype ids
are ever present, validation should report that error and media diagnostics
should fall back to `project.note_types[index]`, where `index` is the zero-based
Product notetype insertion order.

The mapping strategy:

1. `authoring_core` emits normalized media diagnostics, such as
   `MEDIA.MISSING_REFERENCE`, `MEDIA.UNUSED_BINDING`,
   `MEDIA.DECLARED_MIME_MISMATCH`, and
   `MEDIA.DUPLICATE_FILENAME_CONFLICT`.
2. `anki_forge::product` records source provenance while lowering:
   media filename to `project.media[...]`, note fields to
   `project.notes[...]`, and template/CSS text to `project.note_types[...]`.
3. `anki_forge::build` maps lower-level diagnostics into stable
   `Diagnostic { code, severity, message, source, help }`.

Diagnostics should include targeted help where possible:

- Missing reference: register the media with `project.media_mut().add_file(...)`
  or update the local filename in the field/template/CSS.
- Filename collision: choose a unique `export_as(...)` name.
- Unused binding: remove the registration or reference it from a note, template,
  or CSS.
- MIME mismatch: include the declared MIME and observed MIME, then suggest
  changing the export filename/declared MIME or replacing the source file.
- Unsafe reference: use a bare local filename for packaged media.

CSS-originated diagnostics should include the raw `url(...)` value in the
message and a 1-based line hint for the opening `url(` token, computed by
counting newlines in the original CSS text. The source path remains
`project.note_types["id"].css`; the raw reference and line hint provide the
local context.

## Build Report

`BuildReport` should expose a Product-facing media summary without making the
entire normalized media schema stable:

```rust
pub struct MediaSummary {
    pub objects: usize,
    pub bindings: usize,
    pub references: usize,
    pub missing_references: usize,
    pub unsafe_references: usize,
    pub unused_bindings: usize,
    pub unique_bytes: u64,
}
```

`BuildReport` should include this summary alongside existing counts,
diagnostics, metrics, artifact path, and inspect summary. A pretty report or
summary method should be available for README output, examples, and CI logs.

Summary semantics:

- `objects` is `normalized_ir.media_objects.len()`.
- `bindings` is `normalized_ir.media_bindings.len()` after Product registry
  collapse and normalization; duplicate export filenames are invalid and do not
  count as additional bindings.
- `references` is `normalized_ir.media_references.len()`.
- `missing_references` counts references with missing resolution.
- `unsafe_references` counts references that produced `MEDIA.UNSAFE_REFERENCE`.
- `unused_bindings` counts all unreferenced bindings, independent of whether
  policy emits `MEDIA.UNUSED_BINDING` as ignore, info, warning, or error.
- `unique_bytes` is the sum of unique CAS object sizes from `media_objects`, not
  the per-binding byte total.

Pretty report output should be plain ASCII, deterministic, and suitable for
README and CI logs. The target shape is one section with `key: value` rows:

```text
Media:
  objects: 2
  bindings: 3
  references: 4
  missing_references: 1
  unsafe_references: 0
  unused_bindings: 1
  unique_bytes: 48213
```

Diagnostics may be printed after the summary as one diagnostic per line:

```text
[warning MEDIA.UNUSED_BINDING] project.media["unused.png"]: registered media is not referenced. Remove it or reference it from a note, template, or CSS.
```

Diagnostic lines are sorted by final severity (`error`, `warning`, `info`),
then source path, then diagnostic code, then UTF-8 message bytes. The layout is
ASCII, but diagnostic messages may include user-provided UTF-8 text.

`BuildReport::ensure_success()` should continue to fail only when:

- no artifact exists,
- build status is not success, or
- at least one diagnostic has error severity.

Warnings such as unused media should not fail strict default builds unless an
advanced policy promotes that diagnostic to an error.
`ensure_success()` evaluates the final severity already stored in
`BuildReport.diagnostics` after policy application. It does not re-evaluate base
diagnostic severities.

## Reference Scanning

Phase 2 expands static reference scanning from note fields to the Product
surfaces where users can author media references:

- note fields,
- custom template `front`,
- custom template `back`,
- custom template `browser_front`,
- custom template `browser_back`,
- custom notetype CSS,
- Product-authored stock template/CSS changes produced by helpers, bundled
  assets, or future stock-template customization APIs.

Supported static forms remain:

- `[sound:filename]`,
- HTML media `src`,
- HTML `<object data="...">`,
- CSS `url(...)`.

Resolution rules:

- Safe local filenames must match a registered media binding.
- Missing local filenames produce `MEDIA.MISSING_REFERENCE`.
- Unsafe local references produce `MEDIA.UNSAFE_REFERENCE`.
- External URLs, `data:` URIs, protocol-relative URLs, and dynamic template
  expressions are skipped rather than treated as missing media.
- Query strings and fragments do not participate in binding resolution.
- The system records references and diagnostics; it never rewrites user
  HTML/CSS or field content.

Unsafe local references use the same helper-safe filename rules as `export_as`
after reference-specific decoding. Empty references are skipped with
`skip_reason = "empty-ref"` and do not produce missing or unsafe diagnostics. A
non-empty local reference is unsafe if it is absolute, path-like, contains `.`
or `..` path components, contains separators, contains control characters, or
contains characters outside ASCII alphanumeric plus `.`, `_`, and `-`.

Dynamic template expressions are skipped with
`skip_reason = "dynamic-template-expression"`. For Phase 2, a reference value is
dynamic when the candidate filename or local URL path contains Anki template
markers `{{` or `}}` after HTML/CSS string decoding and before helper-safe
filename validation. Conditional blocks around a static reference do not make
the static reference dynamic; only markers inside the candidate reference value
do.

Reference-specific decoding rules:

- The decoding order is fixed: decode the reference form, check for dynamic
  template markers, classify empty/external references, strip query/fragment
  where URL semantics apply, percent-decode URL paths where applicable, then
  apply helper-safe filename validation.
- `[sound:...]` is an Anki media filename reference, not a URL. It is HTML-entity
  decoded and trimmed, but it is not percent-decoded and it does not use query
  or fragment semantics.
- HTML `src` and `<object data>` references are HTML-entity decoded, classified
  as URLs, stripped of query and fragment for binding resolution, and
  percent-decoded as UTF-8 for local path matching.
- CSS `url(...)` references are parsed with the existing
  `authoring_core::media_refs` scanner scope: block comments and script/raw
  text are ignored; quoted and unquoted top-level `url(...)` values are
  supported; local URL path components are percent-decoded as UTF-8 before
  matching.
- Full CSS backslash escape unescaping is out of Phase 2 scope. Escaped local
  filenames that still contain backslashes after the Phase 2 scanner extracts
  the raw URL value are treated as helper-unsafe local references.
- CSS `url(...)` scanning is context-agnostic. It applies inside `@import`,
  `@font-face`, normal declarations, and custom properties when a complete
  top-level `url(...)` token is present.
- Local `@import url("theme.css")` is treated as a packaged local asset
  reference. Users should register that file as media, use an external URL, or
  remove the local import; Phase 2 does not special-case CSS imports away from
  diagnostics.
  The rationale is that local Anki package references must be packaged
  explicitly; anki-forge should not silently assume web-style runtime import
  resolution.
- Bare-string CSS imports such as `@import "theme.css"` are out of Phase 2
  scanning scope because they are not `url(...)` tokens.
- Nested `url(url(...))`, unbalanced parentheses, and incomplete `url(` values
  are skipped as malformed CSS URL candidates rather than treated as missing
  references.
- Invalid percent escapes, invalid UTF-8, decoded separators, decoded `.` or
  `..`, or decoded helper-unsafe characters produce `MEDIA.UNSAFE_REFERENCE`.

Malformed HTML and CSS are handled with best-effort scanning. Complete
recognized references produce resolved, missing, skipped, or unsafe records.
Incomplete constructs, such as an unclosed HTML attribute quote or an unclosed
CSS `url(`, are skipped without a parse diagnostic. Phase 2 diagnostics should
avoid pretending an incomplete fragment is a definite missing media reference.

When one media object is bound to multiple export filenames, diagnostics and
summary output remain binding-oriented. Binding-specific problems point to the
specific `project.media["filename"]` source. Phase 2 pretty output does not list
dedupe groups; it only reports the summary fields defined above.

Generated stock/helper references are scanned only after Product lowering has
registered the corresponding generated media binding. If a bundled helper
produces an unregistered media reference, that is a Product lowering/internal
diagnostic with code `PRODUCT.MEDIA_HELPER_REFERENCE_UNREGISTERED`, not a
user-authored missing-reference diagnostic.

The ordering invariant is explicit: Product lowering must finalize all media
bindings before normalization scans note, template, browser-template, or CSS
text. Implementations should assert this invariant in tests for bundled helper
scenarios. There is no partial helper-scanning phase.

Scanning scope is based on lowered Product-authored surfaces, not on whether the
notetype started as stock or custom. Note fields for stock notes are scanned as
note fields. Template and CSS text for custom notetypes is scanned. Template and
CSS text for stock notetypes is also scanned when the Product API has allowed a
user, helper, or bundled asset to alter that text. Built-in stock templates with
no Product-authored media reference are not special scanning targets.

Product lowering validates helper-generated media references against
helper-generated bindings before normalization. A missing helper binding emits
`PRODUCT.MEDIA_HELPER_REFERENCE_UNREGISTERED`. Once helper text reaches
normalization without that Product diagnostic, remaining missing references are
treated as normal `MEDIA.MISSING_REFERENCE` records on the authored surface.

Source path selection:

- use `project.notes["stable_id"]...` only when the stable id is present and
  unique;
- use `project.notes[index]...` for notes without a stable id, with an empty
  stable id, or with a duplicate stable id;
- `index` is the zero-based Product note insertion order; for `Deck`-backed
  projects, it is the zero-based deck note order at conversion time;
- duplicate stable ids remain validation errors, but media diagnostics should
  still use index-based source paths.

Stable-id uniqueness and blank stable ids are determined by `Project::validate()`
during build preparation before Product source mappings are finalized and before
normalization scans media references. `add_note(...)` does not need to reject
duplicates or blanks eagerly. Build preparation first validates and records
blank/duplicate ids, then builds source mappings from that validation result.
Blank stable ids produce `AFID.STABLE_ID_BLANK` with error severity.

Duplicate notetype ids produce `NOTETYPE.ID_DUPLICATE` with error severity.
That diagnostic should include the display names, if present, plus index-based
source paths so users can distinguish the duplicated definitions.

Diagnostic code ownership:

- `authoring_core` is the source of truth for normalization media codes in the
  `MEDIA.*` family.
- `anki_forge::diagnostics::DiagnosticCode` exposes those codes as stable opaque
  strings and must not rename them while enriching source paths and help text.
- Product-only wiring and lowering failures use `PRODUCT.*` or existing
  Product-domain code families, such as
  `PRODUCT.MEDIA_HELPER_REFERENCE_UNREGISTERED`.

Scanning recovery:

- scanning is continue-on-error across surfaces;
- a malformed candidate in one field, template, browser template, or CSS block
  must not abort scanning of other surfaces;
- complete references discovered elsewhere in the same normalization pass still
  produce their normal resolved/missing/skipped/unsafe records.

Product API note/template/CSS surfaces are Rust `String` values and therefore
valid UTF-8. Invalid UTF-8 JSON or contract input fails before Product-level
reference scanning; Phase 2 does not define lossy scanning of invalid UTF-8
surfaces.

Scanning belongs in or near `authoring_core::normalize`, because normalization
already produces `media_references`. Product lowering should provide source
mapping so those references can become Product-facing diagnostics. The writer
must not redo semantic reference scanning.

## Thin-Slice Delivery Plan

Implementation should be planned as thin slices:

1. Product registry provenance and strict collision behavior.
2. Product source-path mapping for note fields, media bindings, templates, and
   CSS.
3. Template/browser-template/CSS reference scanning in normalization.
4. Diagnostic mapping with `source` and `help`.
5. `BuildReport` media summary and pretty report output.
6. Advanced media policy plumbing outside the primary stable `prelude` path.
7. Examples, README troubleshooting, and Python shape updates.

Each slice should include focused tests and at least one end-to-end assertion
where behavior crosses Product API, normalization, and report output.
For early registry slices, "report output" can mean the existing build
diagnostics returned through `BuildReport`; the full `MediaSummary` and pretty
report are introduced in the report slice.
The minimum early-slice end-to-end assertion is exact diagnostic code, final
severity, Product source path, and one stable message/help substring. Full
message snapshots are not required.

## Testing Strategy

Product media API tests:

- hash dedupe for identical bytes,
- same filename with different content fails,
- same content with different filenames succeeds,
- large file inputs remain path-backed until normalization,
- unsafe export names are rejected.

Reference scanning tests:

- note field resolved/missing references,
- template front/back resolved/missing references,
- browser template references,
- CSS `url(...)` references,
- unused bindings,
- unsafe local references,
- skipped external or dynamic references.

Diagnostics tests:

- major media diagnostics include code, severity, source, and help,
- Product source paths point to the authored object,
- lower-level normalize failures do not hide specific media diagnostics behind a
  generic `PROJECT.NORMALIZE_FAILED`.

Report tests:

- media summary counts objects, bindings, references, missing references, unused
  bindings, and unique bytes,
- warning diagnostics do not fail `ensure_success()`,
- advanced policy can promote selected media diagnostics to errors.

End-to-end examples:

- update `target_api_media` to cover audio, image, template/CSS references, and
  unused media warnings,
- keep examples runnable through `cargo run -q -p anki_forge --example
  target_api_media`.

Expected verification:

```bash
cargo test -p authoring_core -v
cargo test -p anki_forge --test project_media_api_tests -v
cargo test -p anki_forge --test build_report_tests -v
cargo test -p anki_forge -v
cargo run -q -p anki_forge --example target_api_media
```

The listed test file names are preferred implementation targets, not a public
contract. Existing files may be extended where they already exist; otherwise the
implementation plan may create equivalent focused tests with clear names.

Reference scanning tests should include concrete dynamic examples:

- `<img src="{{Image}}">` is skipped with
  `dynamic-template-expression`;
- `{{#HasImage}}<img src="heart.png">{{/HasImage}}` still resolves
  `heart.png` as a static reference.
- CSS diagnostics include the raw `url(...)` value and the 1-based line hint.

## Documentation And Python Shape

README or docs should add a media troubleshooting section covering:

- filename collision,
- missing media reference,
- unused media binding,
- unsafe media reference,
- MIME mismatch,
- why anki-forge does not automatically rewrite filenames or HTML/CSS.

Diagnostic help text should follow a short two-sentence shape: first state the
problem in Product terms, then suggest the next action. Example:
`project.notes["jp:taberu"].fields["Audio"] references missing media
"taberu.mp3". Register it with project.media_mut().add_file(...).export_as("taberu.mp3") or change the reference.`

CSS missing-reference help should acknowledge conservative scanning:
`project.note_types["jp-vocab"].css references missing media "icon.svg" in url("icon.svg"). Register it, change the URL, or remove the CSS rule if it is unused.`

CSS import help should be equally explicit:
`project.note_types["jp-vocab"].css references local import "theme.css" in url("theme.css"). Register theme.css as packaged media, use an external URL, or remove the import.`

The pretty report is human-facing. Structured machine-readable report export is
a future enhancement, not a Phase 2 requirement.

Python Phase 2 work is documentation only:

- update the target media API example under `bindings/python/examples/`,
- update `bindings/python/README.md` to mark it as a shape sketch,
- include a per-slice check that new Rust API surfaces can be expressed
  naturally in the Python shape without relying on Rust-only chaining or
  ownership patterns.

## Success Criteria

Phase 2 is successful when:

- users can diagnose common media problems from `BuildReport` without reading
  normalized IR,
- media references in notes, templates, browser templates, and CSS are scanned
  consistently,
- diagnostics point to Product source paths and include practical help,
- media summary output is useful in examples and CI logs,
- strict default behavior remains simple and deterministic,
- advanced policy hooks exist without expanding the stable beginner API,
- Python shape docs remain aligned with the Rust Product API.
