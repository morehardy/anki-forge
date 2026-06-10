# Project Add-Time Validation Design

## Goal

`Project::add_note` and `Project::add_notetype` should be honest, useful API
boundaries. When users write `project.add_note(note)?`, the method should have
validated the errors that are knowable at that point instead of only pushing
into a vector and delaying obvious failures until `validate()` or `build()`.

The design keeps one primary user-facing add API. It does not add `try_add_*`
methods because that would make users choose between two similar entry points
and would make the common path less clear.

## Current State

Rust `Project::add_notetype` and `Project::add_note` currently return
`anyhow::Result<&mut Self>`, but both implementations only append to
`note_types` or `notes` and return `Ok(self)`.

That creates two API problems:

- `add_note(...)?` reads like validation happened, but no Project-level note
  validation has actually happened.
- Errors such as duplicate stable ids, unknown note type ids, missing custom
  identity, and unknown field keys surface later during `validate()`, `lower()`,
  or `build()`, farther away from the call that introduced them.

`Project::validate()` already reports many relevant diagnostics:

- blank note `stable_id`;
- duplicate note `stable_id`;
- unsupported note type id;
- duplicate custom note type id, including implicit stock collisions;
- custom note type missing identity recipe warning;
- auto-derived custom field key warning.

Python `Project` already fast-fails similar add-time issues, including unknown
note type ids, duplicate custom note type ids, unknown custom note field keys,
and missing identity fields or explicit stable ids before serialization.

## Public API

Change the Rust Product Project add methods to return a typed error:

```rust
impl Project {
    pub fn add_notetype(
        &mut self,
        note_type: NoteType,
    ) -> Result<&mut Self, ProjectAddError>;

    pub fn add_note(&mut self, note: Note) -> Result<&mut Self, ProjectAddError>;
}
```

`ProjectAddError` is a new public error type in `crate::product::project` and is
re-exported from `crate::product` and `crate::prelude`.

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectAddError {
    diagnostic: crate::diagnostics::Diagnostic,
}

impl ProjectAddError {
    pub fn diagnostic(&self) -> &crate::diagnostics::Diagnostic;
    pub fn code(&self) -> crate::diagnostics::ErrorCode;
}
```

`ProjectAddError` implements:

- `std::fmt::Display`, using `"{code}: {message}"`;
- `std::error::Error`;
- `crate::diagnostics::ErrorCodeExt`, using
  `diagnostic.code.error_code()`.

`ProjectAddError::code()` and the `ErrorCodeExt` implementation both use
`diagnostic.code.error_code()`.

`anyhow::Error` downcasting in `crate::diagnostics::ErrorCodeExt for
anyhow::Error` should recognize `ProjectAddError`. The current implementation
in `anki_forge/src/diagnostics/mod.rs` already downcasts `DeckError`, media
errors, `BuildError`, `ProductLoweringError`, and `ProductMediaPrepareError`;
add `ProjectAddError` to that same downcast chain.

The change is intentionally breaking at the signature level. Existing
`anyhow::Result` call sites using `?` continue to work because
`ProjectAddError` implements `std::error::Error`, but direct type annotations
or code relying on `anyhow::Error` at the method boundary may need updating.

## User-Facing Flow

The recommended long-term Project flow is:

```rust
use anki_forge::prelude::*;

fn main() -> anyhow::Result<()> {
    let mut project = Project::new("Spanish A1")
        .stable_id("spanish-a1")
        .default_deck("Spanish::A1");

    project.add_note(Note::basic("hola", "hello").stable_id("es:hola"))?;
    project.add_note(Note::basic("adios", "goodbye").stable_id("es:adios"))?;

    project.validate().ensure_success()?;
    project.write_apkg("spanish-a1.apkg")?.ensure_success()?;
    Ok(())
}
```

Add-time validation means "this object can be accepted into this Project based
on state that is already available." It does not mean "the whole Project is
fully buildable."

`validate()` remains the way to aggregate whole-project diagnostics before a
build. `build()` remains the complete normalization, media, writer,
comparison, update-safety, and policy boundary.

## ValidationReport Convenience

Add `ensure_success()` to the public Product `ValidationReport`:

```rust
impl ValidationReport {
    pub fn ensure_success(&self) -> Result<(), ValidationError>;
}
```

`ValidationError` is a new public diagnostics error type that clones the failed
`ValidationReport` and stores the first error diagnostic code selected by
`ensure_success()`. The current Product `ValidationReport` already derives
`Clone`; keep that derive as part of this design.

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    report: ValidationReport,
    primary_code: DiagnosticCode,
}

impl ValidationError {
    pub fn report(&self) -> &ValidationReport;
    pub fn primary_code(&self) -> &DiagnosticCode;
}
```

