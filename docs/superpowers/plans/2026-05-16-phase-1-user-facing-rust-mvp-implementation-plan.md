# Phase 1 User-Facing Rust MVP Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver the strict Phase 1 Rust Product API so users can build APKG files through `Project` or the quick `Deck` facade, receive a structured `BuildReport`, use custom note types with stable field/template merge ids, and use minimal media helpers without touching IR.

**Architecture:** Add public `prelude`, `build`, `diagnostics`, and expanded `product` modules above the existing `ProductDocument -> AuthoringDocument -> NormalizedIr -> writer_core` pipeline. `Project` owns user intent and lowers to existing product/authoring contracts; `Deck` delegates build/write to `Project::from(deck)` so all public paths share one implementation. `BuildReport` wraps normalized counts, diagnostics, writer results, and inspect summaries while full diff/risk remains later-phase work.

**Tech Stack:** Rust workspace (`cargo`, `anyhow`, `serde`, `tempfile`, `blake3`, `writer_core`, `authoring_core`), existing media CAS pipeline, product-facing integration tests, README/examples, Python shape sketch

---

## Scope Check

This plan implements one cohesive subsystem: the strict Phase 1 user-facing Rust MVP approved in `docs/superpowers/specs/2026-05-16-phase-1-user-facing-rust-mvp-design.md`.

The plan includes:

1. `Project`, `Note`, `NoteType`, `Field`, `Template`, `GenerationRule`, `Content`, product `MediaRegistry`, and identity recipe API.
2. `BuildOptions`, `BuildReport`, `BuildError`, diagnostics, counts, metrics, artifact path, and inspect summary.
3. `Deck` build/write delegation through `Project::from(deck)`, preserving existing Basic, Cloze, and Image Occlusion deck support.
4. custom normal note types with stable `FieldKey` / `TemplateKey` config id derivation.
5. minimal media helpers: `add_file`, `add_bytes`, `export_as`, `MediaRef::sound`, `MediaRef::image`, `Note::sound`, `Note::image`.
6. README first-screen order, runnable examples, Python shape spike, and exit evidence.

The plan excludes:

1. full diff/risk/CI policy enforcement
2. identity lockfile
3. custom cloze note types
4. new `Note::image_occlusion()` Product API
5. production media collision policy polish beyond existing lower-layer behavior
6. Python wheel release
7. declarative YAML/JSON project format

## Execution Prerequisite

Before implementing, create a dedicated worktree from the current branch:

```bash
git worktree add ../anki-forge-phase-1-rust-mvp -b codex/phase-1-rust-mvp
cd ../anki-forge-phase-1-rust-mvp
```

Run every task in that worktree. Keep the original `docs/api-design.md` draft untouched unless the user explicitly asks to track it.

## File Structure Map

### Public API modules

- Create: `anki_forge/src/prelude.rs` - curated stable imports for examples and users.
- Create: `anki_forge/src/build/mod.rs` - build module exports.
- Create: `anki_forge/src/build/options.rs` - `BuildOptions`, `ProjectNormalizeOptions`, path derivation helpers.
- Create: `anki_forge/src/build/report.rs` - `BuildReport`, `BuildError`, counts, metrics, artifact, inspect summary.
- Create: `anki_forge/src/diagnostics/mod.rs` - stable diagnostic type, code, severity, source path.
- Modify: `anki_forge/src/lib.rs` - expose `prelude`, `build`, `diagnostics`, product API, and move low-level re-exports into `authoring` / `writer` modules.

### Product API

- Modify: `anki_forge/src/product/mod.rs` - export existing DTO bridge plus new Product API types.
- Create: `anki_forge/src/product/project.rs` - `Project`, `Project::from(Deck)`, validate/lower/normalize/build/write logic.
- Create: `anki_forge/src/product/notetype.rs` - `NoteType`, `Field`, `FieldKey`, `IdentityRecipe`.
- Create: `anki_forge/src/product/template.rs` - `Template`, `TemplateKey`, `TemplateSource`, `GenerationRule`, stable config id.
- Create: `anki_forge/src/product/note.rs` - `Note`, note kinds, named field map, tags, stable id, stock constructors.
- Create: `anki_forge/src/product/content.rs` - `Content`, HTML escaping/rendering helpers.
- Create: `anki_forge/src/product/media_registry.rs` - product `MediaRegistry`, product `MediaRef`, pending export name builder.
- Create: `anki_forge/src/product/identity.rs` - product identity recipe and alpha custom identity warnings.
- Modify: `anki_forge/src/product/model.rs` - add `key` fields to `CustomField` and `CustomTemplate` while preserving existing product DTO shape where possible.
- Modify: `anki_forge/src/product/lowering.rs` - lower custom field/template keys to deterministic config ids and generation-rule-modified front templates.
- Modify: `anki_forge/src/product/builders.rs` - add constructors/builders for keyed custom fields/templates used by compatibility tests.

### Deck facade

- Modify: `anki_forge/src/deck/export.rs` - replace public build/write implementation with delegation to `Project::from(deck)`.
- Modify: `anki_forge/src/deck/lowering.rs` - keep internal conversion helpers reusable for `Project::from(deck)`.
- Modify: `anki_forge/src/deck/mod.rs` - export compatibility aliases if needed while `BuildResult` is retired or bridged.
- Modify: `anki_forge/examples/deck_basic_flow.rs` - consume `BuildReport` from `write_apkg`.

### Tests and docs

- Create: `anki_forge/tests/project_api_tests.rs`
- Create: `anki_forge/tests/build_report_tests.rs`
- Create: `anki_forge/tests/deck_project_facade_tests.rs`
- Create: `anki_forge/tests/custom_notetype_api_tests.rs`
- Create: `anki_forge/tests/custom_merge_id_snapshot_tests.rs`
- Create: `anki_forge/tests/project_media_api_tests.rs`
- Modify: `anki_forge/tests/product_lowering_tests.rs`
- Modify: `anki_forge/tests/deck_export_tests.rs`
- Create: `anki_forge/examples/target_api_basic.rs`
- Create: `anki_forge/examples/target_api_custom_notetype.rs`
- Create: `anki_forge/examples/target_api_media.rs`
- Create: `bindings/python/examples/target_api_custom.py`
- Modify: `bindings/python/README.md`
- Modify: `README.md`
- Create: `docs/superpowers/checklists/phase-1-user-facing-rust-mvp-exit-evidence.md`

## Shared Implementation Decisions

1. `Project::build()` writes into a temporary artifacts directory when the caller only supplies `output`, then copies the built APKG to `output`.
2. `Project::normalize()` uses deterministic project-owned normalization paths under a temp directory when the caller does not provide `ProjectNormalizeOptions`. The returned `NormalizedIr` must not contain absolute temp paths.
3. `BuildOptions.normalize_options` must be consumed in `Project::build()`. If the caller provides `base_dir`, `media_store_dir`, or media policy, those values are used instead of the default build-derived paths.
4. `BuildReport.counts.notes` is `normalized_ir.notes.len()`.
5. `BuildReport.counts.media` is `normalized_ir.media_bindings.len()`.
6. `BuildReport.counts.cards` prefers APKG inspect observations when `inspect` is enabled. Only when inspect is disabled or unavailable does it fall back to Phase 1 approximation: normal note card count comes from non-empty generated templates, and stock cloze count comes from distinct `{{cN::...}}` ordinals in cloze text.
7. `BuildReport.diagnostics` combines project validation diagnostics, lowering diagnostics, normalization diagnostics, and writer diagnostics.
8. `Project::validate()` must exist in Phase 1 and must at least report duplicate stable ids, auto-derived custom field keys, and custom note types missing identity recipes.
9. `BuildReport.ensure_success()` returns `Ok(())` only when an artifact path exists, no diagnostic has error severity, and the build status is `success`.
10. `stable_config_id(namespace, note_type_id, key)` uses BLAKE3 over `namespace + "\0" + note_type_id + "\0" + key`, takes the first eight bytes as big-endian `i64`, and clears the sign bit with `raw & i64::MAX`.
11. Phase 1 custom note types lower as `kind = "normal"`.
12. `GenerationRule::Cloze` is rejected for custom normal note types with diagnostic `TEMPLATE.CLOZE_RULE_REQUIRES_STOCK_CLOZE`.
13. Existing `Deck::image_occlusion()` remains supported only through `Project::from(deck)`; no new Product `Note::image_occlusion()` is added.

## Task 0: Codebase API Audit

**Files:**
- Read: `anki_forge/src/lib.rs`
- Read: `anki_forge/src/product/builders.rs`
- Read: `anki_forge/src/deck/model.rs`
- Read: `anki_forge/src/deck/lowering.rs`
- Read: `anki_forge/src/deck/media.rs`
- Read: `writer_core/src/build.rs`
- Read: `writer_core/src/inspect.rs`

- [ ] **Step 1: Verify crate-root and runtime assumptions**

Run:

```bash
rg -n "pub use writer_core::|pub fn build\\(|pub fn inspect_apkg\\(|pub fn load_default_writer_stack" anki_forge/src writer_core/src
```

Expected: output includes:

```text
anki_forge/src/lib.rs: pub use writer_core::{ ... build, ... inspect_apkg, ... }
anki_forge/src/runtime/defaults.rs: pub fn load_default_writer_stack(...)
writer_core/src/build.rs: pub fn build(...)
writer_core/src/inspect.rs: pub fn inspect_apkg(...)
```

- [ ] **Step 2: Verify ProductDocument builder assumptions**

Run:

```bash
rg -n "with_custom_notetype|add_custom_note|add_basic_note_with_tags|add_cloze_note_with_tags" anki_forge/src/product
```

Expected: output confirms these public methods exist in `anki_forge/src/product/builders.rs`.

- [ ] **Step 3: Verify Deck facade assumptions**

Run:

```bash
rg -n "pub fn new\\(name|pub fn name\\(&self\\)|pub fn stable_id\\(&self\\)|into_product_document|to_authoring_media" anki_forge/src/deck
```

Expected: output confirms `Deck::new`, `Deck::name`, `Deck::stable_id`, `Deck::into_product_document`, and `RegisteredMedia::to_authoring_media` exist.

- [ ] **Step 4: Record audit result in the implementation branch**

If any expected method is missing or has an incompatible signature, update this plan before starting Task 1. If all methods are present, commit no code for Task 0 and proceed to Task 1.

## Task 1: Public Module Skeleton And BuildReport Types

**Files:**
- Create: `anki_forge/src/prelude.rs`
- Create: `anki_forge/src/build/mod.rs`
- Create: `anki_forge/src/build/options.rs`
- Create: `anki_forge/src/build/report.rs`
- Create: `anki_forge/src/diagnostics/mod.rs`
- Modify: `anki_forge/src/lib.rs`
- Test: `anki_forge/tests/build_report_tests.rs`

- [ ] **Step 1: Write failing BuildReport unit tests**

```rust
// anki_forge/tests/build_report_tests.rs
use std::path::PathBuf;
use std::time::Duration;

use anki_forge::build::{
    ApkgArtifact, BuildCounts, BuildFailureCause, BuildMetrics, BuildReport,
};
use anki_forge::diagnostics::{Diagnostic, DiagnosticCode, Severity};

#[test]
fn build_report_ensure_success_accepts_successful_artifact() {
    let report = BuildReport {
        artifact: Some(ApkgArtifact {
            path: PathBuf::from("out/spanish.apkg"),
        }),
        counts: BuildCounts {
            notes: 2,
            cards: 2,
            media: 0,
        },
        diagnostics: vec![],
        metrics: BuildMetrics {
            duration: Duration::from_millis(25),
        },
        inspect: None,
        status: "success".into(),
    };

    report.ensure_success().expect("successful report");
    assert_eq!(report.warning_count(), 0);
    assert_eq!(report.diagnostic_codes(), Vec::<String>::new());
}

#[test]
fn build_report_ensure_success_rejects_error_diagnostic() {
    let report = BuildReport {
        artifact: Some(ApkgArtifact {
            path: PathBuf::from("out/spanish.apkg"),
        }),
        counts: BuildCounts {
            notes: 1,
            cards: 1,
            media: 0,
        },
        diagnostics: vec![Diagnostic {
            code: DiagnosticCode::new("MEDIA.MISSING_REFERENCE"),
            severity: Severity::Error,
            message: "missing media reference hola.mp3".into(),
            source: None,
            help: Some("register the media before adding the note".into()),
        }],
        metrics: BuildMetrics {
            duration: Duration::from_millis(1),
        },
        inspect: None,
        status: "invalid".into(),
    };

    let err = report.ensure_success().expect_err("report should fail");
    assert_eq!(err.cause, BuildFailureCause::Diagnostics);
    assert_eq!(err.report.diagnostic_codes(), vec!["MEDIA.MISSING_REFERENCE"]);
}
```

