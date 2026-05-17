# Phase 1 User-Facing Rust MVP Design

## Status

Approved design draft.

Date: 2026-05-16

Source design: `docs/api-design.md`

## Goal

Phase 1 delivers the strict user-facing Rust MVP described in `docs/api-design.md`: users can build APKG files through `Project` or the quick `Deck` facade without touching IR, while every public path still flows through the existing lowering, normalization, writer, inspect, and report pipeline.

The exit standard is intentionally strict. Phase 1 must include:

1. `Project`
2. `Deck` as a `Project` facade
3. `NoteType::custom`
4. `FieldKey` / `TemplateKey` / stable config id derivation
5. `Field` / `Template` / minimal `GenerationRule`
6. `Note::new` with named fields
7. `Note::basic` / `Note::cloze`
8. safe `Content::Text` / explicit `Content::Html`
9. minimal `MediaRegistry::add_file` / `add_bytes` / `export_as`
10. `MediaRef::sound()` / `MediaRef::image()` and `Note::sound()` / `Note::image()`
11. `write_apkg -> BuildReport` basic
12. README first screen and runnable examples
13. Basic, Cloze, FieldKey/TemplateKey, and minimal MediaRef snapshot/oracle gates
14. Python API shape, wheel build approach, and diagnostics exception spike

## Current Context

The repository already has useful pieces:

1. `anki_forge/src/deck/*` exposes a quick Rust facade with `Deck`, stock Basic/Cloze/Image Occlusion notes, media registration, identity snapshots, validation, and export helpers.
2. `anki_forge/src/product/*` contains `ProductDocument`, stock/custom product DTOs, lowering to `AuthoringDocument`, helpers, assets, and metadata.
3. `authoring_core` owns normalization, identity, media ingestion, media reference resolution, and risk primitives.
4. `writer_core` owns staging, APKG emission, inspect, diff, and package build result types.
5. The recent production media pipeline has already routed high-level media through CAS-backed normalization and writer surfaces.

Phase 1 is therefore not a rewrite. It is a product API alignment: introduce the target `Project` model, make `Deck` a thin facade over it, turn build output into a user-facing `BuildReport`, and tighten custom note type identity/merge metadata before custom note types become a public habit.

## Architecture

All public build paths use one pipeline:

```text
Project / Deck
  -> ProductDocument
  -> AuthoringDocument
  -> NormalizedIr
  -> writer_core build
  -> inspect
  -> BuildReport
```

The public API expresses user intent. `ProductDocument` remains the lower-level product contract. `AuthoringDocument` remains the pipeline contract. `NormalizedIr` remains the writer input. Writer code does not learn about `Project`, `Deck`, or Product API convenience types.

The Phase 1 architecture has five vertical delivery packages:

1. `Project` and Build API base
2. `Deck` as `Project` facade
3. custom `NoteType` MVP with stable merge ids
4. minimal media and typed content helpers
5. horizontal gate for docs, examples, oracle/snapshot evidence, Python shape, and public API boundary

Each package must produce running tests and at least one visible user-facing behavior. Intermediate states should not add public API that bypasses the pipeline.

## Public Module Boundaries

Phase 1 should establish these public modules:

```text
anki_forge
  prelude
  product
  build
  diagnostics
  authoring
  writer
```

Recommended responsibilities:

| Module | Stability | Responsibility |
| --- | --- | --- |
| `prelude` | stable | Common user imports: `Project`, `Deck`, `Note`, `NoteType`, `Field`, `Template`, `GenerationRule`, `Content`, `MediaRef`, `BuildOptions`, `BuildReport` |
| `product` | stable | Product-facing card authoring API and lower-level `ProductDocument` bridge |
| `build` | stable | `BuildOptions`, `BuildReport`, `BuildError`, `BuildCounts`, `BuildMetrics`, artifact and inspect summaries |
| `diagnostics` | stable | user-facing diagnostic types, codes, severity, source paths, and helpers |
| `authoring` | advanced/unstable | IR and canonical JSON re-exports for advanced users and tests |
| `writer` | advanced/feature-gated | low-level writer, inspect, diff, and runtime primitives |

The crate root can continue to re-export a small number of common types, but Phase 1 should stop flattening low-level `authoring_core` and `writer_core` APIs into the main user surface. README and examples should prefer `anki_forge::prelude::*`.

Recommended file additions:

```text
anki_forge/src/prelude.rs
anki_forge/src/build/mod.rs
anki_forge/src/build/options.rs
anki_forge/src/build/report.rs
anki_forge/src/diagnostics/mod.rs
anki_forge/src/product/project.rs
anki_forge/src/product/notetype.rs
anki_forge/src/product/note.rs
anki_forge/src/product/template.rs
anki_forge/src/product/content.rs
anki_forge/src/product/media_registry.rs
anki_forge/src/product/identity.rs
```

Existing `deck` modules can remain, but their public build/write methods must delegate through `Project::from(deck)`.

## Build API

`BuildReport` is part of Phase 1, not a later diff/risk feature. The initial report is basic but structured.

Required Phase 1 shape:

```rust
pub struct BuildReport {
    pub artifact: ApkgArtifact,
    pub counts: BuildCounts,
    pub diagnostics: Vec<Diagnostic>,
    pub metrics: BuildMetrics,
    pub inspect: Option<InspectSummary>,
}
```

Required Phase 1 data:

1. artifact path
2. note count
3. card count
4. media count
5. diagnostics and warning count
6. inspect summary
7. duration/metrics
8. `ensure_success()`

`BuildError` must carry a report:

```rust
pub struct BuildError {
    pub report: BuildReport,
    pub cause: BuildFailureCause,
}
```

Rules:

1. `Ok(BuildReport)` means the build flow completed and a report is available. The report can still contain warnings.
2. `Err(BuildError)` means the build could not complete or a policy blocked the artifact. The error still exposes diagnostics and partial report data.
3. `write_apkg(path)` returns `Result<BuildReport, BuildError>`, not `Result<()>`.
4. Counts may be derived from `NormalizedIr` and inspect output in Phase 1. Full diff/risk remains Phase 4.

## Project API

`Project` is the long-term user entry point.

Required Phase 1 behavior:

```rust
let mut project = Project::new("Spanish A1")
    .stable_id("spanish-a1")
    .default_deck("Spanish::A1");

project.add_note(Note::basic("hola", "hello").stable_id("es:hola"))?;

let report = project.write_apkg("spanish-a1.apkg")?;
report.ensure_success()?;
```

`Project` owns:

1. project name and stable id
2. default deck
3. deck specs needed for Phase 1
4. note types
5. notes
6. media registry
7. build defaults

`Project` also owns or derives the normalization defaults used by `normalize()` and `build()`. Phase 1 should use the existing strict default media policy unless `BuildOptions` explicitly overrides it. Media-related paths such as `base_dir` and `media_store_dir` are derived from `BuildOptions` and the artifact/output location during `build()`, and from project defaults during `normalize()`.

`BuildOptions` should at minimum cover:

```rust
pub struct BuildOptions {
    pub output: Option<PathBuf>,
    pub artifacts_dir: Option<PathBuf>,
    pub normalize_options: Option<ProjectNormalizeOptions>,
    pub inspect: bool,
}
```

`ProjectNormalizeOptions` is a product-facing wrapper over the lower-level `NormalizeOptions` concepts. It should not expose every internal field prematurely, but it must make the source of `base_dir`, `media_store_dir`, and media policy explicit enough that `Project::normalize()` has deterministic behavior outside `build()`.

`Project` exposes:

```rust
impl Project {
    pub fn new(name: impl Into<String>) -> Self;
    pub fn stable_id(self, stable_id: impl Into<String>) -> Self;
    pub fn default_deck(self, deck_name: impl Into<String>) -> Self;
    pub fn add_notetype(&mut self, note_type: NoteType) -> Result<&mut Self>;
    pub fn add_note(&mut self, note: Note) -> Result<&mut Self>;
    pub fn media_mut(&mut self) -> &mut MediaRegistry;
    pub fn validate(&self) -> ValidationReport;
    pub fn lower(&self) -> Result<LoweringPlan>;
    pub fn normalize(&self) -> Result<NormalizedIr>;
    pub fn build(&self, options: BuildOptions) -> Result<BuildReport, BuildError>;
    pub fn write_apkg(&self, path: impl AsRef<Path>) -> Result<BuildReport, BuildError>;
}
```

`Project` can lower to existing `ProductDocument` first. That keeps Phase 1 focused and avoids rewriting the lower-level product bridge while the public API settles.

## Deck Facade

`Deck` remains the shortest path for new users, but it is no longer an independent build model.

Required equivalence:

```text
Deck::write_apkg(path)
  == Project::from(deck).write_apkg(path)

Deck::build(options)
  == Project::from(deck).build(options)
```

Parity tests must compare:

1. note/card/media counts
2. diagnostic codes
3. warning count
4. inspect summary
5. identity lowering result
6. media lowering result

Artifact paths can differ when different output paths are requested. Semantic report fields must match.

The current `deck::export::BuildResult` should be retired from the public happy path or bridged into `BuildReport`. Existing `to_apkg_bytes()` can stay as convenience if it delegates through the same `Project` build path.

Existing `Deck::image_occlusion()` support remains in Phase 1. `Project::from(deck)` must convert `DeckNote::ImageOcclusion(IoNote)` through the existing ProductDocument Image Occlusion lowering path, so deck-authored IO notes continue to build. A new `Note::image_occlusion()` Product API constructor is not part of Phase 1.

## Custom Note Type MVP

Phase 1 must make custom note types safe enough to use publicly.

Public API:

```rust
let vocab = NoteType::custom("jp-vocab")
    .name("Japanese Vocabulary")
    .field(Field::new("Expression").key("expr").identity().sort())
    .field(Field::new("Meaning").key("meaning"))
    .template(
        Template::new("Recognition")
            .key("recognition")
            .front("{{Expression}}")
            .back("{{FrontSide}}<hr id=\"answer\">{{Meaning}}")
            .generate_when(GenerationRule::all(["expr"]))
    )
    .identity(IdentityRecipe::fields(["expr"]));
```

Phase 1 custom fields require stable keys:

```rust
pub struct Field {
    key: FieldKey,
    name: String,
    identity: bool,
    sort: bool,
    required: bool,
}
```

Phase 1 custom templates require stable keys:

```rust
pub struct Template {
    key: TemplateKey,
    name: String,
    front: TemplateSource,
    back: TemplateSource,
    generation_rule: GenerationRule,
}
```

Stable merge id derivation:

```text
field.config_id = stable_i64("field", note_type_id, field.key)
template.config_id = stable_i64("template", note_type_id, template.key)
```

The exact derivation function must be deterministic, documented, and snapshot-tested. The recommended Phase 1 implementation is:

```text
payload = namespace + "\0" + note_type_id + "\0" + key
digest = blake3(payload)
config_id = positive signed i64 from the first 8 digest bytes, with the sign bit cleared
```

This rule must be treated as public behavior once APKGs are generated from it.

Phase 1 custom note types are normal note types. Their lowered `notetype.kind` is fixed to `"normal"`; cloze custom note types are out of scope for this phase. The stock cloze path remains available through `Note::cloze(...)` and existing `Deck::cloze()` support.

Phase 1 `GenerationRule`:

```rust
pub enum GenerationRule {
    AnkiDefault,
    All(Vec<FieldKey>),
    Any(Vec<FieldKey>),
    Cloze { field: FieldKey },
}
```

Rules:

1. `AnkiDefault` leaves normal Anki front-template generation semantics intact.
2. `All` and `Any` must lower to Anki-compatible front template behavior and be visible in snapshots.
3. `Cloze` is limited to the stock cloze path in Phase 1 and is not exposed on custom normal note types.
4. Unsupported or contradictory rules produce structured diagnostics with source paths.

## Notes And Content

`Note` expresses note intent, not card intent.

Required Phase 1 API:

```rust
Note::basic("hola", "hello");

Note::cloze("La capital de Espana es {{c1::Madrid}}")
    .extra("Europe");

Note::new("jp-vocab")
    .stable_id("jp-vocab:taberu")
    .text("expr", "食べる")
    .text("meaning", "to eat");
```

`Content` defaults to safe text:

```rust
pub enum Content {
    Text(String),
    Html(String),
    Media(MediaRef),
    Composite(Vec<Content>),
}
```

Rules:

1. `text()` HTML-escapes user text.
2. `html()` preserves explicit raw HTML.
3. media helpers produce Anki-compatible field content.
4. Markdown is out of Phase 1.

## Minimal Media Helpers

Phase 1 adds product-level media ergonomics on top of the existing media pipeline.

Required API:

```rust
let audio = project
    .media_mut()
    .add_file("media/hola.mp3")?
    .export_as("hola.mp3")?;

project.add_note(
    Note::basic("hola", "hello")
        .stable_id("es:hola")
        .sound("Audio", audio)
)?;
```

Required behavior:

1. `add_file`
2. `add_bytes`
3. `export_as`
4. `MediaRef::sound()`
5. `MediaRef::image()`
6. `Note::sound(field, ref)`
7. `Note::image(field, ref)`
8. media count in `BuildReport`
9. existing normalization media diagnostics mapped into `BuildReport`