`ValidationError` implements:

- `Display`, using the first error diagnostic when present;
- `std::error::Error`;
- `ErrorCodeExt`, using `primary_code.error_code()`.

`ensure_success()` returns `Ok(())` when the report has no error diagnostics.
It constructs `ValidationError` only after finding at least one error
diagnostic, so no generic validation-failed fallback code is needed.

This makes the API shown in docs possible without requiring users to inspect
`has_errors()` manually.

## Add-Time Validation Scope

`add_note` validates only deterministic state already available in the `Project`
and the `Note`.

It returns an error and does not mutate the Project when:

- note type id is blank after trimming;
- explicit note `stable_id` is blank after trimming;
- explicit note `stable_id` duplicates any already-added non-blank explicit
  note stable id;
- the note uses a supported stock note type id that is already registered as a
  custom note type, because adding the note would create an implicit stock
  declaration that collides with the custom declaration;
- note type id is neither a supported stock note type nor a registered custom
  note type;
- a stock note contains a field key outside that stock note type's supported
  field key set;
- a custom note contains a field key not declared by the registered custom note
  type;
- a custom note-level `Note::identity(...)` override references an unknown
  custom field key;
- a custom note has no explicit `stable_id` and the registered custom note type
  plus the note itself provide no identity recipe from which a stable id can be
  derived.

"Blank after trimming" means `str::trim().is_empty()` in Rust. This trims
Unicode whitespace according to Rust's standard library behavior.

When multiple add-time problems exist on the same note, return the first error
in this priority order:

1. blank note type id;
2. blank explicit stable id;
3. duplicate explicit stable id;
4. stock note type id collides with an already-registered custom note type;
5. unsupported note type id;
6. unknown note field key;
7. note-level identity override unknown field key;
8. missing derivable custom note identity.

`add_notetype` validates only deterministic state already available in the
`Project` and the `NoteType`.

It returns an error and does not mutate the Project when:

- note type id is blank after trimming;
- note type id is a supported stock note type id such as `basic` or `cloze`;
- note type id duplicates an already-added custom note type id;
- two fields in the note type have the same field key;
- two fields in the note type have the same field name;
- more than one field is marked as sort;
- two templates in the note type have the same template key;
- a template generation rule references an unknown field key;
- an identity recipe references an unknown field key.

When multiple add-time problems exist on the same note type, return the first
error in this priority order:

1. blank note type id;
2. supported stock note type id used as a custom note type id;
3. duplicate custom note type id;
4. duplicate field key;
5. duplicate field name;
6. duplicate sort field;
7. duplicate template key;
8. template generation rule unknown field key;
9. identity recipe unknown field key.

`add_notetype` does not fail for a missing identity recipe by itself. Existing
`validate()` behavior treats that as a warning because the project may still be
buildable when every custom note supplies an explicit stable id.

`add_note` does fail when a custom note lacks an explicit stable id and neither
the note-level identity override stored on the note nor the registered
`NoteType::identity(...)` recipe can derive identity because that specific note
cannot be given a stable product identity by the current Project state.

In the current Rust Product model, the note-level identity override is observed
through `Note::identity_ref()`, and it is set by the builder-style
`Note::identity(fields)` method. There is no requirement to inspect a resolved
identity getter during add-time validation; resolved identity is a later
Project/build concern.

Rust direct `Project` should not treat `Field::identity()` alone as a derivable
identity recipe for this check. In the current direct Rust Project path,
identity derivation uses `Note::identity(...)` first and then
`NoteType::identity(...)`; `Field::identity()` is a field annotation and does
not replace the explicit note type identity recipe.

## Deferred Validation Scope