- [ ] **Step 2: Run the failing tests**

Run:

```bash
cargo test -p anki_forge --test build_report_tests -v
```

Expected: FAIL with unresolved imports for `anki_forge::build` and `anki_forge::diagnostics`.

- [ ] **Step 3: Add diagnostics types**

```rust
// anki_forge/src/diagnostics/mod.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticCode(String);

impl DiagnosticCode {
    pub fn new(code: impl Into<String>) -> Self {
        Self(code.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePath(String);

impl SourcePath {
    pub fn new(path: impl Into<String>) -> Self {
        Self(path.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub severity: Severity,
    pub message: String,
    pub source: Option<SourcePath>,
    pub help: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValidationReport {
    pub diagnostics: Vec<Diagnostic>,
}

impl ValidationReport {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == Severity::Error)
    }
}
```

- [ ] **Step 4: Add BuildOptions and BuildReport**

```rust
// anki_forge/src/build/mod.rs
pub mod options;
pub mod report;

pub use options::{BuildOptions, ProjectMediaPolicy, ProjectNormalizeOptions};
pub use report::{
    ApkgArtifact, BuildCounts, BuildError, BuildFailureCause, BuildMetrics, BuildReport,
    InspectSummary,
};
```

```rust
// anki_forge/src/build/options.rs
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ProjectMediaPolicy {
    #[default]
    Strict,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProjectNormalizeOptions {
    pub base_dir: Option<PathBuf>,
    pub media_store_dir: Option<PathBuf>,
    pub media_policy: ProjectMediaPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildOptions {
    pub output: Option<PathBuf>,
    pub artifacts_dir: Option<PathBuf>,
    pub normalize_options: Option<ProjectNormalizeOptions>,
    pub inspect: bool,
}

impl Default for BuildOptions {
    fn default() -> Self {
        Self {
            output: None,
            artifacts_dir: None,
            normalize_options: None,
            inspect: true,
        }
    }
}

impl BuildOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn output(mut self, path: impl Into<PathBuf>) -> Self {
        self.output = Some(path.into());
        self
    }

    pub fn artifacts_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.artifacts_dir = Some(path.into());
        self
    }

    pub fn normalize_options(mut self, options: ProjectNormalizeOptions) -> Self {
        self.normalize_options = Some(options);
        self
    }

    pub fn inspect(mut self, inspect: bool) -> Self {
        self.inspect = inspect;
        self
    }
}

impl ProjectNormalizeOptions {
    pub fn strict() -> Self {
        Self {
            base_dir: None,
            media_store_dir: None,
            media_policy: ProjectMediaPolicy::Strict,
        }
    }

    pub fn base_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.base_dir = Some(path.into());
        self
    }

    pub fn media_store_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.media_store_dir = Some(path.into());
        self
    }

    pub(crate) fn to_authoring_media_policy(&self) -> authoring_core::MediaPolicy {
        match self.media_policy {
            ProjectMediaPolicy::Strict => authoring_core::MediaPolicy::default_strict(),
        }
    }
}
```

```rust
// anki_forge/src/build/report.rs
use std::path::PathBuf;
use std::time::Duration;

use crate::diagnostics::{Diagnostic, Severity};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApkgArtifact {
    pub path: PathBuf,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BuildCounts {
    pub notes: usize,
    pub cards: usize,
    pub media: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildMetrics {
    pub duration: Duration,
}

impl Default for BuildMetrics {
    fn default() -> Self {
        Self {
            duration: Duration::ZERO,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InspectSummary {
    pub source_kind: String,
    pub observation_status: String,
    pub notes: usize,
    pub cards: usize,
    pub notetypes: usize,
    pub templates: usize,
    pub fields: usize,
    pub media: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildReport {
    pub artifact: Option<ApkgArtifact>,
    pub counts: BuildCounts,
    pub diagnostics: Vec<Diagnostic>,
    pub metrics: BuildMetrics,
    pub inspect: Option<InspectSummary>,
    pub status: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildFailureCause {
    MissingArtifact,
    Diagnostics,
    BuildStatus,
    Io,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildError {
    pub report: BuildReport,
    pub cause: BuildFailureCause,
}

impl BuildReport {
    pub fn ensure_success(&self) -> Result<(), BuildError> {
        if self.artifact.is_none() {
            return Err(BuildError {
                report: self.clone(),
                cause: BuildFailureCause::MissingArtifact,
            });
        }

        if self
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == Severity::Error)
        {
            return Err(BuildError {
                report: self.clone(),
                cause: BuildFailureCause::Diagnostics,
            });
        }

        if self.status != "success" {
            return Err(BuildError {
                report: self.clone(),
                cause: BuildFailureCause::BuildStatus,
            });
        }

        Ok(())
    }

    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == Severity::Warning)
            .count()
    }

    pub fn diagnostic_codes(&self) -> Vec<String> {
        self.diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.as_str().to_string())
            .collect()
    }
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "build failed: {:?}", self.cause)
    }
}

impl std::error::Error for BuildError {}
```

- [ ] **Step 5: Export modules from lib.rs**

```rust
// anki_forge/src/lib.rs
mod deck;

pub mod build;
pub mod diagnostics;
pub mod prelude;
pub mod product;
pub mod runtime;

pub use deck::*;
```

Keep the existing `authoring_core` and `writer_core` re-exports for this step so old tests still compile. Moving them behind `authoring` / `writer` modules happens in Task 8.

```rust
// anki_forge/src/prelude.rs
pub use crate::build::{BuildOptions, BuildReport};
pub use crate::deck::Deck;
pub use crate::diagnostics::{Diagnostic, DiagnosticCode, Severity, SourcePath, ValidationReport};
```

- [ ] **Step 6: Run tests for Task 1**

Run:

```bash
cargo test -p anki_forge --test build_report_tests -v
```

Expected: PASS.

- [ ] **Step 7: Commit Task 1**

```bash
git add anki_forge/src/prelude.rs anki_forge/src/build anki_forge/src/diagnostics anki_forge/src/lib.rs anki_forge/tests/build_report_tests.rs
git commit -m "feat: add build report foundation"
```

## Task 2: Product Note, Content, NoteType, Template, And Identity Types

**Files:**
- Create: `anki_forge/src/product/content.rs`
- Create: `anki_forge/src/product/identity.rs`
- Create: `anki_forge/src/product/note.rs`
- Create: `anki_forge/src/product/notetype.rs`
- Create: `anki_forge/src/product/template.rs`
- Modify: `anki_forge/src/product/mod.rs`
- Modify: `anki_forge/src/prelude.rs`
- Test: `anki_forge/tests/project_api_tests.rs`
- Test: `anki_forge/tests/custom_notetype_api_tests.rs`

- [ ] **Step 1: Write failing product API tests**

```rust
// anki_forge/tests/project_api_tests.rs
use anki_forge::prelude::*;

#[test]
fn note_basic_constructor_uses_stock_basic_fields() {
    let note = Note::basic("AT&T", "<b>phone</b>").stable_id("basic:att");

    assert_eq!(note.stable_id(), Some("basic:att"));
    assert_eq!(note.note_type_id(), "basic");
    assert_eq!(
        note.rendered_fields().get("Front").map(String::as_str),
        Some("AT&amp;T")
    );
    assert_eq!(
        note.rendered_fields().get("Back").map(String::as_str),
        Some("&lt;b&gt;phone&lt;/b&gt;")
    );
}

#[test]
fn note_html_constructor_preserves_raw_html() {
    let note = Note::new("custom")
        .stable_id("custom:1")
        .text("question", "AT&T")
        .html("answer", "<b>Bell</b>");

    assert_eq!(
        note.rendered_fields().get("question").map(String::as_str),
        Some("AT&amp;T")
    );
    assert_eq!(
        note.rendered_fields().get("answer").map(String::as_str),
        Some("<b>Bell</b>")
    );
}
```

```rust
// anki_forge/tests/custom_notetype_api_tests.rs
use anki_forge::prelude::*;

#[test]
fn custom_notetype_builder_records_keys_and_identity_recipe() {
    let vocab = NoteType::custom("jp-vocab")
        .name("Japanese Vocabulary")
        .field(Field::new("Expression").key("expr").identity().sort())
        .field(Field::new("Meaning").key("meaning").required())
        .field(Field::new("Audio").key("audio").optional())
        .template(
            Template::new("Recognition")
                .key("recognition")
                .front("{{Expression}}")
                .back("{{FrontSide}}<hr id=\"answer\">{{Meaning}}")
                .browser_front("{{Expression}}")
                .browser_back("{{Meaning}}")
                .target_deck("Japanese::Recognition")
                .generate_when(GenerationRule::all(["expr"])),
        )
        .identity(IdentityRecipe::fields(["expr"]));

    assert_eq!(vocab.id(), "jp-vocab");
    assert_eq!(vocab.name(), Some("Japanese Vocabulary"));
    assert_eq!(vocab.fields()[0].key_ref().as_str(), "expr");
    assert!(vocab.fields()[0].is_identity());
    assert!(vocab.fields()[0].is_sort());
    assert!(vocab.fields()[1].is_required());
    assert!(vocab.fields()[2].is_optional());
    assert_eq!(vocab.templates()[0].key_ref().as_str(), "recognition");
    assert_eq!(
        vocab.templates()[0].browser_front().map(|source| source.as_str()),
        Some("{{Expression}}")
    );
    assert_eq!(
        vocab.templates()[0].target_deck(),
        Some("Japanese::Recognition")
    );
    assert_eq!(
        vocab.identity().expect("identity").field_keys(),
        vec![FieldKey::new("expr")]
    );
}
```

- [ ] **Step 2: Run failing product API tests**

Run:

```bash
cargo test -p anki_forge --test project_api_tests --test custom_notetype_api_tests -v
```

Expected: FAIL with unresolved `Note`, `NoteType`, `Field`, `Template`, `GenerationRule`, and `IdentityRecipe`.

- [ ] **Step 3: Add Content rendering**

```rust
// anki_forge/src/product/content.rs
use super::media_registry::MediaRef;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Content {
    Text(String),
    Html(String),
    Media(MediaRef),
    Composite(Vec<Content>),
}

impl Content {
    pub fn text(value: impl Into<String>) -> Self {
        Self::Text(value.into())
    }

    pub fn html(value: impl Into<String>) -> Self {
        Self::Html(value.into())
    }

    pub fn render(&self) -> String {
        match self {
            Self::Text(value) => escape_html(value),
            Self::Html(value) => value.clone(),
            Self::Media(media) => media.filename().to_string(),
            Self::Composite(items) => items.iter().map(Self::render).collect::<String>(),
        }
    }
}

pub fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
```

- [ ] **Step 4: Add product media reference shell for Content**

```rust
// anki_forge/src/product/media_registry.rs
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MediaRef {
    filename: String,
}

#[derive(Debug, Clone, Default)]
pub struct MediaRegistry;

impl MediaRef {
    pub(crate) fn new(filename: impl Into<String>) -> Self {
        Self {
            filename: filename.into(),
        }
    }

    pub fn filename(&self) -> &str {
        &self.filename
    }

    pub fn sound(&self) -> crate::product::Content {
        crate::product::Content::Html(format!("[sound:{}]", self.filename))
    }

    pub fn image(&self) -> crate::product::Content {
        crate::product::Content::Html(format!("<img src=\"{}\">", self.filename))
    }
}
```

- [ ] **Step 5: Add identity recipe and stable keys**

```rust
// anki_forge/src/product/identity.rs
use super::notetype::FieldKey;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityRecipe {
    field_keys: Vec<FieldKey>,
}

impl IdentityRecipe {
    pub fn fields<I, S>(fields: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut field_keys = fields
            .into_iter()
            .map(|field| FieldKey::new(field.into()))
            .collect::<Vec<_>>();
        field_keys.sort();
        field_keys.dedup();
        Self { field_keys }
    }

    pub fn field_keys(&self) -> Vec<FieldKey> {
        self.field_keys.clone()
    }
}
```

