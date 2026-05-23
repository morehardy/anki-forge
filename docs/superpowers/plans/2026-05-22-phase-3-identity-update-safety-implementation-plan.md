# Phase 3 Identity Update Safety Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement Phase 3 identity/update-safety so `Project::build` can preserve Anki note GUIDs across previous APKGs, lockfiles, and current Product input while producing contract-backed diagnostics and reports.

**Architecture:** Add an `update_safety` slice in `anki_forge` that owns the identity index, lockfile, reconcile, and diagnostics classifier. Extend `writer_core` with a sorted `WriterGuidPlan` and note metadata embedding/inspection. Keep `Project::build` as the orchestrator: validate and normalize, build current identity, load baselines, reconcile, call writer with a GUID plan, optionally write the lockfile, and return a `BuildReport` summary.

**Tech Stack:** Rust workspace (`anki_forge`, `writer_core`, `contract_tools`), `serde`/`serde_json`, existing canonical JSON helpers, `rusqlite` APKG inspection, JSON Schema contracts, `cargo test`, contract fixture gates.

**Commit Rhythm:** One commit per task after that task's tests pass. During a task, revise freely before committing; do not create a separate commit per step.

---

## Source Inputs

- Design spec: `docs/superpowers/specs/2026-05-22-phase-3-identity-update-safety-design.md`
- Current Product build path: `anki_forge/src/product/project.rs`
- Build API/report models: `anki_forge/src/build/options.rs`, `anki_forge/src/build/report.rs`
- Writer build/APKG path: `writer_core/src/build.rs`, `writer_core/src/apkg.rs`, `writer_core/src/inspect.rs`
- Existing Product identity code: `anki_forge/src/product/notetype.rs`, `anki_forge/src/product/note.rs`, `anki_forge/src/deck/identity.rs`
- Contracts: `contracts/schema`, `contracts/semantics`, `contracts/fixtures`
- Confirmed helper signatures: `writer_core/src/policy.rs` exposes `pub fn policy_ref(id: &str, version: &str) -> String`; `writer_core/src/canonical_json.rs` exposes `pub fn to_canonical_json(value: &impl serde::Serialize) -> anyhow::Result<String>` through `writer_core/src/lib.rs`; `anki_forge/src/diagnostics/mod.rs` exposes `DiagnosticCode::new` and `DiagnosticCode::as_str`; `anki_forge/src/build/report.rs` derives `Default` for `BuildCounts`.

## Scope Boundary

This plan implements Phase 3 update safety. It does not implement the full Phase 4 semantic diff/risk policy, a pruning command for absent lockfile entries, a public typed Rust API for `IdentityIndex`, Python bindings, or automated desktop Anki control. It does include a lightweight manual oracle checklist and an early metadata carrier probe.

## File Structure

Create these focused modules:

- `anki_forge/src/update_safety/mod.rs`: module exports and public crate-internal facade.
- `anki_forge/src/update_safety/model.rs`: identity index, lockfile, summary, source, and GUID plan mirror types.
- `anki_forge/src/update_safety/diagnostics.rs`: `UPDATE.*` code constants, severity mapping, and shared classifier helpers for limitations and diagnostics.
- `anki_forge/src/update_safety/current.rs`: build current `IdentityIndex` from `Project`, normalized IR, writer policy, and source maps.
- `anki_forge/src/update_safety/lockfile.rs`: read, validate, write atomic lockfiles.
- `anki_forge/src/update_safety/baseline.rs`: convert previous APKG inspect output and lockfile data into baseline identity indexes.
- `anki_forge/src/update_safety/reconcile.rs`: reconcile current identity with previous APKG and lockfile baselines.
- `anki_forge/src/update_safety/merge_safety.rs`: compare notetype, field, and template merge metadata.
- `anki_forge/src/update_safety/report.rs`: build `UpdateSafetySummary`, aggregate high-volume diagnostics, and partial reports.

Modify these existing files:

- `anki_forge/src/lib.rs`: expose `update_safety` internally and re-export user-facing build types.
- `anki_forge/src/build/options.rs`: add `UpdateSafetyMode` and update-safety builder fields/methods.
- `anki_forge/src/build/report.rs`: add `UpdateSafetySummary` and `BaselineSourceSummary`.
- `anki_forge/src/build/mod.rs`: export new build/report types.
- `anki_forge/src/product/project.rs`: orchestrate update-safety flow in `Project::build`.
- `writer_core/src/model.rs`: add `WriterGuidPlan`, `WriterGuidAssignment`, and note identity metadata structs.
- `writer_core/src/build.rs`: accept optional writer GUID plan.
- `writer_core/src/apkg.rs`: write selected note GUIDs and merge `notes.data.anki_forge_identity`.
- `writer_core/src/inspect.rs`: read `notes.data` identity metadata from APKGs.
- `writer_core/src/lib.rs`: re-export writer GUID plan types.
- `contracts/schema/*.schema.json`: add identity index, lockfile, and update-safety summary schemas.
- `contracts/semantics/identity-update-safety.md`: document Phase 3 semantics.
- `contracts/errors/error-registry.yaml`: add `UPDATE.*` codes.
- Tests in `anki_forge/tests`, `writer_core/tests`, and `contract_tools/tests`.

---

### Task 1: Contract Schemas and Diagnostic Registry

**Files:**
- Create: `contracts/schema/identity-index.schema.json`
- Create: `contracts/schema/identity-lockfile.schema.json`
- Create: `contracts/schema/update-safety-summary.schema.json`
- Create: `contracts/semantics/identity-update-safety.md`
- Modify: `contracts/errors/error-registry.yaml`
- Modify: `contract_tools/tests/schema_gate_tests.rs`
- Modify: `contract_tools/tests/fixture_gate_tests.rs`

- [ ] **Step 1: Add schema gate tests that fail before schemas exist**

Add this test block to `contract_tools/tests/schema_gate_tests.rs`:

```rust
#[test]
fn phase3_update_safety_schemas_are_valid_json_schema() {
    let root = contracts_root();
    for relative in [
        "schema/identity-index.schema.json",
        "schema/identity-lockfile.schema.json",
        "schema/update-safety-summary.schema.json",
    ] {
        let path = root.join(relative);
        let raw = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("read {relative}: {err}"));
        let json: serde_json::Value = serde_json::from_str(&raw)
            .unwrap_or_else(|err| panic!("parse {relative}: {err}"));
        jsonschema::JSONSchema::compile(&json)
            .unwrap_or_else(|err| panic!("compile {relative}: {err}"));
    }
}
```

Run: `cargo test -p contract_tools phase3_update_safety_schemas_are_valid_json_schema`

Expected: FAIL with a missing file error for `schema/identity-index.schema.json`.

- [ ] **Step 2: Add minimal schemas with exact required fields**

Create `contracts/schema/identity-index.schema.json`:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["schema_version", "source_kind", "source_ref", "writer_policy_ref", "project_stable_id", "notes", "notetypes", "limitations"],
  "additionalProperties": false,
  "properties": {
    "schema_version": { "const": "identity-index-v1" },
    "source_kind": { "enum": ["current", "previous_apkg", "lockfile"] },
    "source_ref": { "enum": ["current", "baseline.previous_apkg.primary", "baseline.identity_lockfile.primary"] },
    "writer_policy_ref": { "type": "string", "pattern": "^[^@\\u0000-\\u001f]+@[^@\\u0000-\\u001f]+$" },
    "project_stable_id": { "type": ["string", "null"] },
    "notes": {
      "type": "array",
      "items": { "$ref": "#/definitions/note_identity_entry" }
    },
    "notetypes": {
      "type": "array",
      "items": { "$ref": "#/definitions/notetype_identity_entry" }
    },
    "limitations": {
      "type": "array",
      "items": { "type": "string", "minLength": 1 },
      "uniqueItems": true
    }
  },
  "definitions": {
    "note_identity_entry": {
      "type": "object",
      "required": ["stable_id", "anki_guid", "current_guid_candidate", "guid_derivation_version", "note_type_id", "recipe_id", "provenance", "used_override", "entry_lifecycle", "source_path", "recovery_method"],
      "additionalProperties": false,
      "properties": {
        "stable_id": { "type": "string", "minLength": 1 },
        "normalized_note_id": { "type": "string", "minLength": 1 },
        "anki_guid": { "type": "string", "minLength": 1 },
        "current_guid_candidate": { "type": "string", "minLength": 1 },
        "guid_derivation_version": { "const": "guid.raw-stable-id.v1" },
        "note_type_id": { "type": "string", "minLength": 1 },
        "recipe_id": { "type": "string", "minLength": 1 },
        "canonical_payload_hash": { "type": "string", "pattern": "^blake3:[0-9a-f]{64}$" },
        "provenance": { "enum": ["ExplicitStableId", "InferredFromNoteFields", "InferredFromNotetypeFields", "InferredFromStockRecipe", "unknown_baseline"] },
        "used_override": { "type": "boolean" },
        "entry_lifecycle": { "enum": ["active", "absent_from_current"] },
        "source_path": { "type": "string" },
        "recovery_method": { "enum": ["current_resolution", "embedded_metadata", "lockfile_join", "guid_equals_stable_id", "unrecoverable"] }
      }
    },
    "notetype_identity_entry": {
      "type": "object",
      "required": ["note_type_id", "anki_model_id", "name", "fields", "templates"],
      "additionalProperties": false,
      "properties": {
        "note_type_id": { "type": "string", "minLength": 1 },
        "anki_model_id": { "type": ["integer", "null"] },
        "name": { "type": "string", "minLength": 1 },
        "fields": { "type": "array", "items": { "$ref": "#/definitions/field_merge_entry" } },
        "templates": { "type": "array", "items": { "$ref": "#/definitions/template_merge_entry" } }
      }
    },
    "field_merge_entry": {
      "type": "object",
      "required": ["field_key", "field_name", "ord", "config_id", "tag"],
      "additionalProperties": false,
      "properties": {
        "field_key": { "type": "string", "minLength": 1 },
        "field_name": { "type": "string", "minLength": 1 },
        "ord": { "type": "integer", "minimum": 0 },
        "config_id": { "type": "integer" },
        "tag": { "type": "integer" }
      }
    },
    "template_merge_entry": {
      "type": "object",
      "required": ["template_key", "template_name", "ord", "config_id"],
      "additionalProperties": false,
      "properties": {
        "template_key": { "type": "string", "minLength": 1 },
        "template_name": { "type": "string", "minLength": 1 },
        "ord": { "type": "integer", "minimum": 0 },
        "config_id": { "type": "integer" }
      }
    }
  }
}
```

Create `contracts/schema/identity-lockfile.schema.json`:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["schema_version", "project_stable_id", "writer_policy_ref", "identity_index", "generated_by"],
  "additionalProperties": false,
  "properties": {
    "schema_version": { "const": "identity-lockfile-v1" },
    "project_stable_id": { "type": "string", "minLength": 1 },
    "writer_policy_ref": { "type": "string", "pattern": "^[^@\\u0000-\\u001f]+@[^@\\u0000-\\u001f]+$" },
    "identity_index": {
      "type": "object",
      "required": ["schema_version", "source_kind", "source_ref", "writer_policy_ref", "project_stable_id", "notes", "notetypes", "limitations"],
      "additionalProperties": true,
      "properties": {
        "schema_version": { "const": "identity-index-v1" },
        "source_kind": { "type": "string", "minLength": 1 },
        "source_ref": { "type": "string", "minLength": 1 },
        "project_stable_id": { "type": "string", "minLength": 1 },
        "writer_policy_ref": { "type": "string", "minLength": 1 },
        "notes": { "type": "array" },
        "notetypes": { "type": "array" },
        "limitations": { "type": "array", "items": { "type": "string" } }
      }
    },
    "generated_by": {
      "type": "object",
      "required": ["tool", "tool_version", "writer_policy_ref"],
      "additionalProperties": false,
      "properties": {
        "tool": { "const": "anki-forge" },
        "tool_version": { "type": "string", "minLength": 1 },
        "writer_policy_ref": { "type": "string", "minLength": 1 }
      }
    }
  }
}
```

Create `contracts/schema/update-safety-summary.schema.json`:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "required": ["mode", "baseline_sources", "notes_preserved", "notes_derived", "notes_failed", "baseline_conflicts", "blocking_diagnostics", "lockfile_written"],
  "additionalProperties": false,
  "properties": {
    "mode": { "enum": ["disabled", "report_only", "strict"] },
    "baseline_sources": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["source_kind", "source_ref", "status", "used_for_reconcile", "limitations", "diagnostic_codes"],
        "additionalProperties": false,
        "properties": {
          "source_kind": { "enum": ["previous_apkg", "lockfile"] },
          "source_ref": { "type": "string", "minLength": 1 },
          "display_path": { "type": "string" },
          "status": { "enum": ["loaded", "partial", "unreadable", "ignored_disabled", "schema_unsupported"] },
          "used_for_reconcile": { "type": "boolean" },
          "limitations": { "type": "array", "items": { "type": "string", "minLength": 1 } },
          "diagnostic_codes": { "type": "array", "items": { "type": "string", "pattern": "^UPDATE\\.[A-Z0-9_]+$" } }
        }
      }
    },
    "notes_preserved": { "type": "integer", "minimum": 0 },
    "notes_derived": { "type": "integer", "minimum": 0 },
    "notes_failed": { "type": "integer", "minimum": 0 },
    "baseline_conflicts": { "type": "integer", "minimum": 0 },
    "blocking_diagnostics": { "type": "array", "items": { "type": "string", "pattern": "^UPDATE\\.[A-Z0-9_]+$" } },
    "lockfile_written": { "type": "boolean" }
  }
}
```

- [ ] **Step 3: Add update-safety semantics document**

Create `contracts/semantics/identity-update-safety.md`:

```markdown
# Identity Update Safety Semantics

Phase 3 update safety is built around `identity-index-v1`, `identity-lockfile-v1`, and `identity-note-v1`.

The only Phase 3 GUID derivation version is `guid.raw-stable-id.v1`. It sets `current_guid_candidate` to the resolved Product `stable_id` with no truncation or hashing. Changing this rule requires a new `guid_derivation_version`.

`IdentityIndex.source_ref` uses stable logical values:

- `current`
- `baseline.previous_apkg.primary`
- `baseline.identity_lockfile.primary`

Lockfile JSON must use lexicographic object-key ordering by Unicode scalar value after JSON string decoding. Arrays with semantic order preserve that order. Identity entries are sorted by `stable_id`.

Limitations describe source evidence and diagnostics describe build events. Implementations must derive overlapping values from one internal classifier pass.
```

- [ ] **Step 4: Register UPDATE diagnostic codes**

Append these entries under the existing top-level `codes:` list in `contracts/errors/error-registry.yaml`. The registry uses `id`, `status`, and `summary` fields:

```yaml
  - id: UPDATE.BASELINE_APKG_UNREADABLE
    status: active
    summary: previous APKG baseline could not be read
  - id: UPDATE.BASELINE_LOCKFILE_UNREADABLE
    status: active
    summary: identity lockfile could not be read or validated
  - id: UPDATE.BASELINE_SCHEMA_UNSUPPORTED
    status: active
    summary: baseline schema version is unsupported
  - id: UPDATE.BASELINE_CONFLICT_GUID
    status: active
    summary: previous APKG and lockfile selected different GUIDs
  - id: UPDATE.BASELINE_IGNORED_DISABLED
    status: active
    summary: baseline inputs were ignored because update safety is disabled
  - id: UPDATE.PROJECT_STABLE_ID_MISSING
    status: active
    summary: project stable id is missing for update-safety proof
  - id: UPDATE.PROJECT_STABLE_ID_MISMATCH
    status: active
    summary: project stable ids differ across current project and baselines
  - id: UPDATE.WRITER_POLICY_MISMATCH
    status: active
    summary: writer policy differs from baseline
  - id: UPDATE.WRITER_POLICY_REF_INVALID
    status: active
    summary: writer policy ref cannot be serialized safely
  - id: UPDATE.WRITER_GUID_PLAN_MISMATCH
    status: active
    summary: writer GUID plan does not match normalized notes
  - id: UPDATE.LOCKFILE_PATH_REQUIRED
    status: active
    summary: lockfile write was requested without a lockfile path
  - id: UPDATE.LOCKFILE_ABSENT_ENTRIES_HIGH
    status: active
    summary: lockfile contains many absent entries
  - id: UPDATE.NORMALIZED_NOTE_ID_MISMATCH
    status: active
    summary: normalized note id differs from stable id for raw stable-id GUID derivation
  - id: UPDATE.ANKI_GUID_INVALID
    status: active
    summary: selected Anki GUID violates writer validation
  - id: UPDATE.STABLE_ID_MISSING_IN_STRICT_MODE
    status: active
    summary: current output note has no resolved stable id in strict mode
  - id: UPDATE.STABLE_ID_DUPLICATE_IN_BASELINE
    status: active
    summary: baseline contains duplicate stable ids
  - id: UPDATE.GUID_DUPLICATE_IN_BASELINE
    status: active
    summary: baseline contains duplicate Anki GUIDs
  - id: UPDATE.GUID_DUPLICATE_AT_RECONCILE
    status: active
    summary: two stable ids selected the same Anki GUID
  - id: UPDATE.STABLE_ID_REMOVED_FROM_CURRENT
    status: active
    summary: lockfile stable id is absent from the current project
  - id: UPDATE.GUID_PRESERVED_FROM_PREVIOUS
    status: active
    summary: GUID was preserved from previous APKG
  - id: UPDATE.GUID_PRESERVED_FROM_LOCKFILE
    status: active
    summary: GUID was preserved from identity lockfile
  - id: UPDATE.GUID_DERIVATION_DRIFT
    status: active
    summary: selected baseline GUID differs from current derivation
  - id: UPDATE.GUID_DERIVED_FOR_NEW_NOTE
    status: active
    summary: GUID was derived for a new note
  - id: UPDATE.IDENTITY_PAYLOAD_CHANGED
    status: active
    summary: comparable identity payload hash changed
  - id: UPDATE.IDENTITY_PAYLOAD_HASH_DROPPED
    status: active
    summary: baseline identity payload hash was dropped
  - id: UPDATE.IDENTITY_PAYLOAD_HASH_ADDED
    status: active
    summary: current identity payload hash was added
  - id: UPDATE.NOTETYPE_SET_CHANGED
    status: active
    summary: notetype set changed
  - id: UPDATE.NOTETYPE_RENAMED
    status: active
    summary: notetype was renamed while stable id remained unchanged
  - id: UPDATE.FIELD_MERGE_ID_CHANGED
    status: active
    summary: field config id drifted for the same field key
  - id: UPDATE.FIELD_RENAMED
    status: active
    summary: field was renamed while config id remained unchanged
  - id: UPDATE.FIELD_ORD_CHANGED
    status: active
    summary: field ord changed
  - id: UPDATE.TEMPLATE_SET_CHANGED
    status: active
    summary: template set changed
  - id: UPDATE.TEMPLATE_MERGE_ID_CHANGED
    status: active
    summary: template config id drifted for the same template key
  - id: UPDATE.TEMPLATE_RENAMED
    status: active
    summary: template was renamed while config id remained unchanged
  - id: UPDATE.TEMPLATE_ORD_CHANGED
    status: active
    summary: template ord changed
  - id: UPDATE.BASELINE_IDENTITY_UNRECOVERABLE
    status: active
    summary: expected baseline identity could not be recovered
  - id: UPDATE.LOCKFILE_WRITTEN
    status: active
    summary: identity lockfile was written
  - id: UPDATE.LOCKFILE_WRITE_FAILED
    status: active
    summary: identity lockfile write failed
  - id: UPDATE.NOTE_DATA_METADATA_UNMERGEABLE
    status: active
    summary: note data could not be merged with identity metadata
```

- [ ] **Step 5: Run contract schema checks**

Run: `cargo test -p contract_tools phase3_update_safety_schemas_are_valid_json_schema`

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add contracts/schema/identity-index.schema.json contracts/schema/identity-lockfile.schema.json contracts/schema/update-safety-summary.schema.json contracts/semantics/identity-update-safety.md contracts/errors/error-registry.yaml contract_tools/tests/schema_gate_tests.rs contract_tools/tests/fixture_gate_tests.rs
git commit -m "contracts: add update safety schemas"
```

