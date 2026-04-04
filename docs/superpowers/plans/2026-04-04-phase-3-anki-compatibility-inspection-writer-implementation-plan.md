# Phase 3 Anki Compatibility, Inspection, and Writer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver a contract-first Phase 3 pipeline that turns writer-ready normalized data into materialized staging artifacts, modern-Anki-compatible `.apkg` artifacts, stable inspection reports, diff reports, and fixture-driven compatibility evidence backed by a controlled compatibility oracle.

**Architecture:** Extend the current minimal Phase 2 normalization contracts just enough to produce writer-ready `Normalized IR` for stock `Basic`, `Cloze`, and scoped `Image Occlusion` lanes, then add a new `writer_core` crate for staging, package emission, inspection, and diffing. Keep `contracts/` as the normative source of truth, `writer_core` as the artifact engine, and `contract_tools` as the CLI/gate layer that runs Tier A and Tier B fixtures, staging-first inspection, staging/apkg semantic consistency checks, and the controlled compatibility oracle.

**Tech Stack:** Rust workspace (`cargo`), `serde`, `serde_json`, `serde_yaml`, `clap`, `jsonschema`, `rusqlite`, `zip`, `zstd`, `sha1`, `prost`, JSON Schema contracts, YAML policies and contexts, contract fixtures, local `docs/source/rslib` reference source

---

## Source Baseline

Do not invent Anki package/layout behavior from memory. Any implementation that touches package layout, media map behavior, stock notetypes, or compatibility acceptance must be grounded in these local reference files:

- `docs/source/rslib/src/import_export/package/apkg/export.rs`
- `docs/source/rslib/src/import_export/package/colpkg/export.rs`
- `docs/source/rslib/src/import_export/package/meta.rs`
- `docs/source/rslib/src/import_export/package/media.rs`
- `docs/source/rslib/src/import_export/package/apkg/tests.rs`
- `docs/source/rslib/src/import_export/package/apkg/import/notes.rs`
- `docs/source/rslib/src/notetype/stock.rs`
- `docs/source/rslib/src/image_occlusion/notetype.rs`
- `docs/source/rslib/src/media/files.rs`
- `docs/source/rslib/src/storage/schema11.sql`
- `docs/source/rslib/src/storage/upgrades/schema18_upgrade.sql`

If an implementation choice is not directly covered by current repository contracts, stop and check these source files before adding or changing behavior.

## Scope Check

This plan still targets one coherent subsystem: `Phase 3 Compatibility + Writer`.

However, the current repository only has a minimal `Normalized IR` (`document_id` + `resolved_identity`) and cannot yet drive a package writer. The first implementation block therefore extends the existing Phase 2 contracts and normalization code just enough to produce writer-ready normalized data for the supported stock lanes. That is enabling work inside the same subsystem, not a separate project.

## File Structure Map

### Workspace and semantic engines

- Modify: `Cargo.toml` - add `writer_core` as a workspace member
- Create: `writer_core/Cargo.toml` - declare crate dependencies for build, inspect, diff, SQLite, and package emission
- Create: `writer_core/src/lib.rs` - public API surface and contract version
- Create: `writer_core/src/model.rs` - build, inspect, diff, staging, and diagnostic DTOs
- Create: `writer_core/src/policy.rs` - writer-policy, verification-policy, and build-context loading helpers
- Create: `writer_core/src/canonical_json.rs` - stable machine output serialization for Phase 3 reports
- Create: `writer_core/src/staging.rs` - staging representation and deterministic fingerprint helpers
- Create: `writer_core/src/build.rs` - `Normalized IR -> staging -> package-build-result`
- Create: `writer_core/src/apkg.rs` - `.apkg` emission using source-grounded package layout
- Create: `writer_core/src/inspect.rs` - staging/apkg inspection into stable observation model
- Create: `writer_core/src/diff.rs` - comparison engine for inspect reports
- Create: `writer_core/tests/build_tests.rs` - writer build tests
- Create: `writer_core/tests/inspect_tests.rs` - inspection tests
- Create: `writer_core/tests/diff_tests.rs` - diff tests

### Existing Phase 2 extensions required for writer-ready data

- Modify: `contracts/schema/authoring-ir.schema.json` - define stock notetype, note, and media input shapes
- Modify: `contracts/schema/normalized-ir.schema.json` - define resolved writer-facing normalized output
- Modify: `contracts/schema/normalization-result.schema.json` - keep normalization envelope aligned with expanded normalized payload
- Modify: `contracts/manifest.yaml` - register `normalization_semantics` and any expanded contract assets
- Create: `contracts/semantics/normalization.md` - document writer-ready normalization behavior with source anchors
- Modify: `authoring_core/src/model.rs` - add writer-ready authoring and normalized DTOs
- Create: `authoring_core/src/stock.rs` - source-grounded stock notetype resolution helpers
- Modify: `authoring_core/src/normalize.rs` - expand normalization to resolved stock notetype lanes
- Modify: `authoring_core/src/lib.rs` - export new DTOs and helpers
- Modify: `authoring_core/tests/normalization_pipeline_tests.rs` - cover Basic, Cloze, and scoped Image Occlusion normalization
- Modify: `contract_tools/src/normalize_cmd.rs` - deserialize the richer authoring input model

### Phase 3 contracts and gate assets

- Create: `contracts/schema/package-build-result.schema.json`
- Create: `contracts/schema/inspect-report.schema.json`
- Create: `contracts/schema/diff-report.schema.json`
- Create: `contracts/schema/writer-policy.schema.json`
- Create: `contracts/schema/verification-policy.schema.json`
- Create: `contracts/schema/build-context.schema.json`
- Create: `contracts/policies/writer-policy.default.yaml`
- Create: `contracts/policies/verification-policy.default.yaml`
- Create: `contracts/contexts/build-context.default.yaml`
- Create: `contracts/semantics/build.md`
- Create: `contracts/semantics/inspect.md`
- Create: `contracts/semantics/diff.md`
- Create: `contracts/semantics/golden-regression.md`

### CLI, gates, and compatibility oracle

- Modify: `contract_tools/Cargo.toml` - add `writer_core`, `rusqlite`, and package inspection dependencies as needed by tests
- Modify: `contract_tools/src/lib.rs` - export new Phase 3 command modules
- Modify: `contract_tools/src/main.rs` - add `build`, `inspect`, and `diff` subcommands
- Create: `contract_tools/src/build_cmd.rs` - build command orchestration
- Create: `contract_tools/src/inspect_cmd.rs` - inspect command orchestration
- Create: `contract_tools/src/diff_cmd.rs` - diff command orchestration
- Create: `contract_tools/src/compat_oracle.rs` - controlled compatibility oracle grounded in local `rslib` source assumptions
- Modify: `contract_tools/src/policies.rs` - validate new writer and verification policy assets
- Modify: `contract_tools/src/semantics.rs` - include Phase 3 semantics docs in semantics gates
- Modify: `contract_tools/src/gates.rs` - run Phase 3 fixture and oracle gates
- Modify: `contract_tools/src/fixtures.rs` - execute Tier A and Tier B Phase 3 cases, golden comparisons, and semantic consistency checks

### Fixtures, tests, and docs

- Modify: `contracts/fixtures/index.yaml` - register Phase 3 writer and e2e cases plus golden bindings
- Create: `contracts/fixtures/phase3/inputs/basic-authoring-ir.json`
- Create: `contracts/fixtures/phase3/inputs/basic-normalized-ir.json`
- Create: `contracts/fixtures/phase3/inputs/cloze-authoring-ir.json`
- Create: `contracts/fixtures/phase3/inputs/cloze-normalized-ir.json`
- Create: `contracts/fixtures/phase3/inputs/image-occlusion-authoring-ir.json`
- Create: `contracts/fixtures/phase3/inputs/image-occlusion-normalized-ir.json`
- Create: `contracts/fixtures/phase3/writer/basic-minimal.case.yaml`
- Create: `contracts/fixtures/phase3/writer/cloze-minimal.case.yaml`
- Create: `contracts/fixtures/phase3/writer/image-occlusion-minimal.case.yaml`
- Create: `contracts/fixtures/phase3/e2e/basic-minimal.case.yaml`
- Create: `contracts/fixtures/phase3/e2e/cloze-minimal.case.yaml`
- Create: `contracts/fixtures/phase3/e2e/image-occlusion-minimal.case.yaml`
- Create: `contracts/fixtures/phase3/expected/basic.build.json`
- Create: `contracts/fixtures/phase3/expected/basic.inspect.json`
- Create: `contracts/fixtures/phase3/expected/basic.diff.json`
- Create: `contracts/fixtures/phase3/expected/cloze.build.json`
- Create: `contracts/fixtures/phase3/expected/cloze.inspect.json`
- Create: `contracts/fixtures/phase3/expected/image-occlusion.build.json`
- Create: `contracts/fixtures/phase3/expected/image-occlusion.inspect.json`
- Modify: `contract_tools/tests/workspace_smoke_tests.rs`
- Modify: `contract_tools/tests/schema_gate_tests.rs`
- Modify: `contract_tools/tests/policy_gate_tests.rs`
- Modify: `contract_tools/tests/fixture_gate_tests.rs`
- Modify: `contract_tools/tests/cli_tests.rs`
- Create: `contract_tools/tests/compat_oracle_tests.rs`
- Modify: `README.md`
- Create: `docs/superpowers/checklists/phase-3-exit-evidence.md`

### Implementation notes

- Keep `writer-policy` and `verification-policy` separate throughout the codebase.
- `package-build-result` must include both `writer_policy_ref` and `build_context_ref`.
- `inspect-report` and `diff-report` must carry degradation/comparison completeness explicitly in schema-governed fields.
- `build`, `inspect`, and `diff` must expose stable `contract-json` surfaces.
- Do not replace the existing `contracts/manifest.yaml` asset map with abbreviated excerpts. Every manifest step in this plan means “append these keys while preserving all existing entries”.
- `writer_core/src/lib.rs` must be extended incrementally. Do not declare or re-export modules before the backing files exist.
- Materialized `staging_ref` and `apkg_ref` must point into caller-owned artifact roots, not into tempdirs that are dropped when a helper returns.
- Writer fast gate is staging-first: `build -> inspect(staging) -> diff`. `.apkg` inspection is added for staging/apkg semantic consistency and compatibility acceptance.
- Goldens must be captured from actual deterministic outputs. Do not hand-author placeholder fingerprints, refs, or hashes and then compare them as if they were real outputs.
- `.apkg` emission must follow the source-grounded package layout from local `rslib` references; do not invent filenames, media-map structure, collection version names, `meta` encoding, or dummy-collection behavior.

### Task 1: Bootstrap `writer_core` in the workspace

**Files:**
- Modify: `Cargo.toml`
- Modify: `contract_tools/Cargo.toml`
- Modify: `contract_tools/tests/workspace_smoke_tests.rs`
- Create: `writer_core/Cargo.toml`
- Create: `writer_core/src/lib.rs`

- [ ] **Step 1: Write the failing workspace smoke test**

```rust
// contract_tools/tests/workspace_smoke_tests.rs
#[test]
fn workspace_exposes_writer_core_contract_version() {
    assert_eq!(writer_core::tool_contract_version(), "phase3-v1");
}
```

- [ ] **Step 2: Run the smoke test to verify it fails**

Run: `cargo test -p contract_tools --test workspace_smoke_tests -v`
Expected: FAIL with unresolved crate/import for `writer_core`.

- [ ] **Step 3: Add the workspace member and minimal crate**

```toml
# Cargo.toml
[workspace]
members = ["contract_tools", "authoring_core", "writer_core"]
resolver = "2"

[workspace.package]
edition = "2021"
rust-version = "1.81"

[workspace.lints.rust]
unsafe_code = "forbid"
```