```rust
// anki_forge/src/product/notetype.rs
use super::{IdentityRecipe, Template};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FieldKey(String);

impl FieldKey {
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    key: FieldKey,
    name: String,
    identity: bool,
    sort: bool,
    required: bool,
    optional: bool,
    key_auto_derived: bool,
}

impl Field {
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            key: FieldKey::new(slug_key(&name)),
            name,
            identity: false,
            sort: false,
            required: false,
            optional: false,
            key_auto_derived: true,
        }
    }

    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = FieldKey::new(key);
        self.key_auto_derived = false;
        self
    }

    pub fn identity(mut self) -> Self {
        self.identity = true;
        self
    }

    pub fn sort(mut self) -> Self {
        self.sort = true;
        self
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self.optional = false;
        self
    }

    pub fn optional(mut self) -> Self {
        self.optional = true;
        self.required = false;
        self
    }

    pub fn key_ref(&self) -> &FieldKey {
        &self.key
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn is_identity(&self) -> bool {
        self.identity
    }

    pub fn is_sort(&self) -> bool {
        self.sort
    }

    pub fn is_required(&self) -> bool {
        self.required
    }

    pub fn is_optional(&self) -> bool {
        self.optional
    }

    pub fn key_auto_derived(&self) -> bool {
        self.key_auto_derived
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteType {
    id: String,
    name: Option<String>,
    fields: Vec<Field>,
    templates: Vec<Template>,
    identity: Option<IdentityRecipe>,
}

impl NoteType {
    pub fn custom(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: None,
            fields: Vec::new(),
            templates: Vec::new(),
            identity: None,
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn field(mut self, field: Field) -> Self {
        self.fields.push(field);
        self
    }

    pub fn template(mut self, template: Template) -> Self {
        self.templates.push(template);
        self
    }

    pub fn identity(mut self, identity: IdentityRecipe) -> Self {
        self.identity = Some(identity);
        self
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn fields(&self) -> &[Field] {
        &self.fields
    }

    pub fn templates(&self) -> &[Template] {
        &self.templates
    }

    pub fn identity(&self) -> Option<&IdentityRecipe> {
        self.identity.as_ref()
    }
}

fn slug_key(name: &str) -> String {
    name.trim()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch.to_ascii_lowercase() } else { '_' })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}
```

- [ ] **Step 6: Add Template and GenerationRule**

```rust
// anki_forge/src/product/template.rs
use super::FieldKey;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TemplateKey(String);

impl TemplateKey {
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateSource(String);

impl TemplateSource {
    pub fn new(source: impl Into<String>) -> Self {
        Self(source.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenerationRule {
    AnkiDefault,
    All(Vec<FieldKey>),
    Any(Vec<FieldKey>),
    Cloze { field: FieldKey },
}

impl GenerationRule {
    pub fn all<I, S>(fields: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::All(fields.into_iter().map(|field| FieldKey::new(field.into())).collect())
    }

    pub fn any<I, S>(fields: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::Any(fields.into_iter().map(|field| FieldKey::new(field.into())).collect())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Template {
    key: TemplateKey,
    name: String,
    front: TemplateSource,
    back: TemplateSource,
    browser_front: Option<TemplateSource>,
    browser_back: Option<TemplateSource>,
    target_deck: Option<String>,
    generation_rule: GenerationRule,
}

impl Template {
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            key: TemplateKey::new(name.to_ascii_lowercase().replace(' ', "_")),
            name,
            front: TemplateSource::new(""),
            back: TemplateSource::new(""),
            browser_front: None,
            browser_back: None,
            target_deck: None,
            generation_rule: GenerationRule::AnkiDefault,
        }
    }

    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.key = TemplateKey::new(key);
        self
    }

    pub fn front(mut self, front: impl Into<String>) -> Self {
        self.front = TemplateSource::new(front);
        self
    }

    pub fn back(mut self, back: impl Into<String>) -> Self {
        self.back = TemplateSource::new(back);
        self
    }

    pub fn browser_front(mut self, source: impl Into<String>) -> Self {
        self.browser_front = Some(TemplateSource::new(source));
        self
    }

    pub fn browser_back(mut self, source: impl Into<String>) -> Self {
        self.browser_back = Some(TemplateSource::new(source));
        self
    }

    pub fn target_deck(mut self, deck_name: impl Into<String>) -> Self {
        self.target_deck = Some(deck_name.into());
        self
    }

    pub fn generate_when(mut self, rule: GenerationRule) -> Self {
        self.generation_rule = rule;
        self
    }

    pub fn key_ref(&self) -> &TemplateKey {
        &self.key
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn front_source(&self) -> &TemplateSource {
        &self.front
    }

    pub fn back_source(&self) -> &TemplateSource {
        &self.back
    }

    pub fn browser_front(&self) -> Option<&TemplateSource> {
        self.browser_front.as_ref()
    }

    pub fn browser_back(&self) -> Option<&TemplateSource> {
        self.browser_back.as_ref()
    }

    pub fn target_deck(&self) -> Option<&str> {
        self.target_deck.as_deref()
    }

    pub fn generation_rule(&self) -> &GenerationRule {
        &self.generation_rule
    }
}
```

- [ ] **Step 7: Add Note type**

```rust
// anki_forge/src/product/note.rs
use std::collections::BTreeMap;

use super::{Content, MediaRef};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    note_type_id: String,
    stable_id: Option<String>,
    deck_name: Option<String>,
    fields: BTreeMap<String, Content>,
    tags: Vec<String>,
}

impl Note {
    pub fn new(note_type_id: impl Into<String>) -> Self {
        Self {
            note_type_id: note_type_id.into(),
            stable_id: None,
            deck_name: None,
            fields: BTreeMap::new(),
            tags: Vec::new(),
        }
    }

    pub fn basic(front: impl Into<String>, back: impl Into<String>) -> Self {
        Self::new("basic").text("Front", front).text("Back", back)
    }

    pub fn cloze(text: impl Into<String>) -> Self {
        Self::new("cloze").html("Text", text).text("Back Extra", "")
    }

    pub fn stable_id(mut self, stable_id: impl Into<String>) -> Self {
        self.stable_id = Some(stable_id.into());
        self
    }

    pub fn deck(mut self, deck_name: impl Into<String>) -> Self {
        self.deck_name = Some(deck_name.into());
        self
    }

    pub fn text(mut self, field: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.insert(field.into(), Content::text(value));
        self
    }

    pub fn html(mut self, field: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.insert(field.into(), Content::html(value));
        self
    }

    pub fn sound(mut self, field: impl Into<String>, media: MediaRef) -> Self {
        self.fields.insert(field.into(), media.sound());
        self
    }

    pub fn image(mut self, field: impl Into<String>, media: MediaRef) -> Self {
        self.fields.insert(field.into(), media.image());
        self
    }

    pub fn extra(mut self, extra: impl Into<String>) -> Self {
        self.fields.insert("Back Extra".into(), Content::text(extra));
        self
    }

    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    pub fn note_type_id(&self) -> &str {
        &self.note_type_id
    }

    pub fn stable_id(&self) -> Option<&str> {
        self.stable_id.as_deref()
    }

    pub fn deck_name(&self) -> Option<&str> {
        self.deck_name.as_deref()
    }

    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    pub fn rendered_fields(&self) -> BTreeMap<String, String> {
        self.fields
            .iter()
            .map(|(field, content)| (field.clone(), content.render()))
            .collect()
    }
}
```

`Note::cloze(...)` intentionally stores the cloze Text field as explicit HTML, because Anki must see raw `{{cN::...}}` markers. This is the one Phase 1 stock constructor that does not apply `Content::Text` escaping to its primary field; document that behavior in README and examples so users do not assume cloze text is escaped like `Note::basic(...)`.

- [ ] **Step 8: Wire product exports and prelude**

```rust
// anki_forge/src/product/mod.rs
pub mod assets;
pub mod builders;
pub mod content;
pub mod diagnostics;
pub mod helpers;
pub mod identity;
pub mod lowering;
pub mod media_registry;
pub mod metadata;
pub mod model;
pub mod note;
pub mod notetype;
pub mod stock;
pub mod template;

pub use content::Content;
pub use identity::IdentityRecipe;
pub use media_registry::{MediaRef, MediaRegistry};
pub use note::Note;
pub use notetype::{Field, FieldKey, NoteType};
pub use template::{GenerationRule, Template, TemplateKey, TemplateSource};

// keep existing re-exports below
pub use assets::{AssetSource, FontBinding};
pub use diagnostics::{LoweringDiagnostic, ProductDiagnostic, ProductLoweringError};
pub use helpers::HelperDeclaration;
pub use lowering::{LoweringMapping, LoweringPlan};
pub use metadata::{
    FieldMetadataDeclaration, TemplateBrowserAppearanceDeclaration, TemplateTargetDeckDeclaration,
};
pub use model::{
    BasicNoteType, CustomField, CustomNote, CustomNoteType, CustomTemplate, ProductDocument,
    ProductNote, ProductNoteType,
};
pub use stock::{
    render_image_occlusion_cloze, STOCK_BASIC_ID, STOCK_CLOZE_ID, STOCK_IMAGE_OCCLUSION_ID,
};
```

```rust
// anki_forge/src/prelude.rs
pub use crate::build::{BuildOptions, BuildReport};
pub use crate::deck::Deck;
pub use crate::diagnostics::{Diagnostic, DiagnosticCode, Severity, SourcePath, ValidationReport};
pub use crate::product::{
    Content, Field, FieldKey, GenerationRule, IdentityRecipe, MediaRef, Note, NoteType, Template,
    TemplateKey,
};
```

- [ ] **Step 9: Run Task 2 tests**

Run:

```bash
cargo test -p anki_forge --test project_api_tests --test custom_notetype_api_tests -v
```

Expected: PASS.

- [ ] **Step 10: Commit Task 2**

```bash
git add anki_forge/src/product/content.rs anki_forge/src/product/identity.rs anki_forge/src/product/note.rs anki_forge/src/product/notetype.rs anki_forge/src/product/template.rs anki_forge/src/product/media_registry.rs anki_forge/src/product/mod.rs anki_forge/src/prelude.rs anki_forge/tests/project_api_tests.rs anki_forge/tests/custom_notetype_api_tests.rs
git commit -m "feat: add product authoring types"
```

## Task 3: Project Basic Build End-To-End

**Files:**
- Create: `anki_forge/src/product/project.rs`
- Modify: `anki_forge/src/product/mod.rs`
- Modify: `anki_forge/src/prelude.rs`
- Modify: `anki_forge/src/build/report.rs`
- Test: `anki_forge/tests/project_api_tests.rs`
- Test: `anki_forge/tests/build_report_tests.rs`

- [ ] **Step 1: Add failing Project build test**

Append to `anki_forge/tests/project_api_tests.rs`:

```rust
use std::path::PathBuf;

#[test]
fn project_basic_note_writes_apkg_and_returns_report() {
    let root = unique_artifacts_dir("project-basic-build");
    let output = root.join("spanish-a1.apkg");

    let mut project = Project::new("Spanish A1")
        .stable_id("spanish-a1")
        .default_deck("Spanish::A1");

    project
        .add_note(Note::basic("hola", "hello").stable_id("es:hola"))
        .expect("add note");

    let report = project.write_apkg(&output).expect("write apkg");

    report.ensure_success().expect("successful report");
    assert_eq!(report.counts.notes, 1);
    assert_eq!(report.counts.cards, 1);
    assert_eq!(report.counts.media, 0);
    assert_eq!(
        report.artifact.as_ref().map(|artifact| artifact.path.as_path()),
        Some(output.as_path())
    );
    assert!(output.exists());
}

#[test]
fn project_normalize_basic_note_returns_normalized_ir() {
    let mut project = Project::new("Spanish A1")
        .stable_id("spanish-a1")
        .default_deck("Spanish::A1");

    project
        .add_note(Note::basic("hola", "hello").stable_id("es:hola"))
        .expect("add note");

    let normalized = project.normalize().expect("normalize");

    assert_eq!(normalized.document_id, "spanish-a1");
    assert_eq!(normalized.notes.len(), 1);
    assert_eq!(normalized.notetypes.len(), 1);
    assert_eq!(normalized.notes[0].fields.get("Front").map(String::as_str), Some("hola"));
}

#[test]
fn project_validate_reports_duplicate_stable_ids() {
    let mut project = Project::new("Spanish A1")
        .stable_id("spanish-a1")
        .default_deck("Spanish::A1");

    project
        .add_note(Note::basic("hola", "hello").stable_id("dup"))
        .expect("add first note");
    project
        .add_note(Note::basic("adios", "goodbye").stable_id("dup"))
        .expect("add second note");

    let report = project.validate();

    assert!(report.has_errors());
    assert!(report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code.as_str() == "AFID.STABLE_ID_DUPLICATE"));
}

#[test]
fn project_validate_warns_for_auto_derived_custom_field_key() {
    let note_type = NoteType::custom("auto-key")
        .field(Field::new("Expression"))
        .template(Template::new("Card 1").front("{{Expression}}").back("{{Expression}}"));
    let mut project = Project::new("Auto Key")
        .stable_id("auto-key")
        .default_deck("Auto Key");
    project.add_notetype(note_type).expect("add note type");

    let report = project.validate();

    assert!(report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code.as_str() == "NOTETYPE.FIELD_KEY_AUTO_DERIVED"));
}

#[test]
fn project_cloze_card_count_fallback_counts_distinct_ords_when_inspect_disabled() {
    let root = unique_artifacts_dir("project-cloze-no-inspect");
    let mut project = Project::new("Cloze")
        .stable_id("cloze")
        .default_deck("Cloze");
    project
        .add_note(
            Note::cloze("{{c1::Madrid}} is in {{c2::Spain}} and {{c1::Europe}}")
                .stable_id("cloze:1"),
        )
        .expect("add cloze");

    let report = project
        .build(
            BuildOptions::new()
                .output(root.join("cloze.apkg"))
                .inspect(false),
        )
        .expect("build cloze");

    assert_eq!(report.counts.cards, 2);
}

fn unique_artifacts_dir(label: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "anki-forge-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("create temp artifacts dir");
    dir
}
```

