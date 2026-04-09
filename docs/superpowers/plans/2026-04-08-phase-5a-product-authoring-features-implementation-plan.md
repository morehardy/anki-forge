# Phase 5A Product Authoring Features Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver a Rust-first `Phase 5A` product-authoring layer in `anki_forge` that gives authors first-class `Basic/Cloze/ImageOcclusion` workflows, structured helper/bundler/metadata declarations, and a single explicit lowering boundary into the existing `Authoring IR -> normalize -> build -> inspect -> diff` pipeline.

**Architecture:** Keep `contracts/` as the only normative pipeline contract source, add the `Phase 5A` author-facing model inside `anki_forge::product`, and route every product feature through a reviewable `LoweringPlan` that produces `AuthoringDocument` plus layered diagnostics. Extend `Authoring IR` and `Normalized IR` only where helper output, explicit template/style lowering, browser appearance, template target-deck declarations, and narrow field-label metadata cannot be represented cleanly today; keep `deck override` aligned with Anki's template/card-lane target-deck semantics instead of rewriting final note `deck_name`.

**Tech Stack:** Rust workspace (`cargo`, `serde`, `serde_json`, `anyhow`), existing `anki_forge`, `authoring_core`, and `writer_core` crates, JSON Schema contracts, contract semantics docs, fixture-driven tests, Markdown docs/checklists

---

## Scope Check

This plan still targets one coherent subsystem: `Phase 5A Product Authoring Features`.

The approved spec intentionally bundles three ordered blocks under one author-facing lowering architecture:

- `Block 1`: high-level authoring API + first-class note types
- `Block 2`: template helper system
- `Block 3`: bundler + field metadata + browser/deck override

These are not separate product specs because they all depend on the same product-layer spine, the same layered diagnostics model, and the same `product layer -> Authoring IR` lowering boundary. The plan still separates them into implementation tasks so execution can stage risk and verification cleanly.

Do not split this into separate implementation plans unless the user explicitly asks to peel off one of these areas:

- `Block 1` only
- `Block 2` only
- `Block 3` only
- portability/docs only

## Execution Prerequisite

Before running any task below, create or switch to a dedicated worktree for this plan, for example:

```bash
git worktree add ../anki-forge-phase5a -b codex/phase-5a-product-authoring
cd ../anki-forge-phase5a
```

Execute the plan in that worktree, not on top of unrelated in-progress work.

## File Structure Map

### Product layer surface in `anki_forge`

- Modify: `anki_forge/src/lib.rs` - export the new `product` module without disturbing the existing runtime facade
- Create: `anki_forge/src/product/mod.rs` - public exports for the product-layer surface
- Create: `anki_forge/src/product/model.rs` - `ProductDocument`, closed note-type variants, note variants, bundler and metadata declaration DTOs
- Create: `anki_forge/src/product/builders.rs` - fluent builders and ergonomic Rust-first authoring entry points
- Create: `anki_forge/src/product/diagnostics.rs` - product diagnostics, lowering diagnostics, and error envelope types
- Create: `anki_forge/src/product/lowering.rs` - `LoweringPlan`, mapping evidence, and `ProductDocument::lower()`
- Create: `anki_forge/src/product/helpers.rs` - closed helper families and helper-specific lowering hooks
- Create: `anki_forge/src/product/assets.rs` - bundled asset declarations, inline/file sources, font bindings
- Create: `anki_forge/src/product/metadata.rs` - field-label metadata plus template browser-appearance and target-deck declarations
- Create: `anki_forge/examples/product_basic_flow.rs` - end-to-end Rust example from product layer to inspectable build
- Create: `anki_forge/tests/product_model_tests.rs` - product object model smoke tests
- Create: `anki_forge/tests/product_lowering_tests.rs` - lowering boundary tests for first-class note types and custom escape hatch
- Create: `anki_forge/tests/product_helper_tests.rs` - helper declaration and lowering tests
- Create: `anki_forge/tests/product_bundler_tests.rs` - asset/font bundler tests
- Create: `anki_forge/tests/product_pipeline_tests.rs` - lower -> normalize -> build -> inspect integration tests
- Create: `anki_forge/tests/product_portability_tests.rs` - data-driven portability constraint tests
- Create: `anki_forge/tests/fixtures/product/basic_answer_divider.case.json` - portable basic helper case
- Create: `anki_forge/tests/fixtures/product/io_font_bundle.case.json` - portable IO + font bundler case

### Phase 2 bridge and contract updates

- Modify: `contracts/schema/authoring-ir.schema.json` - add explicit lowered-notetype identity/config payloads, stock compatibility fields, and template browser/deck configuration
- Modify: `contracts/schema/normalized-ir.schema.json` - keep normalized output aligned with lowered note-type identities, field/template ords and ids, css, browser appearance, template deck targeting, and field-label metadata
- Modify: `contracts/semantics/normalization.md` - document explicit lowered notetype handling, source-grounded stock defaults, native compat identities/config, browser template carry-through, and field-label metadata semantics
- Modify: `authoring_core/src/model.rs` - add explicit authoring/notetype, field, and template DTOs including original stock/original id and field/template identity/config
- Modify: `authoring_core/src/lib.rs` - export the new DTOs
- Modify: `authoring_core/src/stock.rs` - prefer explicit lowered notetype payloads before stock fallback and expose source-grounded stock defaults plus stock compat metadata for product lowering
- Modify: `authoring_core/src/normalize.rs` - preserve explicit lowered field/template ids, ords, stock compat fields, browser appearance/template deck data, and field-label metadata into `NormalizedIr`
- Modify: `authoring_core/tests/normalization_pipeline_tests.rs` - cover explicit lowered notetype identities/config, browser appearance, template deck targeting, IO stock compatibility, and metadata carry-through
- Modify: `contract_tools/tests/schema_gate_tests.rs` - cover the new optional `Authoring IR`/`Normalized IR` shapes and inspect/browser observation schemas

### Writer/build/inspect bridge for Block 3

- Modify: `contracts/schema/inspect-report.schema.json` - define structured browser-template, template-deck, and field-label observations
- Modify: `contracts/semantics/build.md` - document lowered browser appearance, template target deck name-to-id resolution, and bundled template-static asset handling at build time
- Modify: `contracts/semantics/inspect.md` - document browser-template and field-label observations emitted by inspect
- Modify: `writer_core/src/staging.rs` - retain lowered template browser configuration, resolve template target deck names into stable ids, and preserve field-label metadata in staging output
- Modify: `writer_core/src/inspect.rs` - emit browser-template, template-target-deck, resolved deck id, and field-label observations
- Modify: `writer_core/tests/build_tests.rs` - prove bundled template-static assets survive the build path
- Modify: `writer_core/tests/inspect_tests.rs` - prove browser-template and field-label observations surface in inspect output
- Modify: `contract_tools/tests/cli_tests.rs` - assert CLI `inspect --output contract-json` exposes the new structured observations

### Docs and release evidence

- Modify: `contract_tools/Cargo.toml` - add a test-only dependency on the vendored upstream `anki` crate under `docs/source/rslib`
- Modify: `contract_tools/src/compat_oracle.rs` - add Phase 5A build/import/reimport oracle helpers against upstream import behavior
- Create: `contract_tools/tests/phase5a_roundtrip_oracle_tests.rs` - black-box repeated APKG import tests against vendored upstream rslib
- Modify: `README.md` - add a Rust-first Phase 5A example and explain the product layer boundary
- Create: `docs/superpowers/checklists/phase-5a-exit-evidence.md` - exact verification commands and evidence checklist for Phase 5A

## Implementation Notes

- Keep the `product layer` inside `anki_forge`; do not create a second semantic core crate.
- `LoweringPlan`, product diagnostics, and portability fixtures belong to `anki_forge`, not `contracts/`.
- Extend `Authoring IR` and `Normalized IR` only for explicit lowered notetype identities/config, fields, templates, css, browser template configuration, template target-deck data, and field-label metadata. Do not add builder-specific or helper-runtime-only fields to contracts.
- Keep `Basic/Cloze/ImageOcclusion` as the documented happy path. The custom/generic path is only a light escape hatch and should not receive more polish than needed to keep it usable.
- Keep Image Occlusion defaults source-grounded. Use the local upstream copies in `docs/source/rslib/src/image_occlusion/notetype.rs` and `docs/source/rslib/src/image_occlusion/notetype.css` as the baseline for default fields, templates, and css. Do not replace them with simplified hand-authored strings.
- Keep Image Occlusion field semantics source-grounded as well: preserve the stock `Comments` field for compatibility, but do not assume stock rendering must surface it directly.
- Keep downstream/native-facing notetype kinds aligned with Anki's real model: only `normal` and `cloze`. Represent Image Occlusion in lowered IR as `kind = "cloze"` plus explicit stock compatibility metadata such as `original_stock_kind = "image_occlusion"`, not as a third native kind.
- Preserve import/update compatibility data in lowered IR: `original_id`, field/template config ids, field/template ords, and field-level `tag`/`prevent_deletion` must survive lowering and normalization when present.
- Treat browser appearance and deck override as template/card-lane declarations grounded in the local upstream template config fields (`q_format_browser`, `a_format_browser`, `target_deck_id`, `browser_font_name`, `browser_font_size`), not as note-level overrides.
- Keep field metadata narrower and explicitly Forge-local: labels and field-role hints are acceptable, but do not present them as direct Anki Browser Appearance equivalents.
- Keep helper families closed and documented. No plugin or user-defined helper registration in this phase.
- Treat template-static assets as underscore-prefixed media with deterministic namespace/hash naming. Font/media examples and lowering rules should follow the underscore/static convention validated in `docs/source/rslib/src/media/check.rs`, `docs/source/rslib/src/import_export/package/apkg/import/media.rs`, and `docs/source/rslib/src/text.rs`, and should avoid bare shared filenames that import would silently skip.
- Build must resolve template target deck names into stable target deck ids before emitting native template config. Missing target decks should be handled by an explicit writer policy instead of implicit runtime behavior.
- Use inline base64 assets in tests whenever possible to avoid unnecessary fixture IO complexity.
- Portability tests must deserialize cases from `anki_forge/tests/fixtures/product/*.json`; they must not construct all cases only through Rust builder methods.
- Phase 5A exit evidence must include a real APKG round-trip oracle: build APKG A, import into a temporary upstream collection, rebuild APKG B from evolved product input, re-import, then assert notetype identity stability, field/template ord stability, hashed static-media update behavior, and valid template target deck resolution under upstream importer behavior.
- The custom escape hatch may remain in product-layer terminology, but it must lower to explicit `normal` or `cloze`-compatible downstream notetype shapes. Do not persist a new downstream `kind: "custom"` taxonomy into the pipeline.
- Add explicit regression coverage for IO field/template ord stability and import-compatible identity matching, not just happy-path lowering.
- Add frequent commits exactly as written in the tasks; each task should leave the repository in a passing state for its focused test slice.

### Task 1: Bootstrap the `anki_forge::product` surface

**Files:**
- Modify: `anki_forge/src/lib.rs`
- Create: `anki_forge/src/product/mod.rs`
- Create: `anki_forge/src/product/model.rs`
- Create: `anki_forge/src/product/diagnostics.rs`
- Test: `anki_forge/tests/product_model_tests.rs`

- [ ] **Step 1: Write the failing product-module smoke test**