```toml
# writer_core/Cargo.toml
[package]
name = "writer_core"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true

[dependencies]
anyhow = "1"
authoring_core = { path = "../authoring_core" }
hex = "0.4"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sha1 = "0.10"

[lints]
workspace = true
```

```toml
# contract_tools/Cargo.toml
[dependencies]
anyhow = "1"
clap = { version = "=4.5.20", features = ["derive"] }
flate2 = "=1.0.35"
jsonschema = { version = "0.18.3", default-features = false }
authoring_core = { path = "../authoring_core" }
writer_core = { path = "../writer_core" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
tar = "=0.4.42"
url = "2.5.2"
```

```rust
// writer_core/src/lib.rs
pub fn tool_contract_version() -> &'static str {
    "phase3-v1"
}
```

- [ ] **Step 4: Run the smoke test to verify it passes**

Run: `cargo test -p contract_tools --test workspace_smoke_tests -v`
Expected: PASS, including `workspace_exposes_writer_core_contract_version`.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml contract_tools/Cargo.toml contract_tools/tests/workspace_smoke_tests.rs writer_core/Cargo.toml writer_core/src/lib.rs
git commit -m "feat: bootstrap writer_core workspace crate"
```

### Task 2: Expand authoring and normalized contracts to a writer-ready stock model

**Files:**
- Modify: `contracts/manifest.yaml`
- Modify: `contracts/schema/authoring-ir.schema.json`
- Modify: `contracts/schema/normalized-ir.schema.json`
- Modify: `contracts/schema/normalization-result.schema.json`
- Create: `contracts/semantics/normalization.md`
- Modify: `contract_tools/tests/schema_gate_tests.rs`

- [ ] **Step 1: Write failing schema tests for writer-ready authoring and normalized shapes**

```rust
// contract_tools/tests/schema_gate_tests.rs
#[test]
fn authoring_ir_schema_accepts_stock_notetype_note_and_media_entries() {
    let manifest = load_manifest(contract_manifest_path()).unwrap();
    let schema =
        load_schema(resolve_asset_path(&manifest, "authoring_ir_schema").unwrap()).unwrap();
    let value = json!({
        "kind": "authoring-ir",
        "schema_version": "0.1.0",
        "metadata": { "document_id": "demo-doc" },
        "notetypes": [
            {
                "id": "basic-main",
                "kind": "basic",
                "name": "Basic"
            }
        ],
        "notes": [
            {
                "id": "note-1",
                "notetype_id": "basic-main",
                "deck_name": "Default",
                "fields": {
                    "Front": "front",
                    "Back": "back <img src=\"sample.jpg\"> [sound:sample.mp3]"
                },
                "tags": ["demo"]
            }
        ],
        "media": [
            {
                "filename": "sample.jpg",
                "mime": "image/jpeg",
                "data_base64": "MQ=="
            },
            {
                "filename": "sample.mp3",
                "mime": "audio/mpeg",
                "data_base64": "Mg=="
            }
        ]
    });

    assert!(validate_value(&schema, &value).is_ok());
}

#[test]
fn normalized_ir_schema_accepts_resolved_writer_ready_shape() {
    let manifest = load_manifest(contract_manifest_path()).unwrap();
    let schema =
        load_schema(resolve_asset_path(&manifest, "normalized_ir_schema").unwrap()).unwrap();
    let value = json!({
        "kind": "normalized-ir",
        "schema_version": "0.1.0",
        "document_id": "demo-doc",
        "resolved_identity": "det:demo-doc",
        "notetypes": [
            {
                "id": "basic-main",
                "kind": "basic",
                "name": "Basic",
                "fields": ["Front", "Back"],
                "templates": [
                    {
                        "name": "Card 1",
                        "question_format": "{{Front}}",
                        "answer_format": "{{FrontSide}}\n\n<hr id=answer>\n\n{{Back}}"
                    }
                ],
                "css": ""
            }
        ],
        "notes": [
            {
                "id": "note-1",
                "notetype_id": "basic-main",
                "deck_name": "Default",
                "fields": {
                    "Front": "front",
                    "Back": "back <img src=\"sample.jpg\"> [sound:sample.mp3]"
                },
                "tags": ["demo"]
            }
        ],
        "media": [
            {
                "filename": "sample.jpg",
                "mime": "image/jpeg",
                "data_base64": "MQ=="
            }
        ]
    });

    assert!(validate_value(&schema, &value).is_ok());
}
```

- [ ] **Step 2: Run the schema tests to verify they fail**

Run: `cargo test -p contract_tools --test schema_gate_tests -v`
Expected: FAIL because the current schemas reject the richer notetype/note/media structure and `normalization_semantics` is not yet registered.

- [ ] **Step 3: Expand the manifest and schemas**

```yaml
# contracts/manifest.yaml
# Append these keys to the existing assets map without removing current entries.
assets:
  authoring_ir_schema: schema/authoring-ir.schema.json
  normalized_ir_schema: schema/normalized-ir.schema.json
  normalization_result_schema: schema/normalization-result.schema.json
  normalization_semantics: semantics/normalization.md
```

```json
// contracts/schema/authoring-ir.schema.json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["kind", "schema_version", "metadata", "notetypes", "notes"],
  "additionalProperties": false,
  "properties": {
    "kind": { "const": "authoring-ir" },
    "schema_version": { "type": "string", "minLength": 1 },
    "metadata": {
      "type": "object",
      "required": ["document_id"],
      "additionalProperties": false,
      "properties": {
        "document_id": { "type": "string", "minLength": 1 }
      }
    },
    "notetypes": {
      "type": "array",
      "items": { "$ref": "#/$defs/authoring_notetype" }
    },
    "notes": {
      "type": "array",
      "items": { "$ref": "#/$defs/authoring_note" }
    },
    "media": {
      "type": "array",
      "default": [],
      "items": { "$ref": "#/$defs/authoring_media" }
    }
  },
  "$defs": {
    "authoring_notetype": {
      "type": "object",
      "required": ["id", "kind"],
      "additionalProperties": false,
      "properties": {
        "id": { "type": "string", "minLength": 1 },
        "kind": {
          "type": "string",
          "enum": ["basic", "cloze", "image_occlusion"]
        },
        "name": { "type": "string", "minLength": 1 }
      }
    },
    "authoring_note": {
      "type": "object",
      "required": ["id", "notetype_id", "deck_name", "fields"],
      "additionalProperties": false,
      "properties": {
        "id": { "type": "string", "minLength": 1 },
        "notetype_id": { "type": "string", "minLength": 1 },
        "deck_name": { "type": "string", "minLength": 1 },
        "fields": {
          "type": "object",
          "minProperties": 1,
          "additionalProperties": { "type": "string" }
        },
        "tags": {
          "type": "array",
          "default": [],
          "items": { "type": "string", "minLength": 1 }
        }
      }
    },
    "authoring_media": {
      "type": "object",
      "required": ["filename", "mime", "data_base64"],
      "additionalProperties": false,
      "properties": {
        "filename": { "type": "string", "minLength": 1 },
        "mime": { "type": "string", "minLength": 1 },
        "data_base64": { "type": "string", "minLength": 1 }
      }
    }
  }
}
```

```json
// contracts/schema/normalized-ir.schema.json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": [
    "kind",
    "schema_version",
    "document_id",
    "resolved_identity",
    "notetypes",
    "notes",
    "media"
  ],
  "additionalProperties": false,
  "properties": {
    "kind": { "const": "normalized-ir" },
    "schema_version": { "type": "string", "minLength": 1 },
    "document_id": { "type": "string", "minLength": 1 },
    "resolved_identity": { "type": "string", "minLength": 1 },
    "notetypes": {
      "type": "array",
      "items": { "$ref": "#/$defs/normalized_notetype" }
    },
    "notes": {
      "type": "array",
      "items": { "$ref": "#/$defs/normalized_note" }
    },
    "media": {
      "type": "array",
      "items": { "$ref": "#/$defs/normalized_media" }
    }
  },
  "$defs": {
    "normalized_notetype": {
      "type": "object",
      "required": ["id", "kind", "name", "fields", "templates", "css"],
      "additionalProperties": false,
      "properties": {
        "id": { "type": "string", "minLength": 1 },
        "kind": {
          "type": "string",
          "enum": ["basic", "cloze", "image_occlusion"]
        },
        "name": { "type": "string", "minLength": 1 },
        "fields": {
          "type": "array",
          "minItems": 1,
          "items": { "type": "string", "minLength": 1 }
        },
        "templates": {
          "type": "array",
          "minItems": 1,
          "items": {
            "type": "object",
            "required": ["name", "question_format", "answer_format"],
            "additionalProperties": false,
            "properties": {
              "name": { "type": "string", "minLength": 1 },
              "question_format": { "type": "string", "minLength": 1 },
              "answer_format": { "type": "string", "minLength": 1 }
            }
          }
        },
        "css": { "type": "string" }
      }
    },
    "normalized_note": {
      "type": "object",
      "required": ["id", "notetype_id", "deck_name", "fields", "tags"],
      "additionalProperties": false,
      "properties": {
        "id": { "type": "string", "minLength": 1 },
        "notetype_id": { "type": "string", "minLength": 1 },
        "deck_name": { "type": "string", "minLength": 1 },
        "fields": {
          "type": "object",
          "minProperties": 1,
          "additionalProperties": { "type": "string" }
        },
        "tags": {
          "type": "array",
          "items": { "type": "string", "minLength": 1 }
        }
      }
    },
    "normalized_media": {
      "type": "object",
      "required": ["filename", "mime", "data_base64"],
      "additionalProperties": false,
      "properties": {
        "filename": { "type": "string", "minLength": 1 },
        "mime": { "type": "string", "minLength": 1 },
        "data_base64": { "type": "string", "minLength": 1 }
      }
    }
  }
}
```

```md
<!-- contracts/semantics/normalization.md -->
---
asset_refs:
  - schema/authoring-ir.schema.json
  - schema/normalized-ir.schema.json
---
# Normalization

Phase 3 normalization resolves stock notetype lanes into writer-ready normalized notetype definitions.

Source anchors:

- `docs/source/rslib/src/notetype/stock.rs`
- `docs/source/rslib/src/image_occlusion/notetype.rs`
- `docs/source/rslib/src/media/files.rs`

Rules:

- authoring `kind=basic|cloze|image_occlusion` resolves to source-grounded stock fields, templates, and CSS
- note field maps stay keyed by stable field names
- media entries remain inline in normalized output for the scoped Phase 3 fixtures
- normalization must not invent unsupported stock templates or media filename handling rules
```

- [ ] **Step 4: Run the schema tests to verify they pass**

Run: `cargo test -p contract_tools --test schema_gate_tests -v`
Expected: PASS for the new authoring and normalized shape tests.

- [ ] **Step 5: Commit**

```bash
git add contracts/manifest.yaml contracts/schema/authoring-ir.schema.json contracts/schema/normalized-ir.schema.json contracts/schema/normalization-result.schema.json contracts/semantics/normalization.md contract_tools/tests/schema_gate_tests.rs
git commit -m "feat: define writer-ready authoring and normalized contracts"
```

### Task 3: Extend `authoring_core` to emit writer-ready normalized stock lanes

**Files:**
- Modify: `authoring_core/src/model.rs`
- Create: `authoring_core/src/stock.rs`
- Modify: `authoring_core/src/normalize.rs`
- Modify: `authoring_core/src/lib.rs`
- Modify: `authoring_core/tests/normalization_pipeline_tests.rs`
- Modify: `contract_tools/src/normalize_cmd.rs`

- [ ] **Step 1: Write failing normalization tests for Basic, Cloze, and Image Occlusion**

```rust
// authoring_core/tests/normalization_pipeline_tests.rs
#[test]
fn basic_authoring_input_expands_to_resolved_basic_notetype() {
    let request = request_from_json(json!({
        "input": {
            "kind": "authoring-ir",
            "schema_version": "0.1.0",
            "metadata_document_id": "demo-doc",
            "notetypes": [
                { "id": "basic-main", "kind": "basic", "name": "Basic" }
            ],
            "notes": [
                {
                    "id": "note-1",
                    "notetype_id": "basic-main",
                    "deck_name": "Default",
                    "fields": { "Front": "front", "Back": "back" },
                    "tags": ["demo"]
                }
            ],
            "media": []
        }
    }));

    let result = normalize(request);
    let normalized = result.normalized_ir.expect("normalized_ir");
    assert_eq!(normalized.notetypes[0].kind, "basic");
    assert_eq!(normalized.notetypes[0].fields, vec!["Front", "Back"]);
    assert_eq!(normalized.notetypes[0].templates[0].name, "Card 1");
}