- [ ] **Step 2: Run failing Project build test**

Run:

```bash
cargo test -p anki_forge --test project_api_tests project_basic_note_writes_apkg_and_returns_report -v
```

Expected: FAIL with unresolved `Project`.

- [ ] **Step 3: Implement Project state and basic note lowering**

```rust
// anki_forge/src/product/project.rs
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::Context;
use authoring_core::{normalize_with_options, NormalizationRequest, NormalizeOptions};
use writer_core::{artifact_path_from_ref, BuildArtifactTarget};

use crate::build::{
    ApkgArtifact, BuildCounts, BuildError, BuildFailureCause, BuildMetrics, BuildOptions,
    BuildReport, InspectSummary, ProjectNormalizeOptions,
};
use crate::diagnostics::{Diagnostic, DiagnosticCode, Severity, SourcePath, ValidationReport};
use crate::product::{
    LoweringPlan, Note, NoteType, ProductDocument, STOCK_BASIC_ID, STOCK_CLOZE_ID,
};

#[derive(Debug, Clone)]
pub struct Project {
    name: String,
    stable_id: Option<String>,
    default_deck: Option<String>,
    note_types: Vec<NoteType>,
    notes: Vec<Note>,
    media: crate::product::MediaRegistry,
}

impl Project {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            stable_id: None,
            default_deck: None,
            note_types: Vec::new(),
            notes: Vec::new(),
            media: crate::product::MediaRegistry::default(),
        }
    }

    pub fn stable_id(mut self, stable_id: impl Into<String>) -> Self {
        self.stable_id = Some(stable_id.into());
        self
    }

    pub fn default_deck(mut self, deck_name: impl Into<String>) -> Self {
        self.default_deck = Some(deck_name.into());
        self
    }

    pub fn add_notetype(&mut self, note_type: NoteType) -> anyhow::Result<&mut Self> {
        self.note_types.push(note_type);
        Ok(self)
    }

    pub fn add_note(&mut self, note: Note) -> anyhow::Result<&mut Self> {
        self.notes.push(note);
        Ok(self)
    }

    pub fn media_mut(&mut self) -> &mut crate::product::MediaRegistry {
        &mut self.media
    }

    pub fn validate(&self) -> ValidationReport {
        let mut diagnostics = Vec::new();
        let mut seen_stable_ids = std::collections::BTreeSet::new();

        for note in &self.notes {
            if let Some(stable_id) = note.stable_id() {
                if !seen_stable_ids.insert(stable_id.to_string()) {
                    diagnostics.push(Diagnostic {
                        code: DiagnosticCode::new("AFID.STABLE_ID_DUPLICATE"),
                        severity: Severity::Error,
                        message: format!("duplicate stable_id '{stable_id}'"),
                        source: Some(SourcePath::new(format!("project.notes[\"{stable_id}\"]"))),
                        help: Some("choose a unique stable_id for each note".into()),
                    });
                }
            }
        }

        for note_type in &self.note_types {
            if note_type.identity().is_none() {
                diagnostics.push(Diagnostic {
                    code: DiagnosticCode::new("NOTETYPE.IDENTITY_RECIPE_MISSING"),
                    severity: Severity::Warning,
                    message: format!("custom note type '{}' has no identity recipe", note_type.id()),
                    source: Some(SourcePath::new(format!("project.note_types[\"{}\"]", note_type.id()))),
                    help: Some("add IdentityRecipe::fields([...]) before relying on update-safe builds".into()),
                });
            }

            for field in note_type.fields() {
                if field.key_auto_derived() {
                    diagnostics.push(Diagnostic {
                        code: DiagnosticCode::new("NOTETYPE.FIELD_KEY_AUTO_DERIVED"),
                        severity: Severity::Warning,
                        message: format!(
                            "field '{}' in note type '{}' uses an auto-derived key",
                            field.name(),
                            note_type.id()
                        ),
                        source: Some(SourcePath::new(format!(
                            "project.note_types[\"{}\"].fields[\"{}\"]",
                            note_type.id(),
                            field.name()
                        ))),
                        help: Some("call .key(\"stable-field-key\") explicitly".into()),
                    });
                }
            }
        }

        ValidationReport { diagnostics }
    }

    pub fn lower(&self) -> anyhow::Result<LoweringPlan> {
        self.to_product_document()
            .lower()
            .map_err(|err| anyhow::anyhow!("lower product document: {:?}", err))
    }

    pub fn normalize(&self) -> anyhow::Result<authoring_core::NormalizedIr> {
        let temp_dir = tempfile::Builder::new()
            .prefix("anki-forge-project-normalize-")
            .tempdir()
            .context("create project normalize temp dir")?;
        self.normalize_with_dirs(
            temp_dir.path(),
            temp_dir.path().join(".anki-forge-media"),
            ProjectNormalizeOptions::default(),
        )
        .map(|output| output.normalized_ir)
    }

    pub fn build(&self, options: BuildOptions) -> Result<BuildReport, BuildError> {
        let started = Instant::now();
        let artifacts_dir = options.artifacts_dir.clone().unwrap_or_else(|| {
            tempfile::Builder::new()
                .prefix("anki-forge-project-build-")
                .tempdir()
                .expect("create temp artifacts dir")
                .keep()
        });
        let normalize_options = options.normalize_options.clone().unwrap_or_default();
        let media_input_dir = normalize_options
            .base_dir
            .clone()
            .unwrap_or_else(|| artifacts_dir.join(".anki-forge-media-input"));
        let media_store_dir = normalize_options
            .media_store_dir
            .clone()
            .unwrap_or_else(|| artifacts_dir.join(".anki-forge-media"));

        let validation = self.validate();
        let mut diagnostics = validation.diagnostics;

        let normalized_output = match self.normalize_with_dirs(
            &media_input_dir,
            &media_store_dir,
            normalize_options,
        ) {
            Ok(output) => output,
            Err(error) => {
                return Err(BuildError {
                    report: BuildReport {
                        artifact: None,
                        counts: BuildCounts::default(),
                        diagnostics: {
                            diagnostics.push(Diagnostic {
                                code: DiagnosticCode::new("PROJECT.NORMALIZE_FAILED"),
                                severity: Severity::Error,
                                message: error.to_string(),
                                source: Some(SourcePath::new("project")),
                                help: Some("inspect product notes and media registrations".into()),
                            });
                            diagnostics
                        },
                        metrics: BuildMetrics {
                            duration: started.elapsed(),
                        },
                        inspect: None,
                        status: "invalid".into(),
                    },
                    cause: BuildFailureCause::Diagnostics,
                });
            }
        };
        let normalized = normalized_output.normalized_ir;
        diagnostics.extend(normalized_output.diagnostics);

        if diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity == Severity::Error)
        {
            return Err(BuildError {
                report: BuildReport {
                    artifact: None,
                    counts: BuildCounts {
                        notes: normalized.notes.len(),
                        cards: count_phase1_cards_without_inspect(&normalized),
                        media: normalized.media_bindings.len(),
                    },
                    diagnostics,
                    metrics: BuildMetrics {
                        duration: started.elapsed(),
                    },
                    inspect: None,
                    status: "invalid".into(),
                },
                cause: BuildFailureCause::Diagnostics,
            });
        }

        let current_dir = std::env::current_dir().map_err(|err| BuildError {
            report: failure_report(started, "PROJECT.CURRENT_DIR_FAILED", err.to_string()),
            cause: BuildFailureCause::Io,
        })?;
        let (_runtime, writer_policy, build_context) =
            crate::runtime::load_default_writer_stack(current_dir).map_err(|err| BuildError {
                report: failure_report(started, "PROJECT.RUNTIME_DEFAULTS_FAILED", err.to_string()),
                cause: BuildFailureCause::Io,
            })?;
        let stable_ref_prefix = self
            .stable_id
            .as_deref()
            .map(|stable_id| format!("artifacts/{stable_id}"))
            .unwrap_or_else(|| "artifacts".into());
        let artifact_target =
            BuildArtifactTarget::new(artifacts_dir.clone(), stable_ref_prefix)
                .with_media_store_dir(media_store_dir);
        let package_build_result =
            crate::build(&normalized, &writer_policy, &build_context, &artifact_target).map_err(
                |err| BuildError {
                    report: failure_report(started, "PROJECT.WRITER_FAILED", err.to_string()),
                    cause: BuildFailureCause::BuildStatus,
                },
            )?;

        let mut diagnostics = package_build_result
            .diagnostics
            .items
            .iter()
            .map(|item| Diagnostic {
                code: DiagnosticCode::new(item.code.clone()),
                severity: severity_from_writer_level(&item.level),
                message: item.summary.clone(),
                source: item.path.clone().map(SourcePath::new),
                help: None,
            })
            .collect::<Vec<_>>();
        let mut artifact = None;
        if let Some(apkg_ref) = package_build_result.apkg_ref.as_deref() {
            let built_path = artifact_path_from_ref(&artifact_target, apkg_ref).map_err(|err| {
                BuildError {
                    report: failure_report(started, "PROJECT.ARTIFACT_REF_FAILED", err.to_string()),
                    cause: BuildFailureCause::Io,
                }
            })?;
            let final_path = if let Some(output) = options.output.as_ref() {
                if let Some(parent) = output.parent() {
                    std::fs::create_dir_all(parent).map_err(|err| BuildError {
                        report: failure_report(started, "PROJECT.OUTPUT_DIR_FAILED", err.to_string()),
                        cause: BuildFailureCause::Io,
                    })?;
                }
                std::fs::copy(&built_path, output).map_err(|err| BuildError {
                    report: failure_report(started, "PROJECT.OUTPUT_COPY_FAILED", err.to_string()),
                    cause: BuildFailureCause::Io,
                })?;
                output.clone()
            } else {
                built_path
            };
            artifact = Some(ApkgArtifact { path: final_path });
        }

        let inspect = if options.inspect {
            artifact
                .as_ref()
                .and_then(|artifact| crate::inspect_apkg(&artifact.path).ok())
                .map(|report| InspectSummary {
                    notes: inspect_metadata_count(&report, "note_count"),
                    cards: inspect_metadata_count(&report, "card_count"),
                    source_kind: report.source_kind,
                    observation_status: report.observation_status,
                    notetypes: report.observations.notetypes.len(),
                    templates: report.observations.templates.len(),
                    fields: report.observations.fields.len(),
                    media: report.observations.media.len(),
                })
        } else {
            None
        };

        let counts = BuildCounts {
            notes: normalized.notes.len(),
            cards: inspect
                .as_ref()
                .map(|summary| summary.cards)
                .filter(|cards| *cards > 0)
                .unwrap_or_else(|| count_phase1_cards_without_inspect(&normalized)),
            media: normalized.media_bindings.len(),
        };

        if package_build_result.result_status != "success" && diagnostics.is_empty() {
            diagnostics.push(Diagnostic {
                code: DiagnosticCode::new("PROJECT.BUILD_STATUS_FAILED"),
                severity: Severity::Error,
                message: format!("build status was {}", package_build_result.result_status),
                source: Some(SourcePath::new("project.build")),
                help: Some("inspect writer diagnostics for the failed stage".into()),
            });
        }

        let report = BuildReport {
            artifact,
            counts,
            diagnostics,
            metrics: BuildMetrics {
                duration: started.elapsed(),
            },
            inspect,
            status: package_build_result.result_status,
        };

        report.ensure_success()?;
        Ok(report)
    }

    pub fn write_apkg(&self, path: impl AsRef<Path>) -> Result<BuildReport, BuildError> {
        self.build(BuildOptions::new().output(path.as_ref().to_path_buf()))
    }

    fn to_product_document(&self) -> ProductDocument {
        let document_id = self
            .stable_id
            .clone()
            .unwrap_or_else(|| self.name.clone());
        let default_deck = self
            .default_deck
            .clone()
            .unwrap_or_else(|| self.name.clone());
        let mut product = ProductDocument::new(document_id)
            .with_default_deck(default_deck.clone())
            .with_basic(STOCK_BASIC_ID)
            .with_cloze(STOCK_CLOZE_ID);

        for note in &self.notes {
            let note_id = note
                .stable_id()
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| format!("generated:{}", product.notes().len() + 1));
            let deck_name = note.deck_name().unwrap_or(default_deck.as_str()).to_string();
            let fields = note.rendered_fields();
            if note.note_type_id() == STOCK_BASIC_ID {
                product = product.add_basic_note_with_tags(
                    STOCK_BASIC_ID,
                    note_id,
                    deck_name,
                    fields.get("Front").cloned().unwrap_or_default(),
                    fields.get("Back").cloned().unwrap_or_default(),
                    note.tags().iter().cloned(),
                );
            } else if note.note_type_id() == STOCK_CLOZE_ID {
                product = product.add_cloze_note_with_tags(
                    STOCK_CLOZE_ID,
                    note_id,
                    deck_name,
                    fields.get("Text").cloned().unwrap_or_default(),
                    fields.get("Back Extra").cloned().unwrap_or_default(),
                    note.tags().iter().cloned(),
                );
            }
        }
        product
    }

    fn normalize_with_dirs(
        &self,
        base_dir: impl Into<PathBuf>,
        media_store_dir: impl Into<PathBuf>,
        mut options: ProjectNormalizeOptions,
    ) -> anyhow::Result<ProjectNormalizeOutput> {
        let base_dir = base_dir.into();
        let media_store_dir = media_store_dir.into();
        options.base_dir = options.base_dir.or(Some(base_dir.clone()));
        options.media_store_dir = options.media_store_dir.or(Some(media_store_dir.clone()));
        let lowering = self.lower()?;
        let result = normalize_with_options(
            NormalizationRequest::new(lowering.authoring_document),
            NormalizeOptions {
                base_dir,
                media_store_dir,
                media_policy: options.to_authoring_media_policy(),
            },
        );
        anyhow::ensure!(
            result.result_status == "success",
            "normalization failed with status {}",
            result.result_status
        );
        let diagnostics = result
            .diagnostics
            .items
            .into_iter()
            .map(normalization_diagnostic_to_product_diagnostic)
            .collect();
        let normalized_ir = result
            .normalized_ir
            .context("normalization did not produce normalized_ir")?;
        Ok(ProjectNormalizeOutput {
            normalized_ir,
            diagnostics,
        })
    }
}

struct ProjectNormalizeOutput {
    normalized_ir: authoring_core::NormalizedIr,
    diagnostics: Vec<Diagnostic>,
}

fn normalization_diagnostic_to_product_diagnostic(
    item: authoring_core::model::DiagnosticItem,
) -> Diagnostic {
    Diagnostic {
        code: DiagnosticCode::new(item.code),
        severity: severity_from_writer_level(&item.level),
        message: item.summary,
        source: None,
        help: None,
    }
}

fn failure_report(started: Instant, code: &str, message: String) -> BuildReport {
    BuildReport {
        artifact: None,
        counts: BuildCounts::default(),
        diagnostics: vec![Diagnostic {
            code: DiagnosticCode::new(code),
            severity: Severity::Error,
            message,
            source: Some(SourcePath::new("project.build")),
            help: None,
        }],
        metrics: BuildMetrics {
            duration: started.elapsed(),
        },
        inspect: None,
        status: "error".into(),
    }
}

fn severity_from_writer_level(level: &str) -> Severity {
    match level {
        "error" => Severity::Error,
        "warning" => Severity::Warning,
        _ => Severity::Info,
    }
}

fn inspect_metadata_count(report: &crate::InspectReport, key: &str) -> usize {
    report
        .observations
        .metadata
        .iter()
        .find_map(|value| value.get(key).and_then(serde_json::Value::as_u64))
        .unwrap_or(0) as usize
}

fn count_phase1_cards_without_inspect(normalized: &authoring_core::NormalizedIr) -> usize {
    let templates_by_notetype = normalized
        .notetypes
        .iter()
        .map(|notetype| {
            let template_count = if notetype.kind == "cloze" {
                0
            } else {
                notetype.templates.len()
            };
            (notetype.id.as_str(), (notetype.kind.as_str(), template_count))
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    normalized
        .notes
        .iter()
        .map(|note| {
            let Some((kind, template_count)) = templates_by_notetype.get(note.notetype_id.as_str()) else {
                return 0;
            };
            if *kind == "cloze" {
                distinct_cloze_ords(note.fields.values().map(String::as_str))
            } else {
                *template_count
            }
        })
        .sum()
}

fn distinct_cloze_ords<'a>(fields: impl Iterator<Item = &'a str>) -> usize {
    let mut ords = std::collections::BTreeSet::new();
    for value in fields {
        for part in value.split("{{c").skip(1) {
            let digits = part
                .chars()
                .take_while(|ch| ch.is_ascii_digit())
                .collect::<String>();
            if !digits.is_empty() {
                ords.insert(digits);
            }
        }
    }
    ords.len()
}
```