```rust
// anki_forge/tests/product_model_tests.rs
use anki_forge::product::{ProductDocument, ProductNoteType};

#[test]
fn product_document_registers_a_basic_notetype() {
    let document = ProductDocument::new("demo-doc").with_basic("basic-main");

    assert_eq!(document.document_id(), "demo-doc");
    assert_eq!(document.note_types().len(), 1);
    assert!(matches!(
        &document.note_types()[0],
        ProductNoteType::Basic(notetype) if notetype.id == "basic-main"
    ));
}
```

- [ ] **Step 2: Run the smoke test to verify it fails**

Run: `cargo test -p anki_forge --test product_model_tests product_document_registers_a_basic_notetype -v`
Expected: FAIL with an unresolved import for `anki_forge::product`.

- [ ] **Step 3: Add the minimal product module, model, and diagnostics exports**

```rust
// anki_forge/src/lib.rs
pub mod product;
pub mod runtime;

pub use authoring_core::model::NormalizationResult;
pub use authoring_core::{
    assess_risk, normalize, parse_selector, resolve_identity, resolve_selector,
    to_canonical_json as to_authoring_canonical_json, AuthoringDocument, AuthoringMedia,
    AuthoringNote, AuthoringNotetype, ComparisonContext, MergeRiskReport, NormalizationRequest,
    NormalizedIr, NormalizedMedia, NormalizedNote, NormalizedNotetype, NormalizedTemplate,
    Selector, SelectorError, SelectorResolveError, SelectorTarget,
};
pub use writer_core::{
    build, build_context_ref, diff_reports, extract_media_references, inspect_apkg,
    inspect_build_result, inspect_staging, policy_ref,
    to_canonical_json as to_writer_canonical_json, BuildArtifactTarget, BuildContext, DiffReport,
    InspectReport, PackageBuildResult, VerificationGateRule, VerificationPolicy, WriterPolicy,
};
```

```rust
// anki_forge/src/product/mod.rs
pub mod diagnostics;
pub mod model;

pub use diagnostics::{LoweringDiagnostic, ProductDiagnostic, ProductLoweringError};
pub use model::{BasicNoteType, ProductDocument, ProductNoteType};
```

```rust
// anki_forge/src/product/diagnostics.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductDiagnostic {
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweringDiagnostic {
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductLoweringError {
    pub product_diagnostics: Vec<ProductDiagnostic>,
    pub lowering_diagnostics: Vec<LoweringDiagnostic>,
}
```

```rust
// anki_forge/src/product/model.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductDocument {
    document_id: String,
    note_types: Vec<ProductNoteType>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProductNoteType {
    Basic(BasicNoteType),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BasicNoteType {
    pub id: String,
    pub name: Option<String>,
}

impl ProductDocument {
    pub fn new(document_id: impl Into<String>) -> Self {
        Self {
            document_id: document_id.into(),
            note_types: Vec::new(),
        }
    }

    pub fn with_basic(mut self, id: impl Into<String>) -> Self {
        self.note_types.push(ProductNoteType::Basic(BasicNoteType {
            id: id.into(),
            name: None,
        }));
        self
    }

    pub fn document_id(&self) -> &str {
        &self.document_id
    }

    pub fn note_types(&self) -> &[ProductNoteType] {
        &self.note_types
    }
}
```

- [ ] **Step 4: Run the smoke test to verify it passes**

Run: `cargo test -p anki_forge --test product_model_tests product_document_registers_a_basic_notetype -v`
Expected: PASS with `product_document_registers_a_basic_notetype`.

- [ ] **Step 5: Commit**

```bash
git add anki_forge/src/lib.rs anki_forge/src/product/mod.rs anki_forge/src/product/model.rs anki_forge/src/product/diagnostics.rs anki_forge/tests/product_model_tests.rs
git commit -m "feat: bootstrap phase 5a product module"
```

### Task 2: Add the Block 1 basic builder path and explicit lowering boundary

**Files:**
- Modify: `anki_forge/src/product/mod.rs`
- Modify: `anki_forge/src/product/model.rs`
- Create: `anki_forge/src/product/builders.rs`
- Create: `anki_forge/src/product/lowering.rs`
- Test: `anki_forge/tests/product_lowering_tests.rs`

- [ ] **Step 1: Write the failing basic-lowering test**

```rust
// anki_forge/tests/product_lowering_tests.rs
use anki_forge::product::ProductDocument;

#[test]
fn basic_product_document_lowers_to_authoring_ir_with_mapping_evidence() {
    let lowering = ProductDocument::new("demo-doc")
        .with_basic("basic-main")
        .add_basic_note("basic-main", "note-1", "Default", "front", "back")
        .lower()
        .expect("lower basic product document");

    assert_eq!(lowering.authoring_document.kind, "authoring-ir");
    assert_eq!(lowering.authoring_document.notetypes[0].kind, "normal");
    assert_eq!(
        lowering.authoring_document.notes[0].fields["Front"],
        "front"
    );
    assert_eq!(lowering.mappings.len(), 2);
    assert!(lowering.product_diagnostics.is_empty());
    assert!(lowering.lowering_diagnostics.is_empty());
}
```

- [ ] **Step 2: Run the basic-lowering test to verify it fails**

Run: `cargo test -p anki_forge --test product_lowering_tests basic_product_document_lowers_to_authoring_ir_with_mapping_evidence -v`
Expected: FAIL because `add_basic_note()` and `lower()` do not exist yet.

- [ ] **Step 3: Add builder helpers, `LoweringPlan`, and the basic lowering implementation**

```rust
// anki_forge/src/product/mod.rs
pub mod builders;
pub mod diagnostics;
pub mod lowering;
pub mod model;

pub use diagnostics::{LoweringDiagnostic, ProductDiagnostic, ProductLoweringError};
pub use lowering::{LoweringMapping, LoweringPlan};
pub use model::{BasicNoteType, ProductDocument, ProductNote, ProductNoteType};
```

```rust
// anki_forge/src/product/model.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductDocument {
    document_id: String,
    note_types: Vec<ProductNoteType>,
    notes: Vec<ProductNote>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProductNoteType {
    Basic(BasicNoteType),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProductNote {
    Basic(BasicNote),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BasicNote {
    pub id: String,
    pub note_type_id: String,
    pub deck_name: String,
    pub front: String,
    pub back: String,
}

impl ProductDocument {
    pub fn notes(&self) -> &[ProductNote] {
        &self.notes
    }
}
```

```rust
// anki_forge/src/product/builders.rs
use super::lowering::lower_document;
use super::model::{BasicNote, ProductDocument, ProductNote};

impl ProductDocument {
    pub fn add_basic_note(
        mut self,
        note_type_id: impl Into<String>,
        note_id: impl Into<String>,
        deck_name: impl Into<String>,
        front: impl Into<String>,
        back: impl Into<String>,
    ) -> Self {
        self.notes.push(ProductNote::Basic(BasicNote {
            id: note_id.into(),
            note_type_id: note_type_id.into(),
            deck_name: deck_name.into(),
            front: front.into(),
            back: back.into(),
        }));
        self
    }

    pub fn lower(&self) -> Result<super::lowering::LoweringPlan, super::ProductLoweringError> {
        lower_document(self)
    }
}
```

```rust
// anki_forge/src/product/lowering.rs
use std::collections::BTreeMap;

use authoring_core::{AuthoringDocument, AuthoringNote, AuthoringNotetype};

use super::diagnostics::ProductLoweringError;
use super::model::{ProductDocument, ProductNote, ProductNoteType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweringMapping {
    pub source_kind: &'static str,
    pub source_id: String,
    pub target_kind: &'static str,
    pub target_id: String,
}

#[derive(Debug, Clone)]
pub struct LoweringPlan {
    pub authoring_document: AuthoringDocument,
    pub mappings: Vec<LoweringMapping>,
    pub product_diagnostics: Vec<super::ProductDiagnostic>,
    pub lowering_diagnostics: Vec<super::LoweringDiagnostic>,
}

pub fn lower_document(document: &ProductDocument) -> Result<LoweringPlan, ProductLoweringError> {
    let notetypes = document
        .note_types()
        .iter()
        .map(|note_type| match note_type {
            ProductNoteType::Basic(notetype) => AuthoringNotetype {
                id: notetype.id.clone(),
                kind: "normal".into(),
                name: notetype.name.clone(),
            },
        })
        .collect::<Vec<_>>();

    let notes = document
        .notes()
        .iter()
        .map(|note| match note {
            ProductNote::Basic(note) => AuthoringNote {
                id: note.id.clone(),
                notetype_id: note.note_type_id.clone(),
                deck_name: note.deck_name.clone(),
                fields: BTreeMap::from([
                    ("Front".into(), note.front.clone()),
                    ("Back".into(), note.back.clone()),
                ]),
                tags: Vec::new(),
            },
        })
        .collect::<Vec<_>>();

    Ok(LoweringPlan {
        authoring_document: AuthoringDocument {
            kind: "authoring-ir".into(),
            schema_version: "0.1.0".into(),
            metadata_document_id: document.document_id().into(),
            notetypes,
            notes,
            media: Vec::new(),
        },
        mappings: notetypes
            .iter()
            .map(|notetype| LoweringMapping {
                source_kind: "product_notetype",
                source_id: notetype.id.clone(),
                target_kind: "authoring_notetype",
                target_id: notetype.id.clone(),
            })
            .chain(notes.iter().map(|note| LoweringMapping {
                source_kind: "product_note",
                source_id: note.id.clone(),
                target_kind: "authoring_note",
                target_id: note.id.clone(),
            }))
            .collect(),
        product_diagnostics: Vec::new(),
        lowering_diagnostics: Vec::new(),
    })
}
```

- [ ] **Step 4: Run the basic-lowering test to verify it passes**

Run: `cargo test -p anki_forge --test product_lowering_tests basic_product_document_lowers_to_authoring_ir_with_mapping_evidence -v`
Expected: PASS with `basic_product_document_lowers_to_authoring_ir_with_mapping_evidence`.

- [ ] **Step 5: Commit**

```bash
git add anki_forge/src/product/mod.rs anki_forge/src/product/model.rs anki_forge/src/product/builders.rs anki_forge/src/product/lowering.rs anki_forge/tests/product_lowering_tests.rs
git commit -m "feat: add basic product lowering boundary"
```

### Task 3: Add `Cloze` and `ImageOcclusion`, plus layered diagnostics for invalid product input

**Files:**
- Modify: `anki_forge/src/product/model.rs`
- Modify: `anki_forge/src/product/builders.rs`
- Modify: `anki_forge/src/product/diagnostics.rs`
- Modify: `anki_forge/src/product/lowering.rs`
- Test: `anki_forge/tests/product_lowering_tests.rs`

- [ ] **Step 1: Write the failing variant and diagnostics tests**