#[test]
fn cloze_authoring_input_expands_to_source_grounded_cloze_template() {
    let request = request_from_json(json!({
        "input": {
            "kind": "authoring-ir",
            "schema_version": "0.1.0",
            "metadata_document_id": "demo-doc",
            "notetypes": [
                { "id": "cloze-main", "kind": "cloze", "name": "Cloze" }
            ],
            "notes": [
                {
                    "id": "note-1",
                    "notetype_id": "cloze-main",
                    "deck_name": "Default",
                    "fields": {
                        "Text": "{{c1::front}}",
                        "Back Extra": "extra"
                    },
                    "tags": []
                }
            ],
            "media": []
        }
    }));

    let result = normalize(request);
    let normalized = result.normalized_ir.expect("normalized_ir");
    assert!(normalized.notetypes[0].templates[0]
        .question_format
        .contains("{{cloze:Text}}"));
}

#[test]
fn image_occlusion_lane_uses_source_grounded_fields_and_css() {
    let request = request_from_json(json!({
        "input": {
            "kind": "authoring-ir",
            "schema_version": "0.1.0",
            "metadata_document_id": "demo-doc",
            "notetypes": [
                { "id": "io-main", "kind": "image_occlusion", "name": "Image Occlusion" }
            ],
            "notes": [
                {
                    "id": "note-1",
                    "notetype_id": "io-main",
                    "deck_name": "Default",
                    "fields": {
                        "Occlusion": "{{c1::shape}}",
                        "Image": "<img src=\"mask.png\">",
                        "Header": "header",
                        "Back Extra": "extra",
                        "Comments": "comment"
                    },
                    "tags": []
                }
            ],
            "media": [
                {
                    "filename": "mask.png",
                    "mime": "image/png",
                    "data_base64": "MQ=="
                }
            ]
        }
    }));

    let result = normalize(request);
    let normalized = result.normalized_ir.expect("normalized_ir");
    assert_eq!(normalized.notetypes[0].fields[0], "Occlusion");
    assert!(normalized.notetypes[0].css.contains("#image-occlusion-container"));
}
```

- [ ] **Step 2: Run the normalization tests to verify they fail**

Run: `cargo test -p authoring_core --test normalization_pipeline_tests -v`
Expected: FAIL because the current DTOs cannot deserialize or emit writer-ready authoring and normalized structures.

- [ ] **Step 3: Implement writer-ready authoring and normalized DTOs plus stock resolution**

```rust
// authoring_core/src/model.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthoringDocument {
    pub kind: String,
    pub schema_version: String,
    pub metadata_document_id: String,
    #[serde(default)]
    pub notetypes: Vec<AuthoringNotetype>,
    #[serde(default)]
    pub notes: Vec<AuthoringNote>,
    #[serde(default)]
    pub media: Vec<AuthoringMedia>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthoringNotetype {
    pub id: String,
    pub kind: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthoringNote {
    pub id: String,
    pub notetype_id: String,
    pub deck_name: String,
    pub fields: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthoringMedia {
    pub filename: String,
    pub mime: String,
    pub data_base64: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedNotetype {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub fields: Vec<String>,
    pub templates: Vec<NormalizedTemplate>,
    pub css: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedTemplate {
    pub name: String,
    pub question_format: String,
    pub answer_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedNote {
    pub id: String,
    pub notetype_id: String,
    pub deck_name: String,
    pub fields: std::collections::BTreeMap<String, String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedMedia {
    pub filename: String,
    pub mime: String,
    pub data_base64: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedIr {
    pub kind: String,
    pub schema_version: String,
    pub document_id: String,
    pub resolved_identity: String,
    pub notetypes: Vec<NormalizedNotetype>,
    pub notes: Vec<NormalizedNote>,
    pub media: Vec<NormalizedMedia>,
}
```

```rust
// authoring_core/src/stock.rs
use crate::model::{AuthoringNotetype, NormalizedNotetype, NormalizedTemplate};

pub fn resolve_stock_notetype(input: &AuthoringNotetype) -> anyhow::Result<NormalizedNotetype> {
    match input.kind.as_str() {
        "basic" => Ok(NormalizedNotetype {
            id: input.id.clone(),
            kind: "basic".into(),
            name: input.name.clone().unwrap_or_else(|| "Basic".into()),
            fields: vec!["Front".into(), "Back".into()],
            templates: vec![NormalizedTemplate {
                name: "Card 1".into(),
                question_format: "{{Front}}".into(),
                answer_format: "{{FrontSide}}\n\n<hr id=answer>\n\n{{Back}}".into(),
            }],
            css: String::new(),
        }),
        "cloze" => Ok(NormalizedNotetype {
            id: input.id.clone(),
            kind: "cloze".into(),
            name: input.name.clone().unwrap_or_else(|| "Cloze".into()),
            fields: vec!["Text".into(), "Back Extra".into()],
            templates: vec![NormalizedTemplate {
                name: input.name.clone().unwrap_or_else(|| "Cloze".into()),
                question_format: "{{cloze:Text}}".into(),
                answer_format: "{{cloze:Text}}<br>\n{{Back Extra}}".into(),
            }],
            css: include_str!("../../docs/source/rslib/src/notetype/cloze_styling.css").into(),
        }),
        "image_occlusion" => Ok(NormalizedNotetype {
            id: input.id.clone(),
            kind: "image_occlusion".into(),
            name: input
                .name
                .clone()
                .unwrap_or_else(|| "Image Occlusion".into()),
            fields: vec![
                "Occlusion".into(),
                "Image".into(),
                "Header".into(),
                "Back Extra".into(),
                "Comments".into(),
            ],
            templates: vec![NormalizedTemplate {
                name: input
                    .name
                    .clone()
                    .unwrap_or_else(|| "Image Occlusion".into()),
                question_format: r#"{{#Header}}<div>{{Header}}</div>{{/Header}}
<div style="display: none">{{cloze:Occlusion}}</div>
<div id="err"></div>
<div id="image-occlusion-container">
    {{Image}}
    <canvas id="image-occlusion-canvas"></canvas>
</div>
<script>
try {
    anki.imageOcclusion.setup();
} catch (exc) {
    document.getElementById("err").innerHTML = `Error loading image occlusion<br><br>${exc}`;
}
</script>
"#
                .into(),
                answer_format: r#"{{#Header}}<div>{{Header}}</div>{{/Header}}
<div style="display: none">{{cloze:Occlusion}}</div>
<div id="err"></div>
<div id="image-occlusion-container">
    {{Image}}
    <canvas id="image-occlusion-canvas"></canvas>
</div>
<script>
try {
    anki.imageOcclusion.setup();
} catch (exc) {
    document.getElementById("err").innerHTML = `Error loading image occlusion<br><br>${exc}`;
}
</script>

<div><button id="toggle">Toggle Masks</button></div>
{{#Back Extra}}<div>{{Back Extra}}</div>{{/Back Extra}}
"#
                .into(),
            }],
            css: include_str!("../../docs/source/rslib/src/image_occlusion/notetype.css").into(),
        }),
        other => anyhow::bail!("unsupported stock notetype kind: {other}"),
    }
}
```

```rust
// authoring_core/src/normalize.rs
let normalized_notetypes = request
    .input
    .notetypes
    .iter()
    .map(crate::stock::resolve_stock_notetype)
    .collect::<anyhow::Result<Vec<_>>>()?;

let normalized_notes = request
    .input
    .notes
    .iter()
    .map(|note| crate::model::NormalizedNote {
        id: note.id.clone(),
        notetype_id: note.notetype_id.clone(),
        deck_name: note.deck_name.clone(),
        fields: note.fields.clone(),
        tags: note.tags.clone(),
    })
    .collect();

let normalized_media = request
    .input
    .media
    .iter()
    .map(|media| crate::model::NormalizedMedia {
        filename: media.filename.clone(),
        mime: media.mime.clone(),
        data_base64: media.data_base64.clone(),
    })
    .collect();

let normalized_ir = NormalizedIr {
    kind: "normalized-ir".into(),
    schema_version: request.input.schema_version,
    document_id: metadata_document_id,
    resolved_identity: resolved_identity.clone(),
    notetypes: normalized_notetypes,
    notes: normalized_notes,
    media: normalized_media,
};
```

```rust
// contract_tools/src/normalize_cmd.rs
#[derive(Debug, Deserialize)]
struct AuthoringInputDocument {
    kind: String,
    schema_version: String,
    metadata: AuthoringInputMetadata,
    #[serde(default)]
    notetypes: Vec<authoring_core::AuthoringNotetype>,
    #[serde(default)]
    notes: Vec<authoring_core::AuthoringNote>,
    #[serde(default)]
    media: Vec<authoring_core::AuthoringMedia>,
}

#[derive(Debug, Deserialize)]
struct AuthoringInputMetadata {
    document_id: String,
}

pub fn run(manifest: &str, input: &str, output: &str) -> anyhow::Result<String> {
    let manifest = load_manifest(manifest)?;
    let input_raw =
        fs::read_to_string(input).with_context(|| format!("failed to read input: {input}"))?;
    let input_value: Value = serde_json::from_str(&input_raw)
        .with_context(|| format!("input must be valid JSON: {input}"))?;
    let authoring_schema_path = resolve_asset_path(&manifest, "authoring_ir_schema")?;
    let authoring_schema = load_schema(&authoring_schema_path)?;
    validate_value(&authoring_schema, &input_value)?;

    let input_document: AuthoringInputDocument =
        serde_json::from_value(input_value).context("input must map into normalize input transport model")?;
    let document = authoring_core::AuthoringDocument {
        kind: input_document.kind,
        schema_version: input_document.schema_version,
        metadata_document_id: input_document.metadata.document_id,
        notetypes: input_document.notetypes,
        notes: input_document.notes,
        media: input_document.media,
    };

    let result = authoring_core::normalize(authoring_core::NormalizationRequest::new(document));
    match output {
        "contract-json" => authoring_core::to_canonical_json(&result),
        "human" => Ok(format!("status: {}", result.result_status)),
        other => bail!("unsupported normalize output mode: {other}"),
    }
}
```

```rust
// authoring_core/src/lib.rs
pub mod canonical_json;
pub mod identity;
pub mod model;
pub mod normalize;
pub mod risk;
pub mod selector;
pub mod stock;

pub use canonical_json::to_canonical_json;
pub use identity::{resolve_identity, DefaultNonceSource, NonceSource};
pub use model::{
    AuthoringDocument, AuthoringMedia, AuthoringNote, AuthoringNotetype, ComparisonContext,
    MergeRiskReport, NormalizationRequest, NormalizedIr, NormalizedMedia, NormalizedNote,
    NormalizedNotetype, NormalizedTemplate,
};
pub use normalize::normalize;
pub use risk::assess_risk;
pub use selector::{
    parse_selector, resolve_selector, Selector, SelectorError, SelectorResolveError, SelectorTarget,
};

pub fn tool_contract_version() -> &'static str {
    "phase2-v1"
}
```

- [ ] **Step 4: Run the normalization tests to verify they pass**

Run: `cargo test -p authoring_core --test normalization_pipeline_tests -v`
Expected: PASS for the Basic, Cloze, and Image Occlusion writer-ready normalization cases.

- [ ] **Step 5: Run the CLI normalization test to confirm the richer input path**

Run: `cargo test -p contract_tools --test cli_tests normalize_contract_json_includes_required_top_level_fields -v`
Expected: PASS, with the `normalize` command still returning valid contract JSON.

- [ ] **Step 6: Commit**

```bash
git add authoring_core/src/model.rs authoring_core/src/stock.rs authoring_core/src/normalize.rs authoring_core/src/lib.rs authoring_core/tests/normalization_pipeline_tests.rs contract_tools/src/normalize_cmd.rs
git commit -m "feat: emit writer-ready normalized stock lanes"
```

### Task 4: Add Phase 3 report schemas, policies, and semantics assets

**Files:**
- Modify: `contracts/manifest.yaml`
- Create: `contracts/schema/package-build-result.schema.json`
- Create: `contracts/schema/inspect-report.schema.json`
- Create: `contracts/schema/diff-report.schema.json`
- Create: `contracts/schema/writer-policy.schema.json`
- Create: `contracts/schema/verification-policy.schema.json`
- Create: `contracts/schema/build-context.schema.json`
- Create: `contracts/policies/writer-policy.default.yaml`
- Create: `contracts/policies/verification-policy.default.yaml`
- Create: `contracts/contexts/build-context.default.yaml`
- Create: `contracts/semantics/build.md`
- Create: `contracts/semantics/inspect.md`
- Create: `contracts/semantics/diff.md`
- Create: `contracts/semantics/golden-regression.md`
- Modify: `contract_tools/src/policies.rs`
- Modify: `contract_tools/src/semantics.rs`
- Modify: `contract_tools/tests/schema_gate_tests.rs`
- Modify: `contract_tools/tests/policy_gate_tests.rs`

- [ ] **Step 1: Write failing tests for the Phase 3 asset keys and policy validation**

```rust
// contract_tools/tests/schema_gate_tests.rs
#[test]
fn manifest_registers_phase3_schema_and_semantics_assets() {
    let manifest = load_manifest(contract_manifest_path()).unwrap();
    for asset_key in [
        "package_build_result_schema",
        "inspect_report_schema",
        "diff_report_schema",
        "writer_policy_schema",
        "verification_policy_schema",
        "build_context_schema",
        "build_context_default",
        "build_semantics",
        "inspect_semantics",
        "diff_semantics",
        "golden_regression_semantics",
    ] {
        assert!(
            resolve_asset_path(&manifest, asset_key).is_ok(),
            "manifest is missing asset key {asset_key}"
        );
    }
}
```

```rust
// contract_tools/tests/policy_gate_tests.rs
#[test]
fn phase3_policy_assets_validate_against_declared_schemas() {
    run_policy_gates(contract_manifest_path()).expect("phase3 policy assets should validate");
}
```

- [ ] **Step 2: Run the schema and policy tests to verify they fail**

Run: `cargo test -p contract_tools --test schema_gate_tests --test policy_gate_tests -v`
Expected: FAIL because the Phase 3 schemas, policies, and semantics docs do not exist yet.

- [ ] **Step 3: Add the Phase 3 assets and register them in the manifest**

```yaml
# contracts/manifest.yaml
# Append these keys to the existing assets map without removing current entries.
assets:
  package_build_result_schema: schema/package-build-result.schema.json
  inspect_report_schema: schema/inspect-report.schema.json
  diff_report_schema: schema/diff-report.schema.json
  writer_policy_schema: schema/writer-policy.schema.json
  verification_policy_schema: schema/verification-policy.schema.json
  build_context_schema: schema/build-context.schema.json
  writer_policy: policies/writer-policy.default.yaml
  verification_policy: policies/verification-policy.default.yaml
  build_context_default: contexts/build-context.default.yaml
  build_semantics: semantics/build.md
  inspect_semantics: semantics/inspect.md
  diff_semantics: semantics/diff.md
  golden_regression_semantics: semantics/golden-regression.md
```

```yaml
# contracts/contexts/build-context.default.yaml
id: build-context.default
version: 1.0.0
emit_apkg: true
materialize_staging: true
media_resolution_mode: inline-only
unresolved_asset_behavior: fail
fingerprint_mode: canonical
```

```json
// contracts/schema/package-build-result.schema.json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": [
    "kind",
    "result_status",
    "tool_contract_version",
    "writer_policy_ref",
    "build_context_ref",
    "diagnostics"
  ],
  "additionalProperties": false,
  "properties": {
    "kind": { "const": "package-build-result" },
    "result_status": { "enum": ["success", "invalid", "error"] },
    "tool_contract_version": { "type": "string", "minLength": 1 },
    "writer_policy_ref": { "type": "string", "minLength": 1 },
    "build_context_ref": { "type": "string", "minLength": 1 },
    "staging_ref": { "type": "string", "minLength": 1 },
    "artifact_fingerprint": { "type": "string", "minLength": 1 },
    "package_fingerprint": { "type": "string", "minLength": 1 },
    "apkg_ref": { "type": "string", "minLength": 1 },
    "diagnostics": {
      "type": "object",
      "required": ["kind", "items"],
      "additionalProperties": false,
      "properties": {
        "kind": { "const": "build-diagnostics" },
        "items": {
          "type": "array",
          "items": {
            "type": "object",
            "required": ["level", "code", "summary"],
            "additionalProperties": false,
            "properties": {
              "level": { "enum": ["warning", "error"] },
              "code": { "type": "string", "minLength": 1 },
              "summary": { "type": "string", "minLength": 1 },
              "domain": { "type": "string" },
              "path": { "type": "string" },
              "target_selector": { "type": "string" },
              "stage": { "type": "string" },
              "operation": { "type": "string" }
            }
          }
        }
      }
    }
  },
  "allOf": [
    {
      "if": { "properties": { "result_status": { "const": "success" } } },
      "then": { "required": ["staging_ref", "artifact_fingerprint"] }
    }
  ]
}
```

```json
// contracts/schema/inspect-report.schema.json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": [
    "kind",
    "observation_model_version",
    "source_kind",
    "source_ref",
    "artifact_fingerprint",
    "observation_status",
    "missing_domains",
    "degradation_reasons",
    "observations"
  ],
  "additionalProperties": false,
  "properties": {
    "kind": { "const": "inspect-report" },
    "observation_model_version": { "type": "string", "minLength": 1 },
    "source_kind": { "enum": ["staging", "apkg"] },
    "source_ref": { "type": "string", "minLength": 1 },
    "artifact_fingerprint": { "type": "string", "minLength": 1 },
    "observation_status": { "enum": ["complete", "degraded", "unavailable"] },
    "missing_domains": {
      "type": "array",
      "items": { "type": "string", "minLength": 1 }
    },
    "degradation_reasons": {
      "type": "array",
      "items": { "type": "string", "minLength": 1 }
    },
    "observations": {
      "type": "object",
      "required": ["notetypes", "templates", "fields", "media", "metadata", "references"],
      "additionalProperties": false,
      "properties": {
        "notetypes": { "type": "array" },
        "templates": { "type": "array" },
        "fields": { "type": "array" },
        "media": { "type": "array" },
        "metadata": { "type": "array" },
        "references": { "type": "array" }
      }
    }
  }
}
```

```json
// contracts/schema/diff-report.schema.json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": [
    "kind",
    "comparison_status",
    "left_fingerprint",
    "right_fingerprint",
    "left_observation_model_version",
    "right_observation_model_version",
    "summary",
    "uncompared_domains",
    "comparison_limitations",
    "changes"
  ],
  "additionalProperties": false,
  "properties": {
    "kind": { "const": "diff-report" },
    "comparison_status": { "enum": ["complete", "partial", "unavailable"] },
    "left_fingerprint": { "type": "string", "minLength": 1 },
    "right_fingerprint": { "type": "string", "minLength": 1 },
    "left_observation_model_version": { "type": "string", "minLength": 1 },
    "right_observation_model_version": { "type": "string", "minLength": 1 },
    "summary": { "type": "string", "minLength": 1 },
    "uncompared_domains": {
      "type": "array",
      "items": { "type": "string", "minLength": 1 }
    },
    "comparison_limitations": {
      "type": "array",
      "items": { "type": "string", "minLength": 1 }
    },
    "changes": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["category", "domain", "severity", "selector", "message", "compatibility_hint"],
        "additionalProperties": false,
        "properties": {
          "category": { "type": "string", "minLength": 1 },
          "domain": { "type": "string", "minLength": 1 },
          "severity": { "enum": ["low", "medium", "high"] },
          "selector": { "type": "string", "minLength": 1 },
          "message": { "type": "string", "minLength": 1 },
          "compatibility_hint": { "type": "string", "minLength": 1 },
          "evidence_refs": {
            "type": "array",
            "items": { "type": "string", "minLength": 1 }
          }
        }
      }
    }
  }
}
```

```json
// contracts/schema/writer-policy.schema.json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": [
    "id",
    "version",
    "compatibility_target",
    "stock_notetype_mode",
    "media_entry_mode",
    "apkg_version"
  ],
  "additionalProperties": false,
  "properties": {
    "id": { "type": "string", "minLength": 1 },
    "version": { "type": "string", "minLength": 1 },
    "compatibility_target": { "const": "latest-only" },
    "stock_notetype_mode": { "type": "string", "minLength": 1 },
    "media_entry_mode": { "type": "string", "minLength": 1 },
    "apkg_version": { "type": "string", "minLength": 1 }
  }
}
```

```json
// contracts/schema/verification-policy.schema.json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["id", "version", "writer_fast_gate", "system_gate", "compat_gate"],
  "additionalProperties": false,
  "properties": {
    "id": { "type": "string", "minLength": 1 },
    "version": { "type": "string", "minLength": 1 },
    "writer_fast_gate": { "$ref": "#/$defs/gate_rule" },
    "system_gate": { "$ref": "#/$defs/gate_rule" },
    "compat_gate": { "$ref": "#/$defs/gate_rule" }
  },
  "$defs": {
    "gate_rule": {
      "type": "object",
      "required": [
        "minimum_comparison_status",
        "allowed_observation_statuses",
        "blocking_severities"
      ],
      "additionalProperties": false,
      "properties": {
        "minimum_comparison_status": { "enum": ["complete", "partial", "unavailable"] },
        "allowed_observation_statuses": {
          "type": "array",
          "items": { "enum": ["complete", "degraded", "unavailable"] }
        },
        "blocking_severities": {
          "type": "array",
          "items": { "enum": ["low", "medium", "high"] }
        }
      }
    }
  }
}
```

```json
// contracts/schema/build-context.schema.json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": [
    "emit_apkg",
    "materialize_staging",
    "media_resolution_mode",
    "unresolved_asset_behavior",
    "fingerprint_mode"
  ],
  "additionalProperties": false,
  "properties": {
    "emit_apkg": { "type": "boolean" },
    "materialize_staging": { "type": "boolean" },
    "media_resolution_mode": { "enum": ["inline-only"] },
    "unresolved_asset_behavior": { "enum": ["fail", "warn"] },
    "fingerprint_mode": { "enum": ["canonical"] }
  }
}
```

```yaml
# contracts/policies/writer-policy.default.yaml
id: writer-policy.default
version: 1.0.0
compatibility_target: latest-only
stock_notetype_mode: source-grounded
media_entry_mode: inline
apkg_version: latest
```

```yaml
# contracts/policies/verification-policy.default.yaml
id: verification-policy.default
version: 1.0.0
writer_fast_gate:
  minimum_comparison_status: complete
  allowed_observation_statuses: [complete]
  blocking_severities: [high]