---

### Task 2: Build API and Report Surface

**Files:**
- Modify: `anki_forge/src/build/options.rs`
- Modify: `anki_forge/src/build/report.rs`
- Modify: `anki_forge/src/build/mod.rs`
- Modify: `anki_forge/src/prelude.rs`
- Test: `anki_forge/tests/public_api_boundary_tests.rs`
- Test: `anki_forge/tests/build_report_tests.rs`

- [ ] **Step 1: Add failing public API test for update-safety options**

Add to `anki_forge/tests/public_api_boundary_tests.rs`:

```rust
#[test]
fn build_options_expose_update_safety_builder_methods() {
    use anki_forge::build::{BuildOptions, UpdateSafetyMode};

    let options = BuildOptions::new()
        .compare_to("previous.apkg")
        .identity_lockfile("anki-forge.lock.json")
        .write_identity_lockfile(true)
        .update_safety(UpdateSafetyMode::ReportOnly);

    assert_eq!(options.compare_to.as_deref(), Some(std::path::Path::new("previous.apkg")));
    assert_eq!(
        options.identity_lockfile.as_deref(),
        Some(std::path::Path::new("anki-forge.lock.json"))
    );
    assert!(options.write_identity_lockfile);
    assert_eq!(options.update_safety, Some(UpdateSafetyMode::ReportOnly));
}
```

Run: `cargo test -p anki_forge build_options_expose_update_safety_builder_methods`

Expected: FAIL because `UpdateSafetyMode`, `compare_to`, `identity_lockfile`, `write_identity_lockfile`, and `update_safety` do not exist.

- [ ] **Step 2: Add `UpdateSafetyMode` and builder fields**

In `anki_forge/src/build/options.rs`, add:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateSafetyMode {
    Disabled,
    ReportOnly,
    Strict,
}
```

Extend `BuildOptions`:

```rust
pub struct BuildOptions {
    pub output: Option<PathBuf>,
    pub artifacts_dir: Option<PathBuf>,
    pub normalize_options: Option<ProjectNormalizeOptions>,
    pub inspect: bool,
    pub compare_to: Option<PathBuf>,
    pub identity_lockfile: Option<PathBuf>,
    pub write_identity_lockfile: bool,
    pub update_safety: Option<UpdateSafetyMode>,
}
```

Extend `Default`:

```rust
Self {
    output: None,
    artifacts_dir: None,
    normalize_options: None,
    inspect: true,
    compare_to: None,
    identity_lockfile: None,
    write_identity_lockfile: false,
    update_safety: None,
}
```

Add builder methods:

```rust
pub fn compare_to(mut self, path: impl Into<PathBuf>) -> Self {
    self.compare_to = Some(path.into());
    self
}

pub fn identity_lockfile(mut self, path: impl Into<PathBuf>) -> Self {
    self.identity_lockfile = Some(path.into());
    self
}

pub fn write_identity_lockfile(mut self, write: bool) -> Self {
    self.write_identity_lockfile = write;
    self
}