```rust
// anki_forge/tests/product_lowering_tests.rs
use anki_forge::product::ProductDocument;

#[test]
fn cloze_and_image_occlusion_lanes_lower_to_stock_compatible_authoring_shapes() {
    let cloze = ProductDocument::new("cloze-doc")
        .with_cloze("cloze-main")
        .add_cloze_note("cloze-main", "note-1", "Default", "A {{c1::cloze}} card", "extra")
        .lower()
        .expect("lower cloze document");

    assert_eq!(cloze.authoring_document.notetypes[0].kind, "cloze");
    assert_eq!(cloze.authoring_document.notes[0].fields["Text"], "A {{c1::cloze}} card");

    let io = ProductDocument::new("io-doc")
        .with_image_occlusion("io-main")
        .add_image_occlusion_note(
            "io-main",
            "note-io-1",
            "Default",
            "{{c1::Mask 1}}",
            "<img src=\"mask.png\">",
            "Header",
            "Extra",
            "Comments",
        )
        .lower()
        .expect("lower io document");

    assert_eq!(io.authoring_document.notetypes[0].kind, "cloze");
    assert_eq!(io.authoring_document.notes[0].fields["Header"], "Header");
}

#[test]
fn image_occlusion_missing_image_emits_product_diagnostic() {
    let error = ProductDocument::new("io-doc")
        .with_image_occlusion("io-main")
        .add_image_occlusion_note("io-main", "note-io-1", "Default", "{{c1::Mask 1}}", "", "", "", "")
        .lower()
        .expect_err("expected missing image to fail");

    assert!(error
        .product_diagnostics
        .iter()
        .any(|item| item.code == "PHASE5A.IO_IMAGE_REQUIRED"));
}
```

- [ ] **Step 2: Run the variant tests to verify they fail**

Run: `cargo test -p anki_forge --test product_lowering_tests cloze_and_image_occlusion_lanes_lower_to_stock_compatible_authoring_shapes -v`
Expected: FAIL because the new builders and stock-compatible IO lowering are missing.

Run: `cargo test -p anki_forge --test product_lowering_tests image_occlusion_missing_image_emits_product_diagnostic -v`
Expected: FAIL because the new builders and diagnostics are missing.

- [ ] **Step 3: Add the new note-type variants, note variants, and validation diagnostics**

```rust
// anki_forge/src/product/diagnostics.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductDiagnostic {
    pub code: &'static str,
    pub message: String,
}

impl ProductDiagnostic {
    pub fn io_image_required(note_id: &str) -> Self {
        Self {
            code: "PHASE5A.IO_IMAGE_REQUIRED",
            message: format!("image occlusion note {note_id} requires a non-empty Image field"),
        }
    }
}
```

```rust
// anki_forge/src/product/model.rs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProductNoteType {
    Basic(BasicNoteType),
    Cloze(ClozeNoteType),
    ImageOcclusion(ImageOcclusionNoteType),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProductNote {
    Basic(BasicNote),
    Cloze(ClozeNote),
    ImageOcclusion(ImageOcclusionNote),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClozeNoteType {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageOcclusionNoteType {
    pub id: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClozeNote {
    pub id: String,
    pub note_type_id: String,
    pub deck_name: String,
    pub text: String,
    pub back_extra: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageOcclusionNote {
    pub id: String,
    pub note_type_id: String,
    pub deck_name: String,
    pub occlusion: String,
    pub image: String,
    pub header: String,
    pub back_extra: String,
    pub comments: String,
}
```

```rust
// anki_forge/src/product/builders.rs
use super::model::{
    ClozeNote, ClozeNoteType, ImageOcclusionNote, ImageOcclusionNoteType, ProductDocument,
    ProductNote, ProductNoteType,
};

impl ProductDocument {
    pub fn with_cloze(mut self, id: impl Into<String>) -> Self {
        self.note_types.push(ProductNoteType::Cloze(ClozeNoteType {
            id: id.into(),
            name: None,
        }));
        self
    }

    pub fn with_image_occlusion(mut self, id: impl Into<String>) -> Self {
        self.note_types
            .push(ProductNoteType::ImageOcclusion(ImageOcclusionNoteType {
                id: id.into(),
                name: None,
            }));
        self
    }

    pub fn add_cloze_note(
        mut self,
        note_type_id: impl Into<String>,
        note_id: impl Into<String>,
        deck_name: impl Into<String>,
        text: impl Into<String>,
        back_extra: impl Into<String>,
    ) -> Self {
        self.notes.push(ProductNote::Cloze(ClozeNote {
            id: note_id.into(),
            note_type_id: note_type_id.into(),
            deck_name: deck_name.into(),
            text: text.into(),
            back_extra: back_extra.into(),
        }));
        self
    }

    pub fn add_image_occlusion_note(
        mut self,
        note_type_id: impl Into<String>,
        note_id: impl Into<String>,
        deck_name: impl Into<String>,
        occlusion: impl Into<String>,
        image: impl Into<String>,
        header: impl Into<String>,
        back_extra: impl Into<String>,
        comments: impl Into<String>,
    ) -> Self {
        self.notes.push(ProductNote::ImageOcclusion(ImageOcclusionNote {
            id: note_id.into(),
            note_type_id: note_type_id.into(),
            deck_name: deck_name.into(),
            occlusion: occlusion.into(),
            image: image.into(),
            header: header.into(),
            back_extra: back_extra.into(),
            comments: comments.into(),
        }));
        self
    }
}
```

```rust
// anki_forge/src/product/lowering.rs
use super::model::{ProductDocument, ProductNote, ProductNoteType};

pub fn lower_document(document: &ProductDocument) -> Result<LoweringPlan, ProductLoweringError> {
    let mut product_diagnostics = Vec::new();

    let notetypes = document
        .note_types()
        .iter()
        .map(|note_type| match note_type {
            ProductNoteType::Basic(notetype) => AuthoringNotetype {
                id: notetype.id.clone(),
                kind: "normal".into(),
                name: notetype.name.clone(),
            },
            ProductNoteType::Cloze(notetype) => AuthoringNotetype {
                id: notetype.id.clone(),
                kind: "cloze".into(),
                name: notetype.name.clone(),
            },
            ProductNoteType::ImageOcclusion(notetype) => AuthoringNotetype {
                id: notetype.id.clone(),
                kind: "cloze".into(),
                name: notetype.name.clone(),
            },
        })
        .collect::<Vec<_>>();

    let notes = document
        .notes()
        .iter()
        .filter_map(|note| match note {
            ProductNote::Basic(note) => Some(AuthoringNote {
                id: note.id.clone(),
                notetype_id: note.note_type_id.clone(),
                deck_name: note.deck_name.clone(),
                fields: BTreeMap::from([
                    ("Front".into(), note.front.clone()),
                    ("Back".into(), note.back.clone()),
                ]),
                tags: Vec::new(),
            }),
            ProductNote::Cloze(note) => Some(AuthoringNote {
                id: note.id.clone(),
                notetype_id: note.note_type_id.clone(),
                deck_name: note.deck_name.clone(),
                fields: BTreeMap::from([
                    ("Text".into(), note.text.clone()),
                    ("Back Extra".into(), note.back_extra.clone()),
                ]),
                tags: Vec::new(),
            }),
            ProductNote::ImageOcclusion(note) => {
                if note.image.trim().is_empty() {
                    product_diagnostics.push(super::ProductDiagnostic::io_image_required(&note.id));
                    None
                } else {
                    Some(AuthoringNote {
                        id: note.id.clone(),
                        notetype_id: note.note_type_id.clone(),
                        deck_name: note.deck_name.clone(),
                        fields: BTreeMap::from([
                            ("Occlusion".into(), note.occlusion.clone()),
                            ("Image".into(), note.image.clone()),
                            ("Header".into(), note.header.clone()),
                            ("Back Extra".into(), note.back_extra.clone()),
                            ("Comments".into(), note.comments.clone()),
                        ]),
                        tags: Vec::new(),
                    })
                }
            }
        })
        .collect::<Vec<_>>();

    if !product_diagnostics.is_empty() {
        return Err(ProductLoweringError {
            product_diagnostics,
            lowering_diagnostics: Vec::new(),
        });
    }

    let mappings = notetypes
        .iter()
        .map(|notetype| LoweringMapping {
            source_kind: "product_notetype",
            source_id: notetype.id.clone(),
            target_kind: "authoring_notetype",
            target_id: notetype.id.clone(),
        })
        .chain(notes.iter().map(|note| LoweringMapping {
            source_kind: "product_note",
            source_id: note.id.clone(),
            target_kind: "authoring_note",
            target_id: note.id.clone(),
        }))
        .collect::<Vec<_>>();

    Ok(LoweringPlan {
        authoring_document: AuthoringDocument {
            kind: "authoring-ir".into(),
            schema_version: "0.1.0".into(),
            metadata_document_id: document.document_id().into(),
            notetypes,
            notes,
            media: Vec::new(),
        },
        mappings,
        product_diagnostics: Vec::new(),
        lowering_diagnostics: Vec::new(),
    })
}
```

- [ ] **Step 4: Run the variant tests to verify they pass**

Run: `cargo test -p anki_forge --test product_lowering_tests cloze_and_image_occlusion_lanes_lower_to_stock_compatible_authoring_shapes -v`
Expected: PASS.