The following remain deferred to `validate()`, `lower()`, `normalize()`, or
`build()`:

- warnings such as auto-derived custom field keys;
- full-project diagnostic aggregation after several invalid objects have been
  added through lower-level or future APIs;
- media registry ownership and missing media references;
- file-backed media source existence, readability, size, MIME, and export
  collision checks;
- cloze syntax, cloze card generation, and normalization diagnostics;
- writer-layer failures;
- update-safety evidence, comparison, lockfile, merge safety, and policy
  failures;
- ProductDocument-backed and Deck-backed project mixed-state diagnostics;
- any validation that requires a normalized IR, writer policy, build context, or
  filesystem state.

The add-time layer should be intentionally small. If a check depends on
normalization or I/O, it belongs later.

## Stock Note Type Registry

Introduce a private shared helper for stock note type support:

```rust
fn is_supported_stock_notetype_id(id: &str) -> bool;
```

The helper should include every stock note type supported by direct
`Project` authoring. Today that means at least:

- `STOCK_BASIC_ID`;
- `STOCK_CLOZE_ID`;
- `STOCK_IMAGE_OCCLUSION_ID` once direct Project Image Occlusion support lands.

Add companion helpers for stock field keys:

```rust
fn stock_field_keys(note_type_id: &str) -> Option<&'static [&'static str]>;
```

`Project::validate()`, `add_note`, `implicit_stock_notetype_ids()`, and
`to_product_document()` should share the same stock registry so supported stock
ids do not drift across code paths.

The registry can be a small match-based helper, but it must stay as the single
source of truth so adding a future stock id, such as direct Project Image
Occlusion, does not require rediscovering every stock whitelist.

For the current Rust Product API, stock note constructors use display field
names:

- `Note::basic(...)`: `Front`, `Back`;
- `Note::cloze(...)`: `Text`, `Back Extra`;
- Image Occlusion builder design: `Occlusion`, `Image`, `Header`,
  `Back Extra`, `Comments`.

The Rust add-time stock field validation should therefore use the Rust Product
note field names, not Python product-v2 lowercase keys.

All note type ids, field keys, template keys, and identity recipe field keys are
case-sensitive. For example, `Front` and `front` are different Rust Product
field keys. This matches the current direct Rust Product model, where stock
constructors use display field names and custom note type keys are exact string
identifiers.

## Error Codes And Sources

`ProjectAddError` diagnostics should use the same codes as `validate()` or
lowering where possible:

- blank note stable id: `AFID.STABLE_ID_BLANK`;
- duplicate note stable id: `AFID.STABLE_ID_DUPLICATE`;
- unsupported note type id: `PROJECT.UNSUPPORTED_NOTE_TYPE`;
- duplicate note type id: `NOTETYPE.ID_DUPLICATE`;
- reserved stock note type id used as custom: `NOTETYPE.ID_RESERVED`;
- unknown note field key: `PRODUCT.FIELD_UNKNOWN`;
- missing note identity: `PRODUCT.IDENTITY_MISSING`;
- unknown identity field key: `PRODUCT.IDENTITY_FIELD_UNKNOWN`;
- duplicate field key: `NOTETYPE.FIELD_KEY_DUPLICATE`;
- duplicate template key: `NOTETYPE.TEMPLATE_KEY_DUPLICATE`;
- duplicate field name: `NOTETYPE.FIELD_NAME_DUPLICATE`;
- duplicate sort field: `NOTETYPE.SORT_FIELD_DUPLICATE`;
- template rule unknown field: `TEMPLATE.FIELD_UNKNOWN`;

`AFID.STABLE_ID_BLANK` is already emitted by Product `Project::validate()` but
is not currently mapped by `ErrorCode::from_code`. Add it to the error registry
and map it to `ErrorCode::StableIdBlank` as an alias of the existing
`DECK.BLANK_STABLE_ID` public enum variant. This keeps existing Product
diagnostic strings stable while making `err.code()` useful at the add-time
boundary.

If any other code does not already exist in the registry, add it to the
registry and map it through `ErrorCode::Unknown(...)` unless the code needs a
first-class `ErrorCode` variant for public matching.