system_gate:
  minimum_comparison_status: partial
  allowed_observation_statuses: [complete, degraded]
  blocking_severities: [high]
compat_gate:
  minimum_comparison_status: complete
  allowed_observation_statuses: [complete]
  blocking_severities: [medium, high]
```

```md
<!-- contracts/semantics/build.md -->
---
asset_refs:
  - schema/package-build-result.schema.json
  - schema/writer-policy.schema.json
  - schema/build-context.schema.json
---
# Build

Source anchors:

- `docs/source/rslib/src/import_export/package/apkg/export.rs`
- `docs/source/rslib/src/import_export/package/colpkg/export.rs`
- `docs/source/rslib/src/import_export/package/meta.rs`
- `docs/source/rslib/src/import_export/package/media.rs`

Build outputs a staging representation first and only then packages `.apkg`.
```

```md
<!-- contracts/semantics/inspect.md -->
---
asset_refs:
  - schema/inspect-report.schema.json
---
# Inspect

Inspection reports are stable observation models.
They are not byte dumps and must exclude packaging noise that is not relevant to compatibility behavior.
```

```md
<!-- contracts/semantics/diff.md -->
---
asset_refs:
  - schema/diff-report.schema.json
---
# Diff

Diff reports describe evidence and compatibility hints.
They do not decide workflow success or failure.
```

```md
<!-- contracts/semantics/golden-regression.md -->
---
asset_refs:
  - schema/inspect-report.schema.json
  - schema/diff-report.schema.json