pub fn update_safety(mut self, mode: UpdateSafetyMode) -> Self {
    self.update_safety = Some(mode);
    self
}
```

- [ ] **Step 3: Export the new mode**

In `anki_forge/src/build/mod.rs`, change the options export:

```rust
pub use options::{
    BuildOptions, ProjectDeclaredMimeMismatchBehavior, ProjectMediaDiagnosticBehavior,
    ProjectMediaPolicy, ProjectMediaPolicyError, ProjectNormalizeOptions, UpdateSafetyMode,
};
```

In `anki_forge/src/prelude.rs`, add `UpdateSafetyMode` to the existing build exports.

- [ ] **Step 4: Add failing report-summary serialization-ish unit test**

Add to `anki_forge/tests/build_report_tests.rs`:

```rust
#[test]
fn build_report_can_carry_update_safety_summary() {
    use anki_forge::build::{
        BaselineSourceSummary, BuildCounts, BuildMetrics, BuildReport, MediaSummary,
        UpdateSafetySummary,
    };

    let report = BuildReport {
        artifact: None,
        counts: BuildCounts::default(),
        media: MediaSummary::default(),
        diagnostics: vec![],
        metrics: BuildMetrics::default(),
        inspect: None,
        update_safety: Some(UpdateSafetySummary {
            mode: "strict".into(),
            baseline_sources: vec![BaselineSourceSummary {
                source_kind: "previous_apkg".into(),
                source_ref: "baseline.previous_apkg.primary".into(),
                display_path: Some("previous.apkg".into()),
                status: "loaded".into(),
                used_for_reconcile: true,
                limitations: vec![],
                diagnostic_codes: vec![],
            }],
            notes_preserved: 1,
            notes_derived: 0,
            notes_failed: 0,
            baseline_conflicts: 0,
            blocking_diagnostics: vec![],
            lockfile_written: false,
        }),
        status: "success".into(),
    };

    let summary = report.update_safety.as_ref().expect("summary");
    assert_eq!(summary.mode, "strict");
    assert_eq!(summary.baseline_sources[0].source_ref, "baseline.previous_apkg.primary");
}
```

Run: `cargo test -p anki_forge build_report_can_carry_update_safety_summary`

Expected: FAIL because `UpdateSafetySummary`, `BaselineSourceSummary`, and `BuildReport.update_safety` do not exist.

- [ ] **Step 5: Implement report types**

In `anki_forge/src/build/report.rs`, add:

```rust
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BaselineSourceSummary {
    pub source_kind: String,
    pub source_ref: String,
    pub display_path: Option<String>,
    pub status: String,
    pub used_for_reconcile: bool,
    pub limitations: Vec<String>,
    pub diagnostic_codes: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UpdateSafetySummary {
    pub mode: String,
    pub baseline_sources: Vec<BaselineSourceSummary>,
    pub notes_preserved: usize,
    pub notes_derived: usize,
    pub notes_failed: usize,
    pub baseline_conflicts: usize,
    pub blocking_diagnostics: Vec<String>,
    pub lockfile_written: bool,
}
```

Extend `BuildReport`:

```rust
pub update_safety: Option<UpdateSafetySummary>,
```

Update every `BuildReport` struct literal in `anki_forge/src/product/project.rs` and tests to set `update_safety: None`.

Export in `anki_forge/src/build/mod.rs`:

```rust
pub use report::{
    ApkgArtifact, BaselineSourceSummary, BuildCounts, BuildError, BuildFailureCause, BuildMetrics,
    BuildReport, InspectSummary, MediaSummary, UpdateSafetySummary,
};
```

- [ ] **Step 6: Run focused tests**

Run:

```bash
cargo test -p anki_forge build_options_expose_update_safety_builder_methods
cargo test -p anki_forge build_report_can_carry_update_safety_summary
```

Expected: both PASS.

- [ ] **Step 7: Commit**

```bash
git add anki_forge/src/build/options.rs anki_forge/src/build/report.rs anki_forge/src/build/mod.rs anki_forge/src/prelude.rs anki_forge/tests/public_api_boundary_tests.rs anki_forge/tests/build_report_tests.rs anki_forge/src/product/project.rs
git commit -m "api: add update safety build options"
```

---

### Task 3: Identity Models, Diagnostics Classifier, and Mode Selection

**Files:**
- Create: `anki_forge/src/update_safety/mod.rs`
- Create: `anki_forge/src/update_safety/model.rs`
- Create: `anki_forge/src/update_safety/diagnostics.rs`
- Create: `anki_forge/src/update_safety/report.rs`
- Modify: `anki_forge/src/lib.rs`
- Test: `anki_forge/tests/update_safety_model_tests.rs`

- [ ] **Step 1: Add failing tests for mode selection and classifier dual output**

Create `anki_forge/tests/update_safety_model_tests.rs`:

```rust
use anki_forge::build::{BuildOptions, UpdateSafetyMode};
use anki_forge::update_safety::{
    classify_project_stable_id_missing, effective_mode, validate_writer_policy_ref,
    EvidenceCondition, EffectiveMode,
};

#[test]
fn effective_mode_uses_explicit_disabled_even_with_baselines() {
    let options = BuildOptions::new()
        .compare_to("previous.apkg")
        .identity_lockfile("anki-forge.lock.json")
        .write_identity_lockfile(true)
        .update_safety(UpdateSafetyMode::Disabled);

    assert_eq!(effective_mode(&options).unwrap(), EffectiveMode::Disabled);
}

#[test]
fn effective_mode_requires_lockfile_path_when_writing() {
    let err = effective_mode(&BuildOptions::new().write_identity_lockfile(true))
        .expect_err("missing lockfile path should fail option validation");

    assert_eq!(err.code.as_str(), "UPDATE.LOCKFILE_PATH_REQUIRED");
}

#[test]
fn effective_mode_upgrades_identity_lockfile_to_strict() {
    let options = BuildOptions::new()
        .identity_lockfile("anki-forge.lock.json")
        .write_identity_lockfile(true);

    assert_eq!(effective_mode(&options).unwrap(), EffectiveMode::Strict);
}

#[test]
fn classifier_returns_limitation_and_diagnostic() {
    let classified = classify_project_stable_id_missing(EvidenceCondition::StrictCompareOnly);

    assert_eq!(classified.limitation.as_deref(), Some("project_stable_id_missing"));
    assert_eq!(classified.diagnostic_code.as_deref(), Some("UPDATE.PROJECT_STABLE_ID_MISSING"));
    assert_eq!(classified.severity, anki_forge::diagnostics::Severity::Warning);

    let classified = classify_project_stable_id_missing(EvidenceCondition::LockfileRequired);
    assert_eq!(classified.diagnostic_code.as_deref(), Some("UPDATE.PROJECT_STABLE_ID_MISSING"));
    assert_eq!(classified.severity, anki_forge::diagnostics::Severity::Error);
}

#[test]
fn writer_policy_ref_rejects_at_sign_and_control_characters() {
    let err = validate_writer_policy_ref("writer@policy", "1.0.0")
        .expect_err("policy id containing @ is invalid");

    assert_eq!(err.code.as_str(), "UPDATE.WRITER_POLICY_REF_INVALID");

    let err = validate_writer_policy_ref("writer-policy.default", "1.0\n0")
        .expect_err("policy version containing control character is invalid");

    assert_eq!(err.code.as_str(), "UPDATE.WRITER_POLICY_REF_INVALID");
}
```

Run: `cargo test -p anki_forge update_safety_model_tests`

Expected: FAIL because the `update_safety` module does not exist.

- [ ] **Step 2: Add module exports**

Create `anki_forge/src/update_safety/mod.rs`:

```rust
pub mod diagnostics;
pub mod model;
pub mod report;

pub use diagnostics::{
    classify_project_stable_id_missing, EvidenceCondition, UpdateDiagnosticClass,
};
pub use model::{EffectiveMode, ModeSelectionError};
pub use model::{effective_mode, validate_writer_policy_ref};
```

Modify `anki_forge/src/lib.rs`:

```rust
pub mod update_safety;
```

- [ ] **Step 3: Implement mode selection**

Create `anki_forge/src/update_safety/model.rs` with these initial definitions:

```rust
use crate::build::{BuildOptions, UpdateSafetyMode};
use crate::diagnostics::{DiagnosticCode, Severity};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectiveMode {
    Disabled,
    ReportOnly,
    Strict,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModeSelectionError {
    pub code: DiagnosticCode,
    pub severity: Severity,
    pub message: String,
}

pub fn effective_mode(options: &BuildOptions) -> Result<EffectiveMode, ModeSelectionError> {
    if options.write_identity_lockfile && options.identity_lockfile.is_none() {
        return Err(ModeSelectionError {
            code: DiagnosticCode::new("UPDATE.LOCKFILE_PATH_REQUIRED"),
            severity: Severity::Error,
            message: "write_identity_lockfile(true) requires identity_lockfile(path)".into(),
        });
    }

    if let Some(mode) = options.update_safety {
        return Ok(match mode {
            UpdateSafetyMode::Disabled => EffectiveMode::Disabled,
            UpdateSafetyMode::ReportOnly => EffectiveMode::ReportOnly,
            UpdateSafetyMode::Strict => EffectiveMode::Strict,
        });
    }

    if options.identity_lockfile.is_some() || options.compare_to.is_some() {
        return Ok(EffectiveMode::Strict);
    }

    Ok(EffectiveMode::Disabled)
}

pub fn validate_writer_policy_ref(id: &str, version: &str) -> Result<String, ModeSelectionError> {
    let invalid = id.is_empty()
        || version.is_empty()
        || id.contains('@')
        || version.contains('@')
        || id.chars().any(char::is_control)
        || version.chars().any(char::is_control);
    if invalid {
        return Err(ModeSelectionError {
            code: DiagnosticCode::new("UPDATE.WRITER_POLICY_REF_INVALID"),
            severity: Severity::Error,
            message: "writer policy id and version must be non-empty and must not contain @ or control characters".into(),
        });
    }
    Ok(writer_core::policy_ref(id, version))
}
```

- [ ] **Step 4: Implement shared classifier skeleton**

Create `anki_forge/src/update_safety/diagnostics.rs`:

```rust
use crate::diagnostics::Severity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceCondition {
    StrictCompareOnly,
    LockfileRequired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateDiagnosticClass {
    pub limitation: Option<&'static str>,
    pub diagnostic_code: Option<&'static str>,
    pub severity: Severity,
}

pub fn classify_project_stable_id_missing(condition: EvidenceCondition) -> UpdateDiagnosticClass {
    match condition {
        EvidenceCondition::StrictCompareOnly => UpdateDiagnosticClass {
            limitation: Some("project_stable_id_missing"),
            diagnostic_code: Some("UPDATE.PROJECT_STABLE_ID_MISSING"),
            severity: Severity::Warning,
        },
        EvidenceCondition::LockfileRequired => UpdateDiagnosticClass {
            limitation: Some("project_stable_id_missing"),
            diagnostic_code: Some("UPDATE.PROJECT_STABLE_ID_MISSING"),
            severity: Severity::Error,
        },
    }
}
```

- [ ] **Step 5: Run focused tests**

Run: `cargo test -p anki_forge update_safety_model_tests`

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add anki_forge/src/update_safety/mod.rs anki_forge/src/update_safety/model.rs anki_forge/src/update_safety/diagnostics.rs anki_forge/src/update_safety/report.rs anki_forge/src/lib.rs anki_forge/tests/update_safety_model_tests.rs
git commit -m "build: add update safety mode model"
```

---

### Task 4: Complete Product Identity Inputs for Current Index Generation

**Files:**
- Modify: `anki_forge/src/product/project.rs`
- Create: `anki_forge/src/update_safety/current.rs`
- Test: `anki_forge/tests/update_safety_current_index_tests.rs`

- [ ] **Step 1: Add failing test for strict current notes requiring stable ids**

Create `anki_forge/tests/update_safety_current_index_tests.rs`:

```rust
use anki_forge::build::{BuildOptions, UpdateSafetyMode};
use anki_forge::prelude::*;

#[test]
fn strict_update_safety_blocks_note_without_resolved_stable_id() {
    let root = tempfile::tempdir().expect("tempdir");
    let output = root.path().join("missing-stable-id.apkg");
    let mut project = Project::new("Strict Missing Identity").stable_id("strict-missing");

    project
        .add_note(Note::basic("hola", "hello"))
        .expect("add note without stable id");

    let err = project
        .build(
            BuildOptions::new()
                .output(&output)
                .update_safety(UpdateSafetyMode::Strict),
        )
        .expect_err("strict update safety should require stable ids");

    assert!(err.report.diagnostic_codes().contains(&"UPDATE.STABLE_ID_MISSING_IN_STRICT_MODE".into()));
    assert!(!output.exists());
}
```

Run: `cargo test -p anki_forge strict_update_safety_blocks_note_without_resolved_stable_id`

Expected: FAIL because strict mode does not yet create this diagnostic.

- [ ] **Step 2: Add current identity index builder API**

Create `anki_forge/src/update_safety/current.rs`:

```rust
use authoring_core::NormalizedIr;

use crate::diagnostics::{Diagnostic, DiagnosticCode, Severity, SourcePath};
use writer_core::WriterPolicy;

use super::model::{validate_writer_policy_ref, EffectiveMode, IdentityIndex};

pub struct CurrentIdentityInput<'a> {
    pub project_stable_id: Option<&'a str>,
    pub normalized: &'a NormalizedIr,
    pub writer_policy: &'a WriterPolicy,
    pub mode: EffectiveMode,
}

pub struct CurrentIdentityOutput {
    pub index: IdentityIndex,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn build_current_identity_index(input: CurrentIdentityInput<'_>) -> CurrentIdentityOutput {
    let mut diagnostics = Vec::new();
    let mut index = IdentityIndex::current(input.project_stable_id, input.writer_policy);
    if let Err(err) = validate_writer_policy_ref(&input.writer_policy.id, &input.writer_policy.version) {
        diagnostics.push(Diagnostic {
            code: err.code,
            severity: err.severity,
            message: err.message,
            source: Some(SourcePath::new("writer_policy")),
            help: Some("remove @ and control characters from writer policy id/version".into()),
        });
    }

    for note in &input.normalized.notes {
        let stable_id = note.id.as_str();
        if matches!(input.mode, EffectiveMode::Strict) && stable_id.trim().is_empty() {
            diagnostics.push(Diagnostic {
                code: DiagnosticCode::new("UPDATE.STABLE_ID_MISSING_IN_STRICT_MODE"),
                severity: Severity::Error,
                message: "current output note has no resolved stable id in strict mode".into(),
                source: Some(SourcePath::new(format!("note[id='{}']", note.id))),
            help: Some("provide Note::stable_id(value) or an identity recipe".into()),
            });
            continue;
        }
        if is_invalid_anki_guid_candidate(stable_id) {
            diagnostics.push(Diagnostic {
                code: DiagnosticCode::new("UPDATE.ANKI_GUID_INVALID"),
                severity: Severity::Error,
                message: format!("stable id {stable_id:?} cannot be used as a Phase 3 Anki GUID candidate"),
                source: Some(SourcePath::new(format!("note[id='{}']", note.id))),
                help: Some("use a non-empty stable id without ASCII control characters and at most 255 bytes".into()),
            });
            continue;
        }
        index.push_current_note(note);
    }

    for notetype in &input.normalized.notetypes {
        index.push_current_notetype(notetype);
    }

    CurrentIdentityOutput { index, diagnostics }
}

fn is_invalid_anki_guid_candidate(value: &str) -> bool {
    value.is_empty() || value.len() > 255 || value.chars().any(char::is_control)
}
```

- [ ] **Step 3: Extend `model.rs` with identity index structs**

In `anki_forge/src/update_safety/model.rs`, add:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdentityIndex {
    pub schema_version: String,
    pub source_kind: String,
    pub source_ref: String,
    pub writer_policy_ref: String,
    pub project_stable_id: Option<String>,
    pub notes: Vec<NoteIdentityEntry>,
    pub notetypes: Vec<NotetypeIdentityEntry>,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NoteIdentityEntry {
    pub stable_id: String,
    pub normalized_note_id: Option<String>,
    pub anki_guid: String,
    pub current_guid_candidate: String,
    pub guid_derivation_version: String,
    pub note_type_id: String,
    pub recipe_id: String,
    pub canonical_payload_hash: Option<String>,
    pub provenance: String,
    pub used_override: bool,
    pub entry_lifecycle: String,
    pub source_path: String,
    pub recovery_method: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NotetypeIdentityEntry {
    pub note_type_id: String,
    pub anki_model_id: Option<i64>,
    pub name: String,
    pub fields: Vec<FieldMergeEntry>,
    pub templates: Vec<TemplateMergeEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FieldMergeEntry {
    pub field_key: String,
    pub field_name: String,
    pub ord: u32,
    pub config_id: i64,
    pub tag: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TemplateMergeEntry {
    pub template_key: String,
    pub template_name: String,
    pub ord: u32,
    pub config_id: i64,
}
```

Add methods:

```rust
impl IdentityIndex {
    pub fn current(project_stable_id: Option<&str>, writer_policy: &writer_core::WriterPolicy) -> Self {
        Self {
            schema_version: "identity-index-v1".into(),
            source_kind: "current".into(),
            source_ref: "current".into(),
            writer_policy_ref: writer_core::policy_ref(&writer_policy.id, &writer_policy.version),
            project_stable_id: project_stable_id.map(str::to_string),
            notes: vec![],
            notetypes: vec![],
            limitations: vec![],
        }
    }

    pub fn push_current_note(&mut self, note: &authoring_core::NormalizedNote) {
        self.notes.push(NoteIdentityEntry {
            stable_id: note.id.clone(),
            normalized_note_id: Some(note.id.clone()),
            anki_guid: note.id.clone(),
            current_guid_candidate: note.id.clone(),
            guid_derivation_version: "guid.raw-stable-id.v1".into(),
            note_type_id: note.notetype_id.clone(),
            recipe_id: "product.explicit-or-normalized.v1".into(),
            canonical_payload_hash: None,
            provenance: "ExplicitStableId".into(),
            used_override: false,
            entry_lifecycle: "active".into(),
            source_path: format!("note[id='{}']", note.id),
            recovery_method: "current_resolution".into(),
        });
    }

    pub fn push_current_notetype(&mut self, notetype: &authoring_core::NormalizedNotetype) {
        self.notetypes.push(NotetypeIdentityEntry {
            note_type_id: notetype.id.clone(),
            anki_model_id: None,
            name: notetype.name.clone(),
            fields: notetype
                .fields
                .iter()
                .enumerate()
                .map(|(ord, field)| FieldMergeEntry {
                    field_key: field.name.clone(),
                    field_name: field.name.clone(),
                    ord: ord as u32,
                    config_id: field.config_id,
                    tag: field.tag,
                })
                .collect(),
            templates: notetype
                .templates
                .iter()
                .enumerate()
                .map(|(ord, template)| TemplateMergeEntry {
                    template_key: template.name.clone(),
                    template_name: template.name.clone(),
                    ord: template.ord.unwrap_or(ord as u32),
                    config_id: template.config_id,
                })
                .collect(),
        });
    }
}
```

- [ ] **Step 4: Wire current identity into `Project::build` before writer**

In `anki_forge/src/product/project.rs`, after loading `writer_policy` and before creating `artifact_target`, add:

```rust
let update_mode = match crate::update_safety::effective_mode(&options) {
    Ok(mode) => mode,
    Err(err) => {
        diagnostics.push(Diagnostic {
            code: err.code,
            severity: err.severity,
            message: err.message,
            source: Some(SourcePath::new("build.options")),
            help: Some("provide identity_lockfile(path) when write_identity_lockfile(true) is set".into()),
        });
        let media = MediaSummary::from_normalized_ir(&normalized, &diagnostics);
        return Err(BuildError::new(
            BuildReport {
                artifact: None,
                counts: BuildCounts {
                    notes: normalized.notes.len(),
                    cards: count_phase1_cards_without_inspect(&normalized),
                    media: normalized.media_bindings.len(),
                },
                media,
                diagnostics,
                metrics: BuildMetrics { duration: started.elapsed() },
                inspect: None,
                update_safety: None,
                status: "invalid".into(),
            },
            BuildFailureCause::Diagnostics,
        ));
    }
};

if self.stable_id.is_none()
    && (options.compare_to.is_some()
        || options.identity_lockfile.is_some()
        || options.write_identity_lockfile)
{
    let condition = if options.identity_lockfile.is_some() || options.write_identity_lockfile {
        crate::update_safety::EvidenceCondition::LockfileRequired
    } else {
        crate::update_safety::EvidenceCondition::StrictCompareOnly
    };
    let classified = crate::update_safety::classify_project_stable_id_missing(condition);
    if let Some(code) = classified.diagnostic_code {
        diagnostics.push(Diagnostic {
            code: DiagnosticCode::new(code),
            severity: classified.severity,
            message: "project stable id is missing for update-safety proof".into(),
            source: Some(SourcePath::new("project.stable_id")),
            help: Some("set Project::stable_id(value) for update-safe builds".into()),
        });
    }
}

let current_identity = crate::update_safety::current::build_current_identity_index(
    crate::update_safety::current::CurrentIdentityInput {
        project_stable_id: self.stable_id.as_deref(),
        normalized: &normalized,
        writer_policy: &writer_policy,
        mode: update_mode,
    },
);
diagnostics.extend(current_identity.diagnostics);
if diagnostics.iter().any(|diagnostic| diagnostic.severity == Severity::Error) {
    let media = MediaSummary::from_normalized_ir(&normalized, &diagnostics);
    return Err(BuildError::new(
        BuildReport {
            artifact: None,
            counts: BuildCounts {
                notes: normalized.notes.len(),
                cards: count_phase1_cards_without_inspect(&normalized),
                media: normalized.media_bindings.len(),
            },
            media,
            diagnostics,
            metrics: BuildMetrics { duration: started.elapsed() },
            inspect: None,
            update_safety: None,
            status: "invalid".into(),
        },
        BuildFailureCause::Diagnostics,
    ));
}
```

If this makes `current` private, update `anki_forge/src/update_safety/mod.rs` to `pub mod current;`.

- [ ] **Step 5: Run failing test again**

Run: `cargo test -p anki_forge strict_update_safety_blocks_note_without_resolved_stable_id`

Expected: PASS.

- [ ] **Step 6: Add positive current-index test**

Append to `anki_forge/tests/update_safety_current_index_tests.rs`:

```rust
#[test]
fn strict_update_safety_allows_explicit_stable_id() {
    let root = tempfile::tempdir().expect("tempdir");
    let output = root.path().join("stable.apkg");
    let mut project = Project::new("Strict Stable").stable_id("strict-stable");

    project
        .add_note(Note::basic("hola", "hello").stable_id("es:hola"))
        .expect("add note");

    let report = project
        .build(
            BuildOptions::new()
                .output(&output)
                .update_safety(UpdateSafetyMode::Strict),
        )
        .expect("strict build with stable id");

    assert!(report.ensure_success().is_ok());
}

#[test]
fn strict_update_safety_blocks_invalid_anki_guid_candidate() {
    let root = tempfile::tempdir().expect("tempdir");
    let output = root.path().join("invalid-guid.apkg");
    let mut project = Project::new("Strict Invalid Guid").stable_id("strict-invalid-guid");

    project
        .add_note(Note::basic("hola", "hello").stable_id("bad\nid"))
        .expect("add note");

    let err = project
        .build(
            BuildOptions::new()
                .output(&output)
                .update_safety(UpdateSafetyMode::Strict),
        )
        .expect_err("invalid GUID candidate should block writer execution");

    assert!(err.report.diagnostic_codes().contains(&"UPDATE.ANKI_GUID_INVALID".into()));
    assert!(!output.exists());
}
```

Run: `cargo test -p anki_forge update_safety_current_index_tests`

Expected: all current-index tests PASS.

- [ ] **Step 7: Commit**

```bash
git add anki_forge/src/update_safety/model.rs anki_forge/src/update_safety/current.rs anki_forge/src/update_safety/mod.rs anki_forge/src/product/project.rs anki_forge/tests/update_safety_current_index_tests.rs
git commit -m "identity: build current update safety index"
```

---

### Task 5: Writer GUID Plan and Selected GUID Writing

**Files:**
- Modify: `writer_core/src/model.rs`
- Modify: `writer_core/src/build.rs`
- Modify: `writer_core/src/apkg.rs`
- Modify: `writer_core/src/lib.rs`
- Modify: `anki_forge/src/product/project.rs`
- Test: `writer_core/tests/build_tests.rs`

- [ ] **Step 1: Add failing writer test for GUID plan application**

Update the import list in `writer_core/tests/build_tests.rs` to include `build_with_guid_plan`, `inspect_apkg`, `WriterGuidAssignment`, and `WriterGuidPlan`.

Add to `writer_core/tests/build_tests.rs`:

```rust
#[test]
fn writer_guid_plan_overrides_notes_guid() {
    let root = unique_artifact_root("writer-guid-plan-overrides");
    let target = BuildArtifactTarget::new(root.clone(), "artifacts/writer-guid-plan-overrides");
    let mut normalized = sample_basic_normalized_ir();
    normalized.notes[0].id = "stable-note".into();

    let plan = WriterGuidPlan {
        assignments: vec![WriterGuidAssignment {
            normalized_note_id: "stable-note".into(),
            stable_id: "stable-note".into(),
            selected_anki_guid: "old-guid-from-baseline".into(),
            guid_derivation_version: "guid.raw-stable-id.v1".into(),
            source: "previous_apkg".into(),
        }],
    };

    let result = build_with_guid_plan(
        &normalized,
        &sample_writer_policy(),
        &sample_build_context(true),
        &target,
        Some(&plan),
    )
    .expect("build with guid plan");

    assert_eq!(result.result_status, "success");
    let report = inspect_apkg(root.join("package.apkg")).expect("inspect apkg");
    assert!(report
        .observations
        .references
        .iter()
        .any(|value| value["selector"] == "note[id='old-guid-from-baseline']"));
}
```

Run: `cargo test -p writer_core writer_guid_plan_overrides_notes_guid`

Expected: FAIL because `WriterGuidPlan`, `WriterGuidAssignment`, and `build_with_guid_plan` do not exist.

- [ ] **Step 2: Add writer GUID plan types**

In `writer_core/src/model.rs`, add:

```rust
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WriterGuidPlan {
    pub assignments: Vec<WriterGuidAssignment>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WriterGuidAssignment {
    pub normalized_note_id: String,
    pub stable_id: String,
    pub selected_anki_guid: String,
    pub guid_derivation_version: String,
    pub source: String,
}
```

In `writer_core/src/lib.rs`, keep `pub use model::*;` and change the build re-export from:

```rust
pub use build::build;
```

to:

```rust
pub use build::{build, build_with_guid_plan};
```

- [ ] **Step 3: Add build entrypoint with optional plan**

In `writer_core/src/build.rs`, keep the existing `build` signature and delegate:

```rust
pub fn build(
    normalized_ir: &NormalizedIr,
    writer_policy: &WriterPolicy,
    build_context: &BuildContext,
    artifact_target: &BuildArtifactTarget,
) -> Result<PackageBuildResult> {
    build_with_guid_plan(normalized_ir, writer_policy, build_context, artifact_target, None)
}

pub fn build_with_guid_plan(
    normalized_ir: &NormalizedIr,
    writer_policy: &WriterPolicy,
    build_context: &BuildContext,
    artifact_target: &BuildArtifactTarget,
    guid_plan: Option<&crate::model::WriterGuidPlan>,
) -> Result<PackageBuildResult> {
    if !build_context.materialize_staging {
        return Ok(error_result(
            writer_policy,
            build_context,
            "PHASE3.STAGING_DISABLED",
            "build_context.materialize_staging is false",
            "build",
            "materialize_staging",
            Some(format!("build-context={}", build_context.id)),
        ));
    }

    let package = match StagingPackage::from_normalized(normalized_ir, writer_policy, build_context)
    {
        Ok(package) => package,
        Err(diagnostics) => return Ok(invalid_result(writer_policy, build_context, diagnostics)),
    };

    let diagnostics = package.diagnostics().to_vec();
    let materialized = match package.materialize(artifact_target) {
        Ok(materialized) => materialized,
        Err(err) => {
            if let Some(media_err) = err.downcast_ref::<crate::media::MediaWriterError>() {
                return Ok(error_result_with_domain(
                    writer_policy,
                    build_context,
                    ErrorResultDetails {
                        code: media_err.diagnostic_code().into(),
                        summary: err.to_string(),
                        domain: "media".into(),
                        stage: "materialize_staging".into(),
                        operation: "write_media".into(),
                        path: media_err.diagnostic_path(),
                    },
                ));
            }
            return Ok(error_result(
                writer_policy,
                build_context,
                "PHASE3.STAGING_MATERIALIZATION_FAILED",
                err.to_string(),
                "materialize_staging",
                "write_manifest",
                Some(
                    artifact_target
                        .staging_manifest_path()
                        .display()
                        .to_string(),
                ),
            ));
        }
    };

    let apkg = if build_context.emit_apkg {
        match emit_apkg(&materialized, artifact_target, guid_plan) {
            Ok(apkg) => Some(apkg),
            Err(err) => {
                return Ok(apkg_error_result(
                    writer_policy,
                    build_context,
                    artifact_target,
                    err,
                ));
            }
        }
    } else {
        None
    };
    let mut result = success_result(writer_policy, build_context, materialized, diagnostics);
    if let Some(apkg) = apkg {
        result.apkg_ref = Some(apkg.apkg_ref);
        result.package_fingerprint = Some(apkg.package_fingerprint);
    }

    Ok(result)
}
```

- [ ] **Step 4: Validate exact GUID plan set in writer**

In `writer_core/src/apkg.rs`, change `emit_apkg` to accept `guid_plan: Option<&WriterGuidPlan>`. Build a map only after exact set validation:

```rust
fn validate_guid_plan(
    normalized_ir: &NormalizedIr,
    guid_plan: Option<&WriterGuidPlan>,
) -> anyhow::Result<std::collections::BTreeMap<String, WriterGuidAssignment>> {
    let Some(plan) = guid_plan else {
        return Ok(Default::default());
    };

    let expected: std::collections::BTreeSet<_> =
        normalized_ir.notes.iter().map(|note| note.id.as_str()).collect();
    let mut seen = std::collections::BTreeSet::new();
    let mut by_note = std::collections::BTreeMap::new();

    for assignment in &plan.assignments {
        if !seen.insert(assignment.normalized_note_id.as_str()) {
            anyhow::bail!(
                "UPDATE.WRITER_GUID_PLAN_MISMATCH: duplicate assignment for {}",
                assignment.normalized_note_id
            );
        }
        by_note.insert(assignment.normalized_note_id.clone(), assignment.clone());
    }

    let actual: std::collections::BTreeSet<_> = by_note.keys().map(String::as_str).collect();
    if expected != actual {
        anyhow::bail!(
            "UPDATE.WRITER_GUID_PLAN_MISMATCH: plan ids {:?} did not match normalized note ids {:?}",
            actual,
            expected
        );
    }

    Ok(by_note)
}
```

When inserting notes, use:

```rust
let guid = guid_assignments
    .get(&note.id)
    .map(|assignment| assignment.selected_anki_guid.as_str())
    .unwrap_or(note.id.as_str());
```

Use `guid` in the SQL parameter for `notes.guid`.

- [ ] **Step 5: Run writer test**

Run: `cargo test -p writer_core writer_guid_plan_overrides_notes_guid`

Expected: PASS.

- [ ] **Step 6: Add mismatch test**

Add to `writer_core/tests/build_tests.rs`:

```rust
#[test]
fn writer_guid_plan_mismatch_returns_update_diagnostic_error() {
    let root = unique_artifact_root("writer-guid-plan-mismatch");
    let target = BuildArtifactTarget::new(root, "artifacts/writer-guid-plan-mismatch");
    let normalized = sample_basic_normalized_ir();
    let plan = WriterGuidPlan { assignments: vec![] };

    let result = build_with_guid_plan(
        &normalized,
        &sample_writer_policy(),
        &sample_build_context(true),
        &target,
        Some(&plan),
    )
    .expect("build returns package result");

    assert_eq!(result.result_status, "error");
    assert!(result
        .diagnostics
        .items
        .iter()
        .any(|item| item.code == "UPDATE.WRITER_GUID_PLAN_MISMATCH"));
}
```

Update `apkg_error_result` in `writer_core/src/build.rs` before the fallback `PHASE3.APKG_EMISSION_FAILED` result:

```rust
if err.to_string().starts_with("UPDATE.WRITER_GUID_PLAN_MISMATCH") {
    return error_result_with_domain(
        writer_policy,
        build_context,
        ErrorResultDetails {
            code: "UPDATE.WRITER_GUID_PLAN_MISMATCH".into(),
            summary: err.to_string(),
            domain: "identity".into(),
            stage: "emit_apkg".into(),
            operation: "validate_guid_plan".into(),
            path: None,
        },
    );
}
```

- [ ] **Step 7: Update `anki_forge` call site**

In `anki_forge/src/product/project.rs`, keep using `crate::writer_build` during this task. Task 11 changes the orchestrator to call `writer_core::build_with_guid_plan` through a new crate-level alias after reconcile produces a real plan.

- [ ] **Step 8: Run focused writer tests**

Run: `cargo test -p writer_core writer_guid_plan`

Expected: both writer GUID plan tests PASS.

- [ ] **Step 9: Commit**

```bash
git add writer_core/src/model.rs writer_core/src/build.rs writer_core/src/apkg.rs writer_core/src/lib.rs writer_core/tests/build_tests.rs anki_forge/src/product/project.rs
git commit -m "writer: support selected note guid plan"
```

---

### Task 6: APKG Note Identity Metadata Embedding and Inspection

**Files:**
- Modify: `writer_core/src/model.rs`
- Modify: `writer_core/src/apkg.rs`
- Modify: `writer_core/src/inspect.rs`
- Test: `writer_core/tests/build_tests.rs`
- Test: `writer_core/tests/inspect_tests.rs`

- [ ] **Step 1: Add failing inspect test for `notes.data.anki_forge_identity`**

Update the import list in `writer_core/tests/inspect_tests.rs` to include `build_with_guid_plan`, `WriterGuidAssignment`, and `WriterGuidPlan`.

Add to `writer_core/tests/inspect_tests.rs`:

```rust
#[test]
fn inspect_apkg_reports_note_identity_metadata_from_notes_data() {
    let root = unique_artifact_root("inspect-note-identity-metadata");
    let target = BuildArtifactTarget::new(root.clone(), "artifacts/inspect-note-identity-metadata");
    let mut normalized = sample_basic_normalized_ir();
    normalized.notes[0].id = "stable-note".into();
    let plan = WriterGuidPlan {
        assignments: vec![WriterGuidAssignment {
            normalized_note_id: "stable-note".into(),
            stable_id: "stable-note".into(),
            selected_anki_guid: "stable-note".into(),
            guid_derivation_version: "guid.raw-stable-id.v1".into(),
            source: "current_derivation".into(),
        }],
    };

    let result = build_with_guid_plan(
        &normalized,
        &sample_writer_policy(),
        &sample_build_context(true),
        &target,
        Some(&plan),
    )
    .expect("build");
    assert_eq!(result.result_status, "success");

    let report = inspect_apkg(root.join("package.apkg")).expect("inspect apkg");
    let identity = report
        .observations
        .metadata
        .iter()
        .find(|value| value["selector"] == "note[guid='stable-note']::anki_forge_identity")
        .expect("identity metadata observation");

    assert_eq!(identity["stable_id"], "stable-note");
    assert_eq!(identity["selected_anki_guid"], "stable-note");
    assert_eq!(identity["schema_version"], "identity-note-v1");
}
```

Run: `cargo test -p writer_core inspect_apkg_reports_note_identity_metadata_from_notes_data`

Expected: FAIL because note identity metadata is not written or inspected.

- [ ] **Step 2: Add metadata struct**

In `writer_core/src/model.rs`, add:

```rust
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct NoteIdentityMetadata {
    pub schema_version: String,
    pub stable_id: String,
    pub recipe_id: String,
    pub canonical_payload_hash: Option<String>,
    pub current_guid_candidate: String,
    pub selected_anki_guid: String,
    pub guid_derivation_version: String,
    pub guid_source: String,
    pub recovery_method: String,
}
```

- [ ] **Step 3: Merge metadata into `notes.data`**

In `writer_core/src/apkg.rs`, add:

```rust
fn note_identity_metadata_for_assignment(
    assignment: Option<&WriterGuidAssignment>,
    note: &NormalizedNote,
) -> NoteIdentityMetadata {
    let selected = assignment
        .map(|assignment| assignment.selected_anki_guid.clone())
        .unwrap_or_else(|| note.id.clone());
    let source = assignment
        .map(|assignment| assignment.source.clone())
        .unwrap_or_else(|| "current_derivation".into());

    NoteIdentityMetadata {
        schema_version: "identity-note-v1".into(),
        stable_id: assignment
            .map(|assignment| assignment.stable_id.clone())
            .unwrap_or_else(|| note.id.clone()),
        recipe_id: "product.explicit-or-normalized.v1".into(),
        canonical_payload_hash: None,
        current_guid_candidate: note.id.clone(),
        selected_anki_guid: selected,
        guid_derivation_version: "guid.raw-stable-id.v1".into(),
        guid_source: source,
        recovery_method: "current_resolution".into(),
    }
}

fn merge_identity_note_data(existing: &str, metadata: &NoteIdentityMetadata) -> anyhow::Result<String> {
    let mut value = if existing.trim().is_empty() {
        serde_json::json!({})
    } else {
        serde_json::from_str(existing)
            .map_err(|err| anyhow::anyhow!("UPDATE.NOTE_DATA_METADATA_UNMERGEABLE: invalid notes.data JSON: {err}"))?
    };

    let Some(object) = value.as_object_mut() else {
        anyhow::bail!("UPDATE.NOTE_DATA_METADATA_UNMERGEABLE: notes.data must be a JSON object");
    };

    object.insert(
        "anki_forge_identity".into(),
        serde_json::to_value(metadata).expect("identity metadata serializes"),
    );
    Ok(serde_json::to_string(&value).expect("identity note data serializes"))
}
```

Use the function when preparing the `data` SQL parameter. If there is no existing `NormalizedNote` data field, pass `"{}"` as the `existing` value.

The exact integration point is `writer_core/src/apkg.rs` inside `populate_latest_collection`, in the `for note in &normalized_ir.notes` loop immediately before this SQL statement:

```rust
conn.execute(
    "insert into notes (id, guid, mid, mod, usn, tags, flds, sfld, csum, flags, data) values (?1, ?2, ?3, ?4, 0, ?5, ?6, ?7, ?8, 0, ?9)",
    rusqlite::params![
        note_row,
        guid,
        ntid,
        storage.mtime_secs,
        note.tags.join(" "),
        storage.flds,
        storage.sfld,
        storage.csum,
        merge_identity_note_data(
            "{}",
            &note_identity_metadata_for_assignment(guid_assignments.get(&note.id), note),
        )?
    ],
)?;
```

- [ ] **Step 4: Inspect metadata from APKG**

In `writer_core/src/inspect.rs`, add a storage field for APKG note metadata:

```rust
struct CollectionData {
    notetypes: Vec<NormalizedNotetype>,
    notes: Vec<NormalizedNote>,
    template_target_decks: Vec<ResolvedTemplateTargetDeck>,
    actual_card_decks: BTreeMap<(String, usize), String>,
    note_identity_metadata: Vec<serde_json::Value>,
}
```

In `inspect_apkg`, initialize and pass the metadata:

```rust
let mut note_identity_metadata = vec![];

if let Some(collection_bytes) = read_expected_collection_bytes(&mut archive, version)? {
    let collection = read_collection_data(&collection_bytes)?;
    normalized_ir.notetypes = collection.notetypes;
    normalized_ir.notes = collection.notes;
    template_target_decks = collection.template_target_decks;
    actual_card_decks = collection.actual_card_decks;
    note_identity_metadata = collection.note_identity_metadata;
    has_core_data = true;
}

let observations = build_observations(
    &normalized_ir,
    &media,
    &template_target_decks,
    &actual_card_decks,
    &note_identity_metadata,
);
```

Change `build_observations` to accept `note_identity_metadata: &[serde_json::Value]`, and change the counts metadata from an immutable single-entry vector to a mutable vector:

```rust
let mut metadata_entries = vec![json!({
    "selector": "counts",
    "notetype_count": normalized_ir.notetypes.len(),
    "template_count": template_entries.len(),
    "field_count": field_entries.len(),
    "note_count": note_entries.len(),
    "card_count": card_entries.len(),
    "media_count": media.len(),
    "evidence_refs": ["counts"],
})];
metadata_entries.extend(note_identity_metadata.iter().cloned());
```

In `read_collection_data`, change the notes query to include `data`:

```rust
let mut note_rows =
    conn.prepare("select id, guid, mid, mod, tags, flds, data from notes order by id")?;
```

Read `data`:

```rust
let data: String = row.get(6)?;
```

Inside the `query_map` closure, return both the `NormalizedNote` and an optional parsed identity observation:

```rust
let identity_metadata = serde_json::from_str::<serde_json::Value>(&data)
    .ok()
    .and_then(|value| value.get("anki_forge_identity").cloned())
    .map(|mut observed| {
        if let Some(object) = observed.as_object_mut() {
            object.insert(
                "selector".into(),
                serde_json::Value::String(format!("note[guid='{}']::anki_forge_identity", guid)),
            );
            object.insert(
                "evidence_refs".into(),
                serde_json::json!([format!("note-data:{}", guid)]),
            );
        }
        observed
    });

Ok((
    NormalizedNote {
        id: guid,
        notetype_id: notetype.id.clone(),
        deck_name: note_decks_by_row_id
            .get(&id)
            .cloned()
            .unwrap_or_else(|| "Default".into()),
        fields,
        tags: if tags.is_empty() {
            vec![]
        } else {
            tags.split(' ').map(|tag| tag.to_string()).collect()
        },
        mtime_secs: Some(mtime_secs),
    },
    identity_metadata,
))
```

Collect the query rows into `Vec<(NormalizedNote, Option<serde_json::Value>)>`, then split it before returning `CollectionData`:

```rust
let rows = note_rows
    .query_map([], |row| {
        // body above
    })?
    .collect::<std::result::Result<Vec<_>, _>>()?;
let mut notes = Vec::with_capacity(rows.len());
let mut note_identity_metadata = vec![];
for (note, identity) in rows {
    notes.push(note);
    if let Some(identity) = identity {
        note_identity_metadata.push(identity);
    }
}
```

- [ ] **Step 5: Map unmergeable data to update diagnostic**

In `writer_core/src/build.rs`, update `apkg_error_result` to detect `UPDATE.NOTE_DATA_METADATA_UNMERGEABLE` and return that code. Place this branch after the `UPDATE.WRITER_GUID_PLAN_MISMATCH` branch from Task 5 and before the fallback `PHASE3.APKG_EMISSION_FAILED` result:

```rust
if err.to_string().starts_with("UPDATE.NOTE_DATA_METADATA_UNMERGEABLE") {
    return error_result_with_domain(
        writer_policy,
        build_context,
        ErrorResultDetails {
            code: "UPDATE.NOTE_DATA_METADATA_UNMERGEABLE".into(),
            summary: err.to_string(),
            domain: "identity".into(),
            stage: "emit_apkg".into(),
            operation: "merge_note_data".into(),
            path: None,
        },
    );
}
```

- [ ] **Step 6: Run focused tests**

Run:

```bash
cargo test -p writer_core inspect_apkg_reports_note_identity_metadata_from_notes_data
cargo test -p writer_core writer_guid_plan_overrides_notes_guid
```

Expected: both PASS.

- [ ] **Step 7: Commit**

```bash
git add writer_core/src/model.rs writer_core/src/apkg.rs writer_core/src/inspect.rs writer_core/src/build.rs writer_core/tests/build_tests.rs writer_core/tests/inspect_tests.rs
git commit -m "writer: embed note identity metadata"
```

---

### Task 7: Lockfile Read, Validation, and Atomic Write

**Files:**
- Create: `anki_forge/src/update_safety/lockfile.rs`
- Modify: `anki_forge/src/update_safety/model.rs`
- Modify: `anki_forge/src/update_safety/mod.rs`
- Test: `anki_forge/tests/update_safety_lockfile_tests.rs`

- [ ] **Step 1: Add failing lockfile roundtrip test**

Create `anki_forge/tests/update_safety_lockfile_tests.rs`:

```rust
use anki_forge::update_safety::lockfile::{read_lockfile, write_lockfile_atomic};
use anki_forge::update_safety::model::{GeneratedBy, IdentityIndex, IdentityLockfile};

#[test]
fn lockfile_roundtrip_uses_canonical_json_and_generated_by() {
    let root = tempfile::tempdir().expect("tempdir");
    let path = root.path().join("anki-forge.lock.json");
    let lockfile = IdentityLockfile {
        schema_version: "identity-lockfile-v1".into(),
        project_stable_id: "project-a".into(),
        writer_policy_ref: "writer-policy.default@1.0.0".into(),
        identity_index: IdentityIndex::empty_lockfile("project-a", "writer-policy.default@1.0.0"),
        generated_by: GeneratedBy {
            tool: "anki-forge".into(),
            tool_version: env!("CARGO_PKG_VERSION").into(),
            writer_policy_ref: "writer-policy.default@1.0.0".into(),
        },
    };

    write_lockfile_atomic(&path, &lockfile).expect("write lockfile");
    let raw = std::fs::read_to_string(&path).expect("read raw");
    assert!(raw.starts_with("{\"generated_by\""));

    let loaded = read_lockfile(&path).expect("read lockfile");
    assert_eq!(loaded.project_stable_id, "project-a");
    assert_eq!(loaded.generated_by.tool, "anki-forge");
}
```

Run: `cargo test -p anki_forge lockfile_roundtrip_uses_canonical_json_and_generated_by`

Expected: FAIL because lockfile module and types do not exist.

- [ ] **Step 2: Add lockfile types**

In `anki_forge/src/update_safety/model.rs`, add:

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct IdentityLockfile {
    pub schema_version: String,
    pub project_stable_id: String,
    pub writer_policy_ref: String,
    pub identity_index: IdentityIndex,
    pub generated_by: GeneratedBy,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct GeneratedBy {
    pub tool: String,
    pub tool_version: String,
    pub writer_policy_ref: String,
}

impl IdentityIndex {
    pub fn empty_lockfile(project_stable_id: &str, writer_policy_ref: &str) -> Self {
        Self {
            schema_version: "identity-index-v1".into(),
            source_kind: "lockfile".into(),
            source_ref: "baseline.identity_lockfile.primary".into(),
            writer_policy_ref: writer_policy_ref.into(),
            project_stable_id: Some(project_stable_id.into()),
            notes: vec![],
            notetypes: vec![],
            limitations: vec![],
        }
    }
}
```

- [ ] **Step 3: Implement atomic write and read**

Create `anki_forge/src/update_safety/lockfile.rs`:

```rust
use std::collections::BTreeSet;
use std::path::Path;

use anyhow::{Context, Result};

use super::model::IdentityLockfile;

pub fn read_lockfile(path: impl AsRef<Path>) -> Result<IdentityLockfile> {
    let path = path.as_ref();
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("read identity lockfile {}", path.display()))?;
    let lockfile: IdentityLockfile = serde_json::from_str(&raw)
        .with_context(|| format!("parse identity lockfile {}", path.display()))?;
    validate_lockfile(&lockfile)?;
    Ok(lockfile)
}

pub fn write_lockfile_atomic(path: impl AsRef<Path>, lockfile: &IdentityLockfile) -> Result<()> {
    let path = path.as_ref();
    validate_lockfile(lockfile)?;
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(parent)
        .with_context(|| format!("create lockfile directory {}", parent.display()))?;
    let tmp = parent.join(format!(
        ".{}.tmp-{}",
        path.file_name().and_then(|name| name.to_str()).unwrap_or("anki-forge.lock.json"),
        std::process::id()
    ));
    let bytes = writer_core::to_canonical_json(lockfile)
        .context("serialize canonical identity lockfile")?;
    std::fs::write(&tmp, bytes)
        .with_context(|| format!("write temporary lockfile {}", tmp.display()))?;
    std::fs::rename(&tmp, path)
        .with_context(|| format!("replace identity lockfile {}", path.display()))?;
    Ok(())
}

fn validate_lockfile(lockfile: &IdentityLockfile) -> Result<()> {
    anyhow::ensure!(lockfile.schema_version == "identity-lockfile-v1", "UPDATE.BASELINE_SCHEMA_UNSUPPORTED: {}", lockfile.schema_version);
    anyhow::ensure!(!lockfile.project_stable_id.trim().is_empty(), "UPDATE.PROJECT_STABLE_ID_MISSING");
    anyhow::ensure!(lockfile.generated_by.tool == "anki-forge", "invalid generated_by.tool");
    let mut stable_ids = BTreeSet::new();
    let mut anki_guids = BTreeSet::new();
    for note in &lockfile.identity_index.notes {
        if note.entry_lifecycle == "active" {
            anyhow::ensure!(
                note.normalized_note_id.as_deref() == Some(note.stable_id.as_str()),
                "UPDATE.NORMALIZED_NOTE_ID_MISMATCH: stable_id={} normalized_note_id={:?}",
                note.stable_id,
                note.normalized_note_id
            );
        }
        anyhow::ensure!(
            stable_ids.insert(note.stable_id.as_str()),
            "UPDATE.STABLE_ID_DUPLICATE_IN_BASELINE: {}",
            note.stable_id
        );
        anyhow::ensure!(
            anki_guids.insert(note.anki_guid.as_str()),
            "UPDATE.GUID_DUPLICATE_IN_BASELINE: {}",
            note.anki_guid
        );
    }
    Ok(())
}
```

In `anki_forge/src/update_safety/mod.rs`, add `pub mod lockfile;`.

- [ ] **Step 4: Add invalid lockfile test**

Append:

```rust
#[test]
fn lockfile_rejects_unknown_schema_version() {
    let root = tempfile::tempdir().expect("tempdir");
    let path = root.path().join("anki-forge.lock.json");
    std::fs::write(
        &path,
        r#"{"schema_version":"identity-lockfile-v99","project_stable_id":"p","writer_policy_ref":"writer-policy.default@1.0.0","identity_index":{"schema_version":"identity-index-v1","source_kind":"lockfile","source_ref":"baseline.identity_lockfile.primary","writer_policy_ref":"writer-policy.default@1.0.0","project_stable_id":"p","notes":[],"notetypes":[],"limitations":[]},"generated_by":{"tool":"anki-forge","tool_version":"0.0.0","writer_policy_ref":"writer-policy.default@1.0.0"}}"#,
    )
    .expect("write invalid lockfile");

    let err = read_lockfile(&path).expect_err("schema should fail");
    assert!(err.to_string().contains("UPDATE.BASELINE_SCHEMA_UNSUPPORTED"));
}
```

Append duplicate-entry validation tests:

```rust
#[test]
fn lockfile_rejects_duplicate_stable_id_and_guid() {
    let mut lockfile = IdentityLockfile {
        schema_version: "identity-lockfile-v1".into(),
        project_stable_id: "project-a".into(),
        writer_policy_ref: "writer-policy.default@1.0.0".into(),
        identity_index: IdentityIndex::empty_lockfile("project-a", "writer-policy.default@1.0.0"),
        generated_by: GeneratedBy {
            tool: "anki-forge".into(),
            tool_version: env!("CARGO_PKG_VERSION").into(),
            writer_policy_ref: "writer-policy.default@1.0.0".into(),
        },
    };
    lockfile.identity_index.notes.push(note_entry("stable-a", "guid-a"));
    lockfile.identity_index.notes.push(note_entry("stable-a", "guid-b"));

    let root = tempfile::tempdir().expect("tempdir");
    let path = root.path().join("duplicate-stable.lock.json");
    let err = write_lockfile_atomic(&path, &lockfile).expect_err("duplicate stable id");
    assert!(err.to_string().contains("UPDATE.STABLE_ID_DUPLICATE_IN_BASELINE"));

    lockfile.identity_index.notes.clear();
    lockfile.identity_index.notes.push(note_entry("stable-a", "guid-a"));
    lockfile.identity_index.notes.push(note_entry("stable-b", "guid-a"));
    let err = write_lockfile_atomic(&path, &lockfile).expect_err("duplicate guid");
    assert!(err.to_string().contains("UPDATE.GUID_DUPLICATE_IN_BASELINE"));
}

#[test]
fn lockfile_rejects_active_normalized_note_id_mismatch() {
    let mut lockfile = IdentityLockfile {
        schema_version: "identity-lockfile-v1".into(),
        project_stable_id: "project-a".into(),
        writer_policy_ref: "writer-policy.default@1.0.0".into(),
        identity_index: IdentityIndex::empty_lockfile("project-a", "writer-policy.default@1.0.0"),
        generated_by: GeneratedBy {
            tool: "anki-forge".into(),
            tool_version: env!("CARGO_PKG_VERSION").into(),
            writer_policy_ref: "writer-policy.default@1.0.0".into(),
        },
    };
    let mut note = note_entry("stable-a", "guid-a");
    note.normalized_note_id = Some("different-normalized-id".into());
    lockfile.identity_index.notes.push(note);

    let root = tempfile::tempdir().expect("tempdir");
    let path = root.path().join("mismatch.lock.json");
    let err = write_lockfile_atomic(&path, &lockfile).expect_err("mismatch");
    assert!(err.to_string().contains("UPDATE.NORMALIZED_NOTE_ID_MISMATCH"));
}

fn note_entry(stable_id: &str, guid: &str) -> anki_forge::update_safety::model::NoteIdentityEntry {
    anki_forge::update_safety::model::NoteIdentityEntry {
        stable_id: stable_id.into(),
        normalized_note_id: Some(stable_id.into()),
        anki_guid: guid.into(),
        current_guid_candidate: stable_id.into(),
        guid_derivation_version: "guid.raw-stable-id.v1".into(),
        note_type_id: "basic".into(),
        recipe_id: "product.explicit-or-normalized.v1".into(),
        canonical_payload_hash: None,
        provenance: "ExplicitStableId".into(),
        used_override: false,
        entry_lifecycle: "active".into(),
        source_path: "test".into(),
        recovery_method: "current_resolution".into(),
    }
}
```

Run: `cargo test -p anki_forge update_safety_lockfile_tests`

Expected: all lockfile tests PASS.

- [ ] **Step 5: Commit**

```bash
git add anki_forge/src/update_safety/lockfile.rs anki_forge/src/update_safety/model.rs anki_forge/src/update_safety/mod.rs anki_forge/tests/update_safety_lockfile_tests.rs
git commit -m "lockfile: add identity lockfile roundtrip"
```

---

### Task 8: Baseline Loading from Previous APKG and Lockfile

**Files:**
- Create: `anki_forge/src/update_safety/baseline.rs`
- Modify: `anki_forge/src/update_safety/model.rs`
- Modify: `anki_forge/src/update_safety/mod.rs`
- Test: `anki_forge/tests/update_safety_baseline_tests.rs`

- [ ] **Step 1: Add failing APKG baseline recovery test**

Create `anki_forge/tests/update_safety_baseline_tests.rs`:

```rust
use anki_forge::build::BuildOptions;
use anki_forge::prelude::*;
use anki_forge::update_safety::baseline::load_previous_apkg_identity_index;

#[test]
fn previous_apkg_identity_index_recovers_embedded_metadata() {
    let root = tempfile::tempdir().expect("tempdir");
    let previous = root.path().join("previous.apkg");
    let mut project = Project::new("Baseline").stable_id("baseline-project");
    project
        .add_note(Note::basic("hola", "hello").stable_id("es:hola"))
        .expect("add note");
    project
        .build(BuildOptions::new().output(&previous))
        .expect("build previous");

    let index = load_previous_apkg_identity_index(&previous, None, None)
        .expect("load previous apkg identity");

    assert_eq!(index.source_kind, "previous_apkg");
    assert_eq!(index.source_ref, "baseline.previous_apkg.primary");
    assert!(index.notes.iter().any(|note| note.stable_id == "es:hola"));
}
```

Run: `cargo test -p anki_forge previous_apkg_identity_index_recovers_embedded_metadata`

Expected: FAIL because baseline loading does not exist.

- [ ] **Step 2: Implement previous APKG loading from inspect metadata**

Create `anki_forge/src/update_safety/baseline.rs`:

```rust
use std::path::Path;

use anyhow::{Context, Result};

use super::model::{IdentityIndex, NoteIdentityEntry};

pub fn load_previous_apkg_identity_index(
    path: impl AsRef<Path>,
    current: Option<&IdentityIndex>,
    lockfile: Option<&IdentityIndex>,
) -> Result<IdentityIndex> {
    let path = path.as_ref();
    let inspect = writer_core::inspect_apkg(path)
        .with_context(|| format!("inspect previous APKG {}", path.display()))?;
    let mut index = IdentityIndex {
        schema_version: "identity-index-v1".into(),
        source_kind: "previous_apkg".into(),
        source_ref: "baseline.previous_apkg.primary".into(),
        writer_policy_ref: "unknown@unknown".into(),
        project_stable_id: None,
        notes: vec![],
        notetypes: vec![],
        limitations: vec![],
    };

    for metadata in inspect.observations.metadata {
        if metadata.get("schema_version").and_then(|value| value.as_str()) != Some("identity-note-v1") {
            continue;
        }
        let Some(stable_id) = metadata.get("stable_id").and_then(|value| value.as_str()) else {
            index.limitations.push("identity_metadata_malformed".into());
            continue;
        };
        let selected = metadata
            .get("selected_anki_guid")
            .and_then(|value| value.as_str())
            .unwrap_or(stable_id);
        index.limitations.push("unknown_baseline_provenance".into());
        index.notes.push(NoteIdentityEntry {
            stable_id: stable_id.into(),
            normalized_note_id: None,
            anki_guid: selected.into(),
            current_guid_candidate: metadata
                .get("current_guid_candidate")
                .and_then(|value| value.as_str())
                .unwrap_or(stable_id)
                .into(),
            guid_derivation_version: metadata
                .get("guid_derivation_version")
                .and_then(|value| value.as_str())
                .unwrap_or("guid.raw-stable-id.v1")
                .into(),
            note_type_id: "unknown".into(),
            recipe_id: metadata
                .get("recipe_id")
                .and_then(|value| value.as_str())
                .unwrap_or("unknown")
                .into(),
            canonical_payload_hash: metadata
                .get("canonical_payload_hash")
                .and_then(|value| value.as_str())
                .map(str::to_string),
            provenance: "unknown_baseline".into(),
            used_override: false,
            entry_lifecycle: "active".into(),
            source_path: path.display().to_string(),
            recovery_method: "embedded_metadata".into(),
        });
    }

    if index.notes.is_empty() {
        recover_guid_equals_stable_id(&mut index, &inspect, current, lockfile);
    }

    index.limitations.sort();
    index.limitations.dedup();
    Ok(index)
}

fn recover_guid_equals_stable_id(
    index: &mut IdentityIndex,
    inspect: &writer_core::InspectReport,
    current: Option<&IdentityIndex>,
    lockfile: Option<&IdentityIndex>,
) {
    let mut stable_ids = std::collections::BTreeSet::new();
    if let Some(current) = current {
        for note in &current.notes {
            stable_ids.insert(note.stable_id.as_str());
        }
    }
    if let Some(lockfile) = lockfile {
        for note in &lockfile.notes {
            stable_ids.insert(note.stable_id.as_str());
        }
    }
    for note in &inspect.observations.references {
        if !note
            .get("selector")
            .and_then(|value| value.as_str())
            .is_some_and(|selector| selector.starts_with("note[id='"))
        {
            continue;
        }
        let Some(guid) = note.get("id").and_then(|value| value.as_str()) else {
            continue;
        };
        if stable_ids.contains(guid) {
            index.limitations.push("unknown_baseline_provenance".into());
            index.notes.push(NoteIdentityEntry {
                stable_id: guid.into(),
                normalized_note_id: None,
                anki_guid: guid.into(),
                current_guid_candidate: guid.into(),
                guid_derivation_version: "guid.raw-stable-id.v1".into(),
                note_type_id: "unknown".into(),
                recipe_id: "unknown".into(),
                canonical_payload_hash: None,
                provenance: "unknown_baseline".into(),
                used_override: false,
                entry_lifecycle: "active".into(),
                source_path: "inspect.notes".into(),
                recovery_method: "guid_equals_stable_id".into(),
            });
        }
    }
}
```

In `anki_forge/src/update_safety/mod.rs`, add `pub mod baseline;`.

- [ ] **Step 3: Add lockfile baseline helper**

Add to `baseline.rs`:

```rust
pub fn lockfile_identity_index(lockfile: &super::model::IdentityLockfile) -> IdentityIndex {
    lockfile.identity_index.clone()
}
```

- [ ] **Step 4: Run baseline tests**

Run: `cargo test -p anki_forge update_safety_baseline_tests`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add anki_forge/src/update_safety/baseline.rs anki_forge/src/update_safety/mod.rs anki_forge/tests/update_safety_baseline_tests.rs
git commit -m "baseline: recover identity from previous apkg"
```

---

### Task 9: Reconcile Current Identity with Baselines

**Files:**
- Create: `anki_forge/src/update_safety/reconcile.rs`
- Modify: `anki_forge/src/update_safety/model.rs`
- Modify: `anki_forge/src/update_safety/mod.rs`
- Test: `anki_forge/tests/update_safety_reconcile_tests.rs`

- [ ] **Step 1: Add failing reconcile priority test**

Create `anki_forge/tests/update_safety_reconcile_tests.rs`:

```rust
use anki_forge::update_safety::model::{IdentityIndex, NoteIdentityEntry};
use anki_forge::update_safety::reconcile::{reconcile_guid_plan, GuidSource};

#[test]
fn previous_apkg_wins_over_lockfile_for_same_stable_id() {
    let current = index_with_note("current", "note-a", "note-a");
    let previous = index_with_note("previous_apkg", "note-a", "guid-from-apkg");
    let lockfile = index_with_note("lockfile", "note-a", "guid-from-lockfile");

    let output = reconcile_guid_plan(&current, Some(&previous), Some(&lockfile))
        .expect("reconcile");

    assert_eq!(output.assignments[0].selected_anki_guid, "guid-from-apkg");
    assert_eq!(output.assignments[0].source, GuidSource::PreviousApkg.as_str());
    assert!(output
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code.as_str() == "UPDATE.BASELINE_CONFLICT_GUID"));
}

fn index_with_note(source_kind: &str, stable_id: &str, guid: &str) -> IdentityIndex {
    let mut index = IdentityIndex::empty_lockfile("project-a", "writer-policy.default@1.0.0");
    index.source_kind = source_kind.into();
    index.notes.push(NoteIdentityEntry {
        stable_id: stable_id.into(),
        normalized_note_id: Some(stable_id.into()),
        anki_guid: guid.into(),
        current_guid_candidate: stable_id.into(),
        guid_derivation_version: "guid.raw-stable-id.v1".into(),
        note_type_id: "basic".into(),
        recipe_id: "product.explicit-or-normalized.v1".into(),
        canonical_payload_hash: None,
        provenance: "ExplicitStableId".into(),
        used_override: false,
        entry_lifecycle: "active".into(),
        source_path: "test".into(),
        recovery_method: "current_resolution".into(),
    });
    index
}
```

Run: `cargo test -p anki_forge previous_apkg_wins_over_lockfile_for_same_stable_id`

Expected: FAIL because reconcile module does not exist.

- [ ] **Step 2: Add reconcile output model**

In `anki_forge/src/update_safety/reconcile.rs`, add:

```rust
use std::collections::BTreeMap;

use crate::diagnostics::{Diagnostic, DiagnosticCode, Severity, SourcePath};
use writer_core::WriterGuidAssignment;

use super::model::IdentityIndex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuidSource {
    PreviousApkg,
    Lockfile,
    CurrentDerivation,
}

impl GuidSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PreviousApkg => "previous_apkg",
            Self::Lockfile => "lockfile",
            Self::CurrentDerivation => "current_derivation",
        }
    }
}

pub struct ReconcileOutput {
    pub assignments: Vec<WriterGuidAssignment>,
    pub diagnostics: Vec<Diagnostic>,
    pub notes_preserved: usize,
    pub notes_derived: usize,
    pub notes_failed: usize,
    pub baseline_conflicts: usize,
}
```

- [ ] **Step 3: Implement priority and duplicate GUID checks**

Add:

```rust
pub fn reconcile_guid_plan(
    current: &IdentityIndex,
    previous_apkg: Option<&IdentityIndex>,
    lockfile: Option<&IdentityIndex>,
) -> anyhow::Result<ReconcileOutput> {
    let previous_by_stable = previous_apkg.map(index_by_stable_id).unwrap_or_default();
    let lockfile_by_stable = lockfile.map(index_by_stable_id).unwrap_or_default();
    let mut diagnostics = Vec::new();
    let mut assignments = Vec::new();
    let mut selected = BTreeMap::<String, String>::new();
    let mut notes_preserved = 0;
    let mut notes_derived = 0;
    let mut baseline_conflicts = 0;

    push_writer_policy_mismatch_diagnostics(current, previous_apkg, lockfile, &mut diagnostics);

    for note in &current.notes {
        let normalized_note_id = note
            .normalized_note_id
            .clone()
            .unwrap_or_else(|| note.stable_id.clone());
        let previous = previous_by_stable.get(note.stable_id.as_str());
        let locked = lockfile_by_stable.get(note.stable_id.as_str());
        let (guid, source) = if let Some(previous) = previous {
            if let Some(locked) = locked {
                if locked.anki_guid != previous.anki_guid {
                    baseline_conflicts += 1;
                    diagnostics.push(Diagnostic {
                        code: DiagnosticCode::new("UPDATE.BASELINE_CONFLICT_GUID"),
                        severity: Severity::Warning,
                        message: format!(
                            "previous APKG GUID {} overrides lockfile GUID {} for {}",
                            previous.anki_guid, locked.anki_guid, note.stable_id
                        ),
                        source: Some(SourcePath::new(note.source_path.clone())),
                        help: Some("previous APKG is artifact truth for update safety".into()),
                    });
                }
            }
            notes_preserved += 1;
            (previous.anki_guid.clone(), GuidSource::PreviousApkg)
        } else if let Some(locked) = locked {
            notes_preserved += 1;
            (locked.anki_guid.clone(), GuidSource::Lockfile)
        } else {
            notes_derived += 1;
            (note.current_guid_candidate.clone(), GuidSource::CurrentDerivation)
        };

        let info_code = match source {
            GuidSource::PreviousApkg => "UPDATE.GUID_PRESERVED_FROM_PREVIOUS",
            GuidSource::Lockfile => "UPDATE.GUID_PRESERVED_FROM_LOCKFILE",
            GuidSource::CurrentDerivation => "UPDATE.GUID_DERIVED_FOR_NEW_NOTE",
        };
        diagnostics.push(Diagnostic {
            code: DiagnosticCode::new(info_code),
            severity: Severity::Info,
            message: format!("selected GUID {guid} for stable id {}", note.stable_id),
            source: Some(SourcePath::new(note.source_path.clone())),
            help: None,
        });

        if let Some(existing) = selected.insert(guid.clone(), note.stable_id.clone()) {
            anyhow::bail!(
                "UPDATE.GUID_DUPLICATE_AT_RECONCILE: {} and {} selected {}",
                existing,
                note.stable_id,
                guid
            );
        }

        if guid != note.current_guid_candidate && source != GuidSource::CurrentDerivation {
            diagnostics.push(Diagnostic {
                code: DiagnosticCode::new("UPDATE.GUID_DERIVATION_DRIFT"),
                severity: Severity::Warning,
                message: format!("selected GUID {guid} differs from current derivation {}", note.current_guid_candidate),
                source: Some(SourcePath::new(note.source_path.clone())),
                help: Some("update-safe mode preserves existing Anki GUIDs".into()),
            });
        }

        assignments.push(WriterGuidAssignment {
            normalized_note_id,
            stable_id: note.stable_id.clone(),
            selected_anki_guid: guid,
            guid_derivation_version: note.guid_derivation_version.clone(),
            source: source.as_str().into(),
        });
    }

    assignments.sort_by(|left, right| {
        left.normalized_note_id
            .cmp(&right.normalized_note_id)
            .then(left.stable_id.cmp(&right.stable_id))
    });

    Ok(ReconcileOutput {
        assignments,
        diagnostics,
        notes_preserved,
        notes_derived,
        notes_failed: 0,
        baseline_conflicts,
    })
}

fn index_by_stable_id(index: &IdentityIndex) -> BTreeMap<&str, &super::model::NoteIdentityEntry> {
    let mut map = BTreeMap::new();
    for note in &index.notes {
        map.insert(note.stable_id.as_str(), note);
    }
    map
}

pub fn current_only_reconcile(current: &IdentityIndex) -> anyhow::Result<ReconcileOutput> {
    reconcile_guid_plan(current, None, None)
}

fn push_writer_policy_mismatch_diagnostics(
    current: &IdentityIndex,
    previous_apkg: Option<&IdentityIndex>,
    lockfile: Option<&IdentityIndex>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for (label, baseline) in [("previous APKG", previous_apkg), ("lockfile", lockfile)] {
        let Some(baseline) = baseline else {
            continue;
        };
        if baseline.writer_policy_ref == "unknown@unknown"
            || baseline.writer_policy_ref == current.writer_policy_ref
        {
            continue;
        }
        diagnostics.push(Diagnostic {
            code: DiagnosticCode::new("UPDATE.WRITER_POLICY_MISMATCH"),
            severity: Severity::Warning,
            message: format!(
                "{label} writer policy {} differs from current {}",
                baseline.writer_policy_ref, current.writer_policy_ref
            ),
            source: Some(SourcePath::new(baseline.source_ref.clone())),
            help: Some("verify the baseline was produced with a compatible writer policy".into()),
        });
    }
}
```

- [ ] **Step 4: Add duplicate GUID test**

Append:

```rust
#[test]
fn reconcile_rejects_duplicate_selected_guid() {
    let mut current = index_with_note("current", "note-a", "note-a");
    current.notes.push(NoteIdentityEntry {
        stable_id: "note-b".into(),
        normalized_note_id: Some("note-b".into()),
        anki_guid: "note-b".into(),
        current_guid_candidate: "same-guid".into(),
        guid_derivation_version: "guid.raw-stable-id.v1".into(),
        note_type_id: "basic".into(),
        recipe_id: "product.explicit-or-normalized.v1".into(),
        canonical_payload_hash: None,
        provenance: "ExplicitStableId".into(),
        used_override: false,
        entry_lifecycle: "active".into(),
        source_path: "test".into(),
        recovery_method: "current_resolution".into(),
    });
    current.notes[0].current_guid_candidate = "same-guid".into();

    let err = reconcile_guid_plan(&current, None, None).expect_err("duplicate guid");

    assert!(err.to_string().contains("UPDATE.GUID_DUPLICATE_AT_RECONCILE"));
}

#[test]
fn reconcile_emits_info_diagnostics_for_guid_sources() {
    let current = index_with_note("current", "note-a", "note-a");
    let previous = index_with_note("previous_apkg", "note-a", "guid-from-apkg");

    let output = reconcile_guid_plan(&current, Some(&previous), None).expect("reconcile");

    assert!(output
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code.as_str() == "UPDATE.GUID_PRESERVED_FROM_PREVIOUS"));

    let output = reconcile_guid_plan(&current, None, None).expect("reconcile current only");
    assert!(output
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code.as_str() == "UPDATE.GUID_DERIVED_FOR_NEW_NOTE"));
}

#[test]
fn reconcile_warns_on_writer_policy_mismatch() {
    let current = index_with_note("current", "note-a", "note-a");
    let mut previous = index_with_note("previous_apkg", "note-a", "guid-from-apkg");
    previous.writer_policy_ref = "writer-policy.legacy@0.9.0".into();

    let output = reconcile_guid_plan(&current, Some(&previous), None).expect("reconcile");

    assert!(output
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code.as_str() == "UPDATE.WRITER_POLICY_MISMATCH"));
}
```

Run: `cargo test -p anki_forge update_safety_reconcile_tests`

Expected: both tests PASS.

- [ ] **Step 5: Commit**

```bash
git add anki_forge/src/update_safety/reconcile.rs anki_forge/src/update_safety/model.rs anki_forge/src/update_safety/mod.rs anki_forge/tests/update_safety_reconcile_tests.rs
git commit -m "reconcile: select update-safe note guids"
```

---

### Task 10: Field, Template, and Notetype Merge Safety

**Files:**
- Create: `anki_forge/src/update_safety/merge_safety.rs`
- Modify: `anki_forge/src/update_safety/mod.rs`
- Test: `anki_forge/tests/update_safety_merge_safety_tests.rs`

- [ ] **Step 1: Add failing config drift and rename tests**

Create `anki_forge/tests/update_safety_merge_safety_tests.rs`:

```rust
use anki_forge::update_safety::merge_safety::compare_notetype_merge_safety;
use anki_forge::update_safety::model::{
    FieldMergeEntry, IdentityIndex, NotetypeIdentityEntry, TemplateMergeEntry,
};

#[test]
fn field_config_id_drift_is_error() {
    let current = index_with_notetype(field("front", "Front", 0, 111), template("card", "Card", 0, 222));
    let baseline = index_with_notetype(field("front", "Front", 0, 999), template("card", "Card", 0, 222));

    let diagnostics = compare_notetype_merge_safety(&current, &baseline);

    assert!(diagnostics.iter().any(|d| d.code.as_str() == "UPDATE.FIELD_MERGE_ID_CHANGED"));
}

#[test]
fn notetype_field_and_template_renames_are_warnings_when_ids_stay_stable() {
    let current = index_with_named_notetype(
        "Renamed",
        field("front", "Prompt", 0, 111),
        template("card", "Prompt Card", 0, 222),
    );
    let baseline = index_with_named_notetype(
        "Original",
        field("front", "Front", 0, 111),
        template("card", "Card", 0, 222),
    );

    let codes: Vec<_> = compare_notetype_merge_safety(&current, &baseline)
        .into_iter()
        .map(|d| d.code.as_str().to_string())
        .collect();

    assert!(codes.contains(&"UPDATE.NOTETYPE_RENAMED".into()));
    assert!(codes.contains(&"UPDATE.FIELD_RENAMED".into()));
    assert!(codes.contains(&"UPDATE.TEMPLATE_RENAMED".into()));
    assert!(!codes.contains(&"UPDATE.FIELD_MERGE_ID_CHANGED".into()));
}

#[test]
fn notetype_and_template_set_changes_include_change_kind() {
    let mut current = index_with_named_notetype(
        "Basic",
        field("front", "Front", 0, 111),
        template("new-card", "New Card", 0, 333),
    );
    let baseline = index_with_named_notetype(
        "Basic",
        field("front", "Front", 0, 111),
        template("old-card", "Old Card", 0, 222),
    );
    let mut added_removed_current = current.clone();
    added_removed_current.notetypes[0].note_type_id = "basic-new".into();

    let mut diagnostics = compare_notetype_merge_safety(&current, &baseline);
    diagnostics.extend(compare_notetype_merge_safety(&added_removed_current, &baseline));

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code.as_str() == "UPDATE.NOTETYPE_SET_CHANGED"
            && diagnostic.message.contains("change_kind=added")
    }));
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code.as_str() == "UPDATE.NOTETYPE_SET_CHANGED"
            && diagnostic.message.contains("change_kind=removed")
    }));
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code.as_str() == "UPDATE.TEMPLATE_SET_CHANGED"
            && diagnostic.message.contains("change_kind=added")
    }));
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.code.as_str() == "UPDATE.TEMPLATE_SET_CHANGED"
            && diagnostic.message.contains("change_kind=removed")
    }));
}