- [ ] **Step 4: Export Project**

```rust
// anki_forge/src/product/mod.rs
pub mod project;
pub use project::Project;
```

```rust
// anki_forge/src/prelude.rs
pub use crate::product::{
    Content, Field, FieldKey, GenerationRule, IdentityRecipe, MediaRef, Note, NoteType, Project,
    Template, TemplateKey,
};
```

- [ ] **Step 5: Run Task 3 tests**

Run:

```bash
cargo test -p anki_forge --test project_api_tests -v
cargo test -p anki_forge --test build_report_tests -v
```

Expected: PASS.

- [ ] **Step 6: Commit Task 3**

```bash
git add anki_forge/src/product/project.rs anki_forge/src/product/mod.rs anki_forge/src/prelude.rs anki_forge/src/build/report.rs anki_forge/tests/project_api_tests.rs anki_forge/tests/build_report_tests.rs
git commit -m "feat: add project basic build path"
```

## Task 4: Deck As Project Facade

**Files:**
- Modify: `anki_forge/src/deck/export.rs`
- Modify: `anki_forge/src/product/project.rs`
- Modify: `anki_forge/examples/deck_basic_flow.rs`
- Modify: `anki_forge/tests/deck_export_tests.rs`
- Test: `anki_forge/tests/deck_project_facade_tests.rs`

- [ ] **Step 1: Write failing facade parity tests**

```rust
// anki_forge/tests/deck_project_facade_tests.rs
use std::path::PathBuf;

use anki_forge::prelude::*;
use anki_forge::{IoMode, MediaSource};

const PNG: &[u8] = &[
    137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1, 8, 6,
    0, 0, 0, 31, 21, 196, 137, 0, 0, 0, 12, 73, 68, 65, 84, 120, 156, 99, 248, 15, 4, 0, 9, 251,
    3, 253, 167, 102, 129, 94, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
];

#[test]
fn deck_build_matches_project_from_deck_for_stock_notes() {
    let root = unique_artifacts_dir("deck-project-stock");
    let mut deck = Deck::builder("Spanish").stable_id("spanish-v1").build();
    deck.basic()
        .note("hola", "hello")
        .stable_id("es-hola")
        .add()
        .expect("add basic");
    deck.cloze()
        .note("La capital de Espana es {{c1::Madrid}}")
        .stable_id("geo-es-capital")
        .add()
        .expect("add cloze");

    let deck_report = deck
        .build(BuildOptions::new().output(root.join("deck.apkg")))
        .expect("deck build");
    let project_report = Project::from(deck.clone())
        .build(BuildOptions::new().output(root.join("project.apkg")))
        .expect("project build");

    assert_eq!(deck_report.counts, project_report.counts);
    assert_eq!(deck_report.diagnostic_codes(), project_report.diagnostic_codes());
    assert_eq!(
        deck_report.inspect.as_ref().map(|summary| summary.observation_status.as_str()),
        project_report.inspect.as_ref().map(|summary| summary.observation_status.as_str())
    );
}

#[test]
fn project_from_deck_preserves_existing_image_occlusion_support() {
    let root = unique_artifacts_dir("deck-project-io");
    let mut deck = Deck::builder("Anatomy").stable_id("anatomy-v1").build();
    let image = deck
        .media()
        .add(MediaSource::from_bytes("heart.png", PNG.to_vec()))
        .expect("add image");
    deck.image_occlusion()
        .note(image)
        .mode(IoMode::HideAllGuessOne)
        .rect(10, 20, 80, 40)
        .stable_id("heart-io-1")
        .add()
        .expect("add io");

    let report = Project::from(deck)
        .build(BuildOptions::new().output(root.join("io.apkg")))
        .expect("project from deck build");

    assert_eq!(report.counts.notes, 1);
    assert_eq!(report.counts.media, 1);
    assert!(report.counts.cards >= 1);
}

fn unique_artifacts_dir(label: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "anki-forge-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("create temp artifacts dir");
    dir
}
```

- [ ] **Step 2: Run failing facade tests**

Run:

```bash
cargo test -p anki_forge --test deck_project_facade_tests -v
```

Expected: FAIL because `Deck::build(BuildOptions)` is not implemented and `Project::from(deck)` is missing.

- [ ] **Step 3: Implement `Project::from(Deck)`**

Add to `anki_forge/src/product/project.rs`:

```rust
impl From<crate::deck::Deck> for Project {
    fn from(deck: crate::deck::Deck) -> Self {
        let mut project = Project::new(deck.name().to_string());
        if let Some(stable_id) = deck.stable_id() {
            project = project.stable_id(stable_id.to_string());
        }
        project = project.default_deck(deck.name().to_string());
        project.deck_source = Some(deck);
        project
    }
}
```

Add a private field to `Project`:

```rust
deck_source: Option<crate::deck::Deck>,
```

Update `Project::new` to initialize `deck_source: None`.

Update `Project::lower()`:

```rust
pub fn lower(&self) -> anyhow::Result<LoweringPlan> {
    if let Some(deck) = &self.deck_source {
        let product = deck.clone().into_product_document()?;
        return product
            .lower()
            .map_err(|err| anyhow::anyhow!("lower deck product document: {:?}", err));
    }

    self.to_product_document()
        .lower()
        .map_err(|err| anyhow::anyhow!("lower product document: {:?}", err))
}
```

Update `normalize_with_dirs` so deck media is included:

```rust
let mut lowering = self.lower()?;
if let Some(deck) = &self.deck_source {
    let media = deck
        .registered_media()
        .values()
        .map(|media| media.to_authoring_media(&base_dir))
        .collect::<anyhow::Result<Vec<_>>>()?;
    lowering.authoring_document.media.extend(media);
}
```

Expose `registered_media()` from `Deck` with `pub(crate)` visibility:

```rust
impl Deck {
    pub(crate) fn registered_media(
        &self,
    ) -> &std::collections::BTreeMap<String, crate::deck::model::RegisteredMedia> {
        &self.media
    }
}
```

- [ ] **Step 4: Delegate Deck build/write**

Replace public methods in `anki_forge/src/deck/export.rs`:

```rust
impl Deck {
    pub fn build(
        &self,
        options: crate::build::BuildOptions,
    ) -> Result<crate::build::BuildReport, crate::build::BuildError> {
        crate::product::Project::from(self.clone()).build(options)
    }

    pub fn write_apkg(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<crate::build::BuildReport, crate::build::BuildError> {
        crate::product::Project::from(self.clone()).write_apkg(path)
    }
}
```

Keep old `Package::build(artifacts_dir)` internals as private compatibility helpers until all tests are migrated. Rename the old `Deck::build(&Path)` behavior to `build_legacy_artifacts_dir` only if existing tests still need the exact `BuildResult` wrapper. Do not leave two public `Deck::build` methods with different semantic paths.

- [ ] **Step 5: Update deck export tests to BuildReport**

In `anki_forge/tests/deck_export_tests.rs`, replace `deck.build(&artifacts_dir)` assertions with:

```rust
let build = deck
    .build(anki_forge::build::BuildOptions::new().output(artifacts_dir.join("deck.apkg")))
    .expect("build facade");

assert!(build.artifact.as_ref().expect("artifact").path.exists());
assert_eq!(build.status, "success");
assert_eq!(build.counts.notes, 1);
```

Replace `deck.write_apkg(&write_path).expect("write apkg to path");` with:

```rust
let report = deck.write_apkg(&write_path).expect("write apkg to path");
report.ensure_success().expect("successful deck write");
```

- [ ] **Step 6: Run deck facade tests**

Run:

```bash
cargo test -p anki_forge --test deck_project_facade_tests -v
cargo test -p anki_forge --test deck_export_tests -v
```

Expected: PASS.

- [ ] **Step 7: Commit Task 4**

```bash
git add anki_forge/src/deck/export.rs anki_forge/src/deck/model.rs anki_forge/src/product/project.rs anki_forge/examples/deck_basic_flow.rs anki_forge/tests/deck_export_tests.rs anki_forge/tests/deck_project_facade_tests.rs
git commit -m "feat: route deck builds through project"
```

