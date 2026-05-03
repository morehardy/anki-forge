# Production Media Pipeline Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace inline normalized media payloads with CAS-backed media objects, export filename bindings, reference indexing, and writer-side integrity checks.

**Architecture:** Authoring input accepts path or small inline byte sources, normalization ingests them into a BLAKE3-addressed store, and normalized IR carries only object/binding/reference metadata. Writer stages and packages media by reading original bytes from CAS, validating invariants, and reporting only writer-level CAS/invariant/APKG failures.

**Tech Stack:** Rust workspace, `serde`, `serde_json`, JSON Schema, `base64`, `blake3`, `sha1`, `hex`, `html-escape`, `zip`, `zstd`, `rusqlite`, Cargo test runner.

---

## Scope Check

This is one subsystem plan: production media ingestion and materialization. It touches contracts, normalization, writer, inspect, and high-level Rust facade code because those pieces currently share the inline `data_base64` contract. Node/Python stream APIs, remote media fetching, media transforms, automatic filename rewriting, and payload-level APKG dedupe remain out of scope.

## File Structure Map

- Modify: `contracts/schema/authoring-ir.schema.json` - authoring media source schema with path and inline bytes variants.
- Modify: `contracts/schema/normalized-ir.schema.json` - media object, binding, and reference schema; remove normalized payload schema.
- Modify: `contract_tools/tests/schema_gate_tests.rs` - schema acceptance/rejection and invariant gate tests.
- Modify: `contracts/semantics/normalization.md` - path base, CAS ingest, MIME, reference, and diagnostics semantics.
- Modify: `contracts/semantics/build.md` - CAS-backed writer and staging/APKG media semantics.
- Modify: `authoring_core/Cargo.toml` - add media ingest dependencies.
- Modify: `authoring_core/src/lib.rs` - export media model, options, and normalize entry points.
- Modify: `authoring_core/src/model.rs` - replace media structs and normalized IR fields.
- Create: `authoring_core/src/media.rs` - media options, policy, CAS write, sniffing, filename/path validation, object/binding construction.
- Create: `authoring_core/src/media_refs.rs` - static media reference extraction and classification.
- Modify: `authoring_core/src/normalize.rs` - call media ingest/reference validation and output new normalized fields.
- Create: `authoring_core/tests/media_ingest_tests.rs` - CAS, source path, inline bytes, MIME, duplicate id/filename, and policy tests.
- Create: `authoring_core/tests/media_refs_tests.rs` - sound/HTML/CSS reference parsing and unsafe/skipped/missing behavior.
- Modify: `writer_core/src/staging.rs` - validate normalized media invariants and materialize staging media from CAS by copy.
- Modify: `writer_core/src/build.rs` - pass CAS-aware target through staging/APKG.
- Modify: `writer_core/src/apkg.rs` - read media bytes from CAS, not `staging/media`.
- Modify: `writer_core/src/inspect.rs` - staging media observations include manifest-declared Forge metadata; APKG observations stay APKG-only.
- Modify: `writer_core/tests/build_tests.rs` - CAS-backed staging/APKG and writer invariant tests.
- Modify: `writer_core/tests/inspect_tests.rs` - staging/APKG inspect media observation tests.
- Modify: `anki_forge/src/deck/media.rs` - keep ergonomic file/bytes/reader registration while lowering to new authoring media sources.
- Modify: `anki_forge/src/deck/lowering.rs` - emit new authoring media declarations.
- Modify: `anki_forge/src/deck/export.rs` - call `normalize_with_options` with package-local media store.
- Modify: `anki_forge/src/product/assets.rs` and `anki_forge/src/product/lowering.rs` - lower bundled font/static assets as ordinary authoring media sources.
- Modify: `anki_forge/src/runtime/normalize.rs` - normalize path input with input-parent `base_dir` and sibling `.anki-forge-media` store.
- Modify: `anki_forge/src/runtime/build.rs` - build normalized path input with an explicit media store location.
- Modify: `anki_forge/src/lib.rs` - re-export new media types and normalize options.

## Task 1: Contract Schema Gates

**Files:**
- Modify: `contract_tools/tests/schema_gate_tests.rs`
- Modify: `contracts/schema/authoring-ir.schema.json`
- Modify: `contracts/schema/normalized-ir.schema.json`
- Modify: `contracts/semantics/normalization.md`
- Modify: `contracts/semantics/build.md`

- [ ] **Step 1: Add failing authoring schema tests**

Add these tests to `contract_tools/tests/schema_gate_tests.rs` near the existing authoring media schema test:

```rust
#[test]
fn authoring_ir_schema_accepts_path_and_inline_media_sources() {
    let manifest = load_manifest(contract_manifest_path()).unwrap();
    let schema =
        load_schema(resolve_asset_path(&manifest, "authoring_ir_schema").unwrap()).unwrap();
    let value = json!({
        "kind": "authoring-ir",
        "schema_version": "0.1.0",
        "metadata": { "document_id": "demo-doc" },
        "notetypes": [],
        "notes": [],
        "media": [
            {
                "id": "media:heart",
                "desired_filename": "heart.png",
                "source": { "kind": "path", "path": "assets/heart.png" },
                "declared_mime": "image/png"
            },
            {
                "id": "media:tiny",
                "desired_filename": "tiny.txt",
                "source": { "kind": "inline_bytes", "data_base64": "aGk=" }
            }
        ]
    });

    assert!(validate_value(&schema, &value).is_ok());
}

#[test]
fn authoring_ir_schema_rejects_legacy_inline_media_payloads() {
    let manifest = load_manifest(contract_manifest_path()).unwrap();
    let schema =
        load_schema(resolve_asset_path(&manifest, "authoring_ir_schema").unwrap()).unwrap();
    let value = json!({
        "kind": "authoring-ir",
        "schema_version": "0.1.0",
        "metadata": { "document_id": "demo-doc" },
        "notetypes": [],
        "notes": [],
        "media": [
            {
                "filename": "sample.jpg",
                "mime": "image/jpeg",
                "data_base64": "MQ=="
            }
        ]
    });

    assert!(validate_value(&schema, &value).is_err());
}
```

- [ ] **Step 2: Add failing normalized schema tests**

Add these tests to `contract_tools/tests/schema_gate_tests.rs` near `normalization_result_schema_allows_null_comparison_context_without_merge_risk_report`:

```rust
#[test]
fn normalized_ir_schema_accepts_media_objects_bindings_and_reference_states() {
    let manifest = load_manifest(contract_manifest_path()).unwrap();
    let schema =
        load_schema(resolve_asset_path(&manifest, "normalized_ir_schema").unwrap()).unwrap();
    let value = writer_ready_normalized_ir_value_with_media_v2();

    assert!(validate_value(&schema, &value).is_ok());
}

#[test]
fn normalized_ir_schema_rejects_media_payload_fields() {
    let manifest = load_manifest(contract_manifest_path()).unwrap();
    let schema =
        load_schema(resolve_asset_path(&manifest, "normalized_ir_schema").unwrap()).unwrap();
    let mut value = writer_ready_normalized_ir_value_with_media_v2();
    value.as_object_mut().unwrap().insert(
        "media".into(),
        json!([{ "filename": "sample.jpg", "mime": "image/jpeg", "data_base64": "MQ==" }]),
    );

    assert!(validate_value(&schema, &value).is_err());
}

#[test]
fn normalized_ir_schema_requires_reference_state_fields() {
    let manifest = load_manifest(contract_manifest_path()).unwrap();
    let schema =
        load_schema(resolve_asset_path(&manifest, "normalized_ir_schema").unwrap()).unwrap();
    let mut value = writer_ready_normalized_ir_value_with_media_v2();
    value["media_references"][0]
        .as_object_mut()
        .unwrap()
        .remove("media_id");

    assert!(validate_value(&schema, &value).is_err());
}

fn writer_ready_normalized_ir_value_with_media_v2() -> Value {
    let mut value = writer_ready_normalized_ir_value();
    let object_id = "obj:blake3:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    value.as_object_mut().unwrap().remove("media");
    value.as_object_mut().unwrap().insert(
        "media_objects".into(),
        json!([
            {
                "id": object_id,
                "object_ref": "media://blake3/0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
                "blake3": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
                "sha1": "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d",
                "size_bytes": 5,
                "mime": "text/plain"
            }
        ]),
    );
    value.as_object_mut().unwrap().insert(
        "media_bindings".into(),
        json!([
            {
                "id": "media:hello",
                "export_filename": "hello.txt",
                "object_id": object_id
            }
        ]),
    );
    value.as_object_mut().unwrap().insert(
        "media_references".into(),
        json!([
            {
                "owner_kind": "note",
                "owner_id": "note-1",
                "location_kind": "field",
                "location_name": "Front",
                "raw_ref": "hello.txt",
                "ref_kind": "html_src",
                "resolution_status": "resolved",
                "media_id": "media:hello"
            },
            {
                "owner_kind": "note",
                "owner_id": "note-1",
                "location_kind": "field",
                "location_name": "Back",
                "raw_ref": "missing.png",
                "ref_kind": "html_src",
                "resolution_status": "missing"
            },
            {
                "owner_kind": "note",
                "owner_id": "note-1",
                "location_kind": "field",
                "location_name": "Back",
                "raw_ref": "https://example.com/x.png",
                "ref_kind": "html_src",
                "resolution_status": "skipped",
                "skip_reason": "external-url"
            }
        ]),
    );
    value
}
```

- [ ] **Step 3: Run schema tests to verify failure**

Run:

```bash
cargo test -p contract_tools --test schema_gate_tests authoring_ir_schema_accepts_path_and_inline_media_sources -- --nocapture
cargo test -p contract_tools --test schema_gate_tests normalized_ir_schema_accepts_media_objects_bindings_and_reference_states -- --nocapture
```

Expected: FAIL because both schemas still require legacy `filename/mime/data_base64` media.

- [ ] **Step 4: Update authoring schema media definition**

In `contracts/schema/authoring-ir.schema.json`, replace the `authoring_media` definition with:

```json
"authoring_media": {
  "type": "object",
  "required": ["id", "desired_filename", "source"],
  "additionalProperties": false,
  "properties": {
    "id": {
      "type": "string",
      "minLength": 1
    },
    "desired_filename": {
      "type": "string",
      "minLength": 1
    },
    "source": {
      "oneOf": [
        { "$ref": "#/$defs/authoring_media_path_source" },
        { "$ref": "#/$defs/authoring_media_inline_bytes_source" }
      ]
    },
    "declared_mime": {
      "type": "string",
      "minLength": 1
    }
  }
},
"authoring_media_path_source": {
  "type": "object",
  "required": ["kind", "path"],
  "additionalProperties": false,
  "properties": {
    "kind": { "const": "path" },
    "path": { "type": "string", "minLength": 1 }
  }
},
"authoring_media_inline_bytes_source": {
  "type": "object",
  "required": ["kind", "data_base64"],
  "additionalProperties": false,
  "properties": {
    "kind": { "const": "inline_bytes" },
    "data_base64": { "type": "string", "minLength": 1 }
  }
}
```

- [ ] **Step 5: Update normalized schema media definitions**

In `contracts/schema/normalized-ir.schema.json`:

1. Replace the root `required` list item `"media"` with `"media_objects"`, `"media_bindings"`, and `"media_references"`.
2. Replace the root `media` property with:

```json
"media_objects": {
  "type": "array",
  "items": { "$ref": "#/$defs/media_object" }
},
"media_bindings": {
  "type": "array",
  "items": { "$ref": "#/$defs/media_binding" }
},
"media_references": {
  "type": "array",
  "items": { "$ref": "#/$defs/media_reference" }
}
```

3. Replace the `normalized_media` definition with these definitions:

```json
"media_object": {
  "type": "object",
  "required": ["id", "object_ref", "blake3", "sha1", "size_bytes", "mime"],
  "additionalProperties": false,
  "properties": {
    "id": { "type": "string", "pattern": "^obj:blake3:[0-9a-f]{64}$" },
    "object_ref": { "type": "string", "pattern": "^media://blake3/[0-9a-f]{64}$" },
    "blake3": { "type": "string", "pattern": "^[0-9a-f]{64}$" },
    "sha1": { "type": "string", "pattern": "^[0-9a-f]{40}$" },
    "size_bytes": { "type": "integer", "minimum": 0 },
    "mime": { "type": "string", "minLength": 1 }
  }
},
"media_binding": {
  "type": "object",
  "required": ["id", "export_filename", "object_id"],
  "additionalProperties": false,
  "properties": {
    "id": { "type": "string", "minLength": 1 },
    "export_filename": { "type": "string", "minLength": 1 },
    "object_id": { "type": "string", "pattern": "^obj:blake3:[0-9a-f]{64}$" }
  }
},
"media_reference": {
  "oneOf": [
    { "$ref": "#/$defs/resolved_media_reference" },
    { "$ref": "#/$defs/missing_media_reference" },
    { "$ref": "#/$defs/skipped_media_reference" }
  ]
},
"resolved_media_reference": {
  "type": "object",
  "required": ["owner_kind", "owner_id", "location_kind", "location_name", "raw_ref", "ref_kind", "resolution_status", "media_id"],
  "additionalProperties": false,
  "properties": {
    "owner_kind": { "type": "string", "minLength": 1 },
    "owner_id": { "type": "string", "minLength": 1 },
    "location_kind": { "type": "string", "minLength": 1 },
    "location_name": { "type": "string", "minLength": 1 },
    "raw_ref": { "type": "string", "minLength": 1 },
    "ref_kind": { "type": "string", "minLength": 1 },
    "resolution_status": { "const": "resolved" },
    "media_id": { "type": "string", "minLength": 1 }
  }
},
"missing_media_reference": {
  "type": "object",
  "required": ["owner_kind", "owner_id", "location_kind", "location_name", "raw_ref", "ref_kind", "resolution_status"],
  "additionalProperties": false,
  "properties": {
    "owner_kind": { "type": "string", "minLength": 1 },
    "owner_id": { "type": "string", "minLength": 1 },
    "location_kind": { "type": "string", "minLength": 1 },
    "location_name": { "type": "string", "minLength": 1 },
    "raw_ref": { "type": "string", "minLength": 1 },
    "ref_kind": { "type": "string", "minLength": 1 },
    "resolution_status": { "const": "missing" }
  }
},
"skipped_media_reference": {
  "type": "object",
  "required": ["owner_kind", "owner_id", "location_kind", "location_name", "raw_ref", "ref_kind", "resolution_status", "skip_reason"],
  "additionalProperties": false,
  "properties": {
    "owner_kind": { "type": "string", "minLength": 1 },
    "owner_id": { "type": "string", "minLength": 1 },
    "location_kind": { "type": "string", "minLength": 1 },
    "location_name": { "type": "string", "minLength": 1 },
    "raw_ref": { "type": "string", "minLength": 1 },
    "ref_kind": { "type": "string", "minLength": 1 },
    "resolution_status": { "const": "skipped" },
    "skip_reason": { "type": "string", "minLength": 1 }
  }
}
```

- [ ] **Step 6: Update semantics docs**

In `contracts/semantics/normalization.md`, replace the paragraph beginning `Authoring notes may reference media entries inline` with:

```markdown
Authoring media declarations may reference relative local paths or small inline
byte payloads. Path sources are resolved against explicit normalization options,
not the process working directory. Inline byte payloads are authoring-only and
must be ingested into the media store before normalized IR is produced.

Normalized media is represented by `media_objects`, `media_bindings`, and
`media_references`. It must not contain `data_base64` or inline byte payloads.
`media_objects` describe CAS-backed content, `media_bindings` describe APKG
export filenames, and `media_references` describe resolved, missing, or skipped
static references discovered in notes/templates.
```

In `contracts/semantics/build.md`, add this paragraph after the APKG v3 media paragraph:

```markdown
Writer media payloads are read from the content-addressed media store. Staging
media files are copy/reflink-derived inspect artifacts and are not the writer's
source of truth. The writer validates normalized media invariants and CAS object
integrity, but media semantics such as unused bindings, missing references,
unsafe references, and MIME mismatch are normalization responsibilities.
```

- [ ] **Step 7: Run schema tests**

Run:

```bash
cargo test -p contract_tools --test schema_gate_tests -v
```

Expected: PASS.

- [ ] **Step 8: Commit contract schema changes**

Run:

```bash
git add contracts/schema/authoring-ir.schema.json contracts/schema/normalized-ir.schema.json contracts/semantics/normalization.md contracts/semantics/build.md contract_tools/tests/schema_gate_tests.rs
git commit -m "feat: update media schemas for cas-backed normalized ir"
```

## Task 2: Core Media Model and Stable Ordering

**Files:**
- Modify: `authoring_core/Cargo.toml`
- Modify: `authoring_core/src/lib.rs`
- Modify: `authoring_core/src/model.rs`
- Create: `authoring_core/src/media.rs`
- Test: `authoring_core/tests/media_ingest_tests.rs`

- [ ] **Step 1: Add failing model serialization tests**

Create `authoring_core/tests/media_ingest_tests.rs` with:

```rust
use authoring_core::{
    AuthoringMedia, AuthoringMediaSource, DiagnosticBehavior, MediaBinding, MediaObject,
    MediaPolicy, MediaReference, MediaReferenceResolution, NormalizeOptions, NormalizedIr,
};
use std::path::PathBuf;

#[test]
fn authoring_media_path_source_serializes_without_payload() {
    let media = AuthoringMedia {
        id: "media:heart".into(),
        desired_filename: "heart.png".into(),
        source: AuthoringMediaSource::Path {
            path: "assets/heart.png".into(),
        },
        declared_mime: Some("image/png".into()),
    };

    let json = serde_json::to_value(media).unwrap();

    assert_eq!(json["id"], "media:heart");
    assert_eq!(json["desired_filename"], "heart.png");
    assert_eq!(json["source"]["kind"], "path");
    assert_eq!(json["source"]["path"], "assets/heart.png");
    assert!(json.get("data_base64").is_none());
}

#[test]
fn normalized_ir_serializes_media_objects_bindings_and_references() {
    let hash = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let normalized = NormalizedIr {
        kind: "normalized-ir".into(),
        schema_version: "0.1.0".into(),
        document_id: "doc".into(),
        resolved_identity: "det:doc".into(),
        notetypes: vec![],
        notes: vec![],
        media_objects: vec![MediaObject {
            id: format!("obj:blake3:{hash}"),
            object_ref: format!("media://blake3/{hash}"),
            blake3: hash.into(),
            sha1: "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d".into(),
            size_bytes: 5,
            mime: "text/plain".into(),
        }],
        media_bindings: vec![MediaBinding {
            id: "media:hello".into(),
            export_filename: "hello.txt".into(),
            object_id: format!("obj:blake3:{hash}"),
        }],
        media_references: vec![MediaReference {
            owner_kind: "note".into(),
            owner_id: "note-1".into(),
            location_kind: "field".into(),
            location_name: "Front".into(),
            raw_ref: "hello.txt".into(),
            ref_kind: "html_src".into(),
            resolution: MediaReferenceResolution::Resolved {
                media_id: "media:hello".into(),
            },
        }],
    };

    let json = serde_json::to_value(normalized).unwrap();

    assert!(json.get("media").is_none());
    assert!(json.get("media_objects").is_some());
    assert!(json.get("media_bindings").is_some());
    assert!(json.get("media_references").is_some());
}

#[test]
fn normalize_options_default_policy_is_explicit() {
    let options = NormalizeOptions {
        base_dir: PathBuf::from("/tmp/input"),
        media_store_dir: PathBuf::from("/tmp/store"),
        media_policy: MediaPolicy {
            inline_bytes_max: 64 * 1024,
            max_media_object_bytes: None,
            max_total_media_bytes: None,
            unknown_mime_behavior: DiagnosticBehavior::Warning,
            unused_binding_behavior: DiagnosticBehavior::Warning,
        },
    };

    assert_eq!(options.media_policy.inline_bytes_max, 65536);
}
```

- [ ] **Step 2: Run model tests to verify failure**

Run:

```bash
cargo test -p authoring_core --test media_ingest_tests -v
```

Expected: FAIL because new media types do not exist.

- [ ] **Step 3: Add dependencies**

In `authoring_core/Cargo.toml`, add these dependencies:

```toml
base64 = "0.22"
blake3 = "1"
hex = "0.4"
html-escape = "0.2"
sha1 = "0.10"
```

- [ ] **Step 4: Create media model module**