fn index_with_notetype(field: FieldMergeEntry, template: TemplateMergeEntry) -> IdentityIndex {
    index_with_named_notetype("Basic", field, template)
}

fn index_with_named_notetype(name: &str, field: FieldMergeEntry, template: TemplateMergeEntry) -> IdentityIndex {
    let mut index = IdentityIndex::empty_lockfile("project-a", "writer-policy.default@1.0.0");
    index.notetypes.push(NotetypeIdentityEntry {
        note_type_id: "basic".into(),
        anki_model_id: Some(1),
        name: name.into(),
        fields: vec![field],
        templates: vec![template],
    });
    index
}

fn field(key: &str, name: &str, ord: u32, config_id: i64) -> FieldMergeEntry {
    FieldMergeEntry {
        field_key: key.into(),
        field_name: name.into(),
        ord,
        config_id,
        tag: ord as i32,
    }
}

fn template(key: &str, name: &str, ord: u32, config_id: i64) -> TemplateMergeEntry {
    TemplateMergeEntry {
        template_key: key.into(),
        template_name: name.into(),
        ord,
        config_id,
    }
}
```

Run: `cargo test -p anki_forge update_safety_merge_safety_tests`

Expected: FAIL because `merge_safety` does not exist.

- [ ] **Step 2: Implement merge safety comparison**

Create `anki_forge/src/update_safety/merge_safety.rs`:

The existing `Diagnostic` type has no structured metadata bag. For Phase 3, `UPDATE.NOTETYPE_SET_CHANGED` and `UPDATE.TEMPLATE_SET_CHANGED` carry `change_kind=added` or `change_kind=removed` in the diagnostic message. Do not widen the global diagnostic type in this plan.

```rust
use std::collections::BTreeMap;