## Task 5: Custom NoteType Lowering And Stable Merge Ids

**Files:**
- Modify: `anki_forge/src/product/model.rs`
- Modify: `anki_forge/src/product/builders.rs`
- Modify: `anki_forge/src/product/lowering.rs`
- Modify: `anki_forge/src/product/project.rs`
- Modify: `anki_forge/src/product/template.rs`
- Test: `anki_forge/tests/custom_notetype_api_tests.rs`
- Test: `anki_forge/tests/custom_merge_id_snapshot_tests.rs`
- Modify: `anki_forge/tests/product_lowering_tests.rs`

- [ ] **Step 1: Write stable config id snapshot tests**

```rust
// anki_forge/tests/custom_merge_id_snapshot_tests.rs
use anki_forge::prelude::*;
use anki_forge::product::stable_config_id;

#[test]
fn stable_config_id_snapshot_values_do_not_drift() {
    assert_eq!(
        stable_config_id("field", "jp-vocab", "expr"),
        2_921_591_957_654_962_622
    );
    assert_eq!(
        stable_config_id("field", "jp-vocab", "meaning"),
        8_939_348_238_921_914_692
    );
    assert_eq!(
        stable_config_id("template", "jp-vocab", "recognition"),
        3_934_332_856_449_685_517
    );
}

#[test]
fn custom_notetype_lowers_keys_to_config_ids() {
    let vocab = NoteType::custom("jp-vocab")
        .name("Japanese Vocabulary")
        .field(Field::new("Expression").key("expr").identity().sort())
        .field(Field::new("Meaning").key("meaning").required())
        .template(
            Template::new("Recognition")
                .key("recognition")
                .front("{{Expression}}")
                .back("{{FrontSide}}<hr id=\"answer\">{{Meaning}}")
                .generate_when(GenerationRule::all(["expr"])),
        )
        .identity(IdentityRecipe::fields(["expr"]));

    let mut project = Project::new("Japanese Core")
        .stable_id("jp-core")
        .default_deck("Japanese::Core");
    project.add_notetype(vocab).expect("add notetype");
    project
        .add_note(
            Note::new("jp-vocab")
                .stable_id("jp-vocab:taberu")
                .text("expr", "食べる")
                .text("meaning", "to eat"),
        )
        .expect("add note");

    let normalized = project.normalize().expect("normalize custom");
    let notetype = normalized
        .notetypes
        .iter()
        .find(|notetype| notetype.id == "jp-vocab")
        .expect("jp vocab notetype");

    assert_eq!(notetype.kind, "normal");
    assert_eq!(notetype.fields[0].name, "Expression");
    assert_eq!(
        notetype.fields[0].config_id,
        Some(2_921_591_957_654_962_622)
    );
    assert_eq!(notetype.templates[0].name, "Recognition");
    assert_eq!(
        notetype.templates[0].config_id,
        Some(3_934_332_856_449_685_517)
    );
}

#[test]
fn custom_notetype_rejects_cloze_generation_rule() {
    let vocab = NoteType::custom("jp-vocab")
        .field(Field::new("Expression").key("expr"))
        .template(
            Template::new("Cloze")
                .key("cloze")
                .front("{{cloze:Expression}}")
                .back("{{cloze:Expression}}")
                .generate_when(GenerationRule::Cloze {
                    field: FieldKey::new("expr"),
                }),
        );

    let mut project = Project::new("Japanese Core")
        .stable_id("jp-core")
        .default_deck("Japanese::Core");
    project.add_notetype(vocab).expect("add notetype");

    let err = project.normalize().expect_err("custom cloze is out of scope");
    assert!(
        err.to_string()
            .contains("TEMPLATE.CLOZE_RULE_REQUIRES_STOCK_CLOZE"),
        "unexpected error: {err}"
    );
}
```

- [ ] **Step 2: Run failing custom merge tests**

Run:

```bash
cargo test -p anki_forge --test custom_merge_id_snapshot_tests -v
```

Expected: FAIL with missing `stable_config_id` and missing custom notetype lowering from `Project`.

- [ ] **Step 3: Add stable config id function**

```rust
// anki_forge/src/product/template.rs
pub fn stable_config_id(namespace: &str, note_type_id: &str, key: &str) -> i64 {
    let payload = format!("{namespace}\0{note_type_id}\0{key}");
    let digest = blake3::hash(payload.as_bytes());
    let mut bytes = [0_u8; 8];
    bytes.copy_from_slice(&digest.as_bytes()[0..8]);
    i64::from_be_bytes(bytes) & i64::MAX
}
```

Export from `anki_forge/src/product/mod.rs`:

```rust
pub use template::{stable_config_id, GenerationRule, Template, TemplateKey, TemplateSource};
```

- [ ] **Step 4: Extend product DTOs with keys**

Modify `anki_forge/src/product/model.rs`:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomField {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomTemplate {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    pub question_format: String,
    pub answer_format: String,
    #[serde(default)]
    pub generation_rule: Option<CustomGenerationRule>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CustomGenerationRule {
    AnkiDefault,
    All { fields: Vec<String> },
    Any { fields: Vec<String> },
    Cloze { field: String },
}
```

Update every existing test fixture construction of `CustomField` and `CustomTemplate` to include `key: None` and `generation_rule: None` where they directly instantiate the structs.

- [ ] **Step 5: Lower Project custom note types to ProductDocument custom DTOs**

Add to `Project::to_product_document()` before notes are lowered:

```rust
for note_type in &self.note_types {
    let custom = crate::product::model::CustomNoteType {
        id: note_type.id().to_string(),
        name: note_type.name().map(ToOwned::to_owned),
        fields: note_type
            .fields()
            .iter()
            .map(|field| crate::product::model::CustomField {
                name: field.name().to_string(),
                key: Some(field.key_ref().as_str().to_string()),
            })
            .collect(),
        templates: note_type
            .templates()
            .iter()
            .map(|template| crate::product::model::CustomTemplate {
                name: template.name().to_string(),
                key: Some(template.key_ref().as_str().to_string()),
                question_format: template.front_source().as_str().to_string(),
                answer_format: template.back_source().as_str().to_string(),
                generation_rule: Some(custom_generation_rule(template.generation_rule())),
            })
            .collect(),
        css: None,
    };
    product = product.with_custom_notetype(custom);
    for template in note_type.templates() {
        if template.browser_front().is_some() || template.browser_back().is_some() {
            product = product.with_browser_appearance(
                note_type.id().to_string(),
                crate::product::metadata::TemplateBrowserAppearanceDeclaration {
                    template_name: template.name().to_string(),
                    question_format: template
                        .browser_front()
                        .map(|source| source.as_str().to_string()),
                    answer_format: template
                        .browser_back()
                        .map(|source| source.as_str().to_string()),
                    font_name: None,
                    font_size: None,
                },
            );
        }
        if let Some(deck_name) = template.target_deck() {
            product = product.with_template_target_deck(
                note_type.id().to_string(),
                crate::product::metadata::TemplateTargetDeckDeclaration {
                    template_name: template.name().to_string(),
                    deck_name: deck_name.to_string(),
                },
            );
        }
    }
}
```

Add helper:

```rust
fn custom_generation_rule(
    rule: &crate::product::GenerationRule,
) -> crate::product::model::CustomGenerationRule {
    match rule {
        crate::product::GenerationRule::AnkiDefault => {
            crate::product::model::CustomGenerationRule::AnkiDefault
        }
        crate::product::GenerationRule::All(fields) => {
            crate::product::model::CustomGenerationRule::All {
                fields: fields.iter().map(|field| field.as_str().to_string()).collect(),
            }
        }
        crate::product::GenerationRule::Any(fields) => {
            crate::product::model::CustomGenerationRule::Any {
                fields: fields.iter().map(|field| field.as_str().to_string()).collect(),
            }
        }
        crate::product::GenerationRule::Cloze { field } => {
            crate::product::model::CustomGenerationRule::Cloze {
                field: field.as_str().to_string(),
            }
        }
    }
}
```

When lowering custom notes, add:

```rust
} else {
    let fields = custom_note_fields_for_authoring(self, note);
    product = product.add_custom_note(crate::product::model::CustomNote {
        id: note_id,
        note_type_id: note.note_type_id().to_string(),
        deck_name,
        fields,
        tags: note.tags().to_vec(),
    });
}
```

Add the field-key translation helper so Product notes can use stable field keys while Authoring IR still receives Anki-visible field names:

```rust
fn custom_note_fields_for_authoring(
    project: &Project,
    note: &crate::product::Note,
) -> std::collections::BTreeMap<String, String> {
    let rendered = note.rendered_fields();
    let Some(note_type) = project
        .note_types
        .iter()
        .find(|note_type| note_type.id() == note.note_type_id())
    else {
        return rendered;
    };

    let name_by_key = note_type
        .fields()
        .iter()
        .map(|field| (field.key_ref().as_str(), field.name()))
        .collect::<std::collections::BTreeMap<_, _>>();

    rendered
        .into_iter()
        .map(|(field_key_or_name, value)| {
            let field_name = name_by_key
                .get(field_key_or_name.as_str())
                .copied()
                .unwrap_or(field_key_or_name.as_str())
                .to_string();
            (field_name, value)
        })
        .collect()
}
```

- [ ] **Step 6: Lower field/template config ids and generation rules**

Modify custom branch in `anki_forge/src/product/lowering.rs`:

```rust
let field_name_by_key = custom
    .fields
    .iter()
    .map(|field| {
        let key = field.key.clone().unwrap_or_else(|| field.name.clone());
        (
            key,
            field.name.clone(),
        )
    })
    .collect::<BTreeMap<_, _>>();

let fields = custom
    .fields
    .iter()
    .enumerate()
    .map(|(ord, field)| {
        let key = field.key.clone().unwrap_or_else(|| field.name.clone());
        AuthoringField {
            name: field.name.clone(),
            ord: Some(ord as u32),
            config_id: Some(crate::product::stable_config_id("field", &custom.id, &key)),
            tag: None,
            prevent_deletion: false,
        }
    })
    .collect();

let templates = custom
    .templates
    .iter()
    .enumerate()
    .map(|(ord, template)| {
        let key = template.key.clone().unwrap_or_else(|| template.name.clone());
        let question_format = lower_generation_rule_front(
            &custom.id,
            template,
            &field_name_by_key,
        )?;
        Ok(AuthoringTemplate {
            name: template.name.clone(),
            ord: Some(ord as u32),
            config_id: Some(crate::product::stable_config_id("template", &custom.id, &key)),
            question_format,
            answer_format: template.answer_format.clone(),
            browser_question_format: document
                .browser_appearance_for(&custom.id, &template.name)
                .and_then(|declaration| declaration.question_format),
            browser_answer_format: document
                .browser_appearance_for(&custom.id, &template.name)
                .and_then(|declaration| declaration.answer_format),
            target_deck_name: document
                .template_target_deck_for(&custom.id, &template.name)
                .map(|declaration| declaration.deck_name),
            browser_font_name: document
                .browser_appearance_for(&custom.id, &template.name)
                .and_then(|declaration| declaration.font_name),
            browser_font_size: document
                .browser_appearance_for(&custom.id, &template.name)
                .and_then(|declaration| declaration.font_size),
        })
    })
    .collect::<Result<Vec<_>, ProductDiagnostic>>()?;
```

Add helper:

```rust
fn lower_generation_rule_front(
    note_type_id: &str,
    template: &crate::product::model::CustomTemplate,
    field_name_by_key: &BTreeMap<String, String>,
) -> Result<String, ProductDiagnostic> {
    let Some(rule) = &template.generation_rule else {
        return Ok(template.question_format.clone());
    };

    match rule {
        crate::product::model::CustomGenerationRule::AnkiDefault => {
            Ok(template.question_format.clone())
        }
        crate::product::model::CustomGenerationRule::All { fields } => {
            let field_names =
                generation_field_names(note_type_id, template, fields, field_name_by_key)?;
            Ok(wrap_front_with_all_conditions(&template.question_format, &field_names))
        }
        crate::product::model::CustomGenerationRule::Any { fields } => {
            let field_names =
                generation_field_names(note_type_id, template, fields, field_name_by_key)?;
            Ok(wrap_front_with_any_conditions(&template.question_format, &field_names))
        }
        crate::product::model::CustomGenerationRule::Cloze { .. } => {
            Err(ProductDiagnostic {
                code: "TEMPLATE.CLOZE_RULE_REQUIRES_STOCK_CLOZE".into(),
                message: format!(
                    "custom normal note type '{}' template '{}' cannot use cloze generation",
                    note_type_id, template.name
                ),
            })
        }
    }
}

fn generation_field_names(
    note_type_id: &str,
    template: &crate::product::model::CustomTemplate,
    fields: &[String],
    field_name_by_key: &BTreeMap<String, String>,
) -> Result<Vec<String>, ProductDiagnostic> {
    let mut field_names = Vec::with_capacity(fields.len());
    for field in fields {
        let Some(field_name) = field_name_by_key.get(field) else {
            return Err(ProductDiagnostic {
                code: "TEMPLATE.REQUIRED_FIELD_MISSING".into(),
                message: format!(
                    "template '{}' in note type '{}' references unknown field key '{}'",
                    template.name, note_type_id, field
                ),
            });
        };
        field_names.push(field_name.clone());
    }
    Ok(field_names)
}

fn wrap_front_with_all_conditions(front: &str, field_keys: &[String]) -> String {
    field_keys.iter().rev().fold(front.to_string(), |inner, field| {
        format!("{{{{#{field}}}}}{inner}{{{{/{field}}}}}")
    })
}

fn wrap_front_with_any_conditions(front: &str, field_keys: &[String]) -> String {
    let guards = field_keys
        .iter()
        .map(|field| format!("{{{{#{field}}}}}{front}{{{{/{field}}}}}"))
        .collect::<Vec<_>>();
    guards.join("")
}
```

- [ ] **Step 7: Run custom lowering tests**

Run:

```bash
cargo test -p anki_forge --test custom_merge_id_snapshot_tests -v
cargo test -p anki_forge --test custom_notetype_api_tests -v
cargo test -p anki_forge --test product_lowering_tests -v
```

Expected: PASS.

- [ ] **Step 8: Commit Task 5**

```bash
git add anki_forge/src/product/model.rs anki_forge/src/product/builders.rs anki_forge/src/product/lowering.rs anki_forge/src/product/project.rs anki_forge/src/product/template.rs anki_forge/src/product/mod.rs anki_forge/tests/custom_notetype_api_tests.rs anki_forge/tests/custom_merge_id_snapshot_tests.rs anki_forge/tests/product_lowering_tests.rs
git commit -m "feat: add stable custom notetype lowering"
```

## Task 6: Product Media Registry And Note Media Helpers

**Files:**
- Modify: `anki_forge/src/product/media_registry.rs`
- Modify: `anki_forge/src/product/project.rs`
- Modify: `anki_forge/src/product/note.rs`
- Test: `anki_forge/tests/project_media_api_tests.rs`
- Test: `anki_forge/tests/build_report_tests.rs`

- [ ] **Step 1: Write failing product media tests**

```rust
// anki_forge/tests/project_media_api_tests.rs
use std::path::PathBuf;

use anki_forge::prelude::*;

const MP3: &[u8] = b"fake-mp3-bytes-for-package-test";
const PNG: &[u8] = &[
    137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1, 8, 6,
    0, 0, 0, 31, 21, 196, 137, 0, 0, 0, 12, 73, 68, 65, 84, 120, 156, 99, 248, 15, 4, 0, 9, 251,
    3, 253, 167, 102, 129, 94, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
];

#[test]
fn product_media_helpers_render_anki_compatible_content() {
    let mut project = Project::new("Media").stable_id("media").default_deck("Media");
    let audio = project
        .media_mut()
        .add_bytes("raw-audio.bin", MP3.to_vec())
        .export_as("hola.mp3")
        .expect("audio media");
    let image = project
        .media_mut()
        .add_bytes("raw-image.bin", PNG.to_vec())
        .export_as("chart.png")
        .expect("image media");

    let note = Note::new("basic")
        .stable_id("media:1")
        .text("Front", "hola")
        .sound("Back", audio.clone())
        .image("Picture", image.clone());

    assert_eq!(audio.sound().render(), "[sound:hola.mp3]");
    assert_eq!(image.image().render(), "<img src=\"chart.png\">");
    assert_eq!(
        note.rendered_fields().get("Back").map(String::as_str),
        Some("[sound:hola.mp3]")
    );
    assert_eq!(
        note.rendered_fields().get("Picture").map(String::as_str),
        Some("<img src=\"chart.png\">")
    );
}

#[test]
fn project_build_packages_product_media_and_reports_count() {
    let root = unique_artifacts_dir("project-media");
    let mut project = Project::new("Media").stable_id("media").default_deck("Media");
    let audio = project
        .media_mut()
        .add_bytes("hola-source.mp3", MP3.to_vec())
        .export_as("hola.mp3")
        .expect("audio media");

    project
        .add_note(
            Note::basic("hola", "hello")
                .stable_id("media:hola")
                .sound("Back", audio),
        )
        .expect("add note");

    let report = project
        .write_apkg(root.join("media.apkg"))
        .expect("write apkg");

    report.ensure_success().expect("successful media build");
    assert_eq!(report.counts.notes, 1);
    assert_eq!(report.counts.media, 1);
}

fn unique_artifacts_dir(label: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "anki-forge-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("create temp artifacts dir");
    dir
}
```

- [ ] **Step 2: Run failing media tests**

Run:

```bash
cargo test -p anki_forge --test project_media_api_tests -v
```

Expected: FAIL because `MediaRegistry::add_bytes(...).export_as(...)` does not exist and project media is not included in lower/build.

- [ ] **Step 3: Implement product MediaRegistry**

Replace `anki_forge/src/product/media_registry.rs` with:

```rust
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Context;
use base64::Engine as _;
use sha1::{Digest, Sha1};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaRef {
    filename: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProductMediaSource {
    File { path: PathBuf },
    InlineBytes { data_base64: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductMedia {
    pub id: String,
    pub export_filename: String,
    pub source: ProductMediaSource,
    pub declared_mime: Option<String>,
    pub sha1_hex: String,
}

#[derive(Debug, Clone, Default)]
pub struct MediaRegistry {
    media: BTreeMap<String, ProductMedia>,
    pending: BTreeMap<String, ProductMedia>,
}

#[derive(Debug, Clone)]
pub struct PendingMedia<'a> {
    registry: &'a mut MediaRegistry,
    media: ProductMedia,
}

impl MediaRef {
    pub(crate) fn new(filename: impl Into<String>) -> Self {
        Self {
            filename: filename.into(),
        }
    }

    pub fn filename(&self) -> &str {
        &self.filename
    }

    pub fn sound(&self) -> crate::product::Content {
        crate::product::Content::Html(format!("[sound:{}]", self.filename))
    }

    pub fn image(&self) -> crate::product::Content {
        crate::product::Content::Html(format!("<img src=\"{}\">", self.filename))
    }
}

impl MediaRegistry {
    pub fn add_file(&mut self, path: impl AsRef<Path>) -> anyhow::Result<PendingMedia<'_>> {
        let path = path.as_ref().to_path_buf();
        let bytes = std::fs::read(&path)
            .with_context(|| format!("read media source file: {}", path.display()))?;
        let filename = path
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| anyhow::anyhow!("media path must end in a valid filename"))?
            .to_string();
        Ok(PendingMedia {
            registry: self,
            media: ProductMedia {
                id: format!("media:{filename}"),
                export_filename: filename.clone(),
                source: ProductMediaSource::File { path },
                declared_mime: Some(mime_from_name(&filename)),
                sha1_hex: hex::encode(Sha1::digest(bytes)),
            },
        })
    }

    pub fn add_bytes(
        &mut self,
        filename: impl Into<String>,
        bytes: Vec<u8>,
    ) -> PendingMedia<'_> {
        let filename = filename.into();
        PendingMedia {
            registry: self,
            media: ProductMedia {
                id: format!("media:{filename}"),
                export_filename: filename.clone(),
                source: ProductMediaSource::InlineBytes {
                    data_base64: base64::engine::general_purpose::STANDARD.encode(&bytes),
                },
                declared_mime: Some(mime_from_name(&filename)),
                sha1_hex: hex::encode(Sha1::digest(bytes)),
            },
        }
    }

    pub(crate) fn media(&self) -> impl Iterator<Item = &ProductMedia> {
        self.media.values()
    }
}