---
# Golden Regression

Golden files are case-derived outputs.
Updating a golden requires confirming whether the change is an intentional compatibility change or a regression.
```

```rust
// contract_tools/src/policies.rs
pub fn run_policy_gates(manifest_path: impl AsRef<Path>) -> anyhow::Result<()> {
    let manifest = load_manifest(manifest_path)?;

    validate_policy_asset(&manifest, "identity_policy_schema", "identity_policy", "identity policy")?;
    validate_policy_asset(&manifest, "risk_policy_schema", "risk_policy", "risk policy")?;
    validate_policy_asset(&manifest, "writer_policy_schema", "writer_policy", "writer policy")?;
    validate_policy_asset(
        &manifest,
        "verification_policy_schema",
        "verification_policy",
        "verification policy",
    )?;
    validate_policy_asset(
        &manifest,
        "build_context_schema",
        "build_context_default",
        "build context",
    )?;

    Ok(())
}
```

```rust
// contract_tools/src/semantics.rs
for key in [
    "validation_semantics",
    "path_semantics",
    "compatibility_semantics",
    "normalization_semantics",
    "build_semantics",
    "inspect_semantics",
    "diff_semantics",
    "golden_regression_semantics",
] {
    // existing semantics gate body unchanged
}
```

- [ ] **Step 4: Run the schema and policy tests to verify they pass**

Run: `cargo test -p contract_tools --test schema_gate_tests --test policy_gate_tests -v`
Expected: PASS, including the new Phase 3 manifest asset and policy validation tests.

- [ ] **Step 5: Commit**

```bash
git add contracts/manifest.yaml contracts/schema/package-build-result.schema.json contracts/schema/inspect-report.schema.json contracts/schema/diff-report.schema.json contracts/schema/writer-policy.schema.json contracts/schema/verification-policy.schema.json contracts/schema/build-context.schema.json contracts/policies/writer-policy.default.yaml contracts/policies/verification-policy.default.yaml contracts/contexts/build-context.default.yaml contracts/semantics/build.md contracts/semantics/inspect.md contracts/semantics/diff.md contracts/semantics/golden-regression.md contract_tools/src/policies.rs contract_tools/src/semantics.rs contract_tools/tests/schema_gate_tests.rs contract_tools/tests/policy_gate_tests.rs
git commit -m "feat: add phase3 report contracts and policy assets"
```

### Task 5: Implement `writer_core` report models, policy loading, and canonical JSON

**Files:**
- Create: `writer_core/src/model.rs`
- Create: `writer_core/src/policy.rs`
- Create: `writer_core/src/canonical_json.rs`
- Modify: `writer_core/src/lib.rs`
- Create: `writer_core/tests/build_tests.rs`

- [ ] **Step 1: Write the failing unit tests for build result tracing and canonical JSON**

```rust
// writer_core/tests/build_tests.rs
use writer_core::{
    to_canonical_json, BuildContext, BuildDiagnostics, BuildDiagnosticItem, PackageBuildResult,
    VerificationPolicy, WriterPolicy,
};

#[test]
fn package_build_result_carries_writer_and_build_context_refs() {
    let result = PackageBuildResult {
        kind: "package-build-result".into(),
        result_status: "success".into(),
        tool_contract_version: writer_core::tool_contract_version().into(),
        writer_policy_ref: "writer-policy.default@1.0.0".into(),
        build_context_ref: "build-context:abc".into(),
        staging_ref: Some("staging:demo".into()),
        artifact_fingerprint: Some("artifact:demo".into()),
        package_fingerprint: None,
        apkg_ref: None,
        diagnostics: BuildDiagnostics {
            kind: "build-diagnostics".into(),
            items: vec![],
        },
    };

    let json = serde_json::to_value(result).unwrap();
    assert_eq!(json["writer_policy_ref"], "writer-policy.default@1.0.0");
    assert_eq!(json["build_context_ref"], "build-context:abc");
}