use crate::diagnostics::{Diagnostic, DiagnosticCode, Severity, SourcePath};

use super::model::{FieldMergeEntry, IdentityIndex, NotetypeIdentityEntry, TemplateMergeEntry};

pub fn compare_notetype_merge_safety(current: &IdentityIndex, baseline: &IdentityIndex) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let current_by_id: BTreeMap<_, _> = current
        .notetypes
        .iter()
        .map(|notetype| (notetype.note_type_id.as_str(), notetype))
        .collect();
    let baseline_by_id: BTreeMap<_, _> = baseline
        .notetypes
        .iter()
        .map(|notetype| (notetype.note_type_id.as_str(), notetype))
        .collect();

    for current_notetype in &current.notetypes {
        let Some(baseline_notetype) = baseline_by_id.get(current_notetype.note_type_id.as_str()) else {
            diagnostics.push(warning("UPDATE.NOTETYPE_SET_CHANGED", &current_notetype.note_type_id, "change_kind=added; notetype was added"));
            continue;
        };
        compare_notetype(current_notetype, baseline_notetype, &mut diagnostics);
    }
    for baseline_notetype in &baseline.notetypes {
        if !current_by_id.contains_key(baseline_notetype.note_type_id.as_str()) {
            diagnostics.push(warning(
                "UPDATE.NOTETYPE_SET_CHANGED",
                &baseline_notetype.note_type_id,
                "change_kind=removed; notetype was removed",
            ));
        }
    }

    diagnostics
}