Run: `cargo test -p anki_forge --test product_lowering_tests image_occlusion_missing_image_emits_product_diagnostic -v`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add anki_forge/src/product/model.rs anki_forge/src/product/builders.rs anki_forge/src/product/diagnostics.rs anki_forge/src/product/lowering.rs anki_forge/tests/product_lowering_tests.rs
git commit -m "feat: add first-class cloze and io product lanes"
```

### Task 4: Extend the Phase 2 bridge for explicit lowered identity/config payloads, stock compatibility, browser appearance, template target decks, and the custom escape hatch

**Files:**
- Modify: `contracts/schema/authoring-ir.schema.json`
- Modify: `contracts/schema/normalized-ir.schema.json`
- Modify: `contracts/semantics/normalization.md`
- Modify: `authoring_core/src/model.rs`
- Modify: `authoring_core/src/lib.rs`
- Modify: `authoring_core/src/stock.rs`
- Modify: `authoring_core/src/normalize.rs`
- Modify: `authoring_core/tests/normalization_pipeline_tests.rs`
- Modify: `contract_tools/tests/schema_gate_tests.rs`
- Modify: `anki_forge/src/product/model.rs`
- Modify: `anki_forge/src/product/builders.rs`
- Modify: `anki_forge/src/product/lowering.rs`
- Test: `anki_forge/tests/product_lowering_tests.rs`

- [ ] **Step 1: Write the failing schema and normalization tests**

```rust
// contract_tools/tests/schema_gate_tests.rs
#[test]
fn authoring_ir_schema_accepts_explicit_lowered_stock_compatible_notetype_shape() {
    let manifest = load_manifest(contract_manifest_path()).unwrap();
    let schema =
        load_schema(resolve_asset_path(&manifest, "authoring_ir_schema").unwrap()).unwrap();
    let value = json!({
        "kind": "authoring-ir",
        "schema_version": "0.1.0",
        "metadata": { "document_id": "demo-doc" },
        "notetypes": [{
            "id": "io-main",
            "kind": "cloze",
            "original_stock_kind": "image_occlusion",
            "original_id": 1234,
            "name": "Image Occlusion",
            "fields": [{
                "name": "Occlusion",
                "ord": 0,
                "config_id": 101,
                "tag": 0,
                "prevent_deletion": true
            }, {
                "name": "Image",
                "ord": 1,
                "config_id": 102,
                "tag": 1,
                "prevent_deletion": true
            }, {
                "name": "Header",
                "ord": 2,
                "config_id": 103,
                "tag": 2,
                "prevent_deletion": true
            }, {
                "name": "Back Extra",
                "ord": 3,
                "config_id": 104,
                "tag": 3,
                "prevent_deletion": true
            }, {
                "name": "Comments",
                "ord": 4,
                "config_id": 105,
                "tag": 4,
                "prevent_deletion": false
            }],
            "templates": [{
                "name": "Image Occlusion",
                "ord": 0,
                "config_id": 201,
                "question_format": "{{cloze:Occlusion}}",
                "answer_format": "{{cloze:Occlusion}}<br>{{Back Extra}}",
                "question_format_browser": "<span>{{Header}}</span>",
                "answer_format_browser": "<span>{{Back Extra}}</span>",
                "target_deck_name": "Custom::Deck",
                "browser_font_name": "Arial",
                "browser_font_size": 18
            }],
            "css": ".card { color: black; }",
            "field_metadata": [{
                "field_name": "Header",
                "label": "Header",
                "role_hint": "context"
            }]
        }],
        "notes": [{
            "id": "note-1",
            "notetype_id": "io-main",
            "deck_name": "Default",
            "fields": {
                "Occlusion": "{{c1::Mask}}",
                "Image": "<img src=\"mask.png\">",
                "Header": "head",
                "Back Extra": "extra",
                "Comments": "comments"
            }
        }]
    });

    assert!(validate_value(&schema, &value).is_ok());
}
```

```rust
// authoring_core/tests/normalization_pipeline_tests.rs
#[test]
fn explicit_lowered_notetype_identities_and_io_config_survive_normalization() {
    let input = AuthoringDocument {
        kind: "authoring-ir".into(),
        schema_version: "0.1.0".into(),
        metadata_document_id: "demo-doc".into(),
        notetypes: vec![AuthoringNotetype {
            id: "io-main".into(),
            kind: "cloze".into(),
            original_stock_kind: Some("image_occlusion".into()),
            original_id: Some(1234),
            name: Some("Image Occlusion".into()),
            fields: Some(vec![
                AuthoringField {
                    name: "Occlusion".into(),
                    ord: Some(0),
                    config_id: Some(101),
                    tag: Some(0),
                    prevent_deletion: Some(true),
                },
                AuthoringField {
                    name: "Image".into(),
                    ord: Some(1),
                    config_id: Some(102),
                    tag: Some(1),
                    prevent_deletion: Some(true),
                },
                AuthoringField {
                    name: "Header".into(),
                    ord: Some(2),
                    config_id: Some(103),
                    tag: Some(2),
                    prevent_deletion: Some(true),
                },
            ]),
            templates: Some(vec![AuthoringTemplate {
                name: "Image Occlusion".into(),
                ord: Some(0),
                config_id: Some(201),
                question_format: "{{cloze:Occlusion}}".into(),
                answer_format: "{{cloze:Occlusion}}<br>{{Back Extra}}".into(),
                question_format_browser: Some("<span>{{Header}}</span>".into()),
                answer_format_browser: Some("<span>{{Back Extra}}</span>".into()),
                target_deck_name: Some("Custom::Deck".into()),
                browser_font_name: Some("Arial".into()),
                browser_font_size: Some(18),
            }]),
            css: Some(".card { color: black; }".into()),
            field_metadata: vec![AuthoringFieldMetadata {
                field_name: "Header".into(),
                label: Some("Header".into()),
                role_hint: Some("context".into()),
            }],
        }],
        notes: vec![AuthoringNote {
            id: "note-1".into(),
            notetype_id: "io-main".into(),
            deck_name: "Default".into(),
            fields: string_map(json!({
                "Occlusion": "{{c1::Mask}}",
                "Image": "<img src=\"mask.png\">",
                "Header": "head",
                "Back Extra": "extra"
            })),
            tags: vec![],
        }],
        media: vec![],
    };

    let result = normalize(NormalizationRequest::new(input));
    let normalized = result.normalized_ir.expect("normalized ir");

    assert_eq!(normalized.notetypes[0].kind, "cloze");
    assert_eq!(normalized.notetypes[0].original_stock_kind.as_deref(), Some("image_occlusion"));
    assert_eq!(normalized.notetypes[0].original_id, Some(1234));
    assert_eq!(normalized.notetypes[0].fields[0].config_id, Some(101));
    assert_eq!(normalized.notetypes[0].fields[0].ord, Some(0));
    assert_eq!(normalized.notetypes[0].fields[0].tag, Some(0));
    assert_eq!(normalized.notetypes[0].fields[0].prevent_deletion, Some(true));
    assert_eq!(normalized.notetypes[0].templates[0].target_deck_name.as_deref(), Some("Custom::Deck"));
    assert_eq!(normalized.notetypes[0].templates[0].config_id, Some(201));
    assert_eq!(normalized.notetypes[0].templates[0].ord, Some(0));
    assert_eq!(normalized.notetypes[0].templates[0].browser_font_name.as_deref(), Some("Arial"));
    assert_eq!(normalized.notetypes[0].field_metadata[0].role_hint.as_deref(), Some("context"));
}
```

```rust
// anki_forge/tests/product_lowering_tests.rs
#[test]
fn custom_escape_hatch_lowers_to_explicit_authoring_normal_notetype_shape() {
    let lowering = ProductDocument::new("custom-doc")
        .with_custom_notetype(
            "custom-main",
            "Custom Card",
            vec!["Prompt", "Response"],
            vec![("Card 1", "{{Prompt}}", "{{FrontSide}}<hr id=answer>{{Response}}")],
            ".card { color: red; }",
        )
        .add_custom_note(
            "custom-main",
            "note-1",
            "Default",
            vec![("Prompt", "front"), ("Response", "back")],
        )
        .lower()
        .expect("lower custom note type");

    let notetype = &lowering.authoring_document.notetypes[0];
    assert_eq!(notetype.kind, "normal");
    assert_eq!(notetype.fields.as_ref().unwrap()[0].name, "Prompt");
    assert_eq!(notetype.fields.as_ref().unwrap()[1].name, "Response");
    assert_eq!(notetype.templates.as_ref().unwrap()[0].name, "Card 1");
}
```

- [ ] **Step 2: Run the failing schema, normalization, and custom-lowering tests**

Run: `cargo test -p contract_tools --test schema_gate_tests authoring_ir_schema_accepts_explicit_lowered_stock_compatible_notetype_shape -v`
Expected: FAIL because the schema rejects explicit lowered stock-compat fields such as `original_stock_kind`, `original_id`, field/template ids and ords, and field `tag`/`prevent_deletion`.

Run: `cargo test -p authoring_core --test normalization_pipeline_tests explicit_lowered_notetype_identities_and_io_config_survive_normalization -v`
Expected: FAIL because the authoring and normalized models do not yet carry explicit lowered stock compatibility identities/config, template browser fields, template target decks, and field-label metadata.

Run: `cargo test -p anki_forge --test product_lowering_tests custom_escape_hatch_lowers_to_explicit_authoring_normal_notetype_shape -v`
Expected: FAIL because the custom escape hatch does not yet lower into an explicit downstream `normal` notetype shape.

- [ ] **Step 3: Add the minimal contract/model bridge and custom escape hatch**

```rust
// authoring_core/src/model.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthoringField {
    pub name: String,
    #[serde(default)]
    pub ord: Option<u32>,
    #[serde(default)]
    pub config_id: Option<i64>,
    #[serde(default)]
    pub tag: Option<u32>,
    #[serde(default)]
    pub prevent_deletion: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthoringTemplate {
    pub name: String,
    #[serde(default)]
    pub ord: Option<u32>,
    #[serde(default)]
    pub config_id: Option<i64>,
    pub question_format: String,
    pub answer_format: String,
    #[serde(default)]
    pub question_format_browser: Option<String>,
    #[serde(default)]
    pub answer_format_browser: Option<String>,
    #[serde(default)]
    pub target_deck_name: Option<String>,
    #[serde(default)]
    pub browser_font_name: Option<String>,
    #[serde(default)]
    pub browser_font_size: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthoringFieldMetadata {
    pub field_name: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub role_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthoringNotetype {
    pub id: String,
    pub kind: String,
    #[serde(default)]
    pub original_stock_kind: Option<String>,
    #[serde(default)]
    pub original_id: Option<i64>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub fields: Option<Vec<AuthoringField>>,
    #[serde(default)]
    pub templates: Option<Vec<AuthoringTemplate>>,
    #[serde(default)]
    pub css: Option<String>,
    #[serde(default)]
    pub field_metadata: Vec<AuthoringFieldMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedField {
    pub name: String,
    #[serde(default)]
    pub ord: Option<u32>,
    #[serde(default)]
    pub config_id: Option<i64>,
    #[serde(default)]
    pub tag: Option<u32>,
    #[serde(default)]
    pub prevent_deletion: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedFieldMetadata {
    pub field_name: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub role_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedTemplate {
    pub name: String,
    #[serde(default)]
    pub ord: Option<u32>,
    #[serde(default)]
    pub config_id: Option<i64>,
    pub question_format: String,
    pub answer_format: String,
    #[serde(default)]
    pub question_format_browser: Option<String>,
    #[serde(default)]
    pub answer_format_browser: Option<String>,
    #[serde(default)]
    pub target_deck_name: Option<String>,
    #[serde(default)]
    pub browser_font_name: Option<String>,
    #[serde(default)]
    pub browser_font_size: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedNotetype {
    pub id: String,
    pub kind: String,
    #[serde(default)]
    pub original_stock_kind: Option<String>,
    #[serde(default)]
    pub original_id: Option<i64>,
    pub name: String,
    pub fields: Vec<NormalizedField>,
    pub templates: Vec<NormalizedTemplate>,
    pub css: String,
    #[serde(default)]
    pub field_metadata: Vec<NormalizedFieldMetadata>,
}
```

```rust
// authoring_core/src/stock.rs
use crate::model::{
    AuthoringFieldMetadata, AuthoringNotetype, AuthoringTemplate, NormalizedField,
    NormalizedFieldMetadata, NormalizedNotetype, NormalizedTemplate,
};

pub fn resolve_stock_notetype(input: &AuthoringNotetype) -> Result<NormalizedNotetype> {
    if let (Some(fields), Some(templates)) = (&input.fields, &input.templates) {
        return Ok(NormalizedNotetype {
            id: input.id.clone(),
            kind: input.kind.clone(),
            original_stock_kind: input.original_stock_kind.clone(),
            original_id: input.original_id,
            name: normalized_name(input),
            fields: fields
                .iter()
                .map(|field| NormalizedField {
                    name: field.name.clone(),
                    ord: field.ord,
                    config_id: field.config_id,
                    tag: field.tag,
                    prevent_deletion: field.prevent_deletion,
                })
                .collect(),
            templates: templates
                .iter()
                .map(|template| NormalizedTemplate {
                    name: template.name.clone(),
                    ord: template.ord,
                    config_id: template.config_id,
                    question_format: template.question_format.clone(),
                    answer_format: template.answer_format.clone(),
                    question_format_browser: template.question_format_browser.clone(),
                    answer_format_browser: template.answer_format_browser.clone(),
                    target_deck_name: template.target_deck_name.clone(),
                    browser_font_name: template.browser_font_name.clone(),
                    browser_font_size: template.browser_font_size,
                })
                .collect(),
            css: input.css.clone().unwrap_or_default(),
            field_metadata: input
                .field_metadata
                .iter()
                .map(|field| NormalizedFieldMetadata {
                    field_name: field.field_name.clone(),
                    label: field.label.clone(),
                    role_hint: field.role_hint.clone(),
                })
                .collect(),
        });
    }

    // keep the existing stock fallback branches for normal/cloze stock kinds,
    // using original_stock_kind to distinguish Image Occlusion when present
}
```

```rust
// authoring_core/src/stock.rs
// expose stock defaults so product lowering can stay source-grounded
pub struct StockLoweringDefaults {
    pub kind: &'static str,
    pub original_stock_kind: Option<&'static str>,
    pub fields: Vec<AuthoringField>,
    pub templates: Vec<AuthoringTemplate>,
    pub css: String,
}

pub fn stock_lowering_defaults(kind: &str) -> Result<StockLoweringDefaults> {
    // derive Basic/Cloze defaults from the local stock definitions in
    // docs/source/rslib/src/notetype/stock.rs and Image Occlusion defaults from
    // docs/source/rslib/src/image_occlusion/notetype.rs + notetype.css,
    // including original_stock_kind, field/template ids when available,
    // field tag/prevent_deletion flags, and stable ord assignments
}
```

```rust
// anki_forge/src/product/lowering.rs
let defaults = stock_lowering_defaults(product_kind)?;
authoring_notetype.kind = defaults.kind.into();
authoring_notetype.original_stock_kind = defaults.original_stock_kind.map(str::to_owned);
authoring_notetype.fields = Some(defaults.fields);
authoring_notetype.templates = Some(defaults.templates);
authoring_notetype.css = Some(defaults.css);
```

```rust
// contracts/schema/authoring-ir.schema.json
"kind": {
  "enum": ["normal", "cloze"]
},
"original_stock_kind": {
  "type": "string",
  "enum": ["basic", "basic_and_reversed", "basic_optional_reversed", "basic_typing", "cloze", "image_occlusion"]
},
"original_id": {
  "type": "integer"
},
"fields": {
  "type": "array",
  "items": {
    "type": "object",
    "required": ["name"],
    "additionalProperties": false,
    "properties": {
      "name": { "type": "string", "minLength": 1 },
      "ord": { "type": "integer", "minimum": 0 },
      "config_id": { "type": "integer" },
      "tag": { "type": "integer", "minimum": 0 },
      "prevent_deletion": { "type": "boolean" }
    }
  }
},
"templates": {
  "type": "array",
  "items": {
    "type": "object",
    "required": ["name", "question_format", "answer_format"],
    "additionalProperties": false,
    "properties": {
      "name": { "type": "string", "minLength": 1 },
      "ord": { "type": "integer", "minimum": 0 },
      "config_id": { "type": "integer" },
      "question_format": { "type": "string", "minLength": 1 },
      "answer_format": { "type": "string", "minLength": 1 },
      "question_format_browser": { "type": "string", "minLength": 1 },
      "answer_format_browser": { "type": "string", "minLength": 1 },
      "target_deck_name": { "type": "string", "minLength": 1 },
      "browser_font_name": { "type": "string", "minLength": 1 },
      "browser_font_size": { "type": "integer", "minimum": 1 }
    }
  }
},
"field_metadata": {
  "type": "array",
  "items": {
    "type": "object",
    "required": ["field_name"],
    "additionalProperties": false,
    "properties": {
      "field_name": { "type": "string", "minLength": 1 },
      "label": { "type": "string", "minLength": 1 },
      "role_hint": { "type": "string", "minLength": 1 }
    }
  }
}
```

```rust
// anki_forge/src/product/model.rs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomNoteType {
    pub id: String,
    pub name: String,
    pub fields: Vec<String>,
    pub templates: Vec<(String, String, String)>,
    pub css: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomNote {
    pub id: String,
    pub note_type_id: String,
    pub deck_name: String,
    pub fields: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProductNoteType {
    Basic(BasicNoteType),
    Cloze(ClozeNoteType),
    ImageOcclusion(ImageOcclusionNoteType),
    Custom(CustomNoteType),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProductNote {
    Basic(BasicNote),
    Cloze(ClozeNote),
    ImageOcclusion(ImageOcclusionNote),
    Custom(CustomNote),
}
```

```rust
// anki_forge/src/product/lowering.rs
fn lower_custom_notetype(note_type: &CustomNoteType) -> authoring_core::AuthoringNotetype {
    authoring_core::AuthoringNotetype {
        id: note_type.id.clone(),
        kind: "normal".into(),
        original_stock_kind: None,
        original_id: None,
        name: Some(note_type.name.clone()),
        fields: Some(
            note_type
                .fields
                .iter()
                .enumerate()
                .map(|(ord, name)| authoring_core::AuthoringField {
                    name: name.clone(),
                    ord: Some(ord as u32),
                    config_id: None,
                    tag: None,
                    prevent_deletion: None,
                })
                .collect(),
        ),
        templates: Some(
            note_type
                .templates
                .iter()
                .enumerate()
                .map(|(ord, (name, qfmt, afmt))| authoring_core::AuthoringTemplate {
                    name: name.clone(),
                    ord: Some(ord as u32),
                    config_id: None,
                    question_format: qfmt.clone(),
                    answer_format: afmt.clone(),
                    question_format_browser: None,
                    answer_format_browser: None,
                    target_deck_name: None,
                    browser_font_name: None,
                    browser_font_size: None,
                })
                .collect(),
        ),
        css: Some(note_type.css.clone()),
        field_metadata: Vec::new(),
    }
}
```

- [ ] **Step 4: Run the schema, normalization, and custom-lowering tests to verify they pass**

Run: `cargo test -p contract_tools --test schema_gate_tests authoring_ir_schema_accepts_explicit_lowered_stock_compatible_notetype_shape -v`
Expected: PASS.

Run: `cargo test -p authoring_core --test normalization_pipeline_tests explicit_lowered_notetype_identities_and_io_config_survive_normalization -v`
Expected: PASS.

Run: `cargo test -p anki_forge --test product_lowering_tests custom_escape_hatch_lowers_to_explicit_authoring_normal_notetype_shape -v`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add contracts/schema/authoring-ir.schema.json contracts/schema/normalized-ir.schema.json contracts/semantics/normalization.md authoring_core/src/model.rs authoring_core/src/lib.rs authoring_core/src/stock.rs authoring_core/src/normalize.rs authoring_core/tests/normalization_pipeline_tests.rs contract_tools/tests/schema_gate_tests.rs anki_forge/src/product/model.rs anki_forge/src/product/builders.rs anki_forge/src/product/lowering.rs anki_forge/tests/product_lowering_tests.rs
git commit -m "feat: bridge explicit lowered notetypes into phase 2"
```

### Task 5: Implement the closed helper system and helper-aware lowering

**Files:**
- Modify: `anki_forge/src/product/mod.rs`
- Modify: `anki_forge/src/product/model.rs`
- Modify: `anki_forge/src/product/builders.rs`
- Modify: `anki_forge/src/product/lowering.rs`
- Create: `anki_forge/src/product/helpers.rs`
- Test: `anki_forge/tests/product_helper_tests.rs`

- [ ] **Step 1: Write the failing helper tests**

```rust
// anki_forge/tests/product_helper_tests.rs
use anki_forge::product::{HelperDeclaration, ProductDocument};

#[test]
fn answer_divider_helper_injects_a_named_divider_into_basic_answer_template() {
    let lowering = ProductDocument::new("demo-doc")
        .with_basic("basic-main")
        .with_helper(
            "basic-main",
            HelperDeclaration::AnswerDivider {
                title: "Answer".into(),
            },
        )
        .add_basic_note("basic-main", "note-1", "Default", "front", "back")
        .lower()
        .expect("lower helper-enhanced document");

    let template = &lowering.authoring_document.notetypes[0].templates.as_ref().unwrap()[0];
    assert!(template.answer_format.contains("Answer"));
}

#[test]
fn back_extra_panel_helper_rejects_basic_note_types() {
    let error = ProductDocument::new("demo-doc")
        .with_basic("basic-main")
        .with_helper(
            "basic-main",
            HelperDeclaration::BackExtraPanel {
                title: Some("More".into()),
            },
        )
        .lower()
        .expect_err("expected invalid helper scope");

    assert!(error
        .product_diagnostics
        .iter()
        .any(|item| item.code == "PHASE5A.HELPER_SCOPE_INVALID"));
}
```

- [ ] **Step 2: Run the helper tests to verify they fail**

Run: `cargo test -p anki_forge --test product_helper_tests answer_divider_helper_injects_a_named_divider_into_basic_answer_template -v`
Expected: FAIL because helper declarations and helper-aware lowering do not exist yet.

Run: `cargo test -p anki_forge --test product_helper_tests back_extra_panel_helper_rejects_basic_note_types -v`
Expected: FAIL because helper declarations and helper-aware lowering do not exist yet.

- [ ] **Step 3: Add helper declarations and helper-specific lowering**

```rust
// anki_forge/src/product/helpers.rs
use super::diagnostics::ProductDiagnostic;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HelperDeclaration {
    AnswerDivider { title: String },
    BackExtraPanel { title: Option<String> },
}

pub fn apply_helpers(
    note_kind: &str,
    question_format: &str,
    answer_format: &str,
    helpers: &[HelperDeclaration],
) -> Result<(String, String), ProductDiagnostic> {
    let mut next_question = question_format.to_string();
    let mut next_answer = answer_format.to_string();

    for helper in helpers {
        match helper {
            HelperDeclaration::AnswerDivider { title } => {
                if note_kind != "basic" {
                    return Err(ProductDiagnostic {
                        code: "PHASE5A.HELPER_SCOPE_INVALID",
                        message: format!("AnswerDivider is only valid for basic note types, got {note_kind}"),
                    });
                }
                next_answer = next_answer.replace(
                    "<hr id=answer>",
                    &format!("<hr id=answer><div class=\"af-answer-divider\">{title}</div>"),
                );
            }
            HelperDeclaration::BackExtraPanel { title } => {
                if note_kind == "basic" {
                    return Err(ProductDiagnostic {
                        code: "PHASE5A.HELPER_SCOPE_INVALID",
                        message: "BackExtraPanel is only valid for Cloze and ImageOcclusion".into(),
                    });
                }
                let header = title.clone().unwrap_or_else(|| "More".into());
                next_answer.push_str(&format!(
                    "\n<div class=\"af-back-extra-panel\"><h3>{header}</h3>{{{{Back Extra}}}}</div>"
                ));
            }
        }
    }

    Ok((next_question, next_answer))
}
```

```rust
// anki_forge/src/product/model.rs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductDocument {
    document_id: String,
    note_types: Vec<ProductNoteType>,
    notes: Vec<ProductNote>,
    helpers: Vec<(String, super::helpers::HelperDeclaration)>,
}
```

```rust
// anki_forge/src/product/builders.rs
impl ProductDocument {
    pub fn with_helper(
        mut self,
        note_type_id: impl Into<String>,
        helper: super::helpers::HelperDeclaration,
    ) -> Self {
        self.helpers.push((note_type_id.into(), helper));
        self
    }

    pub fn helpers_for(
        &self,
        note_type_id: &str,
    ) -> Vec<super::helpers::HelperDeclaration> {
        self.helpers
            .iter()
            .filter(|(target, _)| target == note_type_id)
            .map(|(_, helper)| helper.clone())
            .collect()
    }
}
```

```rust
// anki_forge/src/product/lowering.rs
use super::helpers::apply_helpers;
use authoring_core::stock::stock_lowering_defaults;

fn lowered_templates_for_kind(
    kind: &str,
    helpers: &[super::helpers::HelperDeclaration],
) -> Result<(Vec<String>, Vec<authoring_core::AuthoringTemplate>, String), super::ProductDiagnostic> {
    let defaults = stock_lowering_defaults(kind).map_err(|error| super::ProductDiagnostic {
        code: "PHASE5A.STOCK_DEFAULTS_UNAVAILABLE",
        message: error.to_string(),
    })?;
    let base_template = defaults.templates.first().ok_or_else(|| super::ProductDiagnostic {
        code: "PHASE5A.STOCK_TEMPLATE_MISSING",
        message: format!("no stock template defaults available for {kind}"),
    })?;

    let (question_format, answer_format) = apply_helpers(
        kind,
        &base_template.question_format,
        &base_template.answer_format,
        helpers,
    )?;

    Ok((
        defaults.fields,
        vec![authoring_core::AuthoringTemplate {
            name: base_template.name.clone(),
            question_format,
            answer_format,
            question_format_browser: base_template.question_format_browser.clone(),
            answer_format_browser: base_template.answer_format_browser.clone(),
            target_deck_name: base_template.target_deck_name.clone(),
            browser_font_name: base_template.browser_font_name.clone(),
            browser_font_size: base_template.browser_font_size,
        }],
        defaults.css,
    ))
}
```

- [ ] **Step 4: Run the helper tests to verify they pass**

Run: `cargo test -p anki_forge --test product_helper_tests answer_divider_helper_injects_a_named_divider_into_basic_answer_template -v`
Expected: PASS.

Run: `cargo test -p anki_forge --test product_helper_tests back_extra_panel_helper_rejects_basic_note_types -v`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add anki_forge/src/product/mod.rs anki_forge/src/product/model.rs anki_forge/src/product/lowering.rs anki_forge/src/product/helpers.rs anki_forge/tests/product_helper_tests.rs
git commit -m "feat: add phase 5a helper lowering system"
```

### Task 6: Add the bundler declarations for inline assets and fonts

**Files:**
- Modify: `anki_forge/src/product/mod.rs`
- Modify: `anki_forge/src/product/model.rs`
- Modify: `anki_forge/src/product/builders.rs`
- Modify: `anki_forge/src/product/lowering.rs`
- Create: `anki_forge/src/product/assets.rs`
- Test: `anki_forge/tests/product_bundler_tests.rs`
- Test: `writer_core/tests/build_tests.rs`

- [ ] **Step 1: Write the failing bundler tests**

```rust
// anki_forge/tests/product_bundler_tests.rs
use anki_forge::product::ProductDocument;

#[test]
fn inline_font_asset_lowers_to_media_and_font_face_css() {
    let lowering = ProductDocument::new("demo-doc")
        .with_basic("basic-main")
        .bundle_inline_template_asset("basic-main", "card-font.woff2", "font/woff2", "AA==")
        .bind_font("basic-main", "Forge Sans", "card-font.woff2")
        .add_basic_note("basic-main", "note-1", "Default", "front", "back")
        .lower()
        .expect("lower bundled asset document");

    assert_eq!(lowering.authoring_document.media.len(), 1);
    assert!(lowering.authoring_document.media[0]
        .filename
        .starts_with("_basic-main_card-font-"));
    assert!(lowering.authoring_document.media[0]
        .filename
        .ends_with(".woff2"));
    assert!(lowering.authoring_document.notetypes[0]
        .css
        .as_deref()
        .unwrap()
        .contains("@font-face"));
    assert!(lowering.mappings.iter().any(|mapping| mapping.source_kind == "asset"));
}
```

```rust
// writer_core/tests/build_tests.rs
#[test]
fn build_preserves_bundled_media_entries() {
    let normalized = basic_normalized_ir();
    let result = writer_core::build(
        &normalized,
        &default_writer_policy(),
        &default_build_context(),
        &artifact_target("bundled-media"),
    )
    .expect("build succeeds");

    assert_eq!(result.result_status, "success");
    assert!(std::fs::read_to_string(
        std::path::Path::new(result.staging_ref.as_ref().unwrap()).join("manifest.json")
    )
    .expect("read manifest")
    .contains("media"));
}
```

- [ ] **Step 2: Run the bundler tests to verify they fail**

Run: `cargo test -p anki_forge --test product_bundler_tests inline_font_asset_lowers_to_media_and_font_face_css -v`
Expected: FAIL because asset declarations, deterministic template-static naming, and font bindings do not exist.

Run: `cargo test -p writer_core --test build_tests build_preserves_bundled_media_entries -v`
Expected: FAIL because the staging/build path does not yet carry the new bundled-media case.

- [ ] **Step 3: Add asset declarations, font bindings, and media lowering**

```rust
// anki_forge/src/product/assets.rs
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AssetSource {
    InlineTemplateStatic {
        namespace: String,
        filename: String,
        mime: String,
        data_base64: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FontBinding {
    pub note_type_id: String,
    pub family: String,
    pub filename: String,
}
```

```rust
// anki_forge/src/product/model.rs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductDocument {
    document_id: String,
    note_types: Vec<ProductNoteType>,
    notes: Vec<ProductNote>,
    helpers: Vec<(String, super::helpers::HelperDeclaration)>,
    assets: Vec<super::assets::AssetSource>,
    font_bindings: Vec<super::assets::FontBinding>,
}
```

```rust
// anki_forge/src/product/builders.rs
impl ProductDocument {
    pub fn bundle_inline_template_asset(
        mut self,
        namespace: impl Into<String>,
        filename: impl Into<String>,
        mime: impl Into<String>,
        data_base64: impl Into<String>,
    ) -> Self {
        self.assets.push(super::assets::AssetSource::InlineTemplateStatic {
            namespace: namespace.into(),
            filename: filename.into(),
            mime: mime.into(),
            data_base64: data_base64.into(),
        });
        self
    }

    pub fn bind_font(
        mut self,
        note_type_id: impl Into<String>,
        family: impl Into<String>,
        filename: impl Into<String>,
    ) -> Self {
        self.font_bindings.push(super::assets::FontBinding {
            note_type_id: note_type_id.into(),
            family: family.into(),
            filename: filename.into(),
        });
        self
    }

    pub fn assets(&self) -> &[super::assets::AssetSource] {
        &self.assets
    }

    pub fn font_bindings(&self) -> &[super::assets::FontBinding] {
        &self.font_bindings
    }
}
```

```rust
// anki_forge/src/product/lowering.rs
fn template_static_filename(namespace: &str, filename: &str, data_base64: &str) -> String {
    let sha1 = short_sha1_from_base64(data_base64);
    let logical_name = filename.trim_start_matches('_');
    let (stem, ext) = split_filename(logical_name);
    format!("_{}_{}-{}.{}", namespace, stem, sha1, ext)
}

let media = document
    .assets()
    .iter()
    .map(|asset| match asset {
        super::assets::AssetSource::InlineTemplateStatic {
            namespace,
            filename,
            mime,
            data_base64,
        } => authoring_core::AuthoringMedia {
            filename: template_static_filename(namespace, filename, data_base64),
            mime: mime.clone(),
            data_base64: data_base64.clone(),
        },
    })
    .collect::<Vec<_>>();

for binding in document.font_bindings() {
    if let Some(notetype) = authoring_notetypes
        .iter_mut()
        .find(|notetype| notetype.id == binding.note_type_id)
    {
        let stored_filename = document
            .assets()
            .iter()
            .find_map(|asset| match asset {
                super::assets::AssetSource::InlineTemplateStatic {
                    namespace,
                    filename,
                    data_base64,
                    ..
                } if filename == &binding.filename && namespace == &binding.note_type_id => {
                    Some(template_static_filename(namespace, filename, data_base64))
                }
                _ => None,
            })
            .expect("bound font asset must exist");
        let next_css = format!(
            "@font-face {{ font-family: '{}'; src: url('{}'); }}\n{}",
            binding.family,
            stored_filename,
            notetype.css.clone().unwrap_or_default()
        );
        notetype.css = Some(next_css);
        mappings.push(LoweringMapping {
            source_kind: "asset",
            source_id: binding.filename.clone(),
            target_kind: "authoring_media",
            target_id: stored_filename,
        });
    }
}
```

- [ ] **Step 4: Run the bundler tests to verify they pass**

Run: `cargo test -p anki_forge --test product_bundler_tests inline_font_asset_lowers_to_media_and_font_face_css -v`
Expected: PASS.

Run: `cargo test -p writer_core --test build_tests build_preserves_bundled_media_entries -v`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add anki_forge/src/product/mod.rs anki_forge/src/product/model.rs anki_forge/src/product/builders.rs anki_forge/src/product/lowering.rs anki_forge/src/product/assets.rs anki_forge/tests/product_bundler_tests.rs writer_core/tests/build_tests.rs
git commit -m "feat: add product asset bundler lowering"
```

### Task 7: Add field-label metadata, browser appearance declarations, template target decks, and downstream inspect propagation

**Files:**
- Modify: `anki_forge/src/product/mod.rs`
- Modify: `anki_forge/src/product/model.rs`
- Modify: `anki_forge/src/product/builders.rs`
- Modify: `anki_forge/src/product/lowering.rs`
- Create: `anki_forge/src/product/metadata.rs`
- Modify: `contracts/schema/inspect-report.schema.json`
- Modify: `contracts/semantics/build.md`
- Modify: `contracts/semantics/inspect.md`
- Modify: `writer_core/src/staging.rs`
- Modify: `writer_core/src/inspect.rs`
- Modify: `contract_tools/tests/cli_tests.rs`
- Test: `anki_forge/tests/product_pipeline_tests.rs`
- Test: `writer_core/tests/inspect_tests.rs`

- [ ] **Step 1: Write the failing metadata and inspect tests**

```rust
// anki_forge/tests/product_pipeline_tests.rs
use anki_forge::{
    build, normalize, BuildArtifactTarget, BuildContext, NormalizationRequest, WriterPolicy,
};
use anki_forge::product::{
    FieldMetadataDeclaration, ProductDocument, TemplateBrowserAppearanceDeclaration,
    TemplateTargetDeckDeclaration,
};

#[test]
fn browser_appearance_and_template_target_deck_survive_lower_normalize_and_build() {
    let lowering = ProductDocument::new("demo-doc")
        .with_default_deck("Default")
        .with_basic("basic-main")
        .with_field_metadata(
            "basic-main",
            FieldMetadataDeclaration {
                field_name: "Front".into(),
                label: Some("Prompt".into()),
                role_hint: Some("question".into()),
            },
        )
        .with_browser_appearance(
            "basic-main",
            TemplateBrowserAppearanceDeclaration {
                template_name: "Card 1".into(),
                question_format: Some("<span class=\"browser-front\">{{Front}}</span>".into()),
                answer_format: Some("<span class=\"browser-back\">{{Back}}</span>".into()),
                font_name: Some("Arial".into()),
                font_size: Some(18),
            },
        )
        .with_template_target_deck(
            "basic-main",
            TemplateTargetDeckDeclaration {
                template_name: "Card 1".into(),
                deck_name: "Custom::Deck".into(),
            },
        )
        .add_basic_note("basic-main", "note-1", "IGNORED", "front", "back")
        .lower()
        .expect("lower product document");

    assert_eq!(lowering.authoring_document.notes[0].deck_name, "Default");

    let normalized = normalize(NormalizationRequest::new(lowering.authoring_document));
    let normalized = normalized.normalized_ir.expect("normalized");
    let template = &normalized.notetypes[0].templates[0];
    assert_eq!(template.target_deck_name.as_deref(), Some("Custom::Deck"));
    assert_eq!(template.question_format_browser.as_deref(), Some("<span class=\"browser-front\">{{Front}}</span>"));
    assert_eq!(normalized.notetypes[0].field_metadata[0].role_hint.as_deref(), Some("question"));

    let build = build(
        &normalized,
        &WriterPolicy::default(),
        &BuildContext::default(),
        &BuildArtifactTarget::StagingOnly,
    )
    .expect("build succeeds");
    assert_eq!(build.result_status, "success");
    let report = anki_forge::inspect_staging(&build.staging_ref.expect("staging ref"))
        .expect("inspect staging");
    assert!(report
        .observations
        .template_target_decks
        .iter()
        .any(|value| value["target_deck_name"] == "Custom::Deck" && value["resolved_target_deck_id"].is_number()));
}
```

```rust
// writer_core/tests/inspect_tests.rs
#[test]
fn inspect_emits_browser_template_and_field_label_observations() {
    let build = build_product_browser_case();
    let report = writer_core::inspect_staging(&build.staging_ref.expect("staging ref"))
        .expect("inspect staging");

    assert!(report
        .observations
        .browser_templates
        .iter()
        .any(|value| value["template_name"] == "Card 1" && value["browser_font_name"] == "Arial"));
    assert!(report
        .observations
        .template_target_decks
        .iter()
        .any(|value| value["template_name"] == "Card 1"
            && value["target_deck_name"] == "Custom::Deck"
            && value["resolved_target_deck_id"].is_number()));
    assert!(report
        .observations
        .field_metadata
        .iter()
        .any(|value| value["field_name"] == "Front" && value["role_hint"] == "question"));
}
```

- [ ] **Step 2: Run the metadata and inspect tests to verify they fail**

Run: `cargo test -p anki_forge --test product_pipeline_tests browser_appearance_and_template_target_deck_survive_lower_normalize_and_build -v`
Expected: FAIL because field-label metadata declarations, browser appearance declarations, and template target-deck lowering do not exist yet, and build does not yet resolve target deck names into stable ids.

Run: `cargo test -p writer_core --test inspect_tests inspect_emits_browser_template_and_field_label_observations -v`
Expected: FAIL because inspect does not yet emit structured browser-template, template-target-deck, resolved deck id, and field-label observations.

- [ ] **Step 3: Add metadata declarations, browser/template lowering, and inspect propagation**

```rust
// anki_forge/src/product/metadata.rs
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FieldMetadataDeclaration {
    pub field_name: String,
    pub label: Option<String>,
    pub role_hint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TemplateBrowserAppearanceDeclaration {
    pub template_name: String,
    pub question_format: Option<String>,
    pub answer_format: Option<String>,
    pub font_name: Option<String>,
    pub font_size: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TemplateTargetDeckDeclaration {
    pub template_name: String,
    pub deck_name: String,
}
```

```rust
// anki_forge/src/product/model.rs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProductDocument {
    document_id: String,
    note_types: Vec<ProductNoteType>,
    notes: Vec<ProductNote>,
    helpers: Vec<(String, super::helpers::HelperDeclaration)>,
    assets: Vec<super::assets::AssetSource>,
    font_bindings: Vec<super::assets::FontBinding>,
    field_metadata: Vec<(String, super::metadata::FieldMetadataDeclaration)>,
    browser_appearance: Vec<(String, super::metadata::TemplateBrowserAppearanceDeclaration)>,
    template_target_decks: Vec<(String, super::metadata::TemplateTargetDeckDeclaration)>,
    default_deck_name: Option<String>,
}
```

```rust
// anki_forge/src/product/builders.rs
impl ProductDocument {
    pub fn with_default_deck(mut self, deck_name: impl Into<String>) -> Self {
        self.default_deck_name = Some(deck_name.into());
        self
    }

    pub fn default_deck_name(&self) -> Option<&str> {
        self.default_deck_name.as_deref()
    }

    pub fn with_field_metadata(
        mut self,
        note_type_id: impl Into<String>,
        field: super::metadata::FieldMetadataDeclaration,
    ) -> Self {
        self.field_metadata.push((note_type_id.into(), field));
        self
    }

    pub fn with_browser_appearance(
        mut self,
        note_type_id: impl Into<String>,
        declaration: super::metadata::TemplateBrowserAppearanceDeclaration,
    ) -> Self {
        self.browser_appearance
            .push((note_type_id.into(), declaration));
        self
    }

    pub fn with_template_target_deck(
        mut self,
        note_type_id: impl Into<String>,
        declaration: super::metadata::TemplateTargetDeckDeclaration,
    ) -> Self {
        self.template_target_decks
            .push((note_type_id.into(), declaration));
        self
    }

    pub fn field_metadata_for(
        &self,
        note_type_id: &str,
    ) -> Vec<super::metadata::FieldMetadataDeclaration> {
        self.field_metadata
            .iter()
            .filter(|(target, _)| target == note_type_id)
            .map(|(_, field)| field.clone())
            .collect()
    }

    pub fn browser_appearance_for(
        &self,
        note_type_id: &str,
        template_name: &str,
    ) -> Option<super::metadata::TemplateBrowserAppearanceDeclaration> {
        self.browser_appearance
            .iter()
            .find(|(target, declaration)| {
                target == note_type_id && declaration.template_name == template_name
            })
            .map(|(_, declaration)| declaration.clone())
    }

    pub fn template_target_deck_for(
        &self,
        note_type_id: &str,
        template_name: &str,
    ) -> Option<super::metadata::TemplateTargetDeckDeclaration> {
        self.template_target_decks
            .iter()
            .find(|(target, declaration)| {
                target == note_type_id && declaration.template_name == template_name
            })
            .map(|(_, declaration)| declaration.clone())
    }
}
```

```rust
// anki_forge/src/product/lowering.rs
let resolved_deck_name = document
    .default_deck_name()
    .map(str::to_owned)
    .unwrap_or_else(|| note.deck_name.clone());

authoring_notetype.field_metadata = document
    .field_metadata_for(&notetype.id)
    .into_iter()
    .map(|field| authoring_core::AuthoringFieldMetadata {
        field_name: field.field_name,
        label: field.label,
        role_hint: field.role_hint,
    })
    .collect();

for template in authoring_notetype.templates.iter_mut().flatten() {
    if let Some(browser) = document.browser_appearance_for(&notetype.id, &template.name) {
        template.question_format_browser = browser.question_format.clone();
        template.answer_format_browser = browser.answer_format.clone();
        template.browser_font_name = browser.font_name.clone();
        template.browser_font_size = browser.font_size;
    }
    if let Some(target) = document.template_target_deck_for(&notetype.id, &template.name) {
        template.target_deck_name = Some(target.deck_name.clone());
    }
}
```

```rust
// writer_core/src/staging.rs
for template in staging_notetype.templates.iter_mut() {
    if let Some(target_name) = template.target_deck_name.clone() {
        let target_deck_id = deck_registry.resolve_or_create(&target_name, writer_policy.deck_resolution())?;
        template.target_deck_id = Some(target_deck_id);
    }
}
```

```rust
// writer_core/src/inspect.rs
observations.field_metadata.extend(
    inspect_input
        .notetypes
        .iter()
        .flat_map(|notetype| notetype.field_metadata.iter().map(|field| {
            serde_json::json!({
                "notetype_id": notetype.id,
                "field_name": field.field_name,
                "label": field.label,
                "role_hint": field.role_hint,
            })
        })),
);

observations.browser_templates.extend(
    inspect_input
        .notetypes
        .iter()
        .flat_map(|notetype| notetype.templates.iter().filter_map(|template| {
            template.question_format_browser.as_ref().map(|_| serde_json::json!({
                "notetype_id": notetype.id,
                "template_name": template.name,
                "question_format_browser": template.question_format_browser,
                "answer_format_browser": template.answer_format_browser,
                "browser_font_name": template.browser_font_name,
                "browser_font_size": template.browser_font_size,
            }))
        })),
);

observations.template_target_decks.extend(
    inspect_input
        .notetypes
        .iter()
        .flat_map(|notetype| notetype.templates.iter().filter_map(|template| {
            template.target_deck_name.as_ref().map(|deck| serde_json::json!({
                "notetype_id": notetype.id,
                "template_name": template.name,
                "target_deck_name": deck,
                "resolved_target_deck_id": template.target_deck_id,
            }))
        })),
);
```

```rust
// contract_tools/tests/cli_tests.rs
#[test]
fn inspect_contract_json_exposes_browser_template_and_target_deck_observations() {
    let report = inspect_fixture_contract_json("product-browser-appearance").expect("inspect json");

    assert!(report["observations"]["browser_templates"]
        .as_array()
        .unwrap()
        .iter()
        .any(|value| value["template_name"] == "Card 1"));
    assert!(report["observations"]["template_target_decks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|value| value["target_deck_name"] == "Custom::Deck"));
}
```

- [ ] **Step 4: Run the metadata and inspect tests to verify they pass**

Run: `cargo test -p anki_forge --test product_pipeline_tests browser_appearance_and_template_target_deck_survive_lower_normalize_and_build -v`
Expected: PASS.

Run: `cargo test -p writer_core --test inspect_tests inspect_emits_browser_template_and_field_label_observations -v`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add anki_forge/src/product/mod.rs anki_forge/src/product/model.rs anki_forge/src/product/builders.rs anki_forge/src/product/lowering.rs anki_forge/src/product/metadata.rs contracts/schema/inspect-report.schema.json contracts/semantics/build.md contracts/semantics/inspect.md writer_core/src/staging.rs writer_core/src/inspect.rs contract_tools/tests/cli_tests.rs anki_forge/tests/product_pipeline_tests.rs writer_core/tests/inspect_tests.rs
git commit -m "feat: add product metadata and browser template propagation"
```

### Task 8: Add portability fixtures, real APKG round-trip oracle tests, docs, example flow, and Phase 5A exit evidence

**Files:**
- Create: `anki_forge/tests/fixtures/product/basic_answer_divider.case.json`
- Create: `anki_forge/tests/fixtures/product/io_font_bundle.case.json`
- Create: `anki_forge/tests/fixtures/product/io_font_bundle_roundtrip_v1.case.json`
- Create: `anki_forge/tests/fixtures/product/io_font_bundle_roundtrip_v2.case.json`
- Create: `anki_forge/tests/product_portability_tests.rs`
- Modify: `contract_tools/Cargo.toml`
- Modify: `contract_tools/src/compat_oracle.rs`
- Create: `contract_tools/tests/phase5a_roundtrip_oracle_tests.rs`
- Create: `anki_forge/examples/product_basic_flow.rs`
- Modify: `README.md`
- Create: `docs/superpowers/checklists/phase-5a-exit-evidence.md`

- [ ] **Step 1: Write the failing portability and docs-facing test**

```rust
// anki_forge/tests/product_portability_tests.rs
use std::fs;
use std::path::PathBuf;

use anki_forge::product::ProductDocument;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/product")
        .join(name)
}

#[test]
fn product_cases_can_be_loaded_from_data_fixtures_and_lowered() {
    let value = fs::read_to_string(fixture("basic_answer_divider.case.json"))
        .expect("read fixture");
    let document: ProductDocument = serde_json::from_str(&value).expect("deserialize product case");
    let lowering = document.lower().expect("lower fixture");

    assert_eq!(lowering.authoring_document.metadata_document_id, "fixture-basic-doc");
    assert!(lowering.authoring_document.notetypes[0]
        .templates
        .as_ref()
        .unwrap()[0]
        .answer_format
        .contains("Answer"));
}
```

```rust
// contract_tools/tests/phase5a_roundtrip_oracle_tests.rs
use contract_tools::compat_oracle::run_phase5a_roundtrip_oracle;

#[test]
fn repeated_import_preserves_phase5a_identity_ords_and_static_media_updates() -> anyhow::Result<()> {
    let roundtrip = run_phase5a_roundtrip_oracle(
        "anki_forge/tests/fixtures/product/io_font_bundle_roundtrip_v1.case.json",
        "anki_forge/tests/fixtures/product/io_font_bundle_roundtrip_v2.case.json",
        "phase5a-io-font-roundtrip",
    )?;

    assert_eq!(
        roundtrip.after_first_import.notetype_count,
        roundtrip.after_second_import.notetype_count
    );
    assert_eq!(
        roundtrip
            .after_second_import
            .field_ords
            .get("io-main")
            .expect("io field ords"),
        &vec![0, 1, 2, 3, 4]
    );
    assert_eq!(
        roundtrip
            .after_second_import
            .template_ords
            .get("io-main")
            .expect("io template ords"),
        &vec![0]
    );
    assert_ne!(
        roundtrip.after_first_import.referenced_static_media,
        roundtrip.after_second_import.referenced_static_media
    );
    assert!(roundtrip
        .after_second_import
        .referenced_static_media
        .iter()
        .any(|name| name.starts_with("_io-main_card-font-")));
    assert!(roundtrip
        .after_second_import
        .template_target_decks
        .iter()
        .any(|item| item.template_name == "Image Occlusion"
            && item.deck_name == "Custom::Deck"
            && item.deck_id > 0));

    Ok(())
}
```

- [ ] **Step 2: Run the portability test to verify it fails**

Run: `cargo test -p anki_forge --test product_portability_tests product_cases_can_be_loaded_from_data_fixtures_and_lowered -v`
Expected: FAIL because the fixture files do not exist yet and the product model is not fully data-driven.

Run: `cargo test -p contract_tools --test phase5a_roundtrip_oracle_tests repeated_import_preserves_phase5a_identity_ords_and_static_media_updates -v`
Expected: FAIL because the upstream import oracle helper, v1/v2 round-trip fixtures, and vendored upstream test dependency do not exist yet.

- [ ] **Step 3: Add the fixture cases, upstream round-trip oracle, example flow, README guidance, and exit checklist**

```json
// anki_forge/tests/fixtures/product/basic_answer_divider.case.json
{
  "document_id": "fixture-basic-doc",
  "note_types": [
    { "Basic": { "id": "basic-main", "name": null } }
  ],
  "notes": [
    {
      "Basic": {
        "id": "note-1",
        "note_type_id": "basic-main",
        "deck_name": "Default",
        "front": "front",
        "back": "back"
      }
    }
  ],
  "helpers": [
    ["basic-main", { "AnswerDivider": { "title": "Answer" } }]
  ],
  "assets": [],
  "font_bindings": [],
  "field_metadata": [],
  "browser_appearance": [],
  "template_target_decks": [],
  "default_deck_name": null
}
```

```json
// anki_forge/tests/fixtures/product/io_font_bundle_roundtrip_v1.case.json
{
  "document_id": "fixture-io-roundtrip-v1",
  "note_types": [
    { "ImageOcclusion": { "id": "io-main", "name": null } }
  ],
  "notes": [
    {
      "ImageOcclusion": {
        "id": "note-io-1",
        "note_type_id": "io-main",
        "deck_name": "Default",
        "occlusion": "{{c1::Mask}}",
        "image": "<img src=\"mask.png\">",
        "header": "Header",
        "back_extra": "Extra",
        "comments": "Comments"
      }
    }
  ],
  "helpers": [],
  "assets": [
    {
      "InlineTemplateStatic": {
        "namespace": "io-main",
        "filename": "card-font.woff2",
        "mime": "font/woff2",
        "data_base64": "AA=="
      }
    }
  ],
  "font_bindings": [
    { "note_type_id": "io-main", "family": "IO Sans", "filename": "card-font.woff2" }
  ],
  "field_metadata": [
    ["io-main", { "field_name": "Header", "label": "Header", "role_hint": "context" }]
  ],
  "browser_appearance": [],
  "template_target_decks": [
    ["io-main", { "template_name": "Image Occlusion", "deck_name": "Custom::Deck" }]
  ],
  "default_deck_name": "Default"
}
```

```json
// anki_forge/tests/fixtures/product/io_font_bundle_roundtrip_v2.case.json
{
  "document_id": "fixture-io-roundtrip-v2",
  "note_types": [
    { "ImageOcclusion": { "id": "io-main", "name": null } }
  ],
  "notes": [
    {
      "ImageOcclusion": {
        "id": "note-io-1",
        "note_type_id": "io-main",
        "deck_name": "Default",
        "occlusion": "{{c1::Mask}}",
        "image": "<img src=\"mask.png\">",
        "header": "Header",
        "back_extra": "Updated Extra",
        "comments": "Comments"
      }
    }
  ],
  "helpers": [],
  "assets": [
    {
      "InlineTemplateStatic": {
        "namespace": "io-main",
        "filename": "card-font.woff2",
        "mime": "font/woff2",
        "data_base64": "AQ=="
      }
    }
  ],
  "font_bindings": [
    { "note_type_id": "io-main", "family": "IO Sans", "filename": "card-font.woff2" }
  ],
  "field_metadata": [
    ["io-main", { "field_name": "Header", "label": "Header", "role_hint": "context" }]
  ],
  "browser_appearance": [],
  "template_target_decks": [
    ["io-main", { "template_name": "Image Occlusion", "deck_name": "Custom::Deck" }]
  ],
  "default_deck_name": "Default"
}
```

```toml
# contract_tools/Cargo.toml
[dev-dependencies]
tempfile = "=3.17.1"
anki = { path = "../docs/source/rslib" }
```

```rust
// contract_tools/src/compat_oracle.rs
pub struct Phase5aRoundTripResult {
    pub after_first_import: Phase5aImportState,
    pub after_second_import: Phase5aImportState,
}

pub struct Phase5aImportState {
    pub notetype_count: usize,
    pub field_ords: BTreeMap<String, Vec<u32>>,
    pub template_ords: BTreeMap<String, Vec<u32>>,
    pub referenced_static_media: BTreeSet<String>,
    pub template_target_decks: Vec<TemplateDeckTarget>,
}

pub struct TemplateDeckTarget {
    pub template_name: String,
    pub deck_name: String,
    pub deck_id: i64,
}

pub fn run_phase5a_roundtrip_oracle(
    first_case: &str,
    second_case: &str,
    label: &str,
) -> anyhow::Result<Phase5aRoundTripResult> {
    let (_first_root, first_apkg) = build_product_case_apkg(first_case, format!("{label}-v1"))?;
    let (_second_root, second_apkg) = build_product_case_apkg(second_case, format!("{label}-v2"))?;

    let temp_dir = tempfile::tempdir()?;
    let col_path = temp_dir.path().join("roundtrip.anki2");
    let mut builder = anki::collection::CollectionBuilder::new(&col_path);
    builder.with_desktop_media_paths();
    let mut col = builder.build()?;

    col.import_apkg(
        &first_apkg,
        anki::import_export::package::ImportAnkiPackageOptions::default(),
    )?;
    let after_first_import = summarize_phase5a_import_state(&mut col)?;

    col.import_apkg(
        &second_apkg,
        anki::import_export::package::ImportAnkiPackageOptions {
            merge_notetypes: true,
            ..Default::default()
        },
    )?;
    let after_second_import = summarize_phase5a_import_state(&mut col)?;

    Ok(Phase5aRoundTripResult {
        after_first_import,
        after_second_import,
    })
}
```

```rust
// anki_forge/examples/product_basic_flow.rs
use anki_forge::product::{HelperDeclaration, ProductDocument};
use anki_forge::{normalize, NormalizationRequest};

fn main() -> anyhow::Result<()> {
    let lowering = ProductDocument::new("example-doc")
        .with_default_deck("Default")
        .with_basic("basic-main")
        .with_helper(
            "basic-main",
            HelperDeclaration::AnswerDivider {
                title: "Answer".into(),
            },
        )
        .add_basic_note("basic-main", "note-1", "Default", "front", "back")
        .lower()?;

    let normalized = normalize(NormalizationRequest::new(lowering.authoring_document));
    println!("{}", anki_forge::to_authoring_canonical_json(&normalized)?);
    Ok(())
}
```

```md
<!-- README.md -->
## Phase 5A product authoring (Rust-first)

`anki_forge::product` is the author-facing API layer for `Phase 5A`.
Its job is to produce a reviewable `LoweringPlan`, then hand off to the existing `Authoring IR -> normalize -> build -> inspect -> diff` pipeline.

Run the example with `cargo run -p anki_forge --example product_basic_flow`.
```

```md
<!-- docs/superpowers/checklists/phase-5a-exit-evidence.md -->
# Phase 5A Exit Evidence

- `cargo test -p anki_forge --test product_model_tests -v`
- `cargo test -p anki_forge --test product_lowering_tests -v`
- `cargo test -p anki_forge --test product_helper_tests -v`
- `cargo test -p anki_forge --test product_bundler_tests -v`
- `cargo test -p anki_forge --test product_pipeline_tests -v`
- `cargo test -p anki_forge --test product_portability_tests -v`
- `cargo test -p authoring_core --test normalization_pipeline_tests -v`
- `cargo test -p writer_core --test build_tests -v`
- `cargo test -p writer_core --test inspect_tests -v`
- `cargo test -p contract_tools --test schema_gate_tests -v`
- `cargo test -p contract_tools --test phase5a_roundtrip_oracle_tests -v`
```

- [ ] **Step 4: Run the portability test to verify it passes**

Run: `cargo test -p anki_forge --test product_portability_tests product_cases_can_be_loaded_from_data_fixtures_and_lowered -v`
Expected: PASS.

Run: `cargo test -p contract_tools --test phase5a_roundtrip_oracle_tests repeated_import_preserves_phase5a_identity_ords_and_static_media_updates -v`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add anki_forge/tests/fixtures/product/basic_answer_divider.case.json anki_forge/tests/fixtures/product/io_font_bundle.case.json anki_forge/tests/fixtures/product/io_font_bundle_roundtrip_v1.case.json anki_forge/tests/fixtures/product/io_font_bundle_roundtrip_v2.case.json anki_forge/tests/product_portability_tests.rs contract_tools/Cargo.toml contract_tools/src/compat_oracle.rs contract_tools/tests/phase5a_roundtrip_oracle_tests.rs anki_forge/examples/product_basic_flow.rs README.md docs/superpowers/checklists/phase-5a-exit-evidence.md
git commit -m "docs: add phase 5a fixtures and release evidence"
```

## Self-Review Checklist

Before handing this plan to an executor, verify the following:

1. `Block 1` coverage exists in Tasks 1-4.
2. `Block 2` coverage exists in Task 5.
3. `Block 3` coverage exists in Tasks 6-7.
4. portability/docs coverage exists in Task 8.
5. No task introduces a product feature that writes directly to `Authoring IR` outside the shared lowering boundary.
6. The custom escape hatch remains lighter than the first-class paths.
7. Phase 5A exit evidence includes a real APKG build -> import -> rebuild -> re-import oracle against vendored upstream `anki`.
8. Every task ends with a passing focused test command and a commit.