#[test]
fn canonical_json_orders_phase3_report_keys_stably() {
    let json = to_canonical_json(&serde_json::json!({
        "z": 1,
        "a": { "d": 4, "b": 2 }
    }))
    .unwrap();

    assert_eq!(json, "{\"a\":{\"b\":2,\"d\":4},\"z\":1}");
}
```

- [ ] **Step 2: Run the writer_core tests to verify they fail**

Run: `cargo test -p writer_core --test build_tests -v`
Expected: FAIL because the Phase 3 DTOs and canonical serializer do not exist yet.

- [ ] **Step 3: Add the core Phase 3 DTOs and policy helpers**

```rust
// writer_core/src/model.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriterPolicy {
    pub id: String,
    pub version: String,
    pub compatibility_target: String,
    pub stock_notetype_mode: String,
    pub media_entry_mode: String,
    pub apkg_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationPolicy {
    pub id: String,
    pub version: String,
    pub writer_fast_gate: VerificationGateRule,
    pub system_gate: VerificationGateRule,
    pub compat_gate: VerificationGateRule,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationGateRule {
    pub minimum_comparison_status: String,
    pub allowed_observation_statuses: Vec<String>,
    pub blocking_severities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildContext {
    pub emit_apkg: bool,
    pub materialize_staging: bool,
    pub media_resolution_mode: String,
    pub unresolved_asset_behavior: String,
    pub fingerprint_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildDiagnosticItem {
    pub level: String,
    pub code: String,
    pub summary: String,
    pub domain: Option<String>,
    pub path: Option<String>,
    pub target_selector: Option<String>,
    pub stage: Option<String>,
    pub operation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildDiagnostics {
    pub kind: String,
    pub items: Vec<BuildDiagnosticItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageBuildResult {
    pub kind: String,
    pub result_status: String,
    pub tool_contract_version: String,
    pub writer_policy_ref: String,
    pub build_context_ref: String,
    pub staging_ref: Option<String>,
    pub artifact_fingerprint: Option<String>,
    pub package_fingerprint: Option<String>,
    pub apkg_ref: Option<String>,
    pub diagnostics: BuildDiagnostics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffChange {
    pub category: String,
    pub domain: String,
    pub severity: String,
    pub selector: String,
    pub message: String,
    pub compatibility_hint: String,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffReport {
    pub kind: String,
    pub comparison_status: String,
    pub left_fingerprint: String,
    pub right_fingerprint: String,
    pub left_observation_model_version: String,
    pub right_observation_model_version: String,
    pub summary: String,
    pub uncompared_domains: Vec<String>,
    pub comparison_limitations: Vec<String>,
    pub changes: Vec<DiffChange>,
}
```

```rust
// writer_core/src/policy.rs
pub fn policy_ref(id: &str, version: &str) -> String {
    format!("{id}@{version}")
}

pub fn build_context_ref(context: &crate::BuildContext) -> anyhow::Result<String> {
    let canonical = crate::to_canonical_json(context)?;
    Ok(format!("build-context:{}", hex::encode(sha1::Sha1::digest(canonical.as_bytes()))))
}
```

```rust
// writer_core/src/canonical_json.rs
pub fn to_canonical_json(value: &impl serde::Serialize) -> anyhow::Result<String> {
    let value = serde_json::to_value(value)?;
    let normalized = normalize(value);
    Ok(serde_json::to_string(&normalized)?)
}

fn normalize(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let mut entries = map.into_iter().collect::<Vec<_>>();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            serde_json::Value::Object(
                entries
                    .into_iter()
                    .map(|(k, v)| (k, normalize(v)))
                    .collect(),
            )
        }
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.into_iter().map(normalize).collect())
        }
        other => other,
    }
}
```

```rust
// writer_core/src/lib.rs
pub mod canonical_json;
pub mod model;
pub mod policy;

pub use canonical_json::to_canonical_json;
pub use model::*;
pub use policy::{build_context_ref, policy_ref};

pub fn tool_contract_version() -> &'static str {
    "phase3-v1"
}
```

Keep `writer_core/src/lib.rs` minimal in this task. Do not declare `build`, `staging`, `apkg`, `inspect`, or `diff` modules until the corresponding files are created in later tasks.

- [ ] **Step 4: Run the writer_core tests to verify they pass**

Run: `cargo test -p writer_core --test build_tests -v`
Expected: PASS for the tracing and canonical JSON tests.

- [ ] **Step 5: Commit**

```bash
git add writer_core/src/model.rs writer_core/src/policy.rs writer_core/src/canonical_json.rs writer_core/src/lib.rs writer_core/tests/build_tests.rs
git commit -m "feat: add phase3 writer report models"
```

### Task 6: Implement deterministic staging build for Basic and Cloze plus materialized staging artifacts

**Files:**
- Create: `writer_core/src/staging.rs`
- Create: `writer_core/src/build.rs`
- Modify: `writer_core/src/lib.rs`
- Modify: `writer_core/tests/build_tests.rs`

- [ ] **Step 1: Write failing build tests for Basic and Cloze staging**

Add tests that prove three things before any implementation lands:

1. `writer_core::build()` materializes a reusable staging artifact into a caller-owned artifact root.
2. `staging_ref` is a stable ref into that artifact root, not an ephemeral temp path.
3. `artifact_fingerprint` is derived from canonical staging contents, not from a path string.

Representative assertions:

```rust
let temp = tempfile::tempdir().unwrap();
let target = writer_core::BuildArtifactTarget::new(
    temp.path().join("phase3/basic"),
    "artifacts/phase3/basic".into(),
);

let result = writer_core::build(
    &sample_basic_normalized_ir(),
    &sample_writer_policy(),
    &sample_build_context(false),
    &target,
)
.unwrap();

assert_eq!(result.result_status, "success");
assert_eq!(
    result.staging_ref.as_deref(),
    Some("artifacts/phase3/basic/staging/manifest.json")
);
assert!(temp.path().join("phase3/basic/staging/manifest.json").exists());
assert!(result.artifact_fingerprint.as_deref().unwrap().starts_with("artifact:"));
```

Keep the existing invalid-build test, but require `invalid` diagnostics to include selector/path-level context.

- [ ] **Step 2: Run the build tests to verify they fail**

Run: `cargo test -p writer_core --test build_tests -v`
Expected: FAIL because the staging representation, materialization target, and build engine do not exist yet.

- [ ] **Step 3: Implement deterministic staging and semantic build validation**

Implement the writer-side staging model with two distinct layers:

1. `StagingPackage`: the canonical in-memory package representation.
2. `MaterializedStaging`: the caller-owned on-disk staging tree, including a stable manifest file plus any media payloads needed by later `inspect` and `.apkg` emission.

Key implementation requirements:

- Add `BuildArtifactTarget { root_dir, stable_ref_prefix }` so the caller, not a tempdir helper, owns artifact lifetime.
- `build()` becomes `build(normalized, writer_policy, build_context, artifact_target)`.
- `StagingPackage::from_normalized()` performs semantic validation for required fields, stock lane shape, and unresolved assets according to `build_context`.
- `materialize_staging()` writes a deterministic staging tree, with a canonical `staging/manifest.json` entry as the primary inspectable artifact.
- `artifact_fingerprint` is computed from canonical staging content bytes, not from any filesystem path.
- `staging_ref` is the stable ref returned by `artifact_target`, for example `artifacts/phase3/basic/staging/manifest.json`.
- `PackageBuildResult` remains `invalid` for semantic precondition failures and `error` for execution failures. `invalid` items must carry `domain`, `path`, and `target_selector` whenever possible.

Now that the files exist, extend `writer_core/src/lib.rs` to export:

```rust
pub mod build;
pub mod canonical_json;
pub mod model;
pub mod policy;
pub mod staging;

pub use build::{build, BuildArtifactTarget};
pub use canonical_json::to_canonical_json;
pub use model::*;
pub use policy::{build_context_ref, policy_ref};
pub use staging::{MaterializedStaging, StagingPackage};
```

- [ ] **Step 4: Run the build tests to verify they pass**

Run: `cargo test -p writer_core --test build_tests -v`
Expected: PASS for Basic/Cloze staging materialization and missing-field diagnostic tests.

- [ ] **Step 5: Commit**

```bash
git add writer_core/src/staging.rs writer_core/src/build.rs writer_core/src/lib.rs writer_core/tests/build_tests.rs
git commit -m "feat: build deterministic staging artifacts for basic and cloze"
```

### Task 7: Add the scoped Image Occlusion lane and source-grounded `.apkg` materialization

**Files:**
- Create: `writer_core/src/apkg.rs`
- Modify: `writer_core/src/build.rs`
- Modify: `writer_core/src/lib.rs`
- Modify: `writer_core/tests/build_tests.rs`
- Modify: `writer_core/Cargo.toml`

- [ ] **Step 1: Write failing tests for Image Occlusion build and `.apkg` emission**

Add tests that verify:

1. The scoped Image Occlusion normalized lane builds successfully into staging and `.apkg`.
2. The emitted `.apkg` lives under the caller-owned artifact root and still exists after `build()` returns.
3. Latest-lane `.apkg` layout matches the local `rslib` references:
   - `meta` exists and encodes the latest package lane
   - `collection.anki21b` exists
   - `collection.anki2` exists as the legacy dummy collection lane
   - `media` exists
4. Media-map encoding for latest packages follows `docs/source/rslib/src/import_export/package/media.rs` and `colpkg/export.rs`, not an ad-hoc JSON map.

- [ ] **Step 2: Run the build tests to verify they fail**

Run: `cargo test -p writer_core --test build_tests -v`
Expected: FAIL because `.apkg` emission and the IO lane do not exist yet.

- [ ] **Step 3: Add package emission using the source-grounded layout**

Implementation requirements for `writer_core/src/apkg.rs`:

- Use caller-owned artifact roots from `BuildArtifactTarget`; never create the returned `.apkg` inside a tempdir that is dropped before the caller can inspect it.
- Ground package layout in these local sources:
  - `docs/source/rslib/src/import_export/package/apkg/export.rs`
  - `docs/source/rslib/src/import_export/package/colpkg/export.rs`
  - `docs/source/rslib/src/import_export/package/meta.rs`
  - `docs/source/rslib/src/import_export/package/media.rs`
  - `docs/source/rslib/src/import_export/package/apkg/tests.rs`
- Materialize the latest lane with `collection.anki21b` plus `meta` semantics from `meta.rs`.
- Write the legacy `collection.anki2` lane as the source-backed dummy collection behavior from `write_dummy_collection()`, not by copying the latest collection bytes.
- Follow the latest-lane media behavior from `media.rs` / `colpkg/export.rs`:
  - media payload files are stored as numbered zip entries
  - media map is encoded in the latest format expected by `Meta::new()` packages
  - compression choices follow the latest-lane rules, rather than a handwritten approximation
- `package_fingerprint` is computed from the final `.apkg` bytes after materialization.
- `apkg_ref` is a stable ref into the artifact root, for example `artifacts/phase3/basic/package.apkg`.

Update `build.rs` so that when `build_context.emit_apkg == true`, the build:

1. Starts from the already materialized staging artifact.
2. Emits the `.apkg` into the same caller-owned artifact root.
3. Stores both `apkg_ref` and `package_fingerprint` in `PackageBuildResult`.

Now that the file exists, extend `writer_core/src/lib.rs` with:

```rust
pub mod apkg;
```

- [ ] **Step 4: Run the build tests to verify they pass**

Run: `cargo test -p writer_core --test build_tests -v`
Expected: PASS for Image Occlusion build and source-grounded `.apkg` layout/materialization tests.

- [ ] **Step 5: Commit**

```bash
git add writer_core/Cargo.toml writer_core/src/apkg.rs writer_core/src/build.rs writer_core/src/lib.rs writer_core/tests/build_tests.rs
git commit -m "feat: emit source-grounded phase3 apkg artifacts"
```

### Task 8: Implement inspection and diff with degradation/comparison completeness

**Files:**
- Create: `writer_core/src/inspect.rs`
- Create: `writer_core/src/diff.rs`
- Modify: `writer_core/src/lib.rs`
- Create: `writer_core/tests/inspect_tests.rs`
- Create: `writer_core/tests/diff_tests.rs`

- [ ] **Step 1: Write failing inspect and diff tests**

Add tests for the inspection-first guarantees we actually need:

1. `inspect_build_result()` prefers staging inspection when `staging_ref` is present, and only falls back to `.apkg` when staging is unavailable.
2. `inspect_staging()` emits a `complete` report with populated `notetypes`, `templates`, `fields`, `media`, `metadata`, and `references` domains.
3. `inspect_apkg()` reconstructs those same observation domains from archive contents and reports `degraded` or `unavailable` only when a domain truly cannot be read.
4. `artifact_fingerprint` is derived from canonical observation content or canonical staging content, never from a filesystem path string.
5. `diff_reports()` yields domain-specific changes with stable `selector` + `evidence_refs`, rather than a single generic “metadata changed” message.
6. A dedicated semantic-consistency test compares `inspect(staging)` and `inspect(apkg)` on a comparable fixture and expects `comparison_status=complete` with no compatibility-significant changes.

- [ ] **Step 2: Run the inspect and diff tests to verify they fail**

Run: `cargo test -p writer_core --test inspect_tests --test diff_tests -v`
Expected: FAIL because inspection and diff code do not exist yet.

- [ ] **Step 3: Implement the stable observation model and comparison engine**

Implementation requirements for `writer_core/src/inspect.rs`:

- Keep `observations` domain-structured: `notetypes`, `templates`, `fields`, `media`, `metadata`, `references`.
- `inspect_staging()` must read the materialized staging manifest/tree, not an ephemeral in-memory value, so the same artifact can drive fixture gates and CLI inspection.
- `inspect_build_result()` must inspect staging first for the writer fast gate, then use `.apkg` only for semantic consistency checks and compat acceptance.
- `inspect_apkg()` must do real archive inspection:
  - decode `meta`
  - validate latest-lane collection filename expectations from `meta.rs`
  - decode the media map according to the current lane from `media.rs`
  - extract/read the collection database and surface note type, note, card, and media-reference observations
- `observation_status`, `missing_domains`, and `degradation_reasons` must be populated from actual read limitations.
- `artifact_fingerprint` must be derived from canonicalized observation content or canonical staging bytes, not from the input path.

Implementation requirements for `writer_core/src/diff.rs`:

- Compare each observation domain separately.
- Produce `DiffChange` entries with stable `category`, `domain`, `severity`, `selector`, `compatibility_hint`, and `evidence_refs`.
- Use `comparison_status`, `uncompared_domains`, and `comparison_limitations` only for actual partial/unavailable comparisons.
- Treat “no changes” as a true empty change list, not as a special-case message hiding domain mismatches.

Now that the files exist, extend `writer_core/src/lib.rs` with:

```rust
pub mod diff;
pub mod inspect;

pub use diff::diff_reports;
pub use inspect::{inspect_apkg, inspect_build_result, inspect_staging, InspectObservations, InspectReport};
```

- [ ] **Step 4: Run the inspect and diff tests to verify they pass**

Run: `cargo test -p writer_core --test inspect_tests --test diff_tests -v`
Expected: PASS for observation status, comparison completeness, and staging/apkg semantic consistency tests.

- [ ] **Step 5: Commit**

```bash
git add writer_core/src/inspect.rs writer_core/src/diff.rs writer_core/src/lib.rs writer_core/tests/inspect_tests.rs writer_core/tests/diff_tests.rs
git commit -m "feat: add phase3 inspect and diff engines"
```

### Task 9: Add `contract_tools` build, inspect, and diff commands with stable `contract-json` output

**Files:**
- Modify: `contract_tools/src/main.rs`
- Modify: `contract_tools/src/lib.rs`
- Modify: `contract_tools/src/policies.rs`
- Create: `contract_tools/src/build_cmd.rs`
- Create: `contract_tools/src/inspect_cmd.rs`
- Create: `contract_tools/src/diff_cmd.rs`
- Modify: `contract_tools/tests/cli_tests.rs`

- [ ] **Step 1: Write failing CLI tests for `build`, `inspect`, and `diff`**

```rust
// contract_tools/tests/cli_tests.rs
#[test]
fn build_command_emits_contract_json_with_policy_and_context_refs() {
    let manifest =
        contract_tools::manifest::load_manifest(contract_tools::contract_manifest_path()).unwrap();
    let temp = tempdir().unwrap();
    let input = temp.path().join("basic-normalized-ir.json");
    fs::write(
        &input,
        serde_json::to_string_pretty(&serde_json::json!({
            "kind": "normalized-ir",
            "schema_version": "0.1.0",
            "document_id": "demo-doc",
            "resolved_identity": "det:demo-doc",
            "notetypes": [{
                "id": "basic-main",
                "kind": "basic",
                "name": "Basic",
                "fields": ["Front", "Back"],
                "templates": [{
                    "name": "Card 1",
                    "question_format": "{{Front}}",
                    "answer_format": "{{FrontSide}}\n\n<hr id=answer>\n\n{{Back}}"
                }],
                "css": ""
            }],
            "notes": [{
                "id": "note-1",
                "notetype_id": "basic-main",
                "deck_name": "Default",
                "fields": {"Front": "front", "Back": "back"},
                "tags": []
            }],
            "media": []
        }))
        .unwrap(),
    )
    .unwrap();

    let output = run_cli(&[
        "build",
        "--manifest",
        manifest.path.to_str().unwrap(),
        "--input",
        input.to_str().unwrap(),
        "--writer-policy",
        "default",
        "--build-context",
        "default",
        "--artifacts-dir",
        temp.path().join("artifacts").to_str().unwrap(),
        "--output",
        "contract-json",
    ]);

    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["kind"], "package-build-result");
    assert!(value["writer_policy_ref"].is_string());
    assert!(value["build_context_ref"].is_string());
}

#[test]
fn inspect_command_emits_stable_contract_json() {
    let temp = tempdir().unwrap();
    let staging = temp.path().join("basic/staging/manifest.json");
    std::fs::create_dir_all(staging.parent().unwrap()).unwrap();
    fs::write(
        &staging,
        serde_json::to_string_pretty(&serde_json::json!({
            "notetypes": [],
            "notes": [],
            "media": []
        }))
        .unwrap(),
    )
    .unwrap();

    let output = run_cli(&[
        "inspect",
        "--staging",
        staging.to_str().unwrap(),
        "--output",
        "contract-json",
    ]);
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
}

#[test]
fn diff_command_emits_diff_report_contract_json() {
    let temp = tempdir().unwrap();
    let left = temp.path().join("left.inspect.json");
    let right = temp.path().join("right.inspect.json");
    let inspect = serde_json::json!({
        "kind": "inspect-report",
        "observation_model_version": "phase3-inspect-v1",
        "source_kind": "staging",
        "source_ref": "staging:demo",
        "artifact_fingerprint": "artifact:demo",
        "observation_status": "complete",
        "missing_domains": [],
        "degradation_reasons": [],
        "observations": {
            "notetypes": [],
            "templates": [],
            "fields": [],
            "media": [],
            "metadata": [],
            "references": []
        }
    });
    fs::write(&left, serde_json::to_string_pretty(&inspect).unwrap()).unwrap();
    fs::write(&right, serde_json::to_string_pretty(&inspect).unwrap()).unwrap();

    let output = run_cli(&[
        "diff",
        "--left",
        left.to_str().unwrap(),
        "--right",
        right.to_str().unwrap(),
        "--output",
        "contract-json",
    ]);
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let value: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["kind"], "diff-report");
}
```

- [ ] **Step 2: Run the CLI tests to verify they fail**

Run: `cargo test -p contract_tools --test cli_tests -v`
Expected: FAIL because the new subcommands do not exist yet.

- [ ] **Step 3: Add the new CLI subcommands and stable `contract-json` renderers**

```rust
// contract_tools/src/main.rs
#[derive(Debug, Subcommand)]
enum Command {
    Verify { #[arg(long)] manifest: String },
    Summary { #[arg(long)] manifest: String },
    Package { #[arg(long)] manifest: String, #[arg(long)] out_dir: String },
    Normalize { #[arg(long)] manifest: String, #[arg(long)] input: String, #[arg(long, default_value = "contract-json")] output: String },
    Build {
        #[arg(long)] manifest: String,
        #[arg(long)] input: String,
        #[arg(long, default_value = "default")] writer_policy: String,
        #[arg(long, default_value = "default")] build_context: String,
        #[arg(long)] artifacts_dir: String,
        #[arg(long, default_value = "contract-json")] output: String,
    },
    Inspect {
        #[arg(long)] staging: Option<String>,
        #[arg(long)] apkg: Option<String>,
        #[arg(long, default_value = "contract-json")] output: String,
    },
    Diff {
        #[arg(long)] left: String,
        #[arg(long)] right: String,
        #[arg(long, default_value = "contract-json")] output: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Verify { manifest } => {
            contract_tools::gates::run_all(&manifest)?;
            println!("verification passed");
        }
        Command::Summary { manifest } => {
            println!("{}", contract_tools::summary::render(&manifest)?);
        }
        Command::Package { manifest, out_dir } => {
            let artifact_path = contract_tools::package::build_artifact(&manifest, &out_dir)?;
            println!("{}", artifact_path.display());
        }
        Command::Normalize { manifest, input, output } => {
            print!("{}", contract_tools::normalize_cmd::run(&manifest, &input, &output)?);
        }
        Command::Build { manifest, input, writer_policy, build_context, artifacts_dir, output } => {
            print!("{}", contract_tools::build_cmd::run(&manifest, &input, &writer_policy, &build_context, &artifacts_dir, &output)?);
        }
        Command::Inspect { staging, apkg, output } => {
            print!("{}", contract_tools::inspect_cmd::run(staging.as_deref(), apkg.as_deref(), &output)?);
        }
        Command::Diff { left, right, output } => {
            print!("{}", contract_tools::diff_cmd::run(&left, &right, &output)?);
        }
    }

    Ok(())
}
```

```rust
// contract_tools/src/build_cmd.rs
pub fn run(
    manifest: &str,
    input: &str,
    writer_policy: &str,
    build_context: &str,
    artifacts_dir: &str,
    output: &str,
) -> anyhow::Result<String> {
    let manifest = crate::manifest::load_manifest(manifest)?;
    let input_value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(input)?)?;
    let schema = crate::schema::load_schema(
        crate::manifest::resolve_asset_path(&manifest, "normalized_ir_schema")?
    )?;
    crate::schema::validate_value(&schema, &input_value)?;

    let normalized: authoring_core::NormalizedIr = serde_json::from_value(input_value)?;
    let writer_policy = crate::policies::load_writer_policy_asset(&manifest, writer_policy)?;
    let build_context = crate::policies::load_build_context_asset(&manifest, build_context)?;
    let artifact_target = writer_core::BuildArtifactTarget::new(
        std::path::PathBuf::from(artifacts_dir),
        "artifacts".into(),
    );
    let result = writer_core::build(&normalized, &writer_policy, &build_context, &artifact_target)?;

    match output {
        "contract-json" => writer_core::to_canonical_json(&result),
        "human" => Ok(format!("status: {}", result.result_status)),
        other => anyhow::bail!("unsupported build output mode: {other}"),
    }
}
```

```rust
// contract_tools/src/policies.rs
pub fn load_writer_policy_asset(
    manifest: &crate::manifest::LoadedManifest,
    selector: &str,
) -> anyhow::Result<writer_core::WriterPolicy> {
    let policy_path = resolve_asset_path(manifest, "writer_policy")?;
    let raw = fs::read_to_string(policy_path)?;
    let policy: writer_core::WriterPolicy = serde_yaml::from_str(&raw)?;
    anyhow::ensure!(selector == "default", "only default writer_policy selector is supported initially");
    Ok(policy)
}

pub fn load_build_context_asset(
    manifest: &crate::manifest::LoadedManifest,
    selector: &str,
) -> anyhow::Result<writer_core::BuildContext> {
    anyhow::ensure!(selector == "default", "only default build_context selector is supported initially");
    let context_path = resolve_asset_path(manifest, "build_context_default")?;
    let raw = fs::read_to_string(context_path)?;
    let context: writer_core::BuildContext = serde_yaml::from_str(&raw)?;
    Ok(context)
}
```

```rust
// contract_tools/src/inspect_cmd.rs
pub fn run(staging: Option<&str>, apkg: Option<&str>, output: &str) -> anyhow::Result<String> {
    let report = match (staging, apkg) {
        (Some(path), None) => writer_core::inspect_staging(path)?,
        (None, Some(path)) => writer_core::inspect_apkg(path)?,
        _ => anyhow::bail!("inspect requires exactly one of --staging or --apkg"),
    };

    match output {
        "contract-json" => writer_core::to_canonical_json(&report),
        "human" => Ok(format!("status: {}", report.observation_status)),
        other => anyhow::bail!("unsupported inspect output mode: {other}"),
    }
}
```

```rust
// contract_tools/src/diff_cmd.rs
pub fn run(left: &str, right: &str, output: &str) -> anyhow::Result<String> {
    let left: writer_core::InspectReport =
        serde_json::from_str(&std::fs::read_to_string(left)?)?;
    let right: writer_core::InspectReport =
        serde_json::from_str(&std::fs::read_to_string(right)?)?;
    let diff = writer_core::diff_reports(&left, &right)?;

    match output {
        "contract-json" => writer_core::to_canonical_json(&diff),
        "human" => Ok(format!("status: {}", diff.comparison_status)),
        other => anyhow::bail!("unsupported diff output mode: {other}"),
    }
}
```

```rust
// contract_tools/src/lib.rs
pub mod build_cmd;
pub mod diff_cmd;
pub mod fixtures;
pub mod gates;
pub mod inspect_cmd;
pub mod manifest;
pub mod normalize_cmd;
pub mod package;
pub mod policies;
pub mod registry;
pub mod schema;
pub mod semantics;
pub mod summary;
pub mod versioning;
```

Do not hardcode `BuildContext` defaults in the command layer. `contract_tools` must load the declared build-context asset from `contracts/` and combine it with a caller-supplied `--artifacts-dir` so materialized refs survive process exit.

- [ ] **Step 4: Run the CLI tests to verify they pass**

Run: `cargo test -p contract_tools --test cli_tests -v`
Expected: PASS for the new `build`, `inspect`, and `diff` contract-json tests.

- [ ] **Step 5: Commit**

```bash
git add contract_tools/src/main.rs contract_tools/src/lib.rs contract_tools/src/policies.rs contract_tools/src/build_cmd.rs contract_tools/src/inspect_cmd.rs contract_tools/src/diff_cmd.rs contract_tools/tests/cli_tests.rs
git commit -m "feat: add phase3 build inspect and diff commands"
```

### Task 10: Add Tier A and Tier B Phase 3 fixtures plus verify-gate execution

**Files:**
- Modify: `contracts/fixtures/index.yaml`
- Create: `contracts/fixtures/phase3/inputs/basic-authoring-ir.json`
- Create: `contracts/fixtures/phase3/inputs/basic-normalized-ir.json`
- Create: `contracts/fixtures/phase3/inputs/cloze-authoring-ir.json`
- Create: `contracts/fixtures/phase3/inputs/cloze-normalized-ir.json`
- Create: `contracts/fixtures/phase3/inputs/image-occlusion-authoring-ir.json`
- Create: `contracts/fixtures/phase3/inputs/image-occlusion-normalized-ir.json`
- Create: `contracts/fixtures/phase3/writer/basic-minimal.case.yaml`
- Create: `contracts/fixtures/phase3/writer/cloze-minimal.case.yaml`
- Create: `contracts/fixtures/phase3/writer/image-occlusion-minimal.case.yaml`
- Create: `contracts/fixtures/phase3/e2e/basic-minimal.case.yaml`
- Create: `contracts/fixtures/phase3/e2e/cloze-minimal.case.yaml`
- Create: `contracts/fixtures/phase3/e2e/image-occlusion-minimal.case.yaml`
- Create: `contracts/fixtures/phase3/expected/basic.build.json`
- Create: `contracts/fixtures/phase3/expected/basic.inspect.json`
- Create: `contracts/fixtures/phase3/expected/basic.diff.json`
- Create: `contracts/fixtures/phase3/expected/cloze.build.json`
- Create: `contracts/fixtures/phase3/expected/cloze.inspect.json`
- Create: `contracts/fixtures/phase3/expected/image-occlusion.build.json`
- Create: `contracts/fixtures/phase3/expected/image-occlusion.inspect.json`
- Modify: `contract_tools/src/fixtures.rs`
- Modify: `contract_tools/src/gates.rs`
- Modify: `contract_tools/tests/fixture_gate_tests.rs`

- [ ] **Step 1: Write failing fixture-gate tests for Phase 3 writer and e2e cases**

```rust
// contract_tools/tests/fixture_gate_tests.rs
fn copy_tree(src: &std::path::Path, dst: &std::path::Path) {
    std::fs::create_dir_all(dst).unwrap();
    for entry in std::fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_tree(&src_path, &dst_path);
        } else {
            std::fs::copy(&src_path, &dst_path).unwrap();
        }
    }
}