fn compare_notetype(current: &NotetypeIdentityEntry, baseline: &NotetypeIdentityEntry, diagnostics: &mut Vec<Diagnostic>) {
    if current.name != baseline.name {
        diagnostics.push(warning("UPDATE.NOTETYPE_RENAMED", &current.note_type_id, "notetype name changed"));
    }
    compare_fields(current, baseline, diagnostics);
    compare_templates(current, baseline, diagnostics);
}

fn compare_fields(current: &NotetypeIdentityEntry, baseline: &NotetypeIdentityEntry, diagnostics: &mut Vec<Diagnostic>) {
    let baseline_by_key: BTreeMap<_, _> = baseline.fields.iter().map(|field| (field.field_key.as_str(), field)).collect();
    for field in &current.fields {
        if let Some(old) = baseline_by_key.get(field.field_key.as_str()) {
            compare_field(field, old, &current.note_type_id, diagnostics);
        }
    }
}

fn compare_field(current: &FieldMergeEntry, baseline: &FieldMergeEntry, notetype_id: &str, diagnostics: &mut Vec<Diagnostic>) {
    if current.config_id != baseline.config_id {
        diagnostics.push(error("UPDATE.FIELD_MERGE_ID_CHANGED", notetype_id, "field config id changed"));
        return;
    }
    if current.field_name != baseline.field_name {
        diagnostics.push(warning("UPDATE.FIELD_RENAMED", notetype_id, "field name changed"));
    }
    if current.ord != baseline.ord {
        diagnostics.push(warning("UPDATE.FIELD_ORD_CHANGED", notetype_id, "field ord changed"));
    }
}

fn compare_templates(current: &NotetypeIdentityEntry, baseline: &NotetypeIdentityEntry, diagnostics: &mut Vec<Diagnostic>) {
    let current_by_key: BTreeMap<_, _> = current.templates.iter().map(|template| (template.template_key.as_str(), template)).collect();
    let baseline_by_key: BTreeMap<_, _> = baseline.templates.iter().map(|template| (template.template_key.as_str(), template)).collect();
    for template in &current.templates {
        if let Some(old) = baseline_by_key.get(template.template_key.as_str()) {
            compare_template(template, old, &current.note_type_id, diagnostics);
        } else {
            diagnostics.push(warning(
                "UPDATE.TEMPLATE_SET_CHANGED",
                &current.note_type_id,
                "change_kind=added; template was added",
            ));
        }
    }
    for template in &baseline.templates {
        if !current_by_key.contains_key(template.template_key.as_str()) {
            diagnostics.push(warning(
                "UPDATE.TEMPLATE_SET_CHANGED",
                &current.note_type_id,
                "change_kind=removed; template was removed",
            ));
        }
    }
}

fn compare_template(current: &TemplateMergeEntry, baseline: &TemplateMergeEntry, notetype_id: &str, diagnostics: &mut Vec<Diagnostic>) {
    if current.config_id != baseline.config_id {
        diagnostics.push(error("UPDATE.TEMPLATE_MERGE_ID_CHANGED", notetype_id, "template config id changed"));
        return;
    }
    if current.template_name != baseline.template_name {
        diagnostics.push(warning("UPDATE.TEMPLATE_RENAMED", notetype_id, "template name changed"));
    }
    if current.ord != baseline.ord {
        diagnostics.push(warning("UPDATE.TEMPLATE_ORD_CHANGED", notetype_id, "template ord changed"));
    }
}

fn error(code: &str, source: &str, message: &str) -> Diagnostic {
    Diagnostic {
        code: DiagnosticCode::new(code),
        severity: Severity::Error,
        message: message.into(),
        source: Some(SourcePath::new(source)),
        help: None,
    }
}

fn warning(code: &str, source: &str, message: &str) -> Diagnostic {
    Diagnostic {
        code: DiagnosticCode::new(code),
        severity: Severity::Warning,
        message: message.into(),
        source: Some(SourcePath::new(source)),
        help: None,
    }
}
```

In `anki_forge/src/update_safety/mod.rs`, add `pub mod merge_safety;`.

- [ ] **Step 3: Run merge-safety tests**

Run: `cargo test -p anki_forge update_safety_merge_safety_tests`

Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add anki_forge/src/update_safety/merge_safety.rs anki_forge/src/update_safety/mod.rs anki_forge/tests/update_safety_merge_safety_tests.rs
git commit -m "identity: compare notetype merge safety"
```

---

### Task 11: Integrate Update Safety into `Project::build`

**Files:**
- Modify: `anki_forge/Cargo.toml`
- Modify: `anki_forge/src/product/project.rs`
- Modify: `anki_forge/src/update_safety/report.rs`
- Modify: `anki_forge/src/update_safety/model.rs`
- Test: `anki_forge/tests/update_safety_build_tests.rs`

- [ ] **Step 1: Add failing end-to-end GUID preservation test**

Create `anki_forge/tests/update_safety_build_tests.rs`:

Add these dev-dependencies to `anki_forge/Cargo.toml` because the test edits APKG ZIP entries and SQLite directly:

```toml
[dev-dependencies]
rusqlite = { version = "0.32", features = ["bundled"] }
zip = { version = "2.2.0", default-features = false, features = ["deflate"] }
zstd = "0.13"
```

```rust
use anki_forge::build::BuildOptions;
use anki_forge::prelude::*;
use rusqlite::Connection;

#[test]
fn project_build_compare_to_preserves_previous_guid() {
    let root = tempfile::tempdir().expect("tempdir");
    let previous = root.path().join("previous.apkg");
    let updated = root.path().join("updated.apkg");

    let mut first = Project::new("Spanish").stable_id("spanish");
    first
        .add_note(Note::basic("hola", "hello").stable_id("es:hola"))
        .expect("add first note");
    first.build(BuildOptions::new().output(&previous)).expect("first build");

    rewrite_single_note_guid(&previous, "legacy-guid");

    let mut second = Project::new("Spanish").stable_id("spanish");
    second
        .add_note(Note::basic("hola", "hello updated").stable_id("es:hola"))
        .expect("add second note");
    let report = second
        .build(BuildOptions::new().output(&updated).compare_to(&previous))
        .expect("update-safe build");

    assert_eq!(report.update_safety.as_ref().unwrap().notes_preserved, 1);
    assert_eq!(read_single_guid(&updated), "legacy-guid");
}

#[test]
fn strict_compare_to_unreadable_previous_apkg_blocks_writer() {
    let root = tempfile::tempdir().expect("tempdir");
    let previous = root.path().join("missing.apkg");
    let output = root.path().join("updated.apkg");

    let mut project = Project::new("Spanish").stable_id("spanish");
    project
        .add_note(Note::basic("hola", "hello").stable_id("es:hola"))
        .expect("add note");

    let err = project
        .build(BuildOptions::new().output(&output).compare_to(&previous))
        .expect_err("strict compare_to should block on unreadable APKG");

    assert!(err
        .report
        .diagnostic_codes()
        .contains(&"UPDATE.BASELINE_APKG_UNREADABLE".into()));
    assert!(!output.exists());
}

fn read_single_guid(path: &std::path::Path) -> String {
    let tmp = tempfile::tempdir().expect("tempdir");
    let collection = read_latest_collection_bytes(path);
    let db_path = tmp.path().join("collection.sqlite");
    std::fs::write(&db_path, collection).expect("write sqlite");
    let conn = Connection::open(db_path).expect("open sqlite");
    conn.query_row("select guid from notes", [], |row| row.get(0)).expect("guid")
}

fn rewrite_single_note_guid(path: &std::path::Path, guid: &str) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let file = std::fs::File::open(path).expect("open apkg");
    let mut zip = zip::ZipArchive::new(file).expect("zip");
    let mut entries = std::collections::BTreeMap::<String, Vec<u8>>::new();
    for index in 0..zip.len() {
        let mut entry = zip.by_index(index).expect("entry");
        let mut bytes = Vec::new();
        std::io::Read::read_to_end(&mut entry, &mut bytes).expect("read entry");
        entries.insert(entry.name().to_string(), bytes);
    }

    let compressed = entries
        .get("collection.anki21b")
        .expect("latest collection")
        .clone();
    let decoded = zstd::stream::decode_all(compressed.as_slice()).expect("decode collection");
    let collection = tmp.path().join("collection.sqlite");
    std::fs::write(&collection, decoded).expect("write sqlite");
    let conn = Connection::open(&collection).expect("open sqlite");
    let data: String = conn
        .query_row("select data from notes", [], |row| row.get(0))
        .expect("read notes.data");
    let mut data_json: serde_json::Value =
        serde_json::from_str(&data).unwrap_or_else(|_| serde_json::json!({}));
    data_json["anki_forge_identity"]["selected_anki_guid"] = serde_json::json!(guid);
    conn.execute(
        "update notes set guid = ?1, data = ?2",
        rusqlite::params![guid, serde_json::to_string(&data_json).expect("serialize data")],
    )
    .expect("update guid and metadata");
    drop(conn);
    let updated = std::fs::read(&collection).expect("read sqlite");
    entries.insert(
        "collection.anki21b".into(),
        zstd::stream::encode_all(updated.as_slice(), 0).expect("encode collection"),
    );

    let output = std::fs::File::create(path).expect("replace apkg");
    let mut writer = zip::ZipWriter::new(output);
    let options: zip::write::FileOptions<'_, ()> =
        zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for (entry, bytes) in entries {
        writer.start_file(entry, options).expect("start file");
        std::io::Write::write_all(&mut writer, &bytes).expect("write entry");
    }
    writer.finish().expect("finish");
}

fn read_latest_collection_bytes(path: &std::path::Path) -> Vec<u8> {
    let file = std::fs::File::open(path).expect("open apkg");
    let mut zip = zip::ZipArchive::new(file).expect("zip");
    let mut entry = zip.by_name("collection.anki21b").expect("latest collection");
    let mut compressed = Vec::new();
    std::io::Read::read_to_end(&mut entry, &mut compressed).expect("read collection");
    zstd::stream::decode_all(compressed.as_slice()).expect("decode collection")
}
```

Run: `cargo test -p anki_forge project_build_compare_to_preserves_previous_guid`

Expected: FAIL because `Project::build` does not pass a reconciled writer GUID plan.

- [ ] **Step 2: Build report summary helper**

In `anki_forge/src/update_safety/report.rs`, add:

```rust
use crate::build::{BaselineSourceSummary, UpdateSafetySummary};
use crate::diagnostics::{Diagnostic, Severity};

use super::model::EffectiveMode;
use super::reconcile::ReconcileOutput;

pub fn summary_from_reconcile(
    mode: EffectiveMode,
    reconcile: &ReconcileOutput,
    diagnostics: &[Diagnostic],
    lockfile_written: bool,
) -> UpdateSafetySummary {
    UpdateSafetySummary {
        mode: match mode {
            EffectiveMode::Disabled => "disabled",
            EffectiveMode::ReportOnly => "report_only",
            EffectiveMode::Strict => "strict",
        }
        .into(),
        baseline_sources: vec![],
        notes_preserved: reconcile.notes_preserved,
        notes_derived: reconcile.notes_derived,
        notes_failed: reconcile.notes_failed,
        baseline_conflicts: reconcile.baseline_conflicts,
        blocking_diagnostics: diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == Severity::Error)
            .map(|diagnostic| diagnostic.code.as_str().to_string())
            .collect(),
        lockfile_written,
    }
}
```

- [ ] **Step 3: Wire baseline load, reconcile, and writer call**

In `anki_forge/src/product/project.rs`, after `current_identity`, add:

```rust
let previous_index = if let Some(path) = options.compare_to.as_ref() {
    match crate::update_safety::baseline::load_previous_apkg_identity_index(
        path,
        Some(&current_identity.index),
        None,
    ) {
        Ok(index) => Some(index),
        Err(err) => {
            diagnostics.push(Diagnostic {
                code: DiagnosticCode::new("UPDATE.BASELINE_APKG_UNREADABLE"),
                severity: Severity::Error,
                message: err.to_string(),
                source: Some(SourcePath::new(path.display().to_string())),
                help: Some("verify the previous APKG path and package contents".into()),
            });
            None
        }
    }
} else {
    None
};

if matches!(update_mode, crate::update_safety::EffectiveMode::Strict)
    && options.compare_to.is_some()
    && previous_index.is_none()
{
    return Err(BuildError::new(
        BuildReport {
            artifact: None,
            counts: BuildCounts {
                notes: normalized.notes.len(),
                cards: count_phase1_cards_without_inspect(&normalized),
                media: normalized.media_bindings.len(),
            },
            media: MediaSummary::from_normalized_ir(&normalized, &diagnostics),
            diagnostics: diagnostics.clone(),
            metrics: BuildMetrics { duration: started.elapsed() },
            inspect: None,
            update_safety: None,
            status: "invalid".into(),
        },
        BuildFailureCause::Diagnostics,
    ));
}

let reconcile = crate::update_safety::reconcile::reconcile_guid_plan(
    &current_identity.index,
    previous_index.as_ref(),
    None,
)
.map_err(|err| {
    diagnostics.push(Diagnostic {
        code: DiagnosticCode::new("UPDATE.GUID_DUPLICATE_AT_RECONCILE"),
        severity: Severity::Error,
        message: err.to_string(),
        source: Some(SourcePath::new("update_safety.reconcile")),
        help: Some("choose unique stable ids or remove conflicting lockfile entries".into()),
    });
    BuildError::new(
        BuildReport {
            artifact: None,
            counts: BuildCounts {
                notes: normalized.notes.len(),
                cards: count_phase1_cards_without_inspect(&normalized),
                media: normalized.media_bindings.len(),
            },
            media: MediaSummary::from_normalized_ir(&normalized, &diagnostics),
            diagnostics: diagnostics.clone(),
            metrics: BuildMetrics { duration: started.elapsed() },
            inspect: None,
            update_safety: None,
            status: "invalid".into(),
        },
        BuildFailureCause::Diagnostics,
    )
})?;
diagnostics.extend(reconcile.diagnostics.clone());
if let Some(baseline_for_merge) = previous_index.as_ref() {
    diagnostics.extend(crate::update_safety::merge_safety::compare_notetype_merge_safety(
        &current_identity.index,
        baseline_for_merge,
    ));
}
if matches!(update_mode, crate::update_safety::EffectiveMode::Strict)
    && diagnostics.iter().any(|diagnostic| diagnostic.severity == Severity::Error)
{
    return Err(BuildError::new(
        BuildReport {
            artifact: None,
            counts: BuildCounts {
                notes: normalized.notes.len(),
                cards: count_phase1_cards_without_inspect(&normalized),
                media: normalized.media_bindings.len(),
            },
            media: MediaSummary::from_normalized_ir(&normalized, &diagnostics),
            diagnostics: diagnostics.clone(),
            metrics: BuildMetrics { duration: started.elapsed() },
            inspect: None,
            update_safety: Some(crate::update_safety::report::summary_from_reconcile(
                update_mode,
                &reconcile,
                &diagnostics,
                false,
            )),
            status: "invalid".into(),
        },
        BuildFailureCause::Diagnostics,
    ));
}
let writer_guid_plan = writer_core::WriterGuidPlan {
    assignments: reconcile.assignments.clone(),
};
```

Change the writer call to:

```rust
let package_build_result = writer_core::build_with_guid_plan(
    &normalized,
    &writer_policy,
    &build_context,
    &artifact_target,
    Some(&writer_guid_plan),
)
```

If `update_mode` is disabled and there are no baseline inputs, passing a current-derivation plan is still valid and lets writer embed identity metadata consistently.

- [ ] **Step 4: Attach update-safety summary**

When creating the final `BuildReport`, set:

```rust
update_safety: Some(crate::update_safety::report::summary_from_reconcile(
    update_mode,
    &reconcile,
    &diagnostics,
    false,
)),
```

For early invalid reports before `current_identity` exists, keep `update_safety: None`.

- [ ] **Step 5: Run end-to-end test**

Run: `cargo test -p anki_forge project_build_compare_to_preserves_previous_guid`

Expected: PASS and SQLite `notes.guid` equals `legacy-guid`.

- [ ] **Step 6: Commit**

```bash
git add anki_forge/src/product/project.rs anki_forge/src/update_safety/report.rs anki_forge/src/update_safety/model.rs anki_forge/tests/update_safety_build_tests.rs
git commit -m "build: preserve guids from previous apkg"
```

---

### Task 12: Lockfile Integration and Absent Entry Carry-Forward

**Files:**
- Modify: `anki_forge/src/product/project.rs`
- Modify: `anki_forge/src/update_safety/lockfile.rs`
- Modify: `anki_forge/src/update_safety/reconcile.rs`
- Test: `anki_forge/tests/update_safety_lockfile_build_tests.rs`

- [ ] **Step 1: Add failing lockfile build/rebuild test**

Create `anki_forge/tests/update_safety_lockfile_build_tests.rs`:

```rust
use anki_forge::build::BuildOptions;
use anki_forge::prelude::*;

#[test]
fn build_writes_lockfile_and_second_build_preserves_guid_from_it() {
    let root = tempfile::tempdir().expect("tempdir");
    let first_apkg = root.path().join("first.apkg");
    let second_apkg = root.path().join("second.apkg");
    let lockfile = root.path().join("anki-forge.lock.json");

    let mut first = Project::new("Spanish").stable_id("spanish");
    first
        .add_note(Note::basic("hola", "hello").stable_id("es:hola"))
        .expect("add first note");
    first
        .build(
            BuildOptions::new()
                .output(&first_apkg)
                .identity_lockfile(&lockfile)
                .write_identity_lockfile(true),
        )
        .expect("write initial lockfile");
    assert!(lockfile.exists());

    let mut second = Project::new("Spanish").stable_id("spanish");
    second
        .add_note(Note::basic("hola", "hello again").stable_id("es:hola"))
        .expect("add second note");
    let report = second
        .build(BuildOptions::new().output(&second_apkg).identity_lockfile(&lockfile))
        .expect("use lockfile");

    assert_eq!(report.update_safety.unwrap().notes_preserved, 1);
}
```