Phase 1 does not need full production media productization. Hash dedupe, collision policy polish, unknown/unused media scanning across templates/CSS, and pretty media summaries are Phase 2 work, unless existing lower layers already provide them cheaply.

## Diagnostics

Diagnostics must be structured and user-facing.

Required shape:

```rust
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub severity: Severity,
    pub message: String,
    pub source: Option<SourcePath>,
    pub help: Option<String>,
}
```

Phase 1 source paths must point back to Product API objects where possible:

```text
project.note_types["jp-vocab"].fields["expr"]
project.note_types["jp-vocab"].templates["recognition"]
project.notes["jp-vocab:taberu"].fields["audio"]
project.media["hola.mp3"]
```

Required Phase 1 diagnostic areas:

1. duplicate stable ids
2. custom note type missing identity recipe as alpha warning
3. generation rule references missing field
4. unsupported generation rule lowering
5. media normalization errors
6. build/writer errors

## Testing And Oracle Gates

Phase 1 test coverage must be product-facing, not only lower-layer fixtures.

Required test groups:

```text
anki_forge/tests/project_api_tests.rs
anki_forge/tests/build_report_tests.rs
anki_forge/tests/deck_project_facade_tests.rs
anki_forge/tests/custom_notetype_api_tests.rs
anki_forge/tests/custom_merge_id_snapshot_tests.rs
anki_forge/tests/project_media_api_tests.rs
```

Required hard gates:

1. `Project + Basic note` writes APKG and returns `BuildReport`.
2. `Project + Custom note type + named fields` normalizes and writes APKG.
3. `Project + MediaRef + sound/image helper` writes APKG with media count.
4. `Deck::build()` and `Project::from(deck).build()` produce equivalent semantic reports.
5. FieldKey/TemplateKey config id derivation has stable snapshot values.
6. Basic and Cloze behavior has an oracle or existing manual scenario reference.
7. minimal MediaRef sound/image behavior has snapshot/oracle evidence.

Existing manual validation scenarios can satisfy oracle gates when they are referenced from the Phase 1 exit evidence.

## Docs, Examples, And Python Shape

README order must change to match the target mental model:

```text
Deck quick entry
Project long-term entry
BuildReport / diagnostics
media / identity
IR / contract / oracle
```

Required examples:

```text
examples/target_api/basic.rs
examples/target_api/custom_notetype.rs
examples/target_api/media.rs
```

Python shape spike:

```text
bindings/python/examples/target_api_custom.py
```

The Python spike does not need a full binding implementation, but it must prove:

1. the Rust API shape does not force awkward Python chaining
2. diagnostics can become structured exceptions
3. wheel/maturin build strategy is documented
4. Python media and note type syntax remains natural

## Execution Order

Recommended implementation sequence:

1. Add `build` module and `BuildReport` basic wrapper.
2. Add `Project` with Basic note end-to-end build.
3. Convert `Deck` build/write to delegate through `Project::from(deck)`.
4. Add custom `NoteType`, `Field`, `Template`, `GenerationRule`, `Note::new`, and stable config id derivation.
5. Add product-level `Content` and media helpers.
6. Add README, examples, Python shape spike, and Phase 1 exit evidence.

Each task should use TDD and produce a passing test before moving on to the next task.

## Out Of Scope

Phase 1 does not include:

1. full artifact diff
2. full semantic diff
3. full import risk report
4. `fail_on` risk policy enforcement
5. identity lockfile
6. production media collision policy polish
7. Markdown authoring
8. full Python package release
9. YAML/JSON declarative project format
10. APKG import back into Project
11. a new `Note::image_occlusion()` Product API constructor; existing `Deck::image_occlusion()` remains supported through `Project::from(deck)`

These belong to later phases in `docs/api-design.md`.

## Completion Criteria

Phase 1 is complete only when all of these are true:

1. New users can write a basic deck in 10 lines or fewer.
2. Long-term users can create a `Project`, add notes, write APKG, and inspect `BuildReport`.
3. `Deck` build/write routes through `Project`.
4. custom fields and templates have stable keys and stable config ids.
5. basic media sound/image helpers require no raw Anki media markup.
6. text content is safe by default.
7. build failures expose structured diagnostics through `BuildError.report`.
8. README and examples teach `Deck` first, `Project` second, and IR later.
9. Basic, Cloze, FieldKey/TemplateKey, and MediaRef have snapshot/oracle evidence.
10. Python API shape and diagnostics exception spike are documented.