impl<'a> PendingMedia<'a> {
    pub fn export_as(mut self, filename: impl Into<String>) -> anyhow::Result<MediaRef> {
        let filename = filename.into();
        validate_media_filename(&filename)?;
        self.media.id = format!("media:{filename}");
        self.media.export_filename = filename.clone();
        if let Some(existing) = self.registry.media.get(&filename) {
            anyhow::ensure!(
                existing.sha1_hex == self.media.sha1_hex,
                "MEDIA.FILENAME_COLLISION: {filename}"
            );
            return Ok(MediaRef::new(filename));
        }
        self.registry.media.insert(filename.clone(), self.media);
        Ok(MediaRef::new(filename))
    }
}

fn validate_media_filename(filename: &str) -> anyhow::Result<()> {
    anyhow::ensure!(!filename.trim().is_empty(), "MEDIA.EXPORT_NAME_EMPTY");
    anyhow::ensure!(!filename.contains('/'), "MEDIA.EXPORT_NAME_CONTAINS_SLASH");
    anyhow::ensure!(!filename.contains('\\'), "MEDIA.EXPORT_NAME_CONTAINS_BACKSLASH");
    Ok(())
}

fn mime_from_name(name: &str) -> String {
    match name.rsplit('.').next().map(str::to_ascii_lowercase) {
        Some(ext) if ext == "png" => "image/png".into(),
        Some(ext) if ext == "jpg" || ext == "jpeg" => "image/jpeg".into(),
        Some(ext) if ext == "mp3" => "audio/mpeg".into(),
        Some(ext) if ext == "wav" => "audio/wav".into(),
        _ => "application/octet-stream".into(),
    }
}
```

- [ ] **Step 4: Lower project media into AuthoringDocument**

In `Project::lower()`, after `ProductDocument::lower()` succeeds for non-deck projects:

Add imports to `anki_forge/src/product/project.rs`:

```rust
use base64::Engine as _;
```

```rust
let mut plan = self
    .to_product_document()
    .lower()
    .map_err(|err| anyhow::anyhow!("lower product document: {:?}", err))?;
plan.authoring_document.media.extend(
    self.media
        .media()
        .map(product_media_to_authoring_media)
        .collect::<anyhow::Result<Vec<_>>>()?,
);
Ok(plan)
```

Add helper:

```rust
fn product_media_to_authoring_media(
    media: &crate::product::media_registry::ProductMedia,
) -> anyhow::Result<crate::AuthoringMedia> {
    let source = match &media.source {
        crate::product::media_registry::ProductMediaSource::File { path } => {
            let bytes = std::fs::read(path)
                .with_context(|| format!("read media source file: {}", path.display()))?;
            crate::AuthoringMediaSource::InlineBytes {
                data_base64: base64::engine::general_purpose::STANDARD.encode(bytes),
            }
        }
        crate::product::media_registry::ProductMediaSource::InlineBytes { data_base64 } => {
            crate::AuthoringMediaSource::InlineBytes {
                data_base64: data_base64.clone(),
            }
        }
    };

    Ok(crate::AuthoringMedia {
        id: media.id.clone(),
        desired_filename: media.export_filename.clone(),
        source,
        declared_mime: media.declared_mime.clone(),
    })
}
```

- [ ] **Step 5: Run media tests**

Run:

```bash
cargo test -p anki_forge --test project_media_api_tests -v
cargo test -p anki_forge --test project_api_tests -v
```

Expected: PASS.

- [ ] **Step 6: Commit Task 6**

```bash
git add anki_forge/src/product/media_registry.rs anki_forge/src/product/project.rs anki_forge/src/product/note.rs anki_forge/tests/project_media_api_tests.rs anki_forge/tests/build_report_tests.rs
git commit -m "feat: add project media helpers"
```

## Task 7: Docs, Examples, Python Shape, And Exit Evidence

**Files:**
- Modify: `README.md`
- Create: `anki_forge/examples/target_api_basic.rs`
- Create: `anki_forge/examples/target_api_custom_notetype.rs`
- Create: `anki_forge/examples/target_api_media.rs`
- Create: `bindings/python/examples/target_api_custom.py`
- Modify: `bindings/python/README.md`
- Create: `docs/superpowers/checklists/phase-1-user-facing-rust-mvp-exit-evidence.md`

- [ ] **Step 1: Add target API examples**

```rust
// anki_forge/examples/target_api_basic.rs
use anki_forge::prelude::*;

fn main() -> anyhow::Result<()> {
    let mut deck = Deck::new("Spanish");
    deck.basic()
        .note("hola", "hello")
        .stable_id("es:hola")
        .add()?;
    deck.write_apkg("spanish.apkg")?.ensure_success()?;
    Ok(())
}
```

```rust
// anki_forge/examples/target_api_custom_notetype.rs
use anki_forge::prelude::*;