Run: `cargo test -p anki_forge build_writes_lockfile_and_second_build_preserves_guid_from_it`

Expected: FAIL because `Project::build` does not read/write lockfiles.

- [ ] **Step 2: Add selected index creation**

In `anki_forge/src/update_safety/reconcile.rs`, change the import to `use std::collections::{BTreeMap, BTreeSet};` and add:

```rust
pub fn selected_identity_index(
    current: &IdentityIndex,
    output: &ReconcileOutput,
    previous_lockfile_index: Option<&IdentityIndex>,
) -> IdentityIndex {
    let by_stable: std::collections::BTreeMap<_, _> = output
        .assignments
        .iter()
        .map(|assignment| (assignment.stable_id.as_str(), assignment))
        .collect();
    let mut selected = current.clone();
    selected.source_kind = "lockfile".into();
    selected.source_ref = "baseline.identity_lockfile.primary".into();
    for note in &mut selected.notes {
        if let Some(assignment) = by_stable.get(note.stable_id.as_str()) {
            note.anki_guid = assignment.selected_anki_guid.clone();
        }
    }
    let current_stable_ids: BTreeSet<String> = selected
        .notes
        .iter()
        .map(|note| note.stable_id.clone())
        .collect();
    if let Some(previous_lockfile_index) = previous_lockfile_index {
        for old_note in &previous_lockfile_index.notes {
            if current_stable_ids.contains(old_note.stable_id.as_str()) {
                continue;
            }
            let mut absent = old_note.clone();
            absent.normalized_note_id = None;
            absent.entry_lifecycle = "absent_from_current".into();
            absent.source_path = "baseline.identity_lockfile.primary".into();
            selected.notes.push(absent);
        }
    }
    selected.notes.sort_by(|left, right| left.stable_id.cmp(&right.stable_id));
    selected
}
```

- [ ] **Step 3: Read lockfile baseline in build**

In `Project::build`, before reconcile:

```rust
let lockfile = if let Some(path) = options.identity_lockfile.as_ref() {
    if path.exists() {
        match crate::update_safety::lockfile::read_lockfile(path) {
            Ok(lockfile) => Some(lockfile),
            Err(err) => {
                diagnostics.push(Diagnostic {
                    code: DiagnosticCode::new("UPDATE.BASELINE_LOCKFILE_UNREADABLE"),
                    severity: Severity::Error,
                    message: err.to_string(),
                    source: Some(SourcePath::new(path.display().to_string())),
                    help: Some("fix or regenerate the identity lockfile".into()),
                });
                None
            }
        }
    } else if options.write_identity_lockfile {
        None
    } else {
        diagnostics.push(Diagnostic {
            code: DiagnosticCode::new("UPDATE.BASELINE_LOCKFILE_UNREADABLE"),
            severity: Severity::Error,
            message: format!("identity lockfile {} does not exist", path.display()),
            source: Some(SourcePath::new(path.display().to_string())),
            help: Some("run with write_identity_lockfile(true) to create the first lockfile".into()),
        });
        None
    }
} else {
    None
};
let lockfile_index = lockfile.as_ref().map(|lockfile| lockfile.identity_index.clone());
```

Move the previous APKG loading block from Task 11 below this lockfile block and change the loader call to:

```rust
let previous_index = if let Some(path) = options.compare_to.as_ref() {
    match crate::update_safety::baseline::load_previous_apkg_identity_index(
        path,
        Some(&current_identity.index),
        lockfile_index.as_ref(),
    ) {
        Ok(index) => Some(index),
        Err(err) => {
            diagnostics.push(Diagnostic {
                code: DiagnosticCode::new("UPDATE.BASELINE_APKG_UNREADABLE"),
                severity: Severity::Error,
                message: err.to_string(),
                source: Some(SourcePath::new(path.display().to_string())),
                help: Some("verify the previous APKG path and package contents".into()),
            });
            None
        }
    }
} else {
    None
};
```

Pass `lockfile_index.as_ref()` into `reconcile_guid_plan`. Also replace the Task 11 merge-safety block with this baseline selection so lockfile-only builds still compare merge metadata:

```rust
let baseline_for_merge = previous_index.as_ref().or(lockfile_index.as_ref());
if let Some(baseline_for_merge) = baseline_for_merge {
    diagnostics.extend(crate::update_safety::merge_safety::compare_notetype_merge_safety(
        &current_identity.index,
        baseline_for_merge,
    ));
}
```

- [ ] **Step 4: Write updated lockfile after successful APKG**

After artifact creation and before final report:

```rust
let mut lockfile_written = false;
if options.write_identity_lockfile {
    if let Some(path) = options.identity_lockfile.as_ref() {
        let Some(project_stable_id) = self.stable_id.clone() else {
            diagnostics.push(Diagnostic {
                code: DiagnosticCode::new("UPDATE.PROJECT_STABLE_ID_MISSING"),
                severity: Severity::Error,
                message: "project stable id is required before writing an identity lockfile".into(),
                source: Some(SourcePath::new("project.stable_id")),
                help: Some("set Project::stable_id(value) before write_identity_lockfile(true)".into()),
            });
            return Err(BuildError::new(
                BuildReport {
                    artifact: artifact.clone(),
                    counts,
                    media: MediaSummary::from_normalized_ir(&normalized, &diagnostics),
                    diagnostics: diagnostics.clone(),
                    metrics: BuildMetrics { duration: started.elapsed() },
                    inspect: inspect.clone(),
                    update_safety: None,
                    status: "error".into(),
                },
                BuildFailureCause::Diagnostics,
            ));
        };
        let selected_index = crate::update_safety::reconcile::selected_identity_index(
            &current_identity.index,
            &reconcile,
            lockfile_index.as_ref(),
        );
        let writer_policy_ref = writer_core::policy_ref(&writer_policy.id, &writer_policy.version);
        let lockfile = crate::update_safety::model::IdentityLockfile {
            schema_version: "identity-lockfile-v1".into(),
            project_stable_id,
            writer_policy_ref: writer_policy_ref.clone(),
            identity_index: selected_index,
            generated_by: crate::update_safety::model::GeneratedBy {
                tool: "anki-forge".into(),
                tool_version: env!("CARGO_PKG_VERSION").into(),
                writer_policy_ref,
            },
        };
        crate::update_safety::lockfile::write_lockfile_atomic(path, &lockfile).map_err(|err| {
            diagnostics.push(Diagnostic {
                code: DiagnosticCode::new("UPDATE.LOCKFILE_WRITE_FAILED"),
                severity: Severity::Error,
                message: err.to_string(),
                source: Some(SourcePath::new(path.display().to_string())),
                help: Some("verify the lockfile path is writable".into()),
            });
            BuildError::new(
                BuildReport {
                    artifact: artifact.clone(),
                    counts,
                    media: MediaSummary::from_normalized_ir(&normalized, &diagnostics),
                    diagnostics: diagnostics.clone(),
                    metrics: BuildMetrics { duration: started.elapsed() },
                    inspect: inspect.clone(),
                    update_safety: None,
                    status: "error".into(),
                },
                BuildFailureCause::Io,
            )
        })?;
        lockfile_written = true;
    }
}
```

Use `lockfile_written` in `summary_from_reconcile`.

- [ ] **Step 5: Run lockfile integration test**

Run: `cargo test -p anki_forge build_writes_lockfile_and_second_build_preserves_guid_from_it`

Expected: PASS.

- [ ] **Step 6: Add absent carry-forward test**

Add to `anki_forge/tests/update_safety_lockfile_build_tests.rs`:

```rust
#[test]
fn lockfile_carries_forward_absent_entries() {
    let root = tempfile::tempdir().expect("tempdir");
    let first_apkg = root.path().join("first.apkg");
    let second_apkg = root.path().join("second.apkg");
    let lockfile = root.path().join("anki-forge.lock.json");

    let mut first = Project::new("Spanish").stable_id("spanish");
    first
        .add_note(Note::basic("hola", "hello").stable_id("es:hola"))
        .expect("add hola");
    first
        .add_note(Note::basic("adios", "goodbye").stable_id("es:adios"))
        .expect("add adios");
    first
        .build(
            BuildOptions::new()
                .output(&first_apkg)
                .identity_lockfile(&lockfile)
                .write_identity_lockfile(true),
        )
        .expect("write initial lockfile");

    let mut second = Project::new("Spanish").stable_id("spanish");
    second
        .add_note(Note::basic("hola", "hello again").stable_id("es:hola"))
        .expect("add hola only");
    second
        .build(
            BuildOptions::new()
                .output(&second_apkg)
                .identity_lockfile(&lockfile)
                .write_identity_lockfile(true),
        )
        .expect("rewrite lockfile");

    let loaded = anki_forge::update_safety::lockfile::read_lockfile(&lockfile)
        .expect("read rewritten lockfile");
    assert!(loaded.identity_index.notes.iter().any(|note| {
        note.stable_id == "es:adios" && note.entry_lifecycle == "absent_from_current"
    }));
}
```

Run: `cargo test -p anki_forge lockfile_carries_forward_absent_entries`

Expected: PASS because `selected_identity_index` appends old lockfile notes not present in current with `entry_lifecycle = "absent_from_current"`.

- [ ] **Step 7: Commit**

```bash
git add anki_forge/src/product/project.rs anki_forge/src/update_safety/lockfile.rs anki_forge/src/update_safety/reconcile.rs anki_forge/tests/update_safety_lockfile_build_tests.rs
git commit -m "lockfile: integrate update safety lockfile builds"
```

---

### Task 13: Report Aggregation, Baseline Source Summary, and Disabled Mode

**Files:**
- Modify: `anki_forge/src/update_safety/report.rs`
- Modify: `anki_forge/src/product/project.rs`
- Test: `anki_forge/tests/update_safety_report_tests.rs`

- [ ] **Step 1: Add failing disabled-mode baseline ignored test**

Create `anki_forge/tests/update_safety_report_tests.rs`:

```rust
use anki_forge::build::{BuildOptions, UpdateSafetyMode};
use anki_forge::prelude::*;

#[test]
fn disabled_mode_ignores_baseline_but_records_summary() {
    let root = tempfile::tempdir().expect("tempdir");
    let previous = root.path().join("missing.apkg");
    let output = root.path().join("out.apkg");
    let mut project = Project::new("Disabled").stable_id("disabled");
    project
        .add_note(Note::basic("hola", "hello").stable_id("es:hola"))
        .expect("add note");

    let report = project
        .build(
            BuildOptions::new()
                .output(&output)
                .compare_to(&previous)
                .update_safety(UpdateSafetyMode::Disabled),
        )
        .expect("disabled ignores missing baseline");

    let summary = report.update_safety.expect("summary");
    assert_eq!(summary.mode, "disabled");
    assert_eq!(summary.baseline_sources[0].status, "ignored_disabled");
    assert!(report
        .diagnostic_codes()
        .contains(&"UPDATE.BASELINE_IGNORED_DISABLED".into()));
}
```

Run: `cargo test -p anki_forge disabled_mode_ignores_baseline_but_records_summary`

Expected: FAIL until disabled mode summary is implemented.

- [ ] **Step 2: Implement baseline source summary helpers**

In `anki_forge/src/update_safety/report.rs`, add:

```rust
use crate::build::BaselineSourceSummary;

pub fn ignored_previous_apkg_source(path: &std::path::Path) -> BaselineSourceSummary {
    BaselineSourceSummary {
        source_kind: "previous_apkg".into(),
        source_ref: "baseline.previous_apkg.primary".into(),
        display_path: Some(path.display().to_string()),
        status: "ignored_disabled".into(),
        used_for_reconcile: false,
        limitations: vec![],
        diagnostic_codes: vec!["UPDATE.BASELINE_IGNORED_DISABLED".into()],
    }
}
```

Add this helper for lockfile paths:

```rust
pub fn ignored_lockfile_source(path: &std::path::Path) -> BaselineSourceSummary {
    BaselineSourceSummary {
        source_kind: "lockfile".into(),
        source_ref: "baseline.identity_lockfile.primary".into(),
        display_path: Some(path.display().to_string()),
        status: "ignored_disabled".into(),
        used_for_reconcile: false,
        limitations: vec![],
        diagnostic_codes: vec!["UPDATE.BASELINE_IGNORED_DISABLED".into()],
    }
}

pub fn summary_from_disabled_mode(
    current: &crate::update_safety::model::IdentityIndex,
    baseline_sources: Vec<BaselineSourceSummary>,
    blocking_diagnostics: Vec<String>,
) -> crate::build::UpdateSafetySummary {
    crate::build::UpdateSafetySummary {
        mode: "disabled".into(),
        baseline_sources,
        notes_preserved: 0,
        notes_derived: current.notes.len(),
        notes_failed: 0,
        baseline_conflicts: 0,
        blocking_diagnostics,
        lockfile_written: false,
    }
}
```

- [ ] **Step 3: Short-circuit disabled baseline loading**

In `Project::build`, refactor the Task 11/12 baseline-loading and reconcile code into an explicit mode branch. Place this branch immediately after current identity generation and the current-identity error gate, before any call to `read_lockfile` or `load_previous_apkg_identity_index`. The disabled branch must not fall through into baseline loading.

```rust
let mut reconcile_output: Option<crate::update_safety::reconcile::ReconcileOutput> = None;
let mut writer_guid_plan: Option<writer_core::WriterGuidPlan> = None;
let mut update_safety_summary: Option<crate::build::UpdateSafetySummary> = None;
let mut lockfile_written = false;
let disabled_update_safety = matches!(update_mode, crate::update_safety::EffectiveMode::Disabled);
if disabled_update_safety {
    let mut baseline_sources = Vec::new();
    if let Some(path) = options.compare_to.as_ref() {
        diagnostics.push(Diagnostic {
            code: DiagnosticCode::new("UPDATE.BASELINE_IGNORED_DISABLED"),
            severity: Severity::Info,
            message: "compare_to baseline ignored because update safety is disabled".into(),
            source: Some(SourcePath::new(path.display().to_string())),
            help: Some("remove update_safety(UpdateSafetyMode::Disabled) to analyze the baseline".into()),
        });
        baseline_sources.push(crate::update_safety::report::ignored_previous_apkg_source(path));
    }
    if let Some(path) = options.identity_lockfile.as_ref() {
        baseline_sources.push(crate::update_safety::report::ignored_lockfile_source(path));
    }
    let reconcile = crate::update_safety::reconcile::current_only_reconcile(&current_identity.index)
        .map_err(|err| BuildError::new(
            BuildReport {
                artifact: None,
                counts: BuildCounts {
                    notes: normalized.notes.len(),
                    cards: count_phase1_cards_without_inspect(&normalized),
                    media: normalized.media_bindings.len(),
                },
                media: MediaSummary::from_normalized_ir(&normalized, &diagnostics),
                diagnostics: diagnostics.clone(),
                metrics: BuildMetrics { duration: started.elapsed() },
                inspect: None,
                update_safety: None,
                status: "invalid".into(),
            },
            BuildFailureCause::Diagnostics,
        ))?;
    writer_guid_plan = Some(writer_core::WriterGuidPlan {
        assignments: reconcile.assignments.clone(),
    });
    update_safety_summary = Some(crate::update_safety::report::summary_from_disabled_mode(
        &current_identity.index,
        baseline_sources,
        diagnostics
            .iter()
            .filter(|item| item.severity == Severity::Error)
            .map(|item| item.code.to_string())
            .collect(),
    ));
    reconcile_output = Some(reconcile);
} 
```

Keep the current-derivation writer plan so metadata is embedded.

After this `if disabled_update_safety` block, wrap the Task 12 enabled-mode code in an `if !disabled_update_safety` branch. That enabled-mode block begins with `let lockfile = if let Some(path) = options.identity_lockfile.as_ref()` and ends after optional lockfile writing sets `lockfile_written`; at the end of that enabled branch, assign `reconcile_output = Some(reconcile)`, `writer_guid_plan = Some(writer_guid_plan)`, and `update_safety_summary = Some(crate::update_safety::report::summary_from_reconcile(update_mode, &reconcile, &diagnostics, lockfile_written))`. Immediately before the writer call, unwrap the branch outputs:

```rust
let reconcile = reconcile_output.expect("update safety branch sets reconcile output");
let writer_guid_plan = writer_guid_plan.expect("update safety branch sets writer GUID plan");
let update_safety_summary =
    update_safety_summary.expect("update safety branch sets update safety summary");
```

The final `Project::build` shape after Task 13 must be:

1. Validate Product input and lower/normalize as before.
2. Select `update_mode`; validate project stable id context; build `current_identity`; return before writer if current identity emitted an error.
3. Initialize `reconcile_output`, `writer_guid_plan`, `update_safety_summary`, and `lockfile_written`.
4. If disabled, record ignored baseline sources, run `current_only_reconcile`, set a current-derivation writer plan, set disabled summary, and skip all baseline loading.
5. If enabled, read lockfile if configured, load previous APKG if configured, reconcile with priority previous APKG then lockfile then current derivation, run merge safety against previous APKG or lockfile baseline, return before writer in strict mode if any update-safety error exists, write lockfile after successful artifact output when requested, and set enabled summary.
6. Call `writer_core::build_with_guid_plan` exactly once with the branch-selected writer plan.
7. Copy artifact, inspect if requested, build final `BuildReport` with the branch-selected update-safety summary, and call `ensure_success`.

- [ ] **Step 4: Aggregate high-volume diagnostics in pretty report**

In `anki_forge/src/build/report.rs`, update `pretty_report` to append update-safety summary lines when present:

```rust
if let Some(update) = &self.update_safety {
    lines.push("Update safety:".into());
    lines.push(format!("  mode: {}", update.mode));
    lines.push(format!("  notes_preserved: {}", update.notes_preserved));
    lines.push(format!("  notes_derived: {}", update.notes_derived));
    lines.push(format!("  notes_failed: {}", update.notes_failed));
}
```

Then aggregate high-volume update diagnostics by code:

```rust
let mut update_diagnostics = std::collections::BTreeMap::<String, (usize, Vec<String>)>::new();
for diagnostic in &self.diagnostics {
    let code = diagnostic.code.as_str();
    if !code.starts_with("UPDATE.") {
        continue;
    }
    let entry = update_diagnostics
        .entry(code.to_string())
        .or_insert_with(|| (0, Vec::new()));
    entry.0 += 1;
    if entry.1.len() < 3 {
        if let Some(source) = diagnostic.source.as_ref() {
            entry.1.push(source.as_str().to_string());
        }
    }
}
if !update_diagnostics.is_empty() {
    lines.push("Update diagnostics:".into());
    for (code, (count, samples)) in update_diagnostics {
        if samples.is_empty() {
            lines.push(format!("  {code}: count={count}"));
        } else {
            lines.push(format!("  {code}: count={count} samples={}", samples.join(", ")));
        }
    }
}
```

Do not remove existing diagnostic lines.

- [ ] **Step 5: Run report tests**