The mixed code prefixes are intentional. Add-time errors should preserve the
domain of the underlying rule instead of inventing a `PROJECT.ADD_*` namespace:
identity problems stay in `AFID`, note type shape problems stay in `NOTETYPE`
or `TEMPLATE`, note content problems stay in `PRODUCT`, and project ownership
or registration problems stay in `PROJECT`. This keeps add-time errors aligned
with `validate()`, lowering, and build diagnostics.

Use source paths that point to the attempted add operation when possible:

- `project.notes[{next_index}]` for note-level add errors;
- `project.notes[{next_index}].fields["field"]` for field key errors;
- `project.note_types[{next_index}]` for note type-level add errors;
- `project.note_types[{next_index}].fields["Field Name"]` for field errors;
- `project.note_types[{next_index}].templates["Template Name"]` for template
  errors.

For duplicate errors, the message should name the already-existing item and the
attempted item. The source should point to the attempted item because that is
what failed to be added.

The stock-vs-custom collision check in `add_note` is defensive. Normal public
use should not be able to register a custom note type with a supported stock id
after `add_notetype` starts rejecting reserved stock ids. The check still belongs
in `add_note` so future lower-level constructors or migration paths cannot
silently create an implicit stock declaration on top of an invalid custom
registration.

## Mutation Semantics

Both add methods must be all-or-nothing:

- run add-time validation first;
- return `Err(ProjectAddError)` without modifying the Project on failure;
- append the note or note type only after validation passes;
- return `Ok(self)` after mutation.

This is important because a failed `add_note(...)?` should not leave hidden
invalid state behind.

## Compatibility And Migration

Update Rust examples, README, and `docs/api-design.md` where they describe the
old push-only semantics or show validation guidance. Most existing call sites
that already use `?` do not need mechanical changes because `ProjectAddError`
converts through `anyhow`.

Where docs show an explicit validation step, prefer:

```rust
project.validate().ensure_success()?;
```

Tests and examples that intentionally build invalid Projects should either:

- assert the add-time error at the add call; or
- construct the invalid state through `Project::from_product_document(...)`
  with a public `ProductDocument` builder when the test is specifically about
  build-time diagnostics and needs to bypass direct `Project::add_*`.

No Python API shape change is required. The Rust changes bring the public Rust
behavior closer to the existing Python binding behavior.

## Testing

Add focused Rust tests for `Project::add_note`:

- blank explicit stable id errors and does not mutate notes;
- duplicate explicit stable id errors on the second add and preserves only the
  first note;
- unsupported custom note type id errors;
- stock note type id colliding with an already-registered custom note type
  errors;
- unknown stock field key errors;
- unknown custom field key errors;
- note-level identity override unknown field errors;
- custom note without explicit stable id and without note-level or note type
  identity recipe errors;
- custom note with explicit stable id succeeds even when notetype identity is
  missing.

Add focused Rust tests for `Project::add_notetype`:

- duplicate custom note type id errors;
- custom note type id using a reserved stock id errors;
- duplicate field key errors;
- duplicate field name errors;
- duplicate sort field errors;
- duplicate template key errors;
- template generation rule unknown field errors;
- identity recipe unknown field errors;
- missing identity recipe succeeds at add-time and remains a validation warning.

Add tests for `ValidationReport::ensure_success()`:

- returns `Ok(())` for reports with no error diagnostics;
- returns `ValidationError` for reports with error diagnostics;
- `ValidationError::code()` uses the first error diagnostic code;
- the error exposes the full report.

Add migration coverage by updating existing Project API tests and examples that
currently call `.expect("add note")` or `.expect("add note type")`.

Each focused add-time error test should construct a single-problem fixture
unless the test is explicitly asserting the documented error priority order.
Priority-order tests are optional but recommended for common collisions such as
blank note type id plus unsupported note type id, and reserved stock note type
id plus duplicate custom note type id.

## Non-Goals

This design does not:

- introduce `try_add_note` or `try_add_notetype`;
- make `add_note` or `add_notetype` infallible;
- remove `validate()` or reduce its diagnostic coverage;
- make build-time media, normalization, writer, comparison, or update-safety
  checks run during add-time;
- guarantee that add-time validation has proven the Project can build.