fn main() -> anyhow::Result<()> {
    let vocab = NoteType::custom("jp-vocab")
        .name("Japanese Vocabulary")
        .field(Field::new("Expression").key("expr").identity().sort())
        .field(Field::new("Meaning").key("meaning").required())
        .template(
            Template::new("Recognition")
                .key("recognition")
                .front("{{Expression}}")
                .back("{{FrontSide}}<hr id=\"answer\">{{Meaning}}")
                .generate_when(GenerationRule::all(["expr"])),
        )
        .identity(IdentityRecipe::fields(["expr"]));

    let mut project = Project::new("Japanese Core")
        .stable_id("jp-core")
        .default_deck("Japanese::Core");
    project.add_notetype(vocab)?;
    project.add_note(
        Note::new("jp-vocab")
            .stable_id("jp-vocab:taberu")
            .text("expr", "食べる")
            .text("meaning", "to eat"),
    )?;

    project.write_apkg("jp-core.apkg")?.ensure_success()?;
    Ok(())
}
```

```rust
// anki_forge/examples/target_api_media.rs
use anki_forge::prelude::*;

fn main() -> anyhow::Result<()> {
    let mut project = Project::new("Spanish Media")
        .stable_id("spanish-media")
        .default_deck("Spanish::Media");
    let audio = project
        .media_mut()
        .add_bytes("hola-source.mp3", b"hello audio".to_vec())
        .export_as("hola.mp3")?;

    project.add_note(
        Note::basic("hola", "hello")
            .stable_id("es:hola")
            .sound("Back", audio),
    )?;

    project.write_apkg("spanish-media.apkg")?.ensure_success()?;
    Ok(())
}
```

- [ ] **Step 2: Add Python shape example**

```python
# bindings/python/examples/target_api_custom.py
from anki_forge import Field, GenerationRule, IdentityRecipe, Note, NoteType, Project, Template


def build_project() -> Project:
    project = Project(
        name="Japanese Core",
        stable_id="jp-core",
        default_deck="Japanese::Core",
    )

    vocab = NoteType.custom("jp-vocab", name="Japanese Vocabulary")
    vocab.field(Field("Expression", key="expr", identity=True, sort=True))
    vocab.field(Field("Meaning", key="meaning", required=True))
    vocab.template(
        Template(
            "Recognition",
            key="recognition",
            front="{{Expression}}",
            back="{{FrontSide}}<hr id='answer'>{{Meaning}}",
            generate_when=GenerationRule.all(["expr"]),
        )
    )
    vocab.identity = IdentityRecipe.fields(["expr"])

    project.add_notetype(vocab)
    project.add_note(
        Note("jp-vocab", stable_id="jp-vocab:taberu")
        .text("expr", "食べる")
        .text("meaning", "to eat")
    )
    return project


if __name__ == "__main__":
    report = build_project().write_apkg("jp-core.apkg")
    report.ensure_success()
```

- [ ] **Step 3: Update README first screen**

Replace the README quick-start narrative with this order:

```markdown
## 2. Quick Start: Deck First

```rust
use anki_forge::prelude::*;

fn main() -> anyhow::Result<()> {
    let mut deck = Deck::new("Spanish");
    deck.basic()
        .note("hola", "hello")
        .stable_id("es:hola")
        .add()?;
    deck.write_apkg("spanish.apkg")?.ensure_success()?;
    Ok(())
}
```

## 3. Project For Long-Term Decks

```rust
use anki_forge::prelude::*;

fn main() -> anyhow::Result<()> {
    let mut project = Project::new("Japanese Core")
        .stable_id("jp-core")
        .default_deck("Japanese::Core");
    project.add_note(Note::basic("食べる", "to eat").stable_id("jp:taberu"))?;
    project.write_apkg("jp-core.apkg")?.ensure_success()?;
    Ok(())
}
```

`BuildReport` includes the artifact path, note/card/media counts, diagnostics, warning count, inspect summary, and duration.
```

Keep the existing contract-tool sections later in the README under an advanced heading.

- [ ] **Step 4: Add exit evidence checklist**

```markdown
# Phase 1 User-Facing Rust MVP Exit Evidence

Recorded against the current worktree on 2026-05-16.

## Required Evidence

- [ ] `cargo test -p anki_forge --test project_api_tests -v` passes.
- [ ] `cargo test -p anki_forge --test deck_project_facade_tests -v` passes.
- [ ] `cargo test -p anki_forge --test custom_merge_id_snapshot_tests -v` passes.
- [ ] `cargo test -p anki_forge --test project_media_api_tests -v` passes.
- [ ] `cargo run -q -p anki_forge --example target_api_basic` writes `spanish.apkg`.
- [ ] `cargo run -q -p anki_forge --example target_api_custom_notetype` writes `jp-core.apkg`.
- [ ] `cargo run -q -p anki_forge --example target_api_media` writes `spanish-media.apkg`.
- [ ] README first screen teaches `Deck`; second screen teaches `Project`.
- [ ] `bindings/python/examples/target_api_custom.py` documents Python Product API shape.
- [ ] Existing manual scenarios `S01_basic_text_minimal`, `S02_cloze_minimal`, `S04_basic_image`, and `S05_basic_audio` are referenced as Phase 1 oracle evidence.

## Oracle References

- Basic: `docs/manual-validation/anki-desktop-v1/S01_basic_text_minimal.md`
- Cloze: `docs/manual-validation/anki-desktop-v1/S02_cloze_minimal.md`
- Image media: `docs/manual-validation/anki-desktop-v1/S04_basic_image.md`
- Audio media: `docs/manual-validation/anki-desktop-v1/S05_basic_audio.md`
- Field/template merge id snapshot: `anki_forge/tests/custom_merge_id_snapshot_tests.rs`
```

- [ ] **Step 5: Run docs/examples checks**

Run:

```bash
cargo run -q -p anki_forge --example target_api_basic
cargo run -q -p anki_forge --example target_api_custom_notetype
cargo run -q -p anki_forge --example target_api_media
```

Expected: each command exits 0 and writes its named APKG in the working directory.

- [ ] **Step 6: Commit Task 7**

```bash
git add README.md anki_forge/examples/target_api_basic.rs anki_forge/examples/target_api_custom_notetype.rs anki_forge/examples/target_api_media.rs bindings/python/examples/target_api_custom.py bindings/python/README.md docs/superpowers/checklists/phase-1-user-facing-rust-mvp-exit-evidence.md
git commit -m "docs: add phase 1 product api examples"
```

## Task 8: Public API Boundary And Full Verification

**Files:**
- Modify: `anki_forge/src/lib.rs`
- Create: `anki_forge/src/authoring.rs`
- Create: `anki_forge/src/writer.rs`
- Modify: `anki_forge/src/prelude.rs`
- Test: `anki_forge/tests/public_api_boundary_tests.rs`

- [ ] **Step 1: Write failing public API boundary tests**

```rust
// anki_forge/tests/public_api_boundary_tests.rs
use anki_forge::prelude::*;

#[test]
fn prelude_exports_product_happy_path_types() {
    let mut project = Project::new("Spanish").stable_id("spanish").default_deck("Spanish");
    project
        .add_note(Note::basic("hola", "hello").stable_id("es:hola"))
        .expect("add note");

    let _options = BuildOptions::new().inspect(true);
}

#[test]
fn advanced_authoring_reexports_are_namespaced() {
    let document = anki_forge::authoring::AuthoringDocument {
        kind: "authoring-ir".into(),
        schema_version: "0.1.0".into(),
        metadata_document_id: "doc".into(),
        notetypes: vec![],
        notes: vec![],
        media: vec![],
    };

    assert_eq!(document.kind, "authoring-ir");
}
```

- [ ] **Step 2: Run failing boundary tests**

Run:

```bash
cargo test -p anki_forge --test public_api_boundary_tests -v
```

Expected: FAIL until `authoring` module exists and prelude exports are stable.

- [ ] **Step 3: Add advanced namespace modules**

```rust
// anki_forge/src/authoring.rs
pub use authoring_core::{
    assess_risk, normalize, normalize_with_options, parse_selector, resolve_identity,
    resolve_selector, to_canonical_json as to_authoring_canonical_json, AuthoringDocument,
    AuthoringField, AuthoringFieldMetadata, AuthoringMedia, AuthoringMediaSource, AuthoringNote,
    AuthoringNotetype, AuthoringTemplate, ComparisonContext, DiagnosticBehavior, MediaBinding,
    MediaObject, MediaPolicy, MediaReference, MediaReferenceResolution, MergeRiskReport,
    NormalizationRequest, NormalizeOptions, NormalizedField, NormalizedFieldMetadata, NormalizedIr,
    NormalizedNote, NormalizedNotetype, NormalizedTemplate, Selector, SelectorError,
    SelectorResolveError, SelectorTarget,
};
```

```rust
// anki_forge/src/writer.rs
pub use writer_core::{
    build, build_context_ref, diff_reports, extract_media_references, inspect_apkg,
    inspect_build_result, inspect_staging, policy_ref,
    to_canonical_json as to_writer_canonical_json, BuildArtifactTarget, BuildContext, DiffReport,
    InspectReport, PackageBuildResult, VerificationGateRule, VerificationPolicy, WriterPolicy,
};
```

In `anki_forge/src/lib.rs`, add:

```rust
pub mod authoring;
pub mod writer;
```

Keep old root re-exports for one release if existing tests still use them. Mark them deprecated and add a doc comment above them:

```rust
#[deprecated(
    note = "use anki_forge::prelude for Product API or anki_forge::authoring / anki_forge::writer for advanced APIs"
)]
// Backward-compatible root re-exports. New user docs should use `prelude`,
// `authoring`, or `writer`.
```

- [ ] **Step 4: Run full verification**

Run:

```bash
cargo fmt --all -- --check
cargo test -p anki_forge -v
cargo test -p authoring_core -v
cargo test -p writer_core -v
```

Expected: PASS for all commands.

- [ ] **Step 5: Run target examples**

Run:

```bash
cargo run -q -p anki_forge --example target_api_basic
cargo run -q -p anki_forge --example target_api_custom_notetype
cargo run -q -p anki_forge --example target_api_media
```

Expected: PASS and APKG files are produced in the current directory.

- [ ] **Step 6: Update exit evidence with actual command output**

Edit `docs/superpowers/checklists/phase-1-user-facing-rust-mvp-exit-evidence.md` so every checked command includes `PASS` evidence and the exact command used.

- [ ] **Step 7: Commit Task 8**

```bash
git add anki_forge/src/lib.rs anki_forge/src/authoring.rs anki_forge/src/writer.rs anki_forge/src/prelude.rs anki_forge/tests/public_api_boundary_tests.rs docs/superpowers/checklists/phase-1-user-facing-rust-mvp-exit-evidence.md
git commit -m "feat: finalize phase 1 public api boundary"
```

## Final Verification

After all tasks are complete, run:

```bash
cargo fmt --all -- --check
cargo test -p anki_forge -v
cargo test -p authoring_core -v
cargo test -p writer_core -v
cargo run -q -p anki_forge --example target_api_basic
cargo run -q -p anki_forge --example target_api_custom_notetype
cargo run -q -p anki_forge --example target_api_media
```

Expected: all commands pass. The three examples write `spanish.apkg`, `jp-core.apkg`, and `spanish-media.apkg`.

## Plan Self-Review Notes

Spec coverage:

1. Codebase API audit and signature drift prevention: Task 0.
2. `Project`: Task 3.
3. `Project::validate()` duplicate stable id, auto key, and missing identity diagnostics: Task 3.
4. `Deck` as `Project` facade: Task 4.
5. `NoteType::custom`: Task 2 and Task 5.
6. `FieldKey` / `TemplateKey` / stable config id: Task 5.
7. `Field` / `Template` / minimal `GenerationRule`: Task 2 and Task 5.
8. `Note::new` with named fields: Task 2.
9. `Note::basic` / `Note::cloze`: Task 2 and Task 3.
10. safe `Content::Text` / explicit `Content::Html`: Task 2.
11. minimal media registry and helpers: Task 6.
12. `write_apkg -> BuildReport`: Task 1 and Task 3.
13. README/examples/Python shape: Task 7.
14. snapshot/oracle gates: Task 5 and Task 7.
15. public API boundary: Task 8.

Type consistency:

1. `BuildReport.artifact` is `Option<ApkgArtifact>` throughout the plan so partial error reports can omit artifacts.
2. `BuildOptions.normalize_options` is consumed by `Project::build`, not left as a write-only field.
3. `Project::build` and `Deck::build` both take `BuildOptions`.
4. `Project::write_apkg` and `Deck::write_apkg` both return `Result<BuildReport, BuildError>`.
5. Product `MediaRef` is separate from existing deck `MediaRef`; imports must use `anki_forge::prelude::MediaRef` for Product API.
6. Phase 1 custom note types lower as normal note types only.
7. Card counts use inspect metadata first and only fall back to approximation when inspect is disabled or unavailable.