Run: `cargo test -p anki_forge update_safety_report_tests`

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add anki_forge/src/update_safety/report.rs anki_forge/src/product/project.rs anki_forge/src/build/report.rs anki_forge/tests/update_safety_report_tests.rs
git commit -m "report: summarize update safety baselines"
```

---

### Task 14: Contract Fixtures, Full Test Matrix, and Oracle Docs

**Files:**
- Create: `contracts/fixtures/update-safety/current-index.json`
- Create: `contracts/fixtures/update-safety/identity-lockfile.json`
- Create: `contracts/fixtures/update-safety/update-safety-summary.json`
- Modify: `contracts/fixtures/index.yaml`
- Modify: `contract_tools/src/fixtures.rs`
- Modify: `contract_tools/tests/fixture_gate_tests.rs`
- Create: `docs/manual-validation/phase3-update-safety-oracle.md`
- Create: `docs/manual-validation/phase3-notes-data-carrier-probe.md`
- Test: `anki_forge/tests/update_safety_matrix_tests.rs`

- [ ] **Step 1: Add fixture gate test for update-safety fixtures**

Add to `contract_tools/tests/fixture_gate_tests.rs`:

```rust
#[test]
fn update_safety_contract_fixtures_validate() {
    let root = contracts_root();
    let fixtures = [
        ("schema/identity-index.schema.json", "fixtures/update-safety/current-index.json"),
        ("schema/identity-lockfile.schema.json", "fixtures/update-safety/identity-lockfile.json"),
        ("schema/update-safety-summary.schema.json", "fixtures/update-safety/update-safety-summary.json"),
    ];
    for (schema_rel, fixture_rel) in fixtures {
        let schema_raw = std::fs::read_to_string(root.join(schema_rel)).expect("schema");
        let fixture_raw = std::fs::read_to_string(root.join(fixture_rel)).expect("fixture");
        let schema_json: serde_json::Value = serde_json::from_str(&schema_raw).expect("schema json");
        let fixture_json: serde_json::Value = serde_json::from_str(&fixture_raw).expect("fixture json");
        let compiled = jsonschema::JSONSchema::compile(&schema_json).expect("compile schema");
        compiled
            .validate(&fixture_json)
            .unwrap_or_else(|errors| panic!("{fixture_rel} failed schema: {}", errors.map(|e| e.to_string()).collect::<Vec<_>>().join("; ")));
    }
}

#[test]
fn update_safety_fixture_catalog_lists_required_scenarios() {
    let root = contracts_root();
    let raw = std::fs::read_to_string(root.join("fixtures/index.yaml"))
        .expect("fixture catalog");
    for id in [
        "update-safety-current-index-generation",
        "update-safety-lockfile-roundtrip",
        "update-safety-previous-apkg-priority",
        "update-safety-guid-preservation",
        "update-safety-new-note-guid-derivation",
        "update-safety-baseline-identity-unrecoverable",
        "update-safety-field-config-id-preservation",
        "update-safety-template-config-id-preservation",
        "update-safety-template-ord-warning",
        "update-safety-absent-entry-reintroduced",
        "update-safety-normalized-note-id-mismatch",
        "update-safety-field-ord-warning",
    ] {
        assert!(raw.contains(id), "fixture catalog missing {id}");
    }
}
```

Run: `cargo test -p contract_tools update_safety_contract_fixtures_validate`

Expected: FAIL because fixtures do not exist.

- [ ] **Step 2: Add minimal contract fixtures**

Create `contracts/fixtures/update-safety/current-index.json`:

```json
{
  "schema_version": "identity-index-v1",
  "source_kind": "current",
  "source_ref": "current",
  "writer_policy_ref": "writer-policy.default@1.0.0",
  "project_stable_id": "spanish",
  "notes": [
    {
      "stable_id": "es:hola",
      "normalized_note_id": "es:hola",
      "anki_guid": "es:hola",
      "current_guid_candidate": "es:hola",
      "guid_derivation_version": "guid.raw-stable-id.v1",
      "note_type_id": "basic",
      "recipe_id": "product.explicit-or-normalized.v1",
      "canonical_payload_hash": null,
      "provenance": "ExplicitStableId",
      "used_override": false,
      "entry_lifecycle": "active",
      "source_path": "note[id='es:hola']",
      "recovery_method": "current_resolution"
    }
  ],
  "notetypes": [
    {
      "note_type_id": "basic",
      "anki_model_id": null,
      "name": "Basic",
      "fields": [
        {
          "field_key": "Front",
          "field_name": "Front",
          "ord": 0,
          "config_id": 1,
          "tag": 0
        },
        {
          "field_key": "Back",
          "field_name": "Back",
          "ord": 1,
          "config_id": 2,
          "tag": 1
        }
      ],
      "templates": [
        {
          "template_key": "Card 1",
          "template_name": "Card 1",
          "ord": 0,
          "config_id": 1
        }
      ]
    }
  ],
  "limitations": []
}
```

Create `contracts/fixtures/update-safety/identity-lockfile.json` with:

```json
{
  "schema_version": "identity-lockfile-v1",
  "project_stable_id": "spanish",
  "writer_policy_ref": "writer-policy.default@1.0.0",
  "identity_index": {
    "schema_version": "identity-index-v1",
    "source_kind": "lockfile",
    "source_ref": "baseline.identity_lockfile.primary",
    "writer_policy_ref": "writer-policy.default@1.0.0",
    "project_stable_id": "spanish",
    "notes": [],
    "notetypes": [],
    "limitations": []
  },
  "generated_by": {
    "tool": "anki-forge",
    "tool_version": "0.0.0-fixture",
    "writer_policy_ref": "writer-policy.default@1.0.0"
  }
}
```

Create `contracts/fixtures/update-safety/update-safety-summary.json` with:

```json
{
  "mode": "strict",
  "baseline_sources": [],
  "notes_preserved": 1,
  "notes_derived": 0,
  "notes_failed": 0,
  "baseline_conflicts": 0,
  "blocking_diagnostics": [],
  "lockfile_written": true
}
```

Add this category arm to `contract_tools/src/fixtures.rs` in `run_fixture_gates`:

```rust
"phase3-update-safety" => {
    let case_value: serde_yaml::Value = load_yaml_model(&input_path)?;
    ensure!(
        case_value
            .get("kind")
            .and_then(|value| value.as_str())
            == Some("phase3-update-safety-case"),
        "phase3 update-safety fixture must declare kind=phase3-update-safety-case: {}",
        case.id
    );
    ensure!(
        case_value.get("scenario").and_then(|value| value.as_str()).is_some(),
        "phase3 update-safety fixture must declare scenario: {}",
        case.id
    );
}
```

Create these twelve YAML files under `contracts/fixtures/update-safety/`:

```yaml
kind: phase3-update-safety-case
scenario: current index generation
expected_diagnostics: []
```

For each file, set `scenario` and `expected_diagnostics` to the mapped values below.

Use this exact mapping from catalog id to file, scenario string, and expected diagnostics:

```text
update-safety-current-index-generation -> current-index-generation.case.yaml -> current index generation -> []
update-safety-lockfile-roundtrip -> lockfile-roundtrip.case.yaml -> lockfile roundtrip -> ["UPDATE.LOCKFILE_WRITTEN"]
update-safety-previous-apkg-priority -> previous-apkg-priority.case.yaml -> previous APKG priority over lockfile -> ["UPDATE.BASELINE_CONFLICT_GUID", "UPDATE.GUID_PRESERVED_FROM_PREVIOUS"]
update-safety-guid-preservation -> guid-preservation.case.yaml -> GUID preservation -> ["UPDATE.GUID_PRESERVED_FROM_PREVIOUS"]
update-safety-new-note-guid-derivation -> new-note-guid-derivation.case.yaml -> new note GUID derivation -> ["UPDATE.GUID_DERIVED_FOR_NEW_NOTE"]
update-safety-baseline-identity-unrecoverable -> baseline-identity-unrecoverable.case.yaml -> baseline identity unrecoverable -> ["UPDATE.BASELINE_IDENTITY_UNRECOVERABLE"]
update-safety-field-config-id-preservation -> field-config-id-preservation.case.yaml -> field config id preservation -> []
update-safety-template-config-id-preservation -> template-config-id-preservation.case.yaml -> template config id preservation -> []
update-safety-template-ord-warning -> template-ord-warning.case.yaml -> template ord warning -> ["UPDATE.TEMPLATE_ORD_CHANGED"]
update-safety-absent-entry-reintroduced -> absent-entry-reintroduced.case.yaml -> absent_from_current entry reintroduced -> ["UPDATE.GUID_PRESERVED_FROM_LOCKFILE"]
update-safety-normalized-note-id-mismatch -> normalized-note-id-mismatch.case.yaml -> normalized_note_id versus stable_id corruption rejected -> ["UPDATE.NORMALIZED_NOTE_ID_MISMATCH"]
update-safety-field-ord-warning -> field-ord-warning.case.yaml -> field ord warning -> ["UPDATE.FIELD_ORD_CHANGED"]
```

Append these entries to `contracts/fixtures/index.yaml`:

```yaml
  - id: update-safety-current-index-generation
    category: phase3-update-safety
    compatibility_class: additive_compatible
    upgrade_rules:
      - fixture_updates_required
    input: fixtures/update-safety/current-index-generation.case.yaml
  - id: update-safety-lockfile-roundtrip
    category: phase3-update-safety
    compatibility_class: additive_compatible
    upgrade_rules:
      - fixture_updates_required
    input: fixtures/update-safety/lockfile-roundtrip.case.yaml
  - id: update-safety-previous-apkg-priority
    category: phase3-update-safety
    compatibility_class: additive_compatible
    upgrade_rules:
      - fixture_updates_required
    input: fixtures/update-safety/previous-apkg-priority.case.yaml
  - id: update-safety-guid-preservation
    category: phase3-update-safety
    compatibility_class: additive_compatible
    upgrade_rules:
      - fixture_updates_required
    input: fixtures/update-safety/guid-preservation.case.yaml
  - id: update-safety-new-note-guid-derivation
    category: phase3-update-safety
    compatibility_class: additive_compatible
    upgrade_rules:
      - fixture_updates_required
    input: fixtures/update-safety/new-note-guid-derivation.case.yaml
  - id: update-safety-baseline-identity-unrecoverable
    category: phase3-update-safety
    compatibility_class: additive_compatible
    upgrade_rules:
      - fixture_updates_required
    input: fixtures/update-safety/baseline-identity-unrecoverable.case.yaml
  - id: update-safety-field-config-id-preservation
    category: phase3-update-safety
    compatibility_class: additive_compatible
    upgrade_rules:
      - fixture_updates_required
    input: fixtures/update-safety/field-config-id-preservation.case.yaml
  - id: update-safety-template-config-id-preservation
    category: phase3-update-safety
    compatibility_class: additive_compatible
    upgrade_rules:
      - fixture_updates_required
    input: fixtures/update-safety/template-config-id-preservation.case.yaml
  - id: update-safety-template-ord-warning
    category: phase3-update-safety
    compatibility_class: additive_compatible
    upgrade_rules:
      - fixture_updates_required
    input: fixtures/update-safety/template-ord-warning.case.yaml
  - id: update-safety-absent-entry-reintroduced
    category: phase3-update-safety
    compatibility_class: additive_compatible
    upgrade_rules:
      - fixture_updates_required
    input: fixtures/update-safety/absent-entry-reintroduced.case.yaml
  - id: update-safety-normalized-note-id-mismatch
    category: phase3-update-safety
    compatibility_class: additive_compatible
    upgrade_rules:
      - fixture_updates_required
    input: fixtures/update-safety/normalized-note-id-mismatch.case.yaml
  - id: update-safety-field-ord-warning
    category: phase3-update-safety
    compatibility_class: additive_compatible
    upgrade_rules:
      - fixture_updates_required
    input: fixtures/update-safety/field-ord-warning.case.yaml
```

- [ ] **Step 3: Add diagnostic coverage matrix test**

Create `anki_forge/tests/update_safety_matrix_tests.rs`:

```rust
#[test]
fn update_safety_diagnostic_matrix_lists_every_update_code() {
    let matrix = include_str!("../../docs/manual-validation/phase3-update-safety-oracle.md");
    for code in [
        "UPDATE.BASELINE_APKG_UNREADABLE",
        "UPDATE.BASELINE_LOCKFILE_UNREADABLE",
        "UPDATE.BASELINE_SCHEMA_UNSUPPORTED",
        "UPDATE.BASELINE_CONFLICT_GUID",
        "UPDATE.GUID_PRESERVED_FROM_PREVIOUS",
        "UPDATE.GUID_DERIVATION_DRIFT",
        "UPDATE.FIELD_MERGE_ID_CHANGED",
        "UPDATE.TEMPLATE_MERGE_ID_CHANGED",
        "UPDATE.NOTE_DATA_METADATA_UNMERGEABLE",
    ] {
        assert!(matrix.contains(code), "matrix missing {code}");
    }
}
```

Run: `cargo test -p anki_forge update_safety_diagnostic_matrix_lists_every_update_code`

Expected: FAIL until the manual validation doc lists the codes.

- [ ] **Step 4: Add manual oracle docs**

Create `docs/manual-validation/phase3-notes-data-carrier-probe.md`:

```markdown
# Phase 3 Notes Data Carrier Probe

Record one row per Anki build tested.

Required fields:

- Date
- Platform
- Anki version
- anki-forge commit
- Input APKG path and SHA-256
- Imported note count
- Exported APKG path and SHA-256
- Whether `notes.data.anki_forge_identity` survived import/export
- Observed fallback path if metadata did not survive

Probe steps:

1. Build a one-note APKG with explicit `stable_id`.
2. Inspect the APKG SQLite `notes.data` and confirm `anki_forge_identity` exists.
3. Import into Anki.
4. Export the deck back to APKG.
5. Inspect exported APKG SQLite `notes.data`.
6. Mark the carrier as preserved only when the JSON object and `stable_id` survived.
```

Create `docs/manual-validation/phase3-update-safety-oracle.md`:

```markdown
# Phase 3 Update Safety Oracle

Required evidence for each scenario:

- Date
- Platform
- Anki version
- anki-forge commit
- Scenario input APKG SHA-256
- Updated APKG SHA-256
- Note count before and after import
- Card count before and after import
- GUID comparison result
- Duplicate note outcome
- Relevant diagnostics

Diagnostic coverage matrix:

- `UPDATE.BASELINE_APKG_UNREADABLE`: unit or integration test
- `UPDATE.BASELINE_LOCKFILE_UNREADABLE`: unit or integration test
- `UPDATE.BASELINE_SCHEMA_UNSUPPORTED`: lockfile schema test
- `UPDATE.BASELINE_CONFLICT_GUID`: reconcile test
- `UPDATE.GUID_PRESERVED_FROM_PREVIOUS`: compare_to integration test
- `UPDATE.GUID_DERIVATION_DRIFT`: reconcile test with legacy GUID
- `UPDATE.FIELD_MERGE_ID_CHANGED`: merge safety test
- `UPDATE.TEMPLATE_MERGE_ID_CHANGED`: merge safety test
- `UPDATE.NOTE_DATA_METADATA_UNMERGEABLE`: writer APKG test

Manual scenarios:

1. First import then update with same stable ids updates existing notes.
2. Adding a new note inserts only the new note.
3. Field rename with stable field key/config id remains update-safe.
4. Field config id drift is caught before import.
5. Template reorder is visible as a scheduling risk signal.
```

- [ ] **Step 5: Run contract and matrix tests**

Run:

```bash
cargo test -p contract_tools update_safety_contract_fixtures_validate
cargo test -p anki_forge update_safety_diagnostic_matrix_lists_every_update_code
```

Expected: both PASS.

- [ ] **Step 6: Add performance benchmark test for lockfile scale**

Add a non-default ignored test in `anki_forge/tests/update_safety_lockfile_tests.rs`:

```rust
#[test]
#[ignore = "manual performance boundary check"]
fn lockfile_parse_scale_100k_entries() {
    let root = tempfile::tempdir().expect("tempdir");
    let path = root.path().join("large.lock.json");
    let mut lockfile = sample_lockfile_with_entries(100_000);
    let start = std::time::Instant::now();
    write_lockfile_atomic(&path, &lockfile).expect("write large lockfile");
    let write_elapsed = start.elapsed();
    let start = std::time::Instant::now();
    lockfile = read_lockfile(&path).expect("read large lockfile");
    let read_elapsed = start.elapsed();
    assert_eq!(lockfile.identity_index.notes.len(), 100_000);
    eprintln!("write={write_elapsed:?} read={read_elapsed:?}");
}

fn sample_lockfile_with_entries(count: usize) -> anki_forge::update_safety::model::IdentityLockfile {
    let mut lockfile = anki_forge::update_safety::model::IdentityLockfile {
        schema_version: "identity-lockfile-v1".into(),
        project_stable_id: "scale-project".into(),
        writer_policy_ref: "writer-policy.default@1.0.0".into(),
        identity_index: anki_forge::update_safety::model::IdentityIndex::empty_lockfile(
            "scale-project",
            "writer-policy.default@1.0.0",
        ),
        generated_by: anki_forge::update_safety::model::GeneratedBy {
            tool: "anki-forge".into(),
            tool_version: env!("CARGO_PKG_VERSION").into(),
            writer_policy_ref: "writer-policy.default@1.0.0".into(),
        },
    };
    lockfile.identity_index.notes = (0..count)
        .map(|index| bench_note_entry(&format!("note-{index:06}"), &format!("guid-{index:06}")))
        .collect();
    lockfile
}

fn bench_note_entry(
    stable_id: &str,
    guid: &str,
) -> anki_forge::update_safety::model::NoteIdentityEntry {
    anki_forge::update_safety::model::NoteIdentityEntry {
        stable_id: stable_id.into(),
        normalized_note_id: Some(stable_id.into()),
        anki_guid: guid.into(),
        current_guid_candidate: stable_id.into(),
        guid_derivation_version: "guid.raw-stable-id.v1".into(),
        note_type_id: "basic".into(),
        recipe_id: "product.explicit-or-normalized.v1".into(),
        canonical_payload_hash: None,
        provenance: "ExplicitStableId".into(),
        used_override: false,
        entry_lifecycle: "active".into(),
        source_path: "benchmark".into(),
        recovery_method: "current_resolution".into(),
    }
}
```

Run manually when assessing pruning urgency:

```bash
cargo test -p anki_forge lockfile_parse_scale_100k_entries -- --ignored --nocapture
```

Expected: PASS and printed read/write timings. If read or write is multi-second on the project baseline machine, lower `UPDATE.LOCKFILE_ABSENT_ENTRIES_HIGH` threshold and schedule pruning before Phase 3 exit.

- [ ] **Step 7: Commit**

```bash
git add contracts/fixtures/update-safety/current-index.json contracts/fixtures/update-safety/identity-lockfile.json contracts/fixtures/update-safety/update-safety-summary.json contracts/fixtures/update-safety/*.case.yaml contracts/fixtures/index.yaml contract_tools/src/fixtures.rs contract_tools/tests/fixture_gate_tests.rs docs/manual-validation/phase3-update-safety-oracle.md docs/manual-validation/phase3-notes-data-carrier-probe.md anki_forge/tests/update_safety_matrix_tests.rs anki_forge/tests/update_safety_lockfile_tests.rs
git commit -m "tests: add update safety contract fixtures"
```

---

## Final Verification

- [ ] Run unit and integration tests:

```bash
cargo test -p anki_forge update_safety
cargo test -p writer_core writer_guid_plan
cargo test -p writer_core inspect_apkg_reports_note_identity_metadata_from_notes_data
cargo test -p contract_tools update_safety
```

Expected: all commands PASS.

- [ ] Run full workspace tests:

```bash
cargo test --workspace
```

Expected: PASS.

- [ ] Run contract fixture gates if they are separated from `cargo test --workspace` in CI:

```bash
cargo test -p contract_tools fixture
cargo test -p contract_tools schema
```

Expected: PASS.

- [ ] Perform early manual metadata carrier probe using `docs/manual-validation/phase3-notes-data-carrier-probe.md`.

Expected: The document has a filled row for the tested Anki platform/version. If `notes.data.anki_forge_identity` is stripped, the release notes must state that re-exported APKG baselines require lockfile join or `guid == stable_id` compatibility recovery.

- [ ] Run final status check:

```bash
git status --short
```

Expected: no unstaged or uncommitted implementation changes.

## Risk Notes for Implementers

- Do not make `notes.data` embedding conditional on update-safety mode. Disabled mode still embeds metadata for notes with resolved stable ids so ordinary APKGs can become future baselines.
- Do not use APKG row ids as Product identity. Use `stable_id` and `note_type_id` as Product keys.
- Do not silently skip writer GUID plan mismatches. The writer must fail before APKG emission.
- Do not group active and absent lockfile entries separately when writing canonical lockfile JSON. Sort identity entries by stable key.
- Do not treat real Anki oracle automation as required for the first implementation pass. The early manual carrier probe is required; full release/nightly oracle scenarios can remain manual until automation exists.