Create `authoring_core/src/media.rs` with:

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AuthoringMediaSource {
    Path { path: String },
    InlineBytes { data_base64: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticBehavior {
    Ignore,
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizeOptions {
    pub base_dir: PathBuf,
    pub media_store_dir: PathBuf,
    pub media_policy: MediaPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaPolicy {
    pub inline_bytes_max: usize,
    pub max_media_object_bytes: Option<u64>,
    pub max_total_media_bytes: Option<u64>,
    pub unknown_mime_behavior: DiagnosticBehavior,
    pub unused_binding_behavior: DiagnosticBehavior,
}

impl MediaPolicy {
    pub fn default_strict() -> Self {
        Self {
            inline_bytes_max: 64 * 1024,
            max_media_object_bytes: None,
            max_total_media_bytes: None,
            unknown_mime_behavior: DiagnosticBehavior::Warning,
            unused_binding_behavior: DiagnosticBehavior::Warning,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaObject {
    pub id: String,
    pub object_ref: String,
    pub blake3: String,
    pub sha1: String,
    pub size_bytes: u64,
    pub mime: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaBinding {
    pub id: String,
    pub export_filename: String,
    pub object_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaReference {
    pub owner_kind: String,
    pub owner_id: String,
    pub location_kind: String,
    pub location_name: String,
    pub raw_ref: String,
    pub ref_kind: String,
    #[serde(flatten)]
    pub resolution: MediaReferenceResolution,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "resolution_status", rename_all = "snake_case")]
pub enum MediaReferenceResolution {
    Resolved { media_id: String },
    Missing,
    Skipped { skip_reason: String },
}

pub fn media_object_id(blake3_hex: &str) -> String {
    format!("obj:blake3:{blake3_hex}")
}

pub fn media_object_ref(blake3_hex: &str) -> String {
    format!("media://blake3/{blake3_hex}")
}

pub fn sort_media_objects(objects: &mut [MediaObject]) {
    objects.sort_by(|left, right| left.id.as_bytes().cmp(right.id.as_bytes()));
}

pub fn sort_media_bindings(bindings: &mut [MediaBinding]) {
    bindings.sort_by(|left, right| {
        left.export_filename
            .as_bytes()
            .cmp(right.export_filename.as_bytes())
            .then_with(|| left.id.as_bytes().cmp(right.id.as_bytes()))
    });
}

pub fn sort_media_references(references: &mut [MediaReference]) {
    references.sort_by(|left, right| media_reference_sort_key(left).cmp(&media_reference_sort_key(right)));
}

fn media_reference_sort_key(reference: &MediaReference) -> Vec<Vec<u8>> {
    let (media_id, skip_reason) = match &reference.resolution {
        MediaReferenceResolution::Resolved { media_id } => (media_id.as_str(), ""),
        MediaReferenceResolution::Missing => ("", ""),
        MediaReferenceResolution::Skipped { skip_reason } => ("", skip_reason.as_str()),
    };
    vec![
        reference.owner_kind.as_bytes().to_vec(),
        reference.owner_id.as_bytes().to_vec(),
        reference.location_kind.as_bytes().to_vec(),
        reference.location_name.as_bytes().to_vec(),
        reference.raw_ref.as_bytes().to_vec(),
        reference.ref_kind.as_bytes().to_vec(),
        serde_json::to_value(&reference.resolution)
            .ok()
            .and_then(|value| value["resolution_status"].as_str().map(str::to_owned))
            .unwrap_or_default()
            .into_bytes(),
        media_id.as_bytes().to_vec(),
        skip_reason.as_bytes().to_vec(),
    ]
}
```

- [ ] **Step 5: Update core model structs**

In `authoring_core/src/model.rs`:

1. Import the media types:

```rust
use crate::media::{AuthoringMediaSource, MediaBinding, MediaObject, MediaReference};
```

2. Replace `AuthoringMedia` with:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthoringMedia {
    pub id: String,
    pub desired_filename: String,
    pub source: AuthoringMediaSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub declared_mime: Option<String>,
}
```

3. Replace the `media: Vec<NormalizedMedia>` field on `NormalizedIr` with:

```rust
pub media_objects: Vec<MediaObject>,
pub media_bindings: Vec<MediaBinding>,
pub media_references: Vec<MediaReference>,
```

4. Delete the `NormalizedMedia` struct.

- [ ] **Step 6: Export media types**

In `authoring_core/src/lib.rs`, add:

```rust
pub mod media;
pub use media::{
    media_object_id, media_object_ref, sort_media_bindings, sort_media_objects,
    sort_media_references, AuthoringMediaSource, DiagnosticBehavior, MediaBinding, MediaObject,
    MediaPolicy, MediaReference, MediaReferenceResolution, NormalizeOptions,
};
```

- [ ] **Step 7: Run model tests**

Run:

```bash
cargo test -p authoring_core --test media_ingest_tests -v
```

Expected: PASS for the three model serialization tests. Existing workspace tests may still fail until normalization and writer tasks remove old `NormalizedMedia` references.

- [ ] **Step 8: Commit core media model**

Run:

```bash
git add authoring_core/Cargo.toml authoring_core/src/lib.rs authoring_core/src/model.rs authoring_core/src/media.rs authoring_core/tests/media_ingest_tests.rs
git commit -m "feat: add cas media model types"
```

## Task 3: CAS Ingest, Path Safety, and MIME Sniffing

**Files:**
- Modify: `authoring_core/src/media.rs`
- Modify: `authoring_core/tests/media_ingest_tests.rs`

- [ ] **Step 1: Add failing ingest tests**

Append these tests to `authoring_core/tests/media_ingest_tests.rs`:

```rust
use authoring_core::{ingest_authoring_media, MediaIngestDiagnostic};
use std::{fs, os::unix::fs as unix_fs};

#[test]
fn ingest_path_source_writes_original_bytes_to_cas() {
    let root = unique_test_root("path-source");
    let base_dir = root.join("input");
    let store_dir = root.join("store");
    fs::create_dir_all(base_dir.join("assets")).unwrap();
    fs::write(base_dir.join("assets/hello.txt"), b"hello").unwrap();
    let options = test_options(&base_dir, &store_dir);
    let media = vec![AuthoringMedia {
        id: "media:hello".into(),
        desired_filename: "hello.txt".into(),
        source: AuthoringMediaSource::Path {
            path: "assets/hello.txt".into(),
        },
        declared_mime: Some("text/plain".into()),
    }];

    let result = ingest_authoring_media(&media, &options).unwrap();

    assert!(result.diagnostics.is_empty());
    assert_eq!(result.objects.len(), 1);
    assert_eq!(result.bindings[0].export_filename, "hello.txt");
    assert_eq!(fs::read(result.object_path(&result.objects[0]).unwrap()).unwrap(), b"hello");
}

#[test]
fn ingest_rejects_symlink_escape() {
    let root = unique_test_root("symlink-escape");
    let base_dir = root.join("input");
    let outside_dir = root.join("outside");
    fs::create_dir_all(base_dir.join("assets")).unwrap();
    fs::create_dir_all(&outside_dir).unwrap();
    fs::write(outside_dir.join("secret.txt"), b"secret").unwrap();
    unix_fs::symlink(outside_dir.join("secret.txt"), base_dir.join("assets/link.txt")).unwrap();
    let options = test_options(&base_dir, &root.join("store"));
    let media = vec![AuthoringMedia {
        id: "media:bad".into(),
        desired_filename: "bad.txt".into(),
        source: AuthoringMediaSource::Path {
            path: "assets/link.txt".into(),
        },
        declared_mime: None,
    }];

    let err = ingest_authoring_media(&media, &options).unwrap_err();

    assert!(err.diagnostics.iter().any(|item| item.code == "MEDIA.UNSAFE_SOURCE_PATH"));
}

#[test]
fn ingest_rejects_inline_base64_decode_failure() {
    let root = unique_test_root("inline-decode");
    let options = test_options(&root, &root.join("store"));
    let media = vec![AuthoringMedia {
        id: "media:inline".into(),
        desired_filename: "inline.txt".into(),
        source: AuthoringMediaSource::InlineBytes {
            data_base64: "%%%".into(),
        },
        declared_mime: None,
    }];

    let err = ingest_authoring_media(&media, &options).unwrap_err();

    assert!(err.diagnostics.iter().any(|item| item.code == "MEDIA.INLINE_BASE64_DECODE_FAILED"));
}

fn test_options(base_dir: &std::path::Path, media_store_dir: &std::path::Path) -> NormalizeOptions {
    NormalizeOptions {
        base_dir: base_dir.to_path_buf(),
        media_store_dir: media_store_dir.to_path_buf(),
        media_policy: MediaPolicy::default_strict(),
    }
}

fn unique_test_root(label: &str) -> std::path::PathBuf {
    let mut root = std::env::temp_dir();
    root.push(format!(
        "anki-forge-media-{label}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&root).unwrap();
    root
}
```

On non-Unix platforms, guard the symlink test with:

```rust
#[cfg(unix)]
```

- [ ] **Step 2: Run ingest tests to verify failure**

Run:

```bash
cargo test -p authoring_core --test media_ingest_tests ingest_path_source_writes_original_bytes_to_cas -v
```

Expected: FAIL because `ingest_authoring_media` does not exist.

- [ ] **Step 3: Implement ingest result and diagnostics**

Add these types and functions to `authoring_core/src/media.rs`:

```rust
use base64::Engine as _;
use sha1::{Digest, Sha1};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaIngestDiagnostic {
    pub level: String,
    pub code: String,
    pub summary: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MediaIngestError {
    pub diagnostics: Vec<MediaIngestDiagnostic>,
}

#[derive(Debug, Clone)]
pub struct MediaIngestResult {
    pub objects: Vec<MediaObject>,
    pub bindings: Vec<MediaBinding>,
    pub diagnostics: Vec<MediaIngestDiagnostic>,
    pub media_store_dir: PathBuf,
}

impl MediaIngestResult {
    pub fn object_path(&self, object: &MediaObject) -> Option<PathBuf> {
        object_store_path(&self.media_store_dir, &object.blake3).ok()
    }
}

pub fn ingest_authoring_media(
    media: &[crate::model::AuthoringMedia],
    options: &NormalizeOptions,
) -> Result<MediaIngestResult, MediaIngestError> {
    let mut diagnostics = Vec::new();
    let mut objects_by_id = BTreeMap::<String, MediaObject>::new();
    let mut bindings = Vec::<MediaBinding>::new();
    let mut seen_media_ids = BTreeSet::<String>::new();
    let mut filename_to_object = BTreeMap::<String, String>::new();

    for item in media {
        if !seen_media_ids.insert(item.id.clone()) {
            diagnostics.push(error("MEDIA.DUPLICATE_MEDIA_ID", format!("duplicate media id {}", item.id), None));
            continue;
        }
        if let Err(message) = validate_bare_filename(&item.desired_filename) {
            diagnostics.push(error("MEDIA.UNSAFE_FILENAME", message, Some(item.desired_filename.clone())));
            continue;
        }

        let bytes = match read_authoring_media_bytes(item, options) {
            Ok(bytes) => bytes,
            Err(mut err) => {
                diagnostics.append(&mut err.diagnostics);
                continue;
            }
        };
        if bytes.len() > options.media_policy.inline_bytes_max
            && matches!(item.source, AuthoringMediaSource::InlineBytes { .. })
        {
            diagnostics.push(error(
                "MEDIA.INLINE_TOO_LARGE",
                format!("inline media {} exceeds inline_bytes_max", item.id),
                Some(item.id.clone()),
            ));
            continue;
        }

        let blake3_hex = blake3::hash(&bytes).to_hex().to_string();
        let sha1_hex = hex::encode(Sha1::digest(&bytes));
        let mime = effective_mime(item.declared_mime.as_deref(), &bytes);
        let object = MediaObject {
            id: media_object_id(&blake3_hex),
            object_ref: media_object_ref(&blake3_hex),
            blake3: blake3_hex.clone(),
            sha1: sha1_hex,
            size_bytes: bytes.len() as u64,
            mime,
        };
        if let Err(message) = write_cas_object(&options.media_store_dir, &blake3_hex, &bytes) {
            diagnostics.push(error("MEDIA.CAS_WRITE_FAILED", message, Some(item.id.clone())));
            continue;
        }

        if let Some(previous_object_id) =
            filename_to_object.insert(item.desired_filename.clone(), object.id.clone())
        {
            if previous_object_id != object.id {
                diagnostics.push(error(
                    "MEDIA.DUPLICATE_FILENAME_CONFLICT",
                    format!("export filename {} maps to multiple objects", item.desired_filename),
                    Some(item.desired_filename.clone()),
                ));
            } else {
                diagnostics.push(error(
                    "MEDIA.DUPLICATE_EXPORT_FILENAME",
                    format!("export filename {} is declared more than once", item.desired_filename),
                    Some(item.desired_filename.clone()),
                ));
            }
            continue;
        }

        objects_by_id.entry(object.id.clone()).or_insert(object.clone());
        bindings.push(MediaBinding {
            id: item.id.clone(),
            export_filename: item.desired_filename.clone(),
            object_id: object.id,
        });
    }

    if diagnostics.iter().any(|item| item.level == "error") {
        return Err(MediaIngestError { diagnostics });
    }

    let mut objects = objects_by_id.into_values().collect::<Vec<_>>();
    sort_media_objects(&mut objects);
    sort_media_bindings(&mut bindings);
    Ok(MediaIngestResult {
        objects,
        bindings,
        diagnostics,
        media_store_dir: options.media_store_dir.clone(),
    })
}
```

- [ ] **Step 4: Implement path, inline, CAS, filename, and MIME helpers**

Add these helpers to `authoring_core/src/media.rs` after `ingest_authoring_media`:

```rust
fn read_authoring_media_bytes(
    item: &crate::model::AuthoringMedia,
    options: &NormalizeOptions,
) -> Result<Vec<u8>, MediaIngestError> {
    match &item.source {
        AuthoringMediaSource::Path { path } => read_path_source(path, options),
        AuthoringMediaSource::InlineBytes { data_base64 } => {
            base64::engine::general_purpose::STANDARD
                .decode(data_base64.as_bytes())
                .map_err(|err| MediaIngestError {
                    diagnostics: vec![error(
                        "MEDIA.INLINE_BASE64_DECODE_FAILED",
                        format!("decode inline bytes for {}: {err}", item.id),
                        Some(item.id.clone()),
                    )],
                })
        }
    }
}

fn read_path_source(path: &str, options: &NormalizeOptions) -> Result<Vec<u8>, MediaIngestError> {
    let raw_path = Path::new(path);
    if raw_path.is_absolute() || has_parent_component(raw_path) {
        return Err(MediaIngestError {
            diagnostics: vec![error(
                "MEDIA.UNSAFE_SOURCE_PATH",
                format!("source.path must be relative and stay below base_dir: {path}"),
                Some(path.into()),
            )],
        });
    }
    let base = options.base_dir.canonicalize().map_err(|err| MediaIngestError {
        diagnostics: vec![error(
            "MEDIA.UNSAFE_SOURCE_PATH",
            format!("canonicalize base_dir {}: {err}", options.base_dir.display()),
            Some(options.base_dir.display().to_string()),
        )],
    })?;
    let candidate = options.base_dir.join(raw_path);
    let canonical = candidate.canonicalize().map_err(|err| MediaIngestError {
        diagnostics: vec![error(
            "MEDIA.SOURCE_MISSING",
            format!("read source.path {path}: {err}"),
            Some(path.into()),
        )],
    })?;
    if !canonical.starts_with(&base) {
        return Err(MediaIngestError {
            diagnostics: vec![error(
                "MEDIA.UNSAFE_SOURCE_PATH",
                format!("source.path escapes base_dir: {path}"),
                Some(path.into()),
            )],
        });
    }
    let metadata = fs::metadata(&canonical).map_err(|err| MediaIngestError {
        diagnostics: vec![error(
            "MEDIA.SOURCE_MISSING",
            format!("stat source.path {path}: {err}"),
            Some(path.into()),
        )],
    })?;
    if !metadata.is_file() {
        return Err(MediaIngestError {
            diagnostics: vec![error(
                "MEDIA.SOURCE_NOT_REGULAR_FILE",
                format!("source.path is not a regular file: {path}"),
                Some(path.into()),
            )],
        });
    }
    fs::read(&canonical).map_err(|err| MediaIngestError {
        diagnostics: vec![error(
            "MEDIA.SOURCE_MISSING",
            format!("read source.path {path}: {err}"),
            Some(path.into()),
        )],
    })
}

fn write_cas_object(store_dir: &Path, blake3_hex: &str, bytes: &[u8]) -> Result<(), String> {
    let final_path = object_store_path(store_dir, blake3_hex)?;
    if final_path.exists() {
        let existing = fs::read(&final_path)
            .map_err(|err| format!("read existing object {}: {err}", final_path.display()))?;
        if blake3::hash(&existing).to_hex().to_string() != blake3_hex || existing.len() != bytes.len()
        {
            return Err(format!("existing object integrity mismatch: {}", final_path.display()));
        }
        return Ok(());
    }
    let parent = final_path
        .parent()
        .ok_or_else(|| format!("object path has no parent: {}", final_path.display()))?;
    fs::create_dir_all(parent).map_err(|err| format!("create object dir {}: {err}", parent.display()))?;
    let temp_path = parent.join(format!(".{blake3_hex}.tmp"));
    {
        let mut file = fs::File::create(&temp_path)
            .map_err(|err| format!("create temp object {}: {err}", temp_path.display()))?;
        file.write_all(bytes)
            .map_err(|err| format!("write temp object {}: {err}", temp_path.display()))?;
        file.sync_all()
            .map_err(|err| format!("sync temp object {}: {err}", temp_path.display()))?;
    }
    fs::rename(&temp_path, &final_path).map_err(|err| {
        let _ = fs::remove_file(&temp_path);
        format!("rename temp object into {}: {err}", final_path.display())
    })?;
    Ok(())
}

pub fn object_store_path(store_dir: &Path, blake3_hex: &str) -> Result<PathBuf, String> {
    if blake3_hex.len() != 64 || !blake3_hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("invalid blake3 hex: {blake3_hex}"));
    }
    Ok(store_dir
        .join("objects")
        .join("blake3")
        .join(&blake3_hex[0..2])
        .join(&blake3_hex[2..4])
        .join(blake3_hex))
}

fn validate_bare_filename(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("media filename must not be empty".into());
    }
    if name.contains(['/', '\\']) || Path::new(name).is_absolute() || has_parent_component(Path::new(name)) {
        return Err(format!("media filename must be a bare filename: {name}"));
    }
    let mut components = Path::new(name).components();
    let is_bare = matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none();
    if is_bare {
        Ok(())
    } else {
        Err(format!("media filename must be a bare filename: {name}"))
    }
}

fn has_parent_component(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, Component::ParentDir | Component::CurDir))
}

fn effective_mime(declared_mime: Option<&str>, bytes: &[u8]) -> String {
    sniff_mime(bytes)
        .or_else(|| declared_mime.map(str::to_string))
        .unwrap_or_else(|| "application/octet-stream".into())
}

fn sniff_mime(bytes: &[u8]) -> Option<String> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some("image/png".into())
    } else if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        Some("image/jpeg".into())
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        Some("image/gif".into())
    } else if bytes.starts_with(b"ID3") {
        Some("audio/mpeg".into())
    } else if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WAVE") {
        Some("audio/wav".into())
    } else if bytes.iter().all(|byte| byte.is_ascii() && !byte.is_ascii_control() || *byte == b'\n' || *byte == b'\r' || *byte == b'\t') {
        Some("text/plain".into())
    } else {
        None
    }
}

fn error(
    code: impl Into<String>,
    summary: impl Into<String>,
    path: Option<String>,
) -> MediaIngestDiagnostic {
    MediaIngestDiagnostic {
        level: "error".into(),
        code: code.into(),
        summary: summary.into(),
        path,
    }
}
```

- [ ] **Step 5: Export ingest helpers**

In `authoring_core/src/lib.rs`, extend the `pub use media::{...}` list with:

```rust
ingest_authoring_media, object_store_path, MediaIngestDiagnostic, MediaIngestError,
MediaIngestResult,
```

- [ ] **Step 6: Run ingest tests**

Run:

```bash
cargo test -p authoring_core --test media_ingest_tests -v
```

Expected: PASS.

- [ ] **Step 7: Commit ingest implementation**

Run:

```bash
git add authoring_core/src/media.rs authoring_core/src/lib.rs authoring_core/tests/media_ingest_tests.rs
git commit -m "feat: ingest authoring media into cas"
```

## Task 4: Media Reference Scanner

**Files:**
- Create: `authoring_core/src/media_refs.rs`
- Modify: `authoring_core/src/lib.rs`
- Create: `authoring_core/tests/media_refs_tests.rs`

- [ ] **Step 1: Add failing reference scanner tests**

Create `authoring_core/tests/media_refs_tests.rs` with:

```rust
use authoring_core::{extract_media_reference_candidates, MediaReferenceCandidateKind};

#[test]
fn extracts_sound_html_object_and_css_refs() {
    let refs = extract_media_reference_candidates(
        "note",
        "note-1",
        "field",
        "Front",
        r#"[sound:hello.mp3]<img src="image.png"><object data="clip.webm"></object><span data-id="not-media"></span><style>.x{background:url(font.woff2?cache=1#frag)}</style>"#,
    );

    let raw_refs = refs.iter().map(|item| item.raw_ref.as_str()).collect::<Vec<_>>();
    assert!(raw_refs.contains(&"hello.mp3"));
    assert!(raw_refs.contains(&"image.png"));
    assert!(raw_refs.contains(&"clip.webm"));
    assert!(raw_refs.contains(&"font.woff2?cache=1#frag"));
    assert!(!raw_refs.contains(&"not-media"));
}

#[test]
fn classifies_external_and_data_uri_as_skipped() {
    let refs = extract_media_reference_candidates(
        "note",
        "note-1",
        "field",
        "Back",
        r#"<img src="https://example.com/a.png"><img src="data:image/png;base64,abc">"#,
    );

    assert_eq!(refs.len(), 2);
    assert!(refs.iter().all(|item| item.skip_reason.is_some()));
}

#[test]
fn percent_decodes_local_url_path_and_rejects_decoded_separators() {
    let refs = extract_media_reference_candidates(
        "note",
        "note-1",
        "field",
        "Front",
        r#"<img src="hello%20world.png?x=1"><img src="bad%2Fname.png">"#,
    );

    assert_eq!(refs[0].normalized_local_ref.as_deref(), Some("hello world.png"));
    assert_eq!(refs[1].unsafe_reason.as_deref(), Some("decoded-path-separator"));
}

#[test]
fn sound_refs_do_not_use_url_percent_decoding() {
    let refs = extract_media_reference_candidates(
        "note",
        "note-1",
        "field",
        "Front",
        "[sound:hello%20world.mp3]",
    );

    assert_eq!(refs[0].normalized_local_ref.as_deref(), Some("hello%20world.mp3"));
}
```

- [ ] **Step 2: Run reference tests to verify failure**

Run:

```bash
cargo test -p authoring_core --test media_refs_tests -v
```

Expected: FAIL because `extract_media_reference_candidates` does not exist.

- [ ] **Step 3: Implement reference candidate model and scanner**

Create `authoring_core/src/media_refs.rs` with:

```rust
use html_escape::decode_html_entities;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaReferenceCandidate {
    pub owner_kind: String,
    pub owner_id: String,
    pub location_kind: String,
    pub location_name: String,
    pub raw_ref: String,
    pub ref_kind: String,
    pub normalized_local_ref: Option<String>,
    pub skip_reason: Option<String>,
    pub unsafe_reason: Option<String>,
    pub kind: MediaReferenceCandidateKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaReferenceCandidateKind {
    Sound,
    HtmlSrc,
    HtmlObjectData,
    CssUrl,
}

pub fn extract_media_reference_candidates(
    owner_kind: &str,
    owner_id: &str,
    location_kind: &str,
    location_name: &str,
    text: &str,
) -> Vec<MediaReferenceCandidate> {
    let mut refs = Vec::new();
    refs.extend(extract_sound_refs(owner_kind, owner_id, location_kind, location_name, text));
    refs.extend(extract_html_src_refs(owner_kind, owner_id, location_kind, location_name, text));
    refs.extend(extract_html_object_data_refs(owner_kind, owner_id, location_kind, location_name, text));
    refs.extend(extract_css_url_refs(owner_kind, owner_id, location_kind, location_name, text));
    refs
}
```

Add the parsing helpers below it:

```rust
fn extract_sound_refs(owner_kind: &str, owner_id: &str, location_kind: &str, location_name: &str, text: &str) -> Vec<MediaReferenceCandidate> {
    let mut refs = Vec::new();
    let mut remaining = text;
    while let Some(start) = remaining.find("[sound:") {
        let after = &remaining[start + "[sound:".len()..];
        let Some(end) = after.find(']') else { break };
        let raw = decode_html_entities(&after[..end]).into_owned();
        refs.push(local_candidate(owner_kind, owner_id, location_kind, location_name, &raw, "sound", MediaReferenceCandidateKind::Sound, false));
        remaining = &after[end + 1..];
    }
    refs
}

fn extract_html_src_refs(owner_kind: &str, owner_id: &str, location_kind: &str, location_name: &str, text: &str) -> Vec<MediaReferenceCandidate> {
    extract_html_attribute_refs(owner_kind, owner_id, location_kind, location_name, text, "src", "html_src", MediaReferenceCandidateKind::HtmlSrc)
}

fn extract_html_object_data_refs(owner_kind: &str, owner_id: &str, location_kind: &str, location_name: &str, text: &str) -> Vec<MediaReferenceCandidate> {
    let mut refs = Vec::new();
    let mut remaining = text;
    while let Some(object_start) = remaining.to_ascii_lowercase().find("<object") {
        let object = &remaining[object_start..];
        let Some(tag_end) = object.find('>') else { break };
        let tag = &object[..tag_end];
        refs.extend(extract_html_attribute_refs(owner_kind, owner_id, location_kind, location_name, tag, "data", "html_object_data", MediaReferenceCandidateKind::HtmlObjectData));
        remaining = &object[tag_end + 1..];
    }
    refs
}

fn extract_css_url_refs(owner_kind: &str, owner_id: &str, location_kind: &str, location_name: &str, text: &str) -> Vec<MediaReferenceCandidate> {
    let mut refs = Vec::new();
    let mut remaining = text;
    while let Some(start) = remaining.to_ascii_lowercase().find("url(") {
        let after = &remaining[start + "url(".len()..];
        let Some(end) = after.find(')') else { break };
        let raw = after[..end].trim().trim_matches('"').trim_matches('\'').to_string();
        refs.push(local_candidate(owner_kind, owner_id, location_kind, location_name, &raw, "css_url", MediaReferenceCandidateKind::CssUrl, true));
        remaining = &after[end + 1..];
    }
    refs
}

fn extract_html_attribute_refs(owner_kind: &str, owner_id: &str, location_kind: &str, location_name: &str, text: &str, attr: &str, ref_kind: &str, kind: MediaReferenceCandidateKind) -> Vec<MediaReferenceCandidate> {
    let mut refs = Vec::new();
    let marker = format!("{attr}=");
    let mut remaining = text;
    while let Some(start) = remaining.to_ascii_lowercase().find(&marker) {
        let after = &remaining[start + marker.len()..];
        let Some(first) = after.chars().next() else { break };
        let (raw, rest) = if first == '"' || first == '\'' {
            let content = &after[first.len_utf8()..];
            let Some(end) = content.find(first) else { break };
            (&content[..end], &content[end + first.len_utf8()..])
        } else {
            let end = after.find(|ch: char| ch.is_whitespace() || ch == '>').unwrap_or(after.len());
            (&after[..end], &after[end..])
        };
        let decoded = decode_html_entities(raw).into_owned();
        refs.push(local_candidate(owner_kind, owner_id, location_kind, location_name, &decoded, ref_kind, kind, true));
        remaining = rest;
    }
    refs
}
```

Add classification helpers:

```rust
fn local_candidate(owner_kind: &str, owner_id: &str, location_kind: &str, location_name: &str, raw: &str, ref_kind: &str, kind: MediaReferenceCandidateKind, url_semantics: bool) -> MediaReferenceCandidate {
    let (normalized_local_ref, skip_reason, unsafe_reason) = classify_ref(raw, url_semantics);
    MediaReferenceCandidate {
        owner_kind: owner_kind.into(),
        owner_id: owner_id.into(),
        location_kind: location_kind.into(),
        location_name: location_name.into(),
        raw_ref: raw.into(),
        ref_kind: ref_kind.into(),
        normalized_local_ref,
        skip_reason,
        unsafe_reason,
        kind,
    }
}

fn classify_ref(raw: &str, url_semantics: bool) -> (Option<String>, Option<String>, Option<String>) {
    let lower = raw.to_ascii_lowercase();
    if lower.starts_with("data:") {
        return (None, Some("data-uri".into()), None);
    }
    if lower.starts_with("http://") || lower.starts_with("https://") || lower.starts_with("//") || lower.contains("://") {
        return (None, Some("external-url".into()), None);
    }
    if raw.contains("{{") || raw.contains("}}") {
        return (None, Some("dynamic-template".into()), None);
    }
    let path = if url_semantics {
        raw.split(['?', '#']).next().unwrap_or(raw)
    } else {
        raw
    };
    let decoded = if url_semantics {
        match percent_decode_utf8(path) {
            Ok(value) => value,
            Err(reason) => return (None, None, Some(reason)),
        }
    } else {
        path.to_string()
    };
    if decoded.is_empty() {
        return (None, None, Some("empty-reference".into()));
    }
    if decoded.contains(['/', '\\']) {
        return (None, None, Some("decoded-path-separator".into()));
    }
    if decoded == "." || decoded == ".." {
        return (None, None, Some("path-component".into()));
    }
    (Some(decoded), None, None)
}

fn percent_decode_utf8(input: &str) -> Result<String, String> {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            if index + 2 >= bytes.len() {
                return Err("invalid-percent-escape".into());
            }
            let hex = std::str::from_utf8(&bytes[index + 1..index + 3]).map_err(|_| "invalid-percent-escape".to_string())?;
            let value = u8::from_str_radix(hex, 16).map_err(|_| "invalid-percent-escape".to_string())?;
            out.push(value);
            index += 3;
        } else {
            out.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(out).map_err(|_| "invalid-utf8".into())
}
```

- [ ] **Step 4: Export scanner**

In `authoring_core/src/lib.rs`, add:

```rust
pub mod media_refs;
pub use media_refs::{
    extract_media_reference_candidates, MediaReferenceCandidate, MediaReferenceCandidateKind,
};
```

- [ ] **Step 5: Run reference tests**

Run:

```bash
cargo test -p authoring_core --test media_refs_tests -v
```

Expected: PASS.

- [ ] **Step 6: Commit reference scanner**

Run:

```bash
git add authoring_core/src/media_refs.rs authoring_core/src/lib.rs authoring_core/tests/media_refs_tests.rs
git commit -m "feat: extract normalized media references"
```

## Task 5: Normalize Integration and Media Diagnostics

**Files:**
- Modify: `authoring_core/src/normalize.rs`
- Modify: `authoring_core/src/media.rs`
- Modify: `authoring_core/tests/media_ingest_tests.rs`
- Modify: existing authoring tests that construct `NormalizedIr`

- [ ] **Step 1: Add failing normalize integration tests**

Append these tests to `authoring_core/tests/media_ingest_tests.rs`:

```rust
use authoring_core::{
    normalize_with_options, AuthoringDocument, AuthoringMedia, AuthoringMediaSource,
    AuthoringNote, AuthoringNotetype, NormalizationRequest,
};
use std::collections::BTreeMap;

#[test]
fn normalize_outputs_media_objects_bindings_and_resolved_references() {
    let root = unique_test_root("normalize-media");
    let base_dir = root.join("input");
    fs::create_dir_all(base_dir.join("assets")).unwrap();
    fs::write(base_dir.join("assets/hello.txt"), b"hello").unwrap();
    let options = test_options(&base_dir, &root.join("store"));
    let request = NormalizationRequest::new(authoring_doc_with_media("<img src=\"hello.txt\">"));

    let result = normalize_with_options(request, options);

    assert_eq!(result.result_status, "success");
    let normalized = result.normalized_ir.unwrap();
    assert_eq!(normalized.media_objects.len(), 1);
    assert_eq!(normalized.media_bindings[0].export_filename, "hello.txt");
    assert_eq!(normalized.media_references[0].resolution_status(), "resolved");
}

#[test]
fn normalize_reports_missing_and_unsafe_references() {
    let root = unique_test_root("normalize-missing");
    let base_dir = root.join("input");
    fs::create_dir_all(base_dir.join("assets")).unwrap();
    fs::write(base_dir.join("assets/hello.txt"), b"hello").unwrap();
    let options = test_options(&base_dir, &root.join("store"));
    let request = NormalizationRequest::new(authoring_doc_with_media(
        "<img src=\"missing.png\"><img src=\"bad%2Fname.png\">",
    ));

    let result = normalize_with_options(request, options);

    assert_eq!(result.result_status, "invalid");
    let codes = result
        .diagnostics
        .items
        .iter()
        .map(|item| item.code.as_str())
        .collect::<Vec<_>>();
    assert!(codes.contains(&"MEDIA.MISSING_REFERENCE"));
    assert!(codes.contains(&"MEDIA.UNSAFE_REFERENCE"));
}

fn authoring_doc_with_media(back: &str) -> AuthoringDocument {
    let mut fields = BTreeMap::new();
    fields.insert("Front".into(), "front".into());
    fields.insert("Back".into(), back.into());
    AuthoringDocument {
        kind: "authoring-ir".into(),
        schema_version: "0.1.0".into(),
        metadata_document_id: "doc".into(),
        notetypes: vec![AuthoringNotetype {
            id: "basic-main".into(),
            kind: "normal".into(),
            name: Some("Basic".into()),
            original_stock_kind: Some("basic".into()),
            original_id: None,
            fields: None,
            templates: None,
            css: None,
            field_metadata: vec![],
        }],
        notes: vec![AuthoringNote {
            id: "note-1".into(),
            notetype_id: "basic-main".into(),
            deck_name: "Default".into(),
            fields,
            tags: vec![],
        }],
        media: vec![AuthoringMedia {
            id: "media:hello".into(),
            desired_filename: "hello.txt".into(),
            source: AuthoringMediaSource::Path {
                path: "assets/hello.txt".into(),
            },
            declared_mime: Some("text/plain".into()),
        }],
    }
}
```

Add this helper method to `MediaReference` in `authoring_core/src/media.rs` for test readability:

```rust
impl MediaReference {
    pub fn resolution_status(&self) -> &'static str {
        match self.resolution {
            MediaReferenceResolution::Resolved { .. } => "resolved",
            MediaReferenceResolution::Missing => "missing",
            MediaReferenceResolution::Skipped { .. } => "skipped",
        }
    }
}
```

- [ ] **Step 2: Run normalize integration tests to verify failure**

Run:

```bash
cargo test -p authoring_core --test media_ingest_tests normalize_outputs_media_objects_bindings_and_resolved_references -v
```

Expected: FAIL because `normalize_with_options` does not exist.

- [ ] **Step 3: Implement normalize_with_options entry point**

In `authoring_core/src/normalize.rs`:

1. Change imports to include media helpers:

```rust
use crate::media::{
    ingest_authoring_media, sort_media_references, MediaReference, MediaReferenceResolution,
    NormalizeOptions,
};
use crate::media_refs::extract_media_reference_candidates;
```

2. Keep the existing `normalize(request)` entry point, but make it valid only for no-media requests:

```rust
pub fn normalize(request: NormalizationRequest) -> NormalizationResult {
    if request.input.media.is_empty() {
        return normalize_with_options(request, NormalizeOptions {
            base_dir: std::env::current_dir().unwrap_or_else(|_| ".".into()),
            media_store_dir: std::env::temp_dir().join("anki-forge-unused-media-store"),
            media_policy: crate::media::MediaPolicy::default_strict(),
        });
    }

    invalid_result(
        PolicyRefs {
            identity_policy_ref: "identity-policy.default@1.0.0".into(),
            risk_policy_ref: request
                .comparison_context
                .as_ref()
                .map(|context| context.risk_policy_ref.clone()),
        },
        request.comparison_context,
        vec![DiagnosticItem {
            level: "error".into(),
            code: "MEDIA.NORMALIZE_OPTIONS_REQUIRED".into(),
            summary: "media normalization requires NormalizeOptions".into(),
        }],
        "det:unavailable".into(),
        "media normalization requires explicit options".into(),
    )
}
```

3. Move the current implementation body of `normalize` into a new
   `normalize_with_options` function with this signature. Keep the existing
   document-id, selector, identity, notetype, and note validation logic inside
   this function. Steps 4-6 replace only the legacy normalized media block and
   the final `NormalizedIr` construction.

```rust
pub fn normalize_with_options(
    request: NormalizationRequest,
    options: NormalizeOptions,
) -> NormalizationResult
```

- [ ] **Step 4: Convert ingest diagnostics into normalization diagnostics**

Add this helper in `authoring_core/src/normalize.rs`:

```rust
fn media_ingest_diagnostic_to_item(
    diagnostic: crate::media::MediaIngestDiagnostic,
) -> DiagnosticItem {
    DiagnosticItem {
        level: diagnostic.level,
        code: diagnostic.code,
        summary: diagnostic.summary,
    }
}
```

In `normalize_with_options`, replace the `normalized_media` block with:

```rust
let ingest = match ingest_authoring_media(&request.input.media, &options) {
    Ok(ingest) => ingest,
    Err(error) => {
        let items = error
            .diagnostics
            .into_iter()
            .map(media_ingest_diagnostic_to_item)
            .collect::<Vec<_>>();
        return invalid_result(
            policy_refs,
            request.comparison_context,
            items,
            format!("det:{metadata_document_id}"),
            "media ingestion failed".into(),
        );
    }
};
```

- [ ] **Step 5: Build references from note fields and resolve against bindings**

Add this helper in `authoring_core/src/normalize.rs`:

```rust
fn resolve_media_references(
    notes: &[NormalizedNote],
    bindings: &[crate::media::MediaBinding],
) -> (Vec<MediaReference>, Vec<DiagnosticItem>) {
    let binding_by_filename = bindings
        .iter()
        .map(|binding| (binding.export_filename.as_str(), binding.id.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut references = Vec::new();
    let mut diagnostics = Vec::new();

    for note in notes {
        for (field_name, field_value) in &note.fields {
            for candidate in extract_media_reference_candidates(
                "note",
                &note.id,
                "field",
                field_name,
                field_value,
            ) {
                let resolution = if let Some(reason) = candidate.unsafe_reason {
                    diagnostics.push(DiagnosticItem {
                        level: "error".into(),
                        code: "MEDIA.UNSAFE_REFERENCE".into(),
                        summary: format!("unsafe media reference {}: {}", candidate.raw_ref, reason),
                    });
                    MediaReferenceResolution::Skipped {
                        skip_reason: reason,
                    }
                } else if let Some(reason) = candidate.skip_reason {
                    MediaReferenceResolution::Skipped {
                        skip_reason: reason,
                    }
                } else if let Some(local_ref) = candidate.normalized_local_ref {
                    if let Some(media_id) = binding_by_filename.get(local_ref.as_str()) {
                        MediaReferenceResolution::Resolved {
                            media_id: (*media_id).to_string(),
                        }
                    } else {
                        diagnostics.push(DiagnosticItem {
                            level: "error".into(),
                            code: "MEDIA.MISSING_REFERENCE".into(),
                            summary: format!("missing media reference {}", candidate.raw_ref),
                        });
                        MediaReferenceResolution::Missing
                    }
                } else {
                    MediaReferenceResolution::Skipped {
                        skip_reason: "unresolved-candidate".into(),
                    }
                };

                references.push(MediaReference {
                    owner_kind: candidate.owner_kind,
                    owner_id: candidate.owner_id,
                    location_kind: candidate.location_kind,
                    location_name: candidate.location_name,
                    raw_ref: candidate.raw_ref,
                    ref_kind: candidate.ref_kind,
                    resolution,
                });
            }
        }
    }

    sort_media_references(&mut references);
    (references, diagnostics)
}
```

- [ ] **Step 6: Emit new media fields in NormalizedIr**

In `normalize_with_options`, after `normalized_notes` is built, call:

```rust
let (media_references, media_reference_diagnostics) =
    resolve_media_references(&normalized_notes, &ingest.bindings);
if media_reference_diagnostics
    .iter()
    .any(|item| item.level == "error")
{
    return invalid_result(
        policy_refs,
        request.comparison_context,
        media_reference_diagnostics,
        format!("det:{metadata_document_id}"),
        "media reference resolution failed".into(),
    );
}
```

Then construct `NormalizedIr` with:

```rust
let normalized_ir = NormalizedIr {
    kind: "normalized-ir".into(),
    schema_version: request.input.schema_version,
    document_id: metadata_document_id,
    resolved_identity: resolved_identity.clone(),
    notetypes: normalized_notetypes,
    notes: normalized_notes,
    media_objects: ingest.objects,
    media_bindings: ingest.bindings,
    media_references,
};
```

- [ ] **Step 7: Export normalize_with_options**

In `authoring_core/src/lib.rs`, export:

```rust
pub use normalize::{normalize, normalize_with_options, selector_resolve_error_code};
```

Keep the existing `normalize` export in place by replacing it with the line above.

- [ ] **Step 8: Run normalize integration tests**

Run:

```bash
cargo test -p authoring_core --test media_ingest_tests normalize_outputs_media_objects_bindings_and_resolved_references -v
cargo test -p authoring_core --test media_ingest_tests normalize_reports_missing_and_unsafe_references -v
```

Expected: PASS.

- [ ] **Step 9: Update existing authoring tests for new NormalizedIr fields**

For every existing construction of `NormalizedIr` in `authoring_core/tests`, replace:

```rust
media: vec![],
```

with:

```rust
media_objects: vec![],
media_bindings: vec![],
media_references: vec![],
```

Run:

```bash
cargo test -p authoring_core -v
```

Expected: PASS.

- [ ] **Step 10: Commit normalize integration**

Run:

```bash
git add authoring_core/src/normalize.rs authoring_core/src/lib.rs authoring_core/src/media.rs authoring_core/tests/media_ingest_tests.rs authoring_core/tests
git commit -m "feat: normalize media into cas metadata"
```

## Task 6: Writer CAS Target, Invariants, and Staging

**Files:**
- Modify: `writer_core/Cargo.toml`
- Modify: `writer_core/src/staging.rs`
- Modify: `writer_core/src/build.rs`
- Modify: `writer_core/tests/build_tests.rs`
- Modify: tests that construct `BuildArtifactTarget`

- [ ] **Step 1: Add writer dependency for CAS verification**

In `writer_core/Cargo.toml`, add:

```toml
blake3 = "1"
```

- [ ] **Step 2: Add failing writer CAS staging tests**

Add these tests to `writer_core/tests/build_tests.rs` near `build_materializes_media_payloads_into_staging_tree`:

```rust
#[test]
fn staging_materializes_media_from_cas_not_inline_payload() {
    let root = unique_artifact_root("cas-staging");
    let media_store = root.join("media-store");
    let normalized = sample_basic_normalized_ir_with_cas_media(&media_store, "hello.txt", b"hello");
    let target = BuildArtifactTarget::new(root.clone(), "artifacts/phase3/cas-staging")
        .with_media_store_dir(media_store.clone());

    let result = build(
        &normalized,
        &sample_writer_policy(),
        &sample_build_context(false),
        &target,
    )
    .unwrap();

    assert_eq!(result.result_status, "success");
    assert_eq!(fs::read(root.join("staging/media/hello.txt")).unwrap(), b"hello");
}

#[test]
fn writer_reports_missing_cas_object_without_semantic_media_diagnostics() {
    let root = unique_artifact_root("missing-cas");
    let media_store = root.join("media-store");
    let normalized = sample_basic_normalized_ir_with_cas_media(&media_store, "hello.txt", b"hello");
    let target = BuildArtifactTarget::new(root.clone(), "artifacts/phase3/missing-cas")
        .with_media_store_dir(media_store.clone());
    fs::remove_dir_all(&media_store).unwrap();

    let result = build(
        &normalized,
        &sample_writer_policy(),
        &sample_build_context(false),
        &target,
    )
    .unwrap();

    let codes = result
        .diagnostics
        .items
        .iter()
        .map(|item| item.code.as_str())
        .collect::<Vec<_>>();
    assert_eq!(result.result_status, "invalid");
    assert!(codes.contains(&"MEDIA.CAS_OBJECT_MISSING"));
    assert!(!codes.contains(&"MEDIA.MISSING_REFERENCE"));
    assert!(!codes.contains(&"MEDIA.UNUSED_BINDING"));
}
```

At the top of `writer_core/tests/build_tests.rs`, add:

```rust
use sha1::Digest;
```

Add this helper near existing sample normalized IR helpers:

```rust
fn sample_basic_normalized_ir_with_cas_media(
    media_store: &Path,
    filename: &str,
    bytes: &[u8],
) -> NormalizedIr {
    let mut normalized = sample_basic_normalized_ir();
    let blake3_hex = blake3::hash(bytes).to_hex().to_string();
    let sha1_hex = hex::encode(sha1::Sha1::digest(bytes));
    let object_id = format!("obj:blake3:{blake3_hex}");
    let object_path = authoring_core::object_store_path(media_store, &blake3_hex).unwrap();
    fs::create_dir_all(object_path.parent().unwrap()).unwrap();
    fs::write(&object_path, bytes).unwrap();
    normalized.media_objects = vec![authoring_core::MediaObject {
        id: object_id.clone(),
        object_ref: format!("media://blake3/{blake3_hex}"),
        blake3: blake3_hex,
        sha1: sha1_hex,
        size_bytes: bytes.len() as u64,
        mime: "text/plain".into(),
    }];
    normalized.media_bindings = vec![authoring_core::MediaBinding {
        id: "media:hello".into(),
        export_filename: filename.into(),
        object_id,
    }];
    normalized.media_references = vec![];
    normalized
}
```

- [ ] **Step 3: Run writer staging tests to verify failure**

Run:

```bash
cargo test -p writer_core --test build_tests staging_materializes_media_from_cas_not_inline_payload -v
```

Expected: FAIL because writer still expects `normalized_ir.media` and `BuildArtifactTarget` has no media store path.

- [ ] **Step 4: Extend BuildArtifactTarget with media_store_dir**

In `writer_core/src/staging.rs`, change `BuildArtifactTarget` to:

```rust
#[derive(Debug, Clone)]
pub struct BuildArtifactTarget {
    pub root_dir: PathBuf,
    pub stable_ref_prefix: String,
    pub media_store_dir: PathBuf,
}
```

Change `BuildArtifactTarget::new` to:

```rust
pub fn new(root_dir: impl Into<PathBuf>, stable_ref_prefix: impl Into<String>) -> Self {
    let root_dir = root_dir.into();
    Self {
        media_store_dir: root_dir.join(".anki-forge-media"),
        root_dir,
        stable_ref_prefix: stable_ref_prefix.into(),
    }
}

pub fn with_media_store_dir(mut self, media_store_dir: impl Into<PathBuf>) -> Self {
    self.media_store_dir = media_store_dir.into();
    self
}
```

Update struct literals in tests from:

```rust
BuildArtifactTarget {
    root_dir: root.clone(),
    stable_ref_prefix: "artifacts/phase3/basic".into(),
}
```

to:

```rust
BuildArtifactTarget::new(root.clone(), "artifacts/phase3/basic")
```

- [ ] **Step 5: Implement writer media invariant validation**

In `writer_core/src/staging.rs`, replace the `media_filenames` construction with:

```rust
let media_filenames: BTreeSet<_> = normalized_ir
    .media_bindings
    .iter()
    .map(|binding| binding.export_filename.as_str())
    .collect();
```

Add this helper before `validate_normalized_ir`:

```rust
fn validate_media_invariants(normalized_ir: &NormalizedIr) -> Vec<BuildDiagnosticItem> {
    let mut diagnostics = Vec::new();
    let mut object_ids = BTreeSet::new();
    for (index, object) in normalized_ir.media_objects.iter().enumerate() {
        if !object_ids.insert(object.id.as_str()) {
            diagnostics.push(media_error("MEDIA.DUPLICATE_MEDIA_ID", format!("duplicate media object id {}", object.id), format!("media_objects[{index}].id")));
        }
        if object.id != format!("obj:blake3:{}", object.blake3)
            || object.object_ref != format!("media://blake3/{}", object.blake3)
        {
            diagnostics.push(media_error("MEDIA.INVALID_MEDIA_OBJECT_INVARIANT", format!("invalid object invariant {}", object.id), format!("media_objects[{index}]")));
        }
    }
    let object_id_set = normalized_ir
        .media_objects
        .iter()
        .map(|object| object.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut binding_ids = BTreeSet::new();
    let mut filenames = BTreeSet::new();
    for (index, binding) in normalized_ir.media_bindings.iter().enumerate() {
        if !binding_ids.insert(binding.id.as_str()) {
            diagnostics.push(media_error("MEDIA.DUPLICATE_MEDIA_ID", format!("duplicate media binding id {}", binding.id), format!("media_bindings[{index}].id")));
        }
        if !filenames.insert(binding.export_filename.as_str()) {
            diagnostics.push(media_error("MEDIA.DUPLICATE_EXPORT_FILENAME", format!("duplicate export filename {}", binding.export_filename), format!("media_bindings[{index}].export_filename")));
        }
        if !object_id_set.contains(binding.object_id.as_str()) {
            diagnostics.push(media_error("MEDIA.MEDIA_OBJECT_MISSING", format!("binding {} references missing object {}", binding.id, binding.object_id), format!("media_bindings[{index}].object_id")));
        }
        if let Err(err) = validated_media_output_path(Path::new("media"), &binding.export_filename) {
            diagnostics.push(media_error("MEDIA.UNSAFE_FILENAME", err.to_string(), format!("media_bindings[{index}].export_filename")));
        }
    }
    diagnostics
}

fn media_error(code: &str, summary: String, path: String) -> BuildDiagnosticItem {
    BuildDiagnosticItem {
        level: "error".into(),
        code: code.into(),
        summary,
        domain: Some("media".into()),
        path: Some(path),
        target_selector: None,
        stage: Some("validate".into()),
        operation: Some("writer-invariant".into()),
    }
}
```

At the start of `validate_normalized_ir`, after `let mut diagnostics = vec![];`, add:

```rust
diagnostics.extend(validate_media_invariants(normalized_ir));
```

- [ ] **Step 6: Materialize staging media from CAS by copy**

In `StagingPackage::materialize`, replace the base64 media block with:

```rust
if !self.manifest.normalized_ir.media_bindings.is_empty() {
    let media_dir = staging_dir.join("media");
    fs::create_dir_all(&media_dir)
        .with_context(|| format!("create staging media directory {}", media_dir.display()))?;
    let objects_by_id = self
        .manifest
        .normalized_ir
        .media_objects
        .iter()
        .map(|object| (object.id.as_str(), object))
        .collect::<BTreeMap<_, _>>();
    for binding in &self.manifest.normalized_ir.media_bindings {
        let object = objects_by_id
            .get(binding.object_id.as_str())
            .with_context(|| format!("binding {} references missing object {}", binding.id, binding.object_id))?;
        let source = authoring_core::object_store_path(&target.media_store_dir, &object.blake3)
            .map_err(anyhow::Error::msg)?;
        verify_cas_object(&source, object)?;
        let media_path = validated_media_output_path(&media_dir, &binding.export_filename)?;
        fs::copy(&source, &media_path).with_context(|| {
            format!(
                "copy media object {} into staging media {}",
                source.display(),
                media_path.display()
            )
        })?;
    }
}
```

Add:

```rust
fn verify_cas_object(path: &Path, object: &authoring_core::MediaObject) -> Result<()> {
    let bytes = fs::read(path)
        .with_context(|| format!("read CAS media object {}", path.display()))?;
    anyhow::ensure!(
        bytes.len() as u64 == object.size_bytes,
        "MEDIA.CAS_OBJECT_SIZE_MISMATCH: {}",
        path.display()
    );
    anyhow::ensure!(
        blake3::hash(&bytes).to_hex().to_string() == object.blake3,
        "MEDIA.CAS_OBJECT_BLAKE3_MISMATCH: {}",
        path.display()
    );
    anyhow::ensure!(
        hex::encode(sha1::Sha1::digest(&bytes)) == object.sha1,
        "MEDIA.CAS_OBJECT_SHA1_MISMATCH: {}",
        path.display()
    );
    Ok(())
}
```

- [ ] **Step 7: Convert materialize errors to media-specific build result codes**

In `writer_core/src/build.rs`, change the error mapping around `package.materialize`:

```rust
let materialized = match package.materialize(artifact_target) {
    Ok(materialized) => materialized,
    Err(err) => {
        let text = err.to_string();
        let code = if text.contains("No such file") && text.contains("media object") {
            "MEDIA.CAS_OBJECT_MISSING"
        } else if text.contains("MEDIA.CAS_OBJECT_SIZE_MISMATCH") {
            "MEDIA.CAS_OBJECT_SIZE_MISMATCH"
        } else if text.contains("MEDIA.CAS_OBJECT_BLAKE3_MISMATCH") {
            "MEDIA.CAS_OBJECT_BLAKE3_MISMATCH"
        } else if text.contains("MEDIA.CAS_OBJECT_SHA1_MISMATCH") {
            "MEDIA.CAS_OBJECT_SHA1_MISMATCH"
        } else {
            "PHASE3.STAGING_MATERIALIZATION_FAILED"
        };
        return Ok(error_result(
            writer_policy,
            build_context,
            code,
            text,
            "materialize_staging",
            "write_manifest",
            Some(artifact_target.staging_manifest_path().display().to_string()),
        ));
    }
};
```

- [ ] **Step 8: Run writer staging tests**

Run:

```bash
cargo test -p writer_core --test build_tests staging_materializes_media_from_cas_not_inline_payload -v
cargo test -p writer_core --test build_tests writer_reports_missing_cas_object_without_semantic_media_diagnostics -v
```

Expected: PASS.

- [ ] **Step 9: Commit writer staging changes**

Run:

```bash
git add writer_core/Cargo.toml writer_core/src/staging.rs writer_core/src/build.rs writer_core/tests/build_tests.rs
git commit -m "feat: materialize staging media from cas"
```

## Task 7: APKG Media From CAS and Stable Ordering

**Files:**
- Modify: `writer_core/src/apkg.rs`
- Modify: `writer_core/tests/build_tests.rs`

- [ ] **Step 1: Add failing APKG CAS tests**

Add these tests to `writer_core/tests/build_tests.rs` near APKG media tests:

```rust
#[test]
fn apkg_media_entries_follow_export_filename_then_media_id_order() {
    let root = unique_artifact_root("apkg-order");
    let media_store = root.join("media-store");
    let mut normalized = sample_basic_normalized_ir_with_cas_media(&media_store, "b.txt", b"b");
    let second = sample_basic_normalized_ir_with_cas_media(&media_store, "a.txt", b"a");
    normalized.media_objects.extend(second.media_objects);
    normalized.media_bindings.extend(second.media_bindings);
    normalized.media_bindings[0].id = "media:b".into();
    normalized.media_bindings[1].id = "media:a".into();
    normalized.media_bindings.sort_by(|left, right| {
        left.export_filename
            .as_bytes()
            .cmp(right.export_filename.as_bytes())
            .then_with(|| left.id.as_bytes().cmp(right.id.as_bytes()))
    });
    let target = BuildArtifactTarget::new(root.clone(), "artifacts/phase3/apkg-order")
        .with_media_store_dir(media_store);

    build(&normalized, &sample_writer_policy(), &sample_build_context(true), &target).unwrap();

    let mut archive = open_zip(&root.join("package.apkg"));
    let media_entries = decode_media_entries(
        zstd::stream::decode_all(read_zip_entry_bytes(&mut archive, "media").as_slice()).unwrap(),
    );
    assert_eq!(media_entries.entries[0].name, "a.txt");
    assert_eq!(media_entries.entries[1].name, "b.txt");
}

#[test]
fn apkg_writes_duplicate_object_once_per_export_filename() {
    let root = unique_artifact_root("apkg-deduped-object");
    let media_store = root.join("media-store");
    let mut normalized = sample_basic_normalized_ir_with_cas_media(&media_store, "a.txt", b"same");
    normalized.media_bindings.push(authoring_core::MediaBinding {
        id: "media:b".into(),
        export_filename: "b.txt".into(),
        object_id: normalized.media_objects[0].id.clone(),
    });
    normalized.media_bindings.sort_by(|left, right| {
        left.export_filename
            .as_bytes()
            .cmp(right.export_filename.as_bytes())
            .then_with(|| left.id.as_bytes().cmp(right.id.as_bytes()))
    });
    let target = BuildArtifactTarget::new(root.clone(), "artifacts/phase3/apkg-deduped-object")
        .with_media_store_dir(media_store);

    build(&normalized, &sample_writer_policy(), &sample_build_context(true), &target).unwrap();

    let mut archive = open_zip(&root.join("package.apkg"));
    let names = archive_names(&mut archive);
    assert!(names.contains("0"));
    assert!(names.contains("1"));
}
```

- [ ] **Step 2: Run APKG tests to verify failure**

Run:

```bash
cargo test -p writer_core --test build_tests apkg_media_entries_follow_export_filename_then_media_id_order -v
```

Expected: FAIL because APKG writer still reads `normalized_ir.media` and staging files.

- [ ] **Step 3: Change APKG writer to read CAS**

In `writer_core/src/apkg.rs`, change `write_media_payloads_and_map` signature to:

```rust
fn write_media_payloads_and_map(
    zip: &mut ZipWriter<File>,
    normalized_ir: &NormalizedIr,
    media_store_dir: &Path,
) -> Result<()> {
```

Change the call in `emit_apkg` from:

```rust
write_media_payloads_and_map(&mut zip, &normalized_ir, staging_dir)?;
```

to:

```rust
write_media_payloads_and_map(&mut zip, &normalized_ir, &artifact_target.media_store_dir)?;
```

- [ ] **Step 4: Implement CAS-backed APKG media loop**

Replace the loop in `write_media_payloads_and_map` with:

```rust
let objects_by_id = normalized_ir
    .media_objects
    .iter()
    .map(|object| (object.id.as_str(), object))
    .collect::<std::collections::BTreeMap<_, _>>();

for (index, binding) in normalized_ir.media_bindings.iter().enumerate() {
    let object = objects_by_id
        .get(binding.object_id.as_str())
        .with_context(|| format!("binding {} references missing object {}", binding.id, binding.object_id))?;
    let payload = read_media_payload(media_store_dir, object)?;
    let encoded = zstd::stream::encode_all(payload.as_slice(), 0)
        .context("compress media payload for apkg")?;
    write_stored_entry(zip, &index.to_string(), &encoded)?;
    entries.push(MediaEntry {
        name: binding.export_filename.clone(),
        size: object.size_bytes as u32,
        sha1: hex::decode(&object.sha1)
            .with_context(|| format!("decode sha1 for media object {}", object.id))?,
        legacy_zip_filename: None,
    });
}
```

Change `read_media_payload` to:

```rust
fn read_media_payload(media_store_dir: &Path, object: &authoring_core::MediaObject) -> Result<Vec<u8>> {
    let path = authoring_core::object_store_path(media_store_dir, &object.blake3)
        .map_err(anyhow::Error::msg)?;
    let bytes = fs::read(&path).with_context(|| format!("read media object {}", path.display()))?;
    anyhow::ensure!(bytes.len() as u64 == object.size_bytes, "media object size mismatch {}", object.id);
    anyhow::ensure!(blake3::hash(&bytes).to_hex().to_string() == object.blake3, "media object blake3 mismatch {}", object.id);
    anyhow::ensure!(hex::encode(Sha1::digest(&bytes)) == object.sha1, "media object sha1 mismatch {}", object.id);
    Ok(bytes)
}
```

- [ ] **Step 5: Run APKG tests**

Run:

```bash
cargo test -p writer_core --test build_tests apkg_media_entries_follow_export_filename_then_media_id_order -v
cargo test -p writer_core --test build_tests apkg_writes_duplicate_object_once_per_export_filename -v
```

Expected: PASS.

- [ ] **Step 6: Commit APKG CAS changes**

Run:

```bash
git add writer_core/src/apkg.rs writer_core/tests/build_tests.rs
git commit -m "feat: write apkg media from cas objects"
```

## Task 8: Inspect Surface Updates

**Files:**
- Modify: `writer_core/src/inspect.rs`
- Modify: `writer_core/tests/inspect_tests.rs`

- [ ] **Step 1: Add failing inspect tests**

Add these tests to `writer_core/tests/inspect_tests.rs`:

```rust
#[test]
fn inspect_staging_reports_manifest_media_object_and_binding_metadata() {
    let root = unique_artifact_root("inspect-staging-cas");
    let media_store = root.join("media-store");
    let normalized = sample_basic_normalized_ir_with_cas_media(&media_store, "hello.txt", b"hello");
    let target = BuildArtifactTarget::new(root.clone(), "artifacts/phase3/inspect-staging-cas")
        .with_media_store_dir(media_store);
    build(&normalized, &sample_writer_policy(), &sample_build_context(false), &target).unwrap();

    let report = inspect_staging(root.join("staging/manifest.json")).unwrap();

    let media = &report.observations.media[0];
    assert_eq!(media["filename"], "hello.txt");
    assert_eq!(media["binding_id"], "media:hello");
    assert!(media["object_id"].as_str().unwrap().starts_with("obj:blake3:"));
}

#[test]
fn inspect_apkg_does_not_report_forge_only_media_ids() {
    let root = unique_artifact_root("inspect-apkg-cas");
    let media_store = root.join("media-store");
    let normalized = sample_basic_normalized_ir_with_cas_media(&media_store, "hello.txt", b"hello");
    let target = BuildArtifactTarget::new(root.clone(), "artifacts/phase3/inspect-apkg-cas")
        .with_media_store_dir(media_store);
    build(&normalized, &sample_writer_policy(), &sample_build_context(true), &target).unwrap();

    let report = inspect_apkg(root.join("package.apkg")).unwrap();

    let media = &report.observations.media[0];
    assert_eq!(media["filename"], "hello.txt");
    assert!(media.get("binding_id").is_none());
    assert!(media.get("object_id").is_none());
}
```

At the top of `writer_core/tests/inspect_tests.rs`, add:

```rust
use sha1::Digest;
```

Add this helper near the existing sample normalized IR helpers in `writer_core/tests/inspect_tests.rs`:

```rust
fn sample_basic_normalized_ir_with_cas_media(
    media_store: &Path,
    filename: &str,
    bytes: &[u8],
) -> NormalizedIr {
    let mut normalized = sample_basic_normalized_ir();
    let blake3_hex = blake3::hash(bytes).to_hex().to_string();
    let sha1_hex = hex::encode(sha1::Sha1::digest(bytes));
    let object_id = format!("obj:blake3:{blake3_hex}");
    let object_path = authoring_core::object_store_path(media_store, &blake3_hex).unwrap();
    fs::create_dir_all(object_path.parent().unwrap()).unwrap();
    fs::write(&object_path, bytes).unwrap();
    normalized.media_objects = vec![authoring_core::MediaObject {
        id: object_id.clone(),
        object_ref: format!("media://blake3/{blake3_hex}"),
        blake3: blake3_hex,
        sha1: sha1_hex,
        size_bytes: bytes.len() as u64,
        mime: "text/plain".into(),
    }];
    normalized.media_bindings = vec![authoring_core::MediaBinding {
        id: "media:hello".into(),
        export_filename: filename.into(),
        object_id,
    }];
    normalized.media_references = vec![];
    normalized
}
```

- [ ] **Step 2: Run inspect tests to verify failure**

Run:

```bash
cargo test -p writer_core --test inspect_tests inspect_staging_reports_manifest_media_object_and_binding_metadata -v
```

Expected: FAIL because staging inspect still expects inline media payloads.

- [ ] **Step 3: Update staging media resolution**

In `writer_core/src/inspect.rs`, replace `resolve_staging_media` with a function that reads `media_bindings`, joins each binding to `media_objects`, reads `staging/media/<export_filename>`, verifies size/SHA-1, and returns `ResolvedMedia` with optional Forge metadata:

```rust
#[derive(Debug, Clone)]
struct ResolvedMedia {
    filename: String,
    size: usize,
    sha1_hex: String,
    binding_id: Option<String>,
    object_id: Option<String>,
    object_ref: Option<String>,
}
```

When constructing staging `ResolvedMedia`, set `binding_id`, `object_id`, and `object_ref`. When constructing APKG `ResolvedMedia`, set those fields to `None`.

- [ ] **Step 4: Update media observations**

In `build_observations`, replace media observation mapping with:

```rust
media: media
    .iter()
    .map(|entry| {
        let mut value = json!({
            "selector": format!("media[filename='{}']", entry.filename),
            "filename": entry.filename,
            "size": entry.size,
            "sha1": entry.sha1_hex,
            "evidence_refs": [format!("media:{}", entry.filename)],
        });
        if let Some(binding_id) = &entry.binding_id {
            value["binding_id"] = json!(binding_id);
        }
        if let Some(object_id) = &entry.object_id {
            value["object_id"] = json!(object_id);
        }
        if let Some(object_ref) = &entry.object_ref {
            value["object_ref"] = json!(object_ref);
        }
        value
    })
    .collect(),
```

- [ ] **Step 5: Run inspect tests**

Run:

```bash
cargo test -p writer_core --test inspect_tests inspect_staging_reports_manifest_media_object_and_binding_metadata -v
cargo test -p writer_core --test inspect_tests inspect_apkg_does_not_report_forge_only_media_ids -v
```

Expected: PASS.

- [ ] **Step 6: Commit inspect updates**

Run:

```bash
git add writer_core/src/inspect.rs writer_core/tests/inspect_tests.rs
git commit -m "feat: inspect cas-backed media surfaces"
```

## Task 9: Rust Facade, Deck, Product, and Runtime Flow

**Files:**
- Modify: `anki_forge/src/deck/media.rs`
- Modify: `anki_forge/src/deck/lowering.rs`
- Modify: `anki_forge/src/deck/export.rs`
- Modify: `anki_forge/src/product/assets.rs`
- Modify: `anki_forge/src/product/lowering.rs`
- Modify: `anki_forge/src/runtime/normalize.rs`
- Modify: `anki_forge/src/runtime/build.rs`
- Modify: `anki_forge/src/lib.rs`
- Modify: affected `anki_forge/tests/*`

- [ ] **Step 1: Add failing high-level deck build test**

In `anki_forge/tests/deck_export_tests.rs`, add:

```rust
#[test]
fn deck_build_uses_cas_media_without_normalized_base64_payload() {
    let root = unique_artifacts_dir("deck-cas-build");
    let mut deck = anki_forge::Deck::builder("Media Deck").stable_id("media-deck").build();
    let media = deck
        .media()
        .add(anki_forge::MediaSource::from_bytes("hello.txt", b"hello".to_vec()))
        .unwrap();
    deck.basic()
        .note("front", format!("<img src=\"{}\">", media.name()))
        .stable_id("n1")
        .add()
        .unwrap();

    let build = deck.build(&root).unwrap();
    let manifest = std::fs::read_to_string(build.staging_manifest_path()).unwrap();

    assert!(manifest.contains("media_objects"));
    assert!(manifest.contains("media_bindings"));
    assert!(!manifest.contains("data_base64"));
}
```

Use the existing `unique_artifacts_dir` helper already defined in
`anki_forge/tests/deck_export_tests.rs`.

- [ ] **Step 2: Run high-level test to verify failure**

Run:

```bash
cargo test -p anki_forge --test deck_export_tests deck_build_uses_cas_media_without_normalized_base64_payload -v
```

Expected: FAIL because deck lowering still emits `data_base64`.

- [ ] **Step 3: Update deck registered media model**

In `anki_forge/src/deck/model.rs`, replace `RegisteredMedia` fields:

```rust
pub(crate) mime: String,
pub(crate) data_base64: String,
pub(crate) sha1_hex: String,
```

with:

```rust
pub(crate) source: crate::AuthoringMediaSource,
pub(crate) declared_mime: Option<String>,
pub(crate) sha1_hex: String,
```

Keep `sha1_hex` and raster metadata for identity and image occlusion behavior.

- [ ] **Step 4: Update deck media registration**

In `anki_forge/src/deck/media.rs`, keep reading bytes for `sha1_hex` and raster metadata, but build an authoring source:

```rust
let source = match source {
    MediaSource::File { path } => {
        let bytes = std::fs::read(&path)?;
        let name = path
            .file_name()
            .and_then(|item| item.to_str())
            .ok_or_else(|| anyhow::anyhow!("media path must end in a valid filename"))?
            .to_string();
        (name, bytes, crate::AuthoringMediaSource::InlineBytes {
            data_base64: base64::engine::general_purpose::STANDARD.encode(&bytes),
        })
    }
    MediaSource::Bytes { name, bytes } => {
        let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
        (name, bytes, crate::AuthoringMediaSource::InlineBytes { data_base64: encoded })
    }
};
```

Store `declared_mime: Some(mime_from_name(&name))`. This keeps the existing high-level API stable while ensuring normalized output is CAS-backed after normalization.

- [ ] **Step 5: Update deck lowering**

In `anki_forge/src/deck/lowering.rs`, replace the media extension block with:

```rust
lowered.media.extend(self.media.values().map(|media| crate::AuthoringMedia {
    id: format!("media:{}", media.name),
    desired_filename: media.name.clone(),
    source: media.source.clone(),
    declared_mime: media.declared_mime.clone(),
}));
```

- [ ] **Step 6: Update package build to call normalize_with_options**

In `anki_forge/src/deck/export.rs`, change imports to:

```rust
use authoring_core::{normalize_with_options, MediaPolicy, NormalizationRequest, NormalizeOptions};
```

Before normalizing in `build_package`, add:

```rust
let artifacts_dir = artifacts_dir.as_ref();
let media_store_dir = artifacts_dir.join(".anki-forge-media");
let normalize_options = NormalizeOptions {
    base_dir: std::env::current_dir().context("resolve current directory")?,
    media_store_dir: media_store_dir.clone(),
    media_policy: MediaPolicy::default_strict(),
};
let normalized = normalize_with_options(NormalizationRequest::new(lowered), normalize_options);
```

Replace the later artifact target construction with:

```rust
let artifact_target =
    BuildArtifactTarget::new(artifacts_dir.to_path_buf(), stable_ref_prefix)
        .with_media_store_dir(media_store_dir);
```

- [ ] **Step 7: Update product asset lowering**

In `anki_forge/src/product/lowering.rs`, replace product asset media construction with:

```rust
media.push(crate::AuthoringMedia {
    id: format!("media:{lowered_filename}"),
    desired_filename: lowered_filename.clone(),
    source: crate::AuthoringMediaSource::InlineBytes {
        data_base64: asset.data_base64().into(),
    },
    declared_mime: Some(asset.mime().into()),
});
```

- [ ] **Step 8: Update runtime normalize path flow**

In `anki_forge/src/runtime/normalize.rs`, change imports to include:

```rust
use crate::{normalize_with_options, MediaPolicy, NormalizeOptions};
```

Before returning, compute options:

```rust
let base_dir = input_path
    .parent()
    .map(Path::to_path_buf)
    .unwrap_or_else(|| Path::new(".").to_path_buf());
let media_store_dir = base_dir.join(".anki-forge-media");

Ok(normalize_with_options(
    NormalizationRequest::new(document),
    NormalizeOptions {
        base_dir,
        media_store_dir,
        media_policy: MediaPolicy::default_strict(),
    },
))
```

- [ ] **Step 9: Update runtime build path flow**

In `anki_forge/src/runtime/build.rs`, change target construction to:

```rust
let media_store_dir = input_path
    .parent()
    .map(|parent| parent.join(".anki-forge-media"))
    .unwrap_or_else(|| Path::new(".anki-forge-media").to_path_buf());
let artifact_target =
    BuildArtifactTarget::new(artifacts_dir.as_ref().to_path_buf(), "artifacts")
        .with_media_store_dir(media_store_dir);
```

- [ ] **Step 10: Update public re-exports**

In `anki_forge/src/lib.rs`, replace `NormalizedMedia` in the re-export list with:

```rust
AuthoringMediaSource, DiagnosticBehavior, MediaBinding, MediaObject, MediaPolicy,
MediaReference, MediaReferenceResolution, NormalizeOptions,
```

Also re-export `normalize_with_options`.

- [ ] **Step 11: Run high-level tests**

Run:

```bash
cargo test -p anki_forge --test deck_export_tests deck_build_uses_cas_media_without_normalized_base64_payload -v
cargo test -p anki_forge -v
```

Expected: PASS.

- [ ] **Step 12: Commit facade updates**

Run:

```bash
git add anki_forge/src/deck/media.rs anki_forge/src/deck/model.rs anki_forge/src/deck/lowering.rs anki_forge/src/deck/export.rs anki_forge/src/product/lowering.rs anki_forge/src/runtime/normalize.rs anki_forge/src/runtime/build.rs anki_forge/src/lib.rs anki_forge/tests
git commit -m "feat: route high-level media through cas normalize"
```

## Task 10: Fixture Migration and Full Verification

**Files:**
- Modify: `contracts/fixtures/**/*.json`
- Modify: `contracts/fixtures/**/*.yaml`
- Modify: `contracts/fixtures/phase3/expected/*.json`
- Modify: affected test helpers in `writer_core/tests/build_tests.rs`, `writer_core/tests/inspect_tests.rs`, `anki_forge/tests/*`, and `authoring_core/tests/*`

- [ ] **Step 1: Locate all remaining legacy media payload references**

Run:

```bash
rg -n '"data_base64"|"NormalizedMedia"|'\''data_base64'\''' .
```

Expected: Output only from authoring inline-byte source schema/tests/helpers and historical spec text. No `NormalizedMedia` type and no writer-side `data_base64` read path should remain.

- [ ] **Step 2: Rewrite JSON fixtures**

For each `contracts/fixtures/**/input/authoring-ir.json` with legacy media entries, change:

```json
{
  "filename": "chart-basic.png",
  "mime": "image/png",
  "data_base64": "..."
}
```

to:

```json
{
  "id": "media:chart-basic.png",
  "desired_filename": "chart-basic.png",
  "source": {
    "kind": "path",
    "path": "../assets/chart-basic.png"
  },
  "declared_mime": "image/png"
}
```

Use the actual asset filename for each fixture. Keep the path relative to the authoring JSON file's parent directory.

- [ ] **Step 3: Update expected fixture files**

Run the fixture gate:

```bash
cargo test -p contract_tools --test fixture_gate_tests -v
```

Expected: FAIL with fixture diffs showing old media shapes.

Update these expected files by replacing legacy `media` arrays with `media_objects`, `media_bindings`, and `media_references`:

```text
contracts/fixtures/phase3/expected/basic.build.json
contracts/fixtures/phase3/expected/basic.inspect.json
contracts/fixtures/phase3/expected/cloze.build.json
contracts/fixtures/phase3/expected/cloze.inspect.json
contracts/fixtures/phase3/expected/image-occlusion.build.json
contracts/fixtures/phase3/expected/image-occlusion.inspect.json
```

Use the values emitted by the failing fixture gate output for each fixture's
BLAKE3, SHA-1, size, MIME, binding id, and reference rows. For inspect expected
files:

- staging inspect observations may include `binding_id`, `object_id`, and `object_ref`
- APKG inspect observations must include only observable fields such as `filename`, `size`, and `sha1`
- no expected normalized or writer fixture may include top-level `media` with `data_base64`

Run the fixture gate again:

```bash
cargo test -p contract_tools --test fixture_gate_tests -v
```

Expected: PASS.

- [ ] **Step 4: Run package and contract tests**

Run:

```bash
cargo test -p authoring_core -v
cargo test -p writer_core -v
cargo test -p anki_forge -v
cargo test -p contract_tools -v
```

Expected: PASS.

- [ ] **Step 5: Run workspace tests**

Run:

```bash
cargo test --workspace -v
```

Expected: PASS.

- [ ] **Step 6: Run final legacy payload scan**

Run:

```bash
rg -n '"media"\s*:\s*\[|"data_base64"|NormalizedMedia|decode\(media\.data_base64|normalized_ir\.media' authoring_core writer_core anki_forge contracts
```

Expected: No writer/normalized legacy payload usage. Matches for authoring inline bytes are allowed only when nested under `"source": { "kind": "inline_bytes" }`.

- [ ] **Step 7: Commit fixture and verification updates**

Run:

```bash
git add contracts authoring_core writer_core anki_forge
git commit -m "test: migrate media fixtures to cas pipeline"
```

## Final Verification

- [ ] **Step 1: Run formatting and tests**

Run:

```bash
cargo fmt --all -- --check
cargo test --workspace -v
```

Expected: both commands PASS.

- [ ] **Step 2: Confirm git status**

Run:

```bash
git status --short
```

Expected: no output.

## Execution Notes

Execute tasks in order. Do not start writer changes before normalized IR compiles and media ingest tests pass. Commit after every task so regressions can be bisected.