#[test]
fn fixture_gates_execute_phase3_writer_and_e2e_cases() {
    run_fixture_gates(contract_manifest_path()).expect("phase3 fixtures should pass");
}

#[test]
fn fixture_gates_reject_phase3_inspect_golden_mismatch() {
    let temp = tempdir().unwrap();
    let root = temp.path().join("bundle");
    copy_tree(
        contract_tools::contract_manifest_path().parent().unwrap(),
        &root,
    );
    let expected = root.join("fixtures/phase3/expected/basic.inspect.json");
    std::fs::write(&expected, r#"{"kind":"inspect-report","broken":true}"#).unwrap();
    let manifest_path = root.join("manifest.yaml");

    let err = run_fixture_gates(&manifest_path).expect_err("golden mismatch should fail");
    assert!(err.to_string().contains("phase3 inspect output mismatch"));
}
```

- [ ] **Step 2: Run the fixture gate tests to verify they fail**

Run: `cargo test -p contract_tools --test fixture_gate_tests -v`
Expected: FAIL because the Phase 3 fixtures and gate execution paths do not exist yet.

- [ ] **Step 3: Add case-first fixtures, deterministic artifact roots, and generated goldens**

Each Phase 3 fixture case must own:

- its input (`authoring` or `normalized`)
- its declared `writer_policy_ref`
- its declared `build_context_ref` or inline build-context payload
- its deterministic `artifacts_dir`
- its expected build/inspect/diff outputs

Representative writer case:

```yaml
# contracts/fixtures/phase3/writer/basic-minimal.case.yaml
kind: phase3-writer-case
normalized_input: fixtures/phase3/inputs/basic-normalized-ir.json
writer_policy_selector: default
build_context_selector: default
artifacts_dir: artifacts/phase3-writer-basic-minimal
expected_build: fixtures/phase3/expected/basic.build.json
expected_inspect: fixtures/phase3/expected/basic.inspect.json
expected_diff: fixtures/phase3/expected/basic.diff.json
```

Representative e2e case:

```yaml
# contracts/fixtures/phase3/e2e/basic-minimal.case.yaml
kind: phase3-e2e-case
authoring_input: fixtures/phase3/inputs/basic-authoring-ir.json
writer_policy_selector: default
build_context_selector: default
artifacts_dir: artifacts/phase3-e2e-basic-minimal
expected_build: fixtures/phase3/expected/basic.build.json
expected_inspect: fixtures/phase3/expected/basic.inspect.json
expected_diff: fixtures/phase3/expected/basic.diff.json
```

Do not hand-author placeholder `build_context_ref`, `staging_ref`, `artifact_fingerprint`, or `package_fingerprint` values. Instead:

1. Implement a small fixture-golden capture helper in `contract_tools/src/fixtures.rs` or the corresponding tests.
2. Materialize each case into its declared `artifacts_dir`.
3. Capture canonical `package-build-result`, `inspect-report`, and optional `diff-report` from the real deterministic outputs.
4. Review the generated files and check them in as the expected goldens.

Because `artifacts_dir` is case-local and deterministic, `staging_ref` / `apkg_ref` can also be stable deterministic refs instead of temp paths.

Update `contract_tools/src/fixtures.rs` so that:

- Tier A writer fixtures call `writer_core::build(..., artifact_target)` directly from normalized input.
- Tier B e2e fixtures call `normalize -> build(..., artifact_target) -> inspect(staging) -> diff`.
- Fixture execution asserts the referenced staging and `.apkg` artifacts actually exist before comparing goldens.
- Writer fast gate compares staging inspection against the case golden.
- Compat-oriented fixture execution also runs `inspect(apkg)` and checks staging/apkg semantic consistency on comparable domains.

- [ ] **Step 4: Run the fixture gate tests to verify they pass**

Run: `cargo test -p contract_tools --test fixture_gate_tests -v`
Expected: PASS for Phase 3 writer/e2e fixture execution, deterministic golden comparison, and staging/apkg semantic consistency checks.

- [ ] **Step 5: Run the repository verify gate to confirm the full loop is wired in**

Run: `cargo run -p contract_tools -- verify --manifest "$(pwd)/contracts/manifest.yaml"`
Expected: PASS with `verification passed`.

- [ ] **Step 6: Commit**

```bash
git add contracts/fixtures/index.yaml contracts/fixtures/phase3 contract_tools/src/fixtures.rs contract_tools/src/gates.rs contract_tools/tests/fixture_gate_tests.rs
git commit -m "feat: add phase3 fixture and verify gate coverage"
```

### Task 11: Add the controlled compatibility oracle and Phase 3 operator docs

**Files:**
- Modify: `contract_tools/Cargo.toml`
- Create: `contract_tools/src/compat_oracle.rs`
- Create: `contract_tools/tests/compat_oracle_tests.rs`
- Modify: `contract_tools/src/gates.rs`
- Modify: `contract_tools/src/lib.rs`
- Modify: `README.md`
- Create: `docs/superpowers/checklists/phase-3-exit-evidence.md`

- [ ] **Step 1: Write the failing compatibility-oracle test**

```rust
// contract_tools/tests/compat_oracle_tests.rs
#[test]
fn compat_oracle_accepts_supported_basic_package_layout_and_semantics() {
    let temp = tempfile::tempdir().unwrap();
    let target = writer_core::BuildArtifactTarget::new(
        temp.path().join("phase3/basic"),
        "artifacts/phase3/basic".into(),
    );
    let result = writer_core::build(
        &sample_basic_normalized_ir(),
        &sample_writer_policy(),
        &sample_build_context(true),
        &target,
    )
    .unwrap();
    let apkg = temp.path().join("phase3/basic/package.apkg");
    let inspect = writer_core::inspect_apkg(&apkg).unwrap();

contract_tools::compat_oracle::validate_supported_package(&apkg, &inspect)
        .expect("basic package should satisfy the controlled compatibility oracle");
}
```

Reuse the normalized/policy/build-context fixture helpers introduced earlier in the Phase 3 tests, or define local equivalents in this test file.

- [ ] **Step 2: Run the compatibility-oracle test to verify it fails**

Run: `cargo test -p contract_tools --test compat_oracle_tests -v`
Expected: FAIL because the oracle does not exist yet.

- [ ] **Step 3: Implement the source-grounded compatibility oracle and wire it into `compat gate`**

Implementation requirements for `contract_tools/src/compat_oracle.rs`:

- Keep the oracle source-grounded and stronger than a zip-entry smoke test.
- Validate package behavior against these local anchors:
  - `docs/source/rslib/src/import_export/package/meta.rs`
  - `docs/source/rslib/src/import_export/package/media.rs`
  - `docs/source/rslib/src/import_export/package/colpkg/export.rs`
  - `docs/source/rslib/src/import_export/package/apkg/tests.rs`
  - `docs/source/rslib/src/notetype/stock.rs`
  - `docs/source/rslib/src/image_occlusion/notetype.rs`
- The oracle must inspect at least:
  - `meta` lane and collection filename expectations
  - latest-lane media-map encoding/decoding behavior
  - collection DB readability plus note/card/notetype counts
  - stock-lane notetype/template invariants for supported fixtures
  - media reference consistency between collection content and media map
- Prefer comparing the package against the already generated `inspect-report`, so the oracle checks real archive semantics instead of reimplementing a second ad-hoc model.
- If a direct import smoke helper becomes feasible later, it is an additive lane. The controlled oracle remains the minimum acceptance requirement in this plan.

`run_compat_oracle_gates()` should reuse the declared writer fixtures, rebuild them into case-local artifact roots, inspect the resulting `.apkg`, and validate each supported core case through the oracle.

```toml
# contract_tools/Cargo.toml
[dependencies]
anyhow = "1"
clap = { version = "=4.5.20", features = ["derive"] }
flate2 = "=1.0.35"
jsonschema = { version = "0.18.3", default-features = false }
authoring_core = { path = "../authoring_core" }
writer_core = { path = "../writer_core" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
tar = "=0.4.42"
url = "2.5.2"
zip = { version = "2.2.0", default-features = false, features = ["deflate"] }
```

```rust
// contract_tools/src/gates.rs
pub fn run_all(manifest_path: impl AsRef<Path>) -> anyhow::Result<()> {
    let manifest_path = manifest_path.as_ref();

    load_manifest(manifest_path)?;
    schema::run_schema_gates(manifest_path)?;
    semantics::run_semantics_gates(manifest_path)?;
    policies::run_policy_gates(manifest_path)?;
    registry::run_registry_gates(manifest_path)?;
    fixtures::run_fixture_gates(manifest_path)?;
    compat_oracle::run_compat_oracle_gates(manifest_path)?;
    versioning::run_versioning_gates(manifest_path)?;

    Ok(())
}
```

```rust
// contract_tools/src/lib.rs
pub mod build_cmd;
pub mod compat_oracle;
pub mod diff_cmd;
pub mod fixtures;
pub mod gates;
pub mod inspect_cmd;
pub mod manifest;
pub mod normalize_cmd;
pub mod package;
pub mod policies;
pub mod registry;
pub mod schema;
pub mod semantics;
pub mod summary;
pub mod versioning;
```

```md
<!-- README.md (Phase 3 excerpt) -->
`build --output contract-json` writes the schema-governed Phase 3 `package-build-result`.
`inspect --output contract-json` writes the stable observation model used by golden regressions.
`diff --output contract-json` writes the stable comparison report used by verification policies.
These three `contract-json` surfaces are stable machine interfaces, not informal debug output.

For Phase 3 readiness, capture the commands and evidence in `docs/superpowers/checklists/phase-3-exit-evidence.md`.
```

```md
<!-- docs/superpowers/checklists/phase-3-exit-evidence.md -->
# Phase 3 Exit Evidence

- [ ] `cargo test -p authoring_core -v`
- [ ] `cargo test -p writer_core -v`
- [ ] `cargo test -p contract_tools -v`
- [ ] `cargo run -p contract_tools -- verify --manifest "$(pwd)/contracts/manifest.yaml"`
- [ ] `cargo run -p contract_tools -- build --manifest "$(pwd)/contracts/manifest.yaml" --input "$(pwd)/contracts/fixtures/phase3/inputs/basic-normalized-ir.json" --writer-policy default --build-context default --artifacts-dir "$(pwd)/tmp/phase3/basic-minimal" --output contract-json`
- [ ] `cargo run -p contract_tools -- inspect --staging "$(pwd)/tmp/phase3/basic-minimal/staging/manifest.json" --output contract-json`
- [ ] `cargo run -p contract_tools -- inspect --apkg "$(pwd)/tmp/phase3/basic-minimal/package.apkg" --output contract-json`
- [ ] `cargo run -p contract_tools -- diff --left "$(pwd)/contracts/fixtures/phase3/expected/basic.inspect.json" --right "$(pwd)/contracts/fixtures/phase3/expected/basic.inspect.json" --output contract-json`
- [ ] package-build-result, inspect-report, and diff-report outputs are schema-valid and stable under repeated runs for the same fixture inputs
```

- [ ] **Step 4: Run the compatibility-oracle and full contract_tools tests**

Run: `cargo test -p contract_tools --test compat_oracle_tests --test cli_tests --test fixture_gate_tests -v`
Expected: PASS, and the oracle checks should validate the supported package layout.

- [ ] **Step 5: Commit**

```bash
git add contract_tools/Cargo.toml contract_tools/src/compat_oracle.rs contract_tools/tests/compat_oracle_tests.rs contract_tools/src/gates.rs contract_tools/src/lib.rs README.md docs/superpowers/checklists/phase-3-exit-evidence.md
git commit -m "feat: add phase3 compatibility oracle and operator docs"
```
