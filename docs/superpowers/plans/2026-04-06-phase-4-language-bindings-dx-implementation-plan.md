# Phase 4 Language Bindings and DX Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver a Phase 4 access layer that adds a normative Rust facade, stable CLI-backed Node/Python bindings, a shared conformance runner, and enough docs/examples to complete the minimal `normalize -> build -> inspect -> diff` flow from every supported surface inside the repository.

**Architecture:** Keep `contracts/` as the only semantic source of truth, keep `authoring_core` and `writer_core` as the core semantic engines, add a new `anki_forge` Rust facade that orchestrates those engines directly for its normative path, and treat `contract_tools --output contract-json` as the stable protocol surface wrapped by Node and Python. Build the language bindings around a common runtime-discovery model, explicit failure classification, and a single conformance runner that proves Rust facade, CLI, Node structured, and Python structured all preserve the same protocol truth.

**Tech Stack:** Rust workspace (`cargo`, `serde`, `serde_json`, `serde_yaml`, `jsonschema`, `clap`), Node.js built-in modules (`node:child_process`, `node:test`, ESM), Python stdlib (`subprocess`, `unittest`, `dataclasses`, `pathlib`), YAML/JSON contract assets, GitHub Actions

---

## Scope Check

This plan still targets one coherent subsystem: `Phase 4 Language Bindings and DX`.

The work touches Rust facade APIs, CLI adapter seams, Node/Python wrappers, examples, and conformance, but those are not independent product areas. They are all different access surfaces over the same contract-defined semantics and need to move together so that shared conformance, runtime discovery, failure classification, and examples stay aligned.

Do not split this into separate implementation plans unless the user explicitly asks to peel off one of these areas into its own sub-project:

- Rust facade only
- Node wrapper only
- Python wrapper only
- shared conformance/CI only

## File Structure Map

### Rust facade workspace

- Modify: `Cargo.toml` - add `anki_forge` as a workspace member
- Create: `anki_forge/Cargo.toml` - crate metadata and dependencies for facade/runtime helpers
- Create: `anki_forge/src/lib.rs` - typed-core facade exports and facade version helpers
- Create: `anki_forge/src/runtime/mod.rs` - runtime module exports
- Create: `anki_forge/src/runtime/discovery.rs` - workspace/runtime discovery and resolved-runtime types
- Create: `anki_forge/src/runtime/assets.rs` - manifest loading, asset lookup, writer-policy/build-context loading
- Create: `anki_forge/src/runtime/schema.rs` - JSON Schema loading/validation for runtime file-oriented operations
- Create: `anki_forge/src/runtime/normalize.rs` - file-oriented normalization orchestration
- Create: `anki_forge/src/runtime/build.rs` - file-oriented build orchestration
- Create: `anki_forge/src/runtime/inspect.rs` - file-oriented inspection helpers
- Create: `anki_forge/src/runtime/diff.rs` - file-oriented diff helpers
- Create: `anki_forge/examples/minimal_flow.rs` - Rust facade example
- Create: `anki_forge/examples/conformance_surface.rs` - parameterized Rust facade adapter for the shared conformance runner
- Create: `anki_forge/tests/typed_core_tests.rs` - typed facade tests
- Create: `anki_forge/tests/runtime_facade_tests.rs` - runtime discovery and file-oriented facade tests

### CLI alignment

- Modify: `contract_tools/Cargo.toml` - add `anki_forge` dependency for command-layer reuse
- Modify: `contract_tools/src/normalize_cmd.rs` - call Rust facade runtime normalization helper
- Modify: `contract_tools/src/build_cmd.rs` - call Rust facade runtime build helper
- Modify: `contract_tools/src/inspect_cmd.rs` - call Rust facade runtime inspect helper
- Modify: `contract_tools/src/diff_cmd.rs` - call Rust facade runtime diff helper
- Modify: `contract_tools/tests/cli_tests.rs` - assert CLI contract-json matches Rust facade runtime output for representative flows

### Node wrapper

- Create: `bindings/node/package.json` - local package metadata, scripts, and future package boundary
- Create: `bindings/node/README.md` - Node wrapper overview, layers, and workspace-mode usage
- Create: `bindings/node/src/version.js` - wrapper API version hook
- Create: `bindings/node/src/errors.js` - `RuntimeInvocationError` and `ProtocolParseError`
- Create: `bindings/node/src/runtime.js` - workspace/installed runtime locator and metadata loader
- Create: `bindings/node/src/raw.js` - raw CLI launcher and argv builder
- Create: `bindings/node/src/contracts.js` - command-specific contract shape and version validators
- Create: `bindings/node/src/helpers.js` - convenience-only helper/view layer
- Create: `bindings/node/src/structured.js` - structured `normalize/build/inspect/diff` wrappers
- Create: `bindings/node/src/index.js` - public exports
- Create: `bindings/node/examples/minimal-flow.mjs` - Node structured-wrapper example
- Create: `bindings/node/test/raw.test.js` - Node raw/runtime tests
- Create: `bindings/node/test/structured.test.js` - Node structured/error/helper tests

### Python wrapper

- Create: `bindings/python/pyproject.toml` - local package metadata and future package boundary
- Create: `bindings/python/README.md` - Python wrapper overview, layers, and workspace-mode usage
- Create: `bindings/python/src/anki_forge_python/version.py` - wrapper API version hook
- Create: `bindings/python/src/anki_forge_python/errors.py` - `RuntimeInvocationError` and `ProtocolParseError`
- Create: `bindings/python/src/anki_forge_python/runtime.py` - workspace/installed runtime locator and metadata loader
- Create: `bindings/python/src/anki_forge_python/raw.py` - raw CLI launcher and argv builder
- Create: `bindings/python/src/anki_forge_python/contracts.py` - command-specific contract shape and version validators
- Create: `bindings/python/src/anki_forge_python/helpers.py` - convenience-only helper/view layer
- Create: `bindings/python/src/anki_forge_python/structured.py` - structured `normalize/build/inspect/diff` wrappers
- Create: `bindings/python/src/anki_forge_python/__init__.py` - public exports
- Create: `bindings/python/examples/minimal_flow.py` - Python structured-wrapper example
- Create: `bindings/python/tests/test_raw.py` - Python raw/runtime tests
- Create: `bindings/python/tests/test_structured.py` - Python structured/error/helper tests

### Shared conformance and docs

- Create: `tests/conformance/README.md` - suite purpose, coverage split, and runner identity
- Create: `tests/conformance/run_phase4_suite.py` - canonical shared conformance runner
- Create: `tests/conformance/node_surface.mjs` - Node structured-surface adapter for shared cases
- Create: `tests/conformance/python_surface.py` - Python structured-surface adapter for shared cases
- Modify: `.github/workflows/contract-ci.yml` - add Node/Python setup and invoke the shared conformance runner
- Modify: `README.md` - document Phase 4 language surfaces and runner entrypoints
- Create: `docs/superpowers/checklists/phase-4-exit-evidence.md` - exact final verification commands and evidence checklist

### Implementation notes

- Keep the Rust facade normative path direct: no shell-out from `anki_forge` runtime helpers to `contract_tools`.
- Keep `contract_tools` as an independent protocol/gate layer even when it reuses the facade runtime for command execution.
- Treat `contracts/manifest.yaml` and the Phase 3 assets already registered there as the authoritative runtime asset source for Phase 4. Task 2 should verify and reuse the existing `writer_policy` and `build_context_default` entries instead of creating a parallel default build-context asset.
- Preserve the current `contract-json` field names. Wrapper helper/view layers may only add derived projections.
- Keep runtime metadata explicit. Every wrapper-facing success or failure path should preserve resolved manifest, bundle root, mode, and launcher/executable details where applicable.
- Keep wrapper API version separate from bundle version and tool contract version. The code should carry all three as separate concepts.
- Node/Python structured layers must validate command-specific contract shape and version hooks after JSON parsing. Detecting malformed JSON is not enough; a syntactically valid payload with missing required fields or mismatched contract-version metadata is a `ProtocolParseError`.
- Contract-valid result-level statuses such as `invalid`, `error`, `degraded`, `partial`, and `unavailable` must return structured results when the CLI exits successfully and emits valid contract JSON. Only invocation failures or protocol/parse failures become wrapper exceptions.
- Raw layers must wrap runtime locator/discovery failures into `RuntimeInvocationError` instead of leaking plain `Error` or `RuntimeError`.
- Helper artifact paths must be derived from returned contract refs such as `staging_ref` and `apkg_ref`, not guessed only from request paths.
- The shared conformance runner must execute the same case corpus across Rust facade, CLI, Node structured, and Python structured surfaces and compare canonical payloads. Raw-layer conformance remains secondary and only compares runtime metadata plus invocation-failure classification.
- Do not introduce Node or Python third-party runtime dependencies in Phase 4. Use built-in Node/Python facilities first.
- Use existing contract fixtures wherever possible instead of inventing new binding-only semantic cases.
- The shared conformance suite is a runner identity, not just a naming convention for scattered tests.

### Task 1: Bootstrap the `anki_forge` Rust facade crate

**Files:**
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Create: `anki_forge/Cargo.toml`
- Create: `anki_forge/src/lib.rs`
- Create: `anki_forge/tests/typed_core_tests.rs`

- [ ] **Step 1: Write the failing typed-facade test**

```rust
// anki_forge/tests/typed_core_tests.rs
use anki_forge::{normalize, writer_tool_contract_version, AuthoringDocument, NormalizationRequest};

#[test]
fn typed_facade_reexports_phase2_and_phase3_core_surfaces() {
    let result = normalize(NormalizationRequest::new(AuthoringDocument {
        kind: "authoring-ir".into(),
        schema_version: "0.1.0".into(),
        metadata_document_id: "demo-doc".into(),
        notetypes: vec![],
        notes: vec![],
        media: vec![],
    }));

    assert_eq!(result.tool_contract_version, "phase2-v1");
    assert_eq!(writer_tool_contract_version(), "phase3-v1");
}
```

- [ ] **Step 2: Run the new crate test to verify it fails**

Run: `cargo test -p anki_forge --test typed_core_tests -v`
Expected: FAIL with `package ID specification 'anki_forge' did not match any packages`.

- [ ] **Step 3: Add the workspace member and minimal typed facade**

```toml
# Cargo.toml
[workspace]
members = ["contract_tools", "authoring_core", "writer_core", "anki_forge"]
resolver = "2"

[workspace.package]
edition = "2021"
rust-version = "1.81"

[workspace.lints.rust]
unsafe_code = "forbid"
```

```toml
# anki_forge/Cargo.toml
[package]
name = "anki_forge"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true

[dependencies]
authoring_core = { path = "../authoring_core" }
writer_core = { path = "../writer_core" }

[lints]
workspace = true
```

```rust
// anki_forge/src/lib.rs
pub use authoring_core::{
    assess_risk, normalize, parse_selector, resolve_identity, resolve_selector,
    to_canonical_json as to_authoring_canonical_json, AuthoringDocument, AuthoringMedia,
    AuthoringNote, AuthoringNotetype, ComparisonContext, MergeRiskReport, NormalizationRequest,
    NormalizationResult, NormalizedIr, NormalizedMedia, NormalizedNote, NormalizedNotetype,
    NormalizedTemplate, Selector, SelectorError, SelectorResolveError, SelectorTarget,
};
pub use writer_core::{
    build, build_context_ref, diff_reports, extract_media_references, inspect_apkg,
    inspect_build_result, inspect_staging, policy_ref, to_canonical_json as to_writer_canonical_json,
    BuildArtifactTarget, BuildContext, DiffReport, InspectReport, PackageBuildResult,
    VerificationGateRule, VerificationPolicy, WriterPolicy,
};

pub fn authoring_tool_contract_version() -> &'static str {
    authoring_core::tool_contract_version()
}

pub fn writer_tool_contract_version() -> &'static str {
    writer_core::tool_contract_version()
}

pub fn facade_api_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
```

- [ ] **Step 4: Run the typed-facade test to verify it passes**

Run: `cargo test -p anki_forge --test typed_core_tests -v`
Expected: PASS with `typed_facade_reexports_phase2_and_phase3_core_surfaces`.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml Cargo.lock anki_forge/Cargo.toml anki_forge/src/lib.rs anki_forge/tests/typed_core_tests.rs
git commit -m "feat: add anki_forge typed facade crate"
```

### Task 2: Add Rust runtime discovery and contract asset loading

**Files:**
- Modify: `anki_forge/Cargo.toml`
- Modify: `anki_forge/src/lib.rs`
- Create: `anki_forge/src/runtime/mod.rs`
- Create: `anki_forge/src/runtime/discovery.rs`
- Create: `anki_forge/src/runtime/assets.rs`
- Create: `anki_forge/tests/runtime_facade_tests.rs`

This task relies on the Phase 3 contract assets that already exist in the repository:

- `contracts/manifest.yaml` already registers `writer_policy` and `build_context_default`
- `contracts/contexts/build-context.default.yaml` already exists and should remain the default runtime build context for Phase 4

Do not create a second default build-context asset in a new location unless those Phase 3 assets are actually missing.

- [ ] **Step 1: Write the failing runtime discovery and asset tests**

```rust
// anki_forge/tests/runtime_facade_tests.rs
use std::path::PathBuf;

use anki_forge::runtime::{
    discover_workspace_runtime, load_build_context, load_bundle_from_manifest, load_writer_policy,
    RuntimeMode,
};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn workspace_runtime_discovers_manifest_bundle_root_and_bundle_version() {
    let resolved = discover_workspace_runtime(repo_root()).expect("discover workspace runtime");

    assert_eq!(resolved.mode, RuntimeMode::Workspace);
    assert!(resolved.manifest_path.ends_with("contracts/manifest.yaml"));
    assert!(resolved.bundle_root.ends_with("contracts"));
    assert_eq!(resolved.bundle_version, "0.1.0");
}

#[test]
fn runtime_loads_default_phase3_assets_from_manifest() {
    let bundle = load_bundle_from_manifest(repo_root().join("contracts/manifest.yaml"))
        .expect("load runtime bundle");

    assert!(bundle.assets.contains_key("writer_policy"));
    assert!(bundle.assets.contains_key("build_context_default"));

    let writer_policy = load_writer_policy(&bundle, "default").expect("load writer policy");
    let build_context = load_build_context(&bundle, "default").expect("load build context");

    assert_eq!(writer_policy.id, "writer-policy.default");
    assert_eq!(build_context.id, "build-context.default");
}
```

- [ ] **Step 2: Run the runtime-facade tests to verify they fail**

Run: `cargo test -p anki_forge --test runtime_facade_tests -v`
Expected: FAIL with unresolved module/import errors for `anki_forge::runtime`.

- [ ] **Step 3: Add runtime discovery and asset-loading modules**

```toml
# anki_forge/Cargo.toml
[package]
name = "anki_forge"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true

[dependencies]
anyhow = "1"
authoring_core = { path = "../authoring_core" }
serde = { version = "1", features = ["derive"] }
serde_yaml = "0.9"
writer_core = { path = "../writer_core" }

[lints]
workspace = true
```

```rust
// anki_forge/src/runtime/mod.rs
pub mod assets;
pub mod discovery;

pub use assets::{
    load_build_context, load_bundle_from_manifest, load_writer_policy, resolve_asset_path,
    RuntimeBundle,
};
pub use discovery::{discover_workspace_runtime, ResolvedRuntime, RuntimeMode};
```

```rust
// anki_forge/src/runtime/discovery.rs
use std::path::{Path, PathBuf};

use anyhow::{bail, Context};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeMode {
    Workspace,
    Installed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedRuntime {
    pub mode: RuntimeMode,
    pub manifest_path: PathBuf,
    pub bundle_root: PathBuf,
    pub bundle_version: String,
}

pub fn discover_workspace_runtime(start: impl AsRef<Path>) -> anyhow::Result<ResolvedRuntime> {
    let start = start
        .as_ref()
        .canonicalize()
        .with_context(|| format!("resolve workspace start path: {}", start.as_ref().display()))?;

    let mut current = if start.is_dir() {
        start
    } else {
        start.parent().unwrap_or_else(|| Path::new("/")).to_path_buf()
    };

    loop {
        let manifest_path = current.join("contracts/manifest.yaml");
        if manifest_path.is_file() {
            return super::assets::load_bundle_from_manifest(manifest_path).map(|bundle| bundle.runtime);
        }

        if !current.pop() {
            break;
        }
    }

    bail!("failed to discover contracts/manifest.yaml from workspace path")
}
```

```rust
// anki_forge/src/runtime/assets.rs
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{bail, ensure, Context};
use serde::Deserialize;
use writer_core::{BuildContext, WriterPolicy};

use super::discovery::{ResolvedRuntime, RuntimeMode};

#[derive(Debug, Deserialize)]
struct Compatibility {
    public_axis: String,
}

#[derive(Debug, Deserialize)]
struct ManifestData {
    bundle_version: String,
    compatibility: Compatibility,
    assets: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct RuntimeBundle {
    pub runtime: ResolvedRuntime,
    pub assets: BTreeMap<String, String>,
}

pub fn load_bundle_from_manifest(manifest_path: impl AsRef<Path>) -> anyhow::Result<RuntimeBundle> {
    let manifest_path = manifest_path
        .as_ref()
        .canonicalize()
        .with_context(|| format!("resolve manifest path: {}", manifest_path.as_ref().display()))?;
    let raw = fs::read_to_string(&manifest_path)
        .with_context(|| format!("read manifest: {}", manifest_path.display()))?;
    let manifest: ManifestData =
        serde_yaml::from_str(&raw).context("decode runtime manifest YAML")?;

    ensure!(
        manifest.compatibility.public_axis == "bundle_version",
        "runtime manifest public_axis must be bundle_version"
    );

    let bundle_root = manifest_path
        .parent()
        .context("runtime manifest must live under contracts/")?
        .to_path_buf();

    Ok(RuntimeBundle {
        runtime: ResolvedRuntime {
            mode: RuntimeMode::Workspace,
            manifest_path,
            bundle_root,
            bundle_version: manifest.bundle_version,
        },
        assets: manifest.assets,
    })
}

pub fn resolve_asset_path(bundle: &RuntimeBundle, key: &str) -> anyhow::Result<PathBuf> {
    let rel = bundle
        .assets
        .get(key)
        .with_context(|| format!("missing asset key: {key}"))?;
    let path = bundle.runtime.bundle_root.join(rel);
    let path = path
        .canonicalize()
        .with_context(|| format!("resolve asset path: {}", path.display()))?;

    ensure!(
        path.starts_with(&bundle.runtime.bundle_root),
        "asset path must stay within contracts/: {}",
        path.display()
    );
    ensure!(path.is_file(), "asset path must resolve to a file: {}", path.display());
    Ok(path)
}

pub fn load_writer_policy(bundle: &RuntimeBundle, selector: &str) -> anyhow::Result<WriterPolicy> {
    if selector != "default" {
        bail!("only default writer_policy selector is supported initially");
    }

    let path = resolve_asset_path(bundle, "writer_policy")?;
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("read writer policy: {}", path.display()))?;
    serde_yaml::from_str(&raw).with_context(|| format!("decode writer policy: {}", path.display()))
}

pub fn load_build_context(bundle: &RuntimeBundle, selector: &str) -> anyhow::Result<BuildContext> {
    if selector != "default" {
        bail!("only default build_context selector is supported initially");
    }

    let path = resolve_asset_path(bundle, "build_context_default")?;
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("read build context: {}", path.display()))?;
    serde_yaml::from_str(&raw).with_context(|| format!("decode build context: {}", path.display()))
}
```

```rust
// anki_forge/src/lib.rs
pub mod runtime;

pub use authoring_core::{
    assess_risk, normalize, parse_selector, resolve_identity, resolve_selector,
    to_canonical_json as to_authoring_canonical_json, AuthoringDocument, AuthoringMedia,
    AuthoringNote, AuthoringNotetype, ComparisonContext, MergeRiskReport, NormalizationRequest,
    NormalizationResult, NormalizedIr, NormalizedMedia, NormalizedNote, NormalizedNotetype,
    NormalizedTemplate, Selector, SelectorError, SelectorResolveError, SelectorTarget,
};
pub use writer_core::{
    build, build_context_ref, diff_reports, extract_media_references, inspect_apkg,
    inspect_build_result, inspect_staging, policy_ref, to_canonical_json as to_writer_canonical_json,
    BuildArtifactTarget, BuildContext, DiffReport, InspectReport, PackageBuildResult,
    VerificationGateRule, VerificationPolicy, WriterPolicy,
};

pub fn authoring_tool_contract_version() -> &'static str {
    authoring_core::tool_contract_version()
}

pub fn writer_tool_contract_version() -> &'static str {
    writer_core::tool_contract_version()
}

pub fn facade_api_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
```

- [ ] **Step 4: Run the runtime-facade tests to verify they pass**

Run: `cargo test -p anki_forge --test runtime_facade_tests -v`
Expected: PASS with `workspace_runtime_discovers_manifest_bundle_root_and_bundle_version` and `runtime_loads_default_phase3_assets_from_manifest`.

- [ ] **Step 5: Commit**

```bash
git add anki_forge/Cargo.toml anki_forge/src/lib.rs anki_forge/src/runtime/mod.rs anki_forge/src/runtime/discovery.rs anki_forge/src/runtime/assets.rs anki_forge/tests/runtime_facade_tests.rs
git commit -m "feat: add rust runtime discovery for anki_forge"
```

### Task 3: Add Rust runtime file-oriented operations and align CLI commands to the facade

**Files:**
- Modify: `anki_forge/Cargo.toml`
- Modify: `anki_forge/src/runtime/mod.rs`
- Create: `anki_forge/src/runtime/schema.rs`
- Create: `anki_forge/src/runtime/normalize.rs`
- Create: `anki_forge/src/runtime/build.rs`
- Create: `anki_forge/src/runtime/inspect.rs`
- Create: `anki_forge/src/runtime/diff.rs`
- Modify: `anki_forge/tests/runtime_facade_tests.rs`
- Create: `anki_forge/examples/minimal_flow.rs`
- Create: `anki_forge/examples/conformance_surface.rs`
- Modify: `contract_tools/Cargo.toml`
- Modify: `contract_tools/src/normalize_cmd.rs`
- Modify: `contract_tools/src/build_cmd.rs`
- Modify: `contract_tools/src/inspect_cmd.rs`
- Modify: `contract_tools/src/diff_cmd.rs`
- Modify: `contract_tools/tests/cli_tests.rs`

- [ ] **Step 1: Write the failing runtime-operation and CLI-alignment tests**

```rust
// anki_forge/tests/runtime_facade_tests.rs
use std::path::PathBuf;

use anki_forge::runtime::{
    build_from_path, discover_workspace_runtime, inspect_apkg_path, normalize_from_path,
};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn runtime_normalize_and_build_from_paths_match_repository_contracts() {
    let runtime = discover_workspace_runtime(repo_root()).expect("discover workspace runtime");
    let authoring_input = repo_root().join("contracts/fixtures/valid/minimal-authoring-ir.json");
    let normalized = normalize_from_path(&runtime, &authoring_input).expect("normalize from path");
    assert_eq!(normalized.kind, "normalization-result");
    assert_eq!(normalized.result_status, "success");

    let build_input = repo_root().join("contracts/fixtures/phase3/inputs/basic-normalized-ir.json");
    let artifacts_dir = repo_root().join("tmp/phase4-runtime-facade/basic");
    let build_result = build_from_path(&runtime, &build_input, "default", "default", &artifacts_dir)
        .expect("build from path");
    assert_eq!(build_result.kind, "package-build-result");
    assert_eq!(build_result.result_status, "success");

    let apkg_report =
        inspect_apkg_path(artifacts_dir.join("package.apkg")).expect("inspect apkg from path");
    assert_eq!(apkg_report.kind, "inspect-report");
    assert_eq!(apkg_report.observation_status, "complete");
}
```

```rust
// contract_tools/tests/cli_tests.rs
#[test]
fn build_command_matches_anki_forge_runtime_output() {
    let manifest = contract_tools::contract_manifest_path();
    let repo_root = manifest.parent().unwrap().parent().unwrap();
    let build_input = repo_root.join("contracts/fixtures/phase3/inputs/basic-normalized-ir.json");
    let artifacts_dir = tempdir().unwrap();

    let runtime = anki_forge::runtime::load_bundle_from_manifest(&manifest).unwrap().runtime;
    let runtime_result = anki_forge::runtime::build_from_path(
        &runtime,
        &build_input,
        "default",
        "default",
        artifacts_dir.path(),
    )
    .unwrap();

    let cli_output = run_cli(&[
        "build",
        "--manifest",
        manifest.to_str().unwrap(),
        "--input",
        build_input.to_str().unwrap(),
        "--writer-policy",
        "default",
        "--build-context",
        "default",
        "--artifacts-dir",
        artifacts_dir.path().to_str().unwrap(),
        "--output",
        "contract-json",
    ]);

    assert!(cli_output.status.success());
    let cli_json: serde_json::Value = serde_json::from_slice(&cli_output.stdout).unwrap();
    let runtime_json = serde_json::to_value(runtime_result).unwrap();
    assert_eq!(cli_json, runtime_json);
}
```

- [ ] **Step 2: Run the new facade and CLI tests to verify they fail**

Run: `cargo test -p anki_forge --test runtime_facade_tests -v`
Expected: FAIL with unresolved imports for `normalize_from_path`, `build_from_path`, or `inspect_apkg_path`.

Run: `cargo test -p contract_tools --test cli_tests -v build_command_matches_anki_forge_runtime_output`
Expected: FAIL with unresolved crate/import for `anki_forge` or missing runtime helpers.

- [ ] **Step 3: Implement runtime schema validation, file-oriented operations, and CLI adapters**

```toml
# anki_forge/Cargo.toml
[package]
name = "anki_forge"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true

[dependencies]
anyhow = "1"
authoring_core = { path = "../authoring_core" }
jsonschema = { version = "0.18.3", default-features = false }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
writer_core = { path = "../writer_core" }

[lints]
workspace = true
```

```rust
// anki_forge/src/runtime/schema.rs
use std::{fs, path::Path};

use anyhow::{bail, Context};
use jsonschema::JSONSchema;
use serde_json::Value;

pub fn load_schema(path: impl AsRef<Path>) -> anyhow::Result<Value> {
    let path = path.as_ref();
    let raw = fs::read_to_string(path)
        .with_context(|| format!("read runtime schema: {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("decode runtime schema: {}", path.display()))
}

pub fn validate_value(schema: &Value, value: &Value) -> anyhow::Result<()> {
    let validator = JSONSchema::compile(schema).context("compile runtime schema")?;
    if let Err(errors) = validator.validate(value) {
        let details = errors.map(|error| error.to_string()).collect::<Vec<_>>().join("; ");
        bail!(details);
    }
    Ok(())
}
```

```rust
// anki_forge/src/runtime/normalize.rs
use std::{fs, path::Path};

use anyhow::Context;
use serde::Deserialize;
use serde_json::Value;

use crate::{
    runtime::{resolve_asset_path, RuntimeBundle},
    AuthoringDocument, AuthoringMedia, AuthoringNote, AuthoringNotetype, NormalizationRequest,
    NormalizationResult,
};

use super::schema::{load_schema, validate_value};

#[derive(Debug, Deserialize)]
struct InputDocument {
    kind: String,
    schema_version: String,
    metadata: InputMetadata,
    #[serde(default)]
    notetypes: Vec<AuthoringNotetype>,
    #[serde(default)]
    notes: Vec<AuthoringNote>,
    #[serde(default)]
    media: Vec<AuthoringMedia>,
}

#[derive(Debug, Deserialize)]
struct InputMetadata {
    document_id: String,
}

pub fn normalize_from_path(
    runtime: &super::discovery::ResolvedRuntime,
    input_path: impl AsRef<Path>,
) -> anyhow::Result<NormalizationResult> {
    let bundle = super::assets::load_bundle_from_manifest(&runtime.manifest_path)?;
    let input_path = input_path.as_ref();
    let raw = fs::read_to_string(input_path)
        .with_context(|| format!("read authoring input: {}", input_path.display()))?;
    let value: Value = serde_json::from_str(&raw)
        .with_context(|| format!("decode authoring input JSON: {}", input_path.display()))?;

    let schema_path = resolve_asset_path(&bundle, "authoring_ir_schema")?;
    let schema = load_schema(&schema_path)?;
    validate_value(&schema, &value).with_context(|| {
        format!(
            "normalize input must satisfy authoring_ir_schema: {}",
            schema_path.display()
        )
    })?;

    let input_document: InputDocument =
        serde_json::from_value(value).context("map runtime input into authoring document")?;

    Ok(crate::normalize(NormalizationRequest::new(AuthoringDocument {
        kind: input_document.kind,
        schema_version: input_document.schema_version,
        metadata_document_id: input_document.metadata.document_id,
        notetypes: input_document.notetypes,
        notes: input_document.notes,
        media: input_document.media,
    })))
}
```

```rust
// anki_forge/src/runtime/build.rs
use std::{fs, path::Path, path::PathBuf};

use anyhow::Context;
use serde_json::Value;

use crate::{BuildArtifactTarget, NormalizedIr, PackageBuildResult};

use super::{
    assets::{load_build_context, load_bundle_from_manifest, load_writer_policy, resolve_asset_path},
    schema::{load_schema, validate_value},
};

pub fn build_from_path(
    runtime: &super::discovery::ResolvedRuntime,
    input_path: impl AsRef<Path>,
    writer_policy_selector: &str,
    build_context_selector: &str,
    artifacts_dir: impl AsRef<Path>,
) -> anyhow::Result<PackageBuildResult> {
    let bundle = load_bundle_from_manifest(&runtime.manifest_path)?;
    let input_path = input_path.as_ref();
    let raw = fs::read_to_string(input_path)
        .with_context(|| format!("read normalized input: {}", input_path.display()))?;
    let value: Value = serde_json::from_str(&raw)
        .with_context(|| format!("decode normalized input JSON: {}", input_path.display()))?;

    let schema_path = resolve_asset_path(&bundle, "normalized_ir_schema")?;
    let schema = load_schema(&schema_path)?;
    validate_value(&schema, &value).with_context(|| {
        format!(
            "build input must satisfy normalized_ir_schema: {}",
            schema_path.display()
        )
    })?;

    let normalized: NormalizedIr =
        serde_json::from_value(value).context("map runtime input into normalized IR")?;
    let writer_policy = load_writer_policy(&bundle, writer_policy_selector)?;
    let build_context = load_build_context(&bundle, build_context_selector)?;
    let artifact_target = BuildArtifactTarget::new(
        PathBuf::from(artifacts_dir.as_ref()),
        "artifacts".to_string(),
    );

    crate::build(&normalized, &writer_policy, &build_context, &artifact_target)
}
```

```rust
// anki_forge/src/runtime/inspect.rs
use std::path::Path;

use crate::InspectReport;

pub fn inspect_staging_path(path: impl AsRef<Path>) -> anyhow::Result<InspectReport> {
    crate::inspect_staging(path)
}

pub fn inspect_apkg_path(path: impl AsRef<Path>) -> anyhow::Result<InspectReport> {
    crate::inspect_apkg(path)
}
```

```rust
// anki_forge/src/runtime/diff.rs
use std::{fs, path::Path};

use anyhow::Context;

use crate::{DiffReport, InspectReport};

pub fn diff_from_paths(left: impl AsRef<Path>, right: impl AsRef<Path>) -> anyhow::Result<DiffReport> {
    let left_path = left.as_ref();
    let right_path = right.as_ref();

    let left_report: InspectReport = serde_json::from_str(
        &fs::read_to_string(left_path)
            .with_context(|| format!("read left inspect report: {}", left_path.display()))?,
    )
    .context("decode left inspect report JSON")?;
    let right_report: InspectReport = serde_json::from_str(
        &fs::read_to_string(right_path)
            .with_context(|| format!("read right inspect report: {}", right_path.display()))?,
    )
    .context("decode right inspect report JSON")?;

    crate::diff_reports(&left_report, &right_report)
}
```

```rust
// anki_forge/src/runtime/mod.rs
pub mod assets;
pub mod build;
pub mod diff;
pub mod discovery;
pub mod inspect;
pub mod normalize;
pub mod schema;

pub use assets::{
    load_build_context, load_bundle_from_manifest, load_writer_policy, resolve_asset_path,
    RuntimeBundle,
};
pub use build::build_from_path;
pub use diff::diff_from_paths;
pub use discovery::{discover_workspace_runtime, ResolvedRuntime, RuntimeMode};
pub use inspect::{inspect_apkg_path, inspect_staging_path};
pub use normalize::normalize_from_path;
```

```rust
// contract_tools/Cargo.toml
[package]
name = "contract_tools"
version = "0.1.0"
edition.workspace = true
rust-version.workspace = true

[dependencies]
anki_forge = { path = "../anki_forge" }
anyhow = "1"
clap = { version = "=4.5.20", features = ["derive"] }
flate2 = "=1.0.35"
html-escape = "0.2"
jsonschema = { version = "0.18.3", default-features = false }
authoring_core = { path = "../authoring_core" }
writer_core = { path = "../writer_core" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
prost = "0.13"
rusqlite = { version = "0.32", features = ["bundled"] }
sha1 = "0.10"
tar = "=0.4.42"
url = "2.5.2"
zip = { version = "2.2.0", default-features = false, features = ["deflate"] }
zstd = "0.13"
```

```rust
// contract_tools/src/normalize_cmd.rs
use anyhow::bail;

pub fn run(manifest: &str, input: &str, output: &str) -> anyhow::Result<String> {
    let runtime = anki_forge::runtime::load_bundle_from_manifest(manifest)?.runtime;
    let result = anki_forge::runtime::normalize_from_path(&runtime, input)?;

    match output {
        "contract-json" => authoring_core::to_canonical_json(&result),
        "human" => Ok(format!("status: {}", result.result_status)),
        other => bail!("unsupported normalize output mode: {other}"),
    }
}
```

```rust
// contract_tools/src/build_cmd.rs
use anyhow::bail;

pub fn run(
    manifest: &str,
    input: &str,
    writer_policy: &str,
    build_context: &str,
    artifacts_dir: &str,
    output: &str,
) -> anyhow::Result<String> {
    let runtime = anki_forge::runtime::load_bundle_from_manifest(manifest)?.runtime;
    let result = anki_forge::runtime::build_from_path(
        &runtime,
        input,
        writer_policy,
        build_context,
        artifacts_dir,
    )?;

    match output {
        "contract-json" => writer_core::to_canonical_json(&result),
        "human" => Ok(format!("status: {}", result.result_status)),
        other => bail!("unsupported build output mode: {other}"),
    }
}
```

```rust
// contract_tools/src/inspect_cmd.rs
use anyhow::bail;

pub fn run(staging: Option<&str>, apkg: Option<&str>, output: &str) -> anyhow::Result<String> {
    let report = match (staging, apkg) {
        (Some(path), None) => anki_forge::runtime::inspect_staging_path(path)?,
        (None, Some(path)) => anki_forge::runtime::inspect_apkg_path(path)?,
        _ => bail!("inspect requires exactly one of --staging or --apkg"),
    };

    match output {
        "contract-json" => writer_core::to_canonical_json(&report),
        "human" => Ok(format!("status: {}", report.observation_status)),
        other => bail!("unsupported inspect output mode: {other}"),
    }
}
```

```rust
// contract_tools/src/diff_cmd.rs
use anyhow::bail;

pub fn run(left: &str, right: &str, output: &str) -> anyhow::Result<String> {
    let diff = anki_forge::runtime::diff_from_paths(left, right)?;

    match output {
        "contract-json" => writer_core::to_canonical_json(&diff),
        "human" => Ok(format!("status: {}", diff.comparison_status)),
        other => bail!("unsupported diff output mode: {other}"),
    }
}
```

```rust
// anki_forge/examples/minimal_flow.rs
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .to_path_buf();
    let runtime = anki_forge::runtime::discover_workspace_runtime(&workspace_root)?;

    println!(
        "resolved runtime => mode={:?} manifest={} bundle_root={}",
        runtime.mode,
        runtime.manifest_path.display(),
        runtime.bundle_root.display()
    );

    let normalized = anki_forge::runtime::normalize_from_path(
        &runtime,
        workspace_root.join("contracts/fixtures/valid/minimal-authoring-ir.json"),
    )?;
    println!("normalize status => {}", normalized.result_status);

    let artifacts_dir = workspace_root.join("tmp/phase4-rust-example/basic");
    let build_result = anki_forge::runtime::build_from_path(
        &runtime,
        workspace_root.join("contracts/fixtures/phase3/inputs/basic-normalized-ir.json"),
        "default",
        "default",
        &artifacts_dir,
    )?;
    println!("build status => {}", build_result.result_status);

    let staging_report =
        anki_forge::runtime::inspect_staging_path(artifacts_dir.join("staging/manifest.json"))?;
    let apkg_report =
        anki_forge::runtime::inspect_apkg_path(artifacts_dir.join("package.apkg"))?;

    println!("inspect staging => {}", staging_report.observation_status);
    println!("inspect apkg => {}", apkg_report.observation_status);

    Ok(())
}
```

```rust
// anki_forge/examples/conformance_surface.rs
use std::{fs, path::PathBuf};

use anyhow::{bail, Context};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct RequestEnvelope {
    command: String,
    request: serde_json::Value,
    #[serde(rename = "runtimeOptions")]
    runtime_options: RuntimeOptions,
}

#[derive(Debug, Deserialize)]
struct RuntimeOptions {
    cwd: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let request_path = std::env::args().nth(1).context("expected request json path")?;
    let envelope: RequestEnvelope = serde_json::from_str(&fs::read_to_string(&request_path)?)?;
    let runtime = anki_forge::runtime::discover_workspace_runtime(&envelope.runtime_options.cwd)?;

    let payload = match envelope.command.as_str() {
        "normalize" => serde_json::to_value(anki_forge::runtime::normalize_from_path(
            &runtime,
            envelope.request["inputPath"].as_str().context("normalize inputPath")?,
        )?)?,
        "build" => serde_json::to_value(anki_forge::runtime::build_from_path(
            &runtime,
            envelope.request["inputPath"].as_str().context("build inputPath")?,
            "default",
            "default",
            envelope.request["artifactsDir"].as_str().context("build artifactsDir")?,
        )?)?,
        "inspect" => {
            if let Some(staging_path) = envelope.request.get("stagingPath").and_then(|value| value.as_str()) {
                serde_json::to_value(anki_forge::runtime::inspect_staging_path(staging_path)?)?
            } else {
                serde_json::to_value(anki_forge::runtime::inspect_apkg_path(
                    envelope.request["apkgPath"].as_str().context("inspect apkgPath")?,
                )?)?
            }
        }
        "diff" => serde_json::to_value(anki_forge::runtime::diff_from_paths(
            envelope.request["leftPath"].as_str().context("diff leftPath")?,
            envelope.request["rightPath"].as_str().context("diff rightPath")?,
        )?)?,
        other => bail!("unsupported conformance command: {other}"),
    };

    println!("{}", serde_json::to_string(&payload)?);
    Ok(())
}
```

- [ ] **Step 4: Run the Rust and CLI tests to verify they pass**

Run: `cargo test -p anki_forge --test runtime_facade_tests -v`
Expected: PASS with the runtime file-oriented facade tests.

Run: `cargo test -p contract_tools --test cli_tests -v build_command_matches_anki_forge_runtime_output`
Expected: PASS with CLI JSON exactly matching runtime-facade JSON.

Run: `cargo run -p anki_forge --example minimal_flow`
Expected: PASS with printed runtime metadata and `normalize/build/inspect` statuses.

- [ ] **Step 5: Commit**

```bash
git add anki_forge/Cargo.toml anki_forge/src/runtime/mod.rs anki_forge/src/runtime/schema.rs anki_forge/src/runtime/normalize.rs anki_forge/src/runtime/build.rs anki_forge/src/runtime/inspect.rs anki_forge/src/runtime/diff.rs anki_forge/tests/runtime_facade_tests.rs anki_forge/examples/minimal_flow.rs anki_forge/examples/conformance_surface.rs contract_tools/Cargo.toml contract_tools/src/normalize_cmd.rs contract_tools/src/build_cmd.rs contract_tools/src/inspect_cmd.rs contract_tools/src/diff_cmd.rs contract_tools/tests/cli_tests.rs
git commit -m "feat: add runtime facade and align cli commands"
```

### Task 4: Add the Node runtime locator and raw command layer

**Files:**
- Create: `bindings/node/package.json`
- Create: `bindings/node/src/version.js`
- Create: `bindings/node/src/errors.js`
- Create: `bindings/node/src/runtime.js`
- Create: `bindings/node/src/raw.js`
- Create: `bindings/node/src/index.js`
- Create: `bindings/node/test/raw.test.js`

- [ ] **Step 1: Write the failing Node raw/runtime tests**

```js
// bindings/node/test/raw.test.js
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { resolveRuntime, runRaw, RuntimeInvocationError, WRAPPER_API_VERSION } from '../src/index.js';

const bindingsNodeRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const repoRoot = path.resolve(bindingsNodeRoot, '../..');
const validAuthoringInput = path.join(repoRoot, 'contracts/fixtures/valid/minimal-authoring-ir.json');

test('resolveRuntime discovers workspace metadata and keeps wrapper version separate', () => {
  const runtime = resolveRuntime({ cwd: bindingsNodeRoot });

  assert.equal(runtime.mode, 'workspace');
  assert.match(runtime.manifestPath, /contracts\/manifest\.yaml$/);
  assert.match(runtime.bundleRoot, /contracts$/);
  assert.equal(runtime.bundleVersion, '0.1.0');
  assert.equal(typeof WRAPPER_API_VERSION, 'string');
});

test('runRaw normalize preserves stdout stderr exit status and argv', async () => {
  const result = await runRaw('normalize', { inputPath: validAuthoringInput }, { cwd: bindingsNodeRoot });

  assert.equal(result.command, 'normalize');
  assert.equal(result.exitStatus, 0);
  assert.equal(typeof result.stdout, 'string');
  assert.equal(typeof result.stderr, 'string');
  assert.equal(Array.isArray(result.argv), true);
  assert.equal(result.resolvedRuntime.mode, 'workspace');
});

test('runRaw raises RuntimeInvocationError when launcher executable is missing', async () => {
  await assert.rejects(
    () =>
      runRaw(
        'normalize',
        { inputPath: validAuthoringInput },
        { cwd: bindingsNodeRoot, launcherExecutable: '/definitely-missing-anki-forge-binary' },
      ),
    (error) =>
      error instanceof RuntimeInvocationError &&
      error.command === 'normalize' &&
      error.resolvedRuntime.mode === 'workspace',
  );
});

test('runRaw wraps runtime discovery failures as RuntimeInvocationError', async () => {
  const detachedDir = fs.mkdtempSync(path.join(os.tmpdir(), 'anki-forge-node-detached-'));

  await assert.rejects(
    () => runRaw('normalize', { inputPath: validAuthoringInput }, { cwd: detachedDir }),
    (error) =>
      error instanceof RuntimeInvocationError &&
      error.command === 'normalize' &&
      error.failurePhase === 'runtime-resolution' &&
      error.resolvedRuntime === null,
  );
});
```

- [ ] **Step 2: Run the Node raw/runtime tests to verify they fail**

Run: `node --test bindings/node/test/raw.test.js`
Expected: FAIL with missing module/package files under `bindings/node/src`.

- [ ] **Step 3: Add the Node package metadata, runtime locator, raw launcher, and runtime errors**

```json
// bindings/node/package.json
{
  "name": "anki-forge-node",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "exports": {
    ".": "./src/index.js"
  },
  "scripts": {
    "test": "node --test test/*.test.js",
    "example:minimal": "node examples/minimal-flow.mjs"
  }
}
```

```js
// bindings/node/src/version.js
export const WRAPPER_API_VERSION = '0.1.0';
```

```js
// bindings/node/src/errors.js
export class RuntimeInvocationError extends Error {
  constructor(message, details) {
    super(message);
    this.name = 'RuntimeInvocationError';
    this.command = details.command;
    this.exitStatus = details.exitStatus ?? null;
    this.stdout = details.stdout ?? '';
    this.stderr = details.stderr ?? '';
    this.resolvedRuntime = details.resolvedRuntime;
    this.failurePhase = details.failurePhase ?? null;
    this.parsePhase = details.parsePhase ?? null;
  }
}

export class ProtocolParseError extends Error {
  constructor(message, details) {
    super(message);
    this.name = 'ProtocolParseError';
    this.command = details.command;
    this.exitStatus = details.exitStatus ?? null;
    this.stdout = details.stdout ?? '';
    this.stderr = details.stderr ?? '';
    this.resolvedRuntime = details.resolvedRuntime;
    this.parsePhase = details.parsePhase ?? 'json';
  }
}
```

```js
// bindings/node/src/runtime.js
import fs from 'node:fs';
import path from 'node:path';

function readBundleVersion(manifestPath) {
  const raw = fs.readFileSync(manifestPath, 'utf8');
  const match = raw.match(/^bundle_version:\s*"([^"]+)"/m);
  return match ? match[1] : 'unknown';
}

export function resolveRuntime(options = {}) {
  if (options.mode === 'installed') {
    return {
      mode: 'installed',
      manifestPath: path.resolve(options.manifestPath),
      bundleRoot: path.resolve(options.bundleRoot),
      bundleVersion: readBundleVersion(path.resolve(options.manifestPath)),
      launcherExecutable: options.launcherExecutable,
      launcherPrefix: [...(options.launcherPrefix ?? [])],
    };
  }

  let current = path.resolve(options.cwd ?? process.cwd());
  while (true) {
    const manifestPath = path.join(current, 'contracts', 'manifest.yaml');
    if (fs.existsSync(manifestPath)) {
      return {
        mode: 'workspace',
        manifestPath,
        bundleRoot: path.dirname(manifestPath),
        bundleVersion: readBundleVersion(manifestPath),
        launcherExecutable: options.launcherExecutable ?? process.env.ANKI_FORGE_CONTRACT_TOOLS ?? 'cargo',
        launcherPrefix: [...(options.launcherPrefix ?? ['run', '-q', '-p', 'contract_tools', '--'])],
      };
    }

    const parent = path.dirname(current);
    if (parent === current) {
      throw new Error('failed to discover contracts/manifest.yaml from workspace path');
    }
    current = parent;
  }
}
```

```js
// bindings/node/src/raw.js
import { spawn } from 'node:child_process';
import path from 'node:path';

import { RuntimeInvocationError } from './errors.js';
import { resolveRuntime } from './runtime.js';

function buildArgs(command, request, runtime) {
  switch (command) {
    case 'normalize':
      return [
        ...runtime.launcherPrefix,
        'normalize',
        '--manifest',
        runtime.manifestPath,
        '--input',
        request.inputPath,
        '--output',
        request.output ?? 'contract-json',
      ];
    case 'build':
      return [
        ...runtime.launcherPrefix,
        'build',
        '--manifest',
        runtime.manifestPath,
        '--input',
        request.inputPath,
        '--writer-policy',
        request.writerPolicy ?? 'default',
        '--build-context',
        request.buildContext ?? 'default',
        '--artifacts-dir',
        request.artifactsDir,
        '--output',
        request.output ?? 'contract-json',
      ];
    case 'inspect':
      return request.stagingPath
        ? [...runtime.launcherPrefix, 'inspect', '--staging', request.stagingPath, '--output', request.output ?? 'contract-json']
        : [...runtime.launcherPrefix, 'inspect', '--apkg', request.apkgPath, '--output', request.output ?? 'contract-json'];
    case 'diff':
      return [
        ...runtime.launcherPrefix,
        'diff',
        '--left',
        request.leftPath,
        '--right',
        request.rightPath,
        '--output',
        request.output ?? 'contract-json',
      ];
    default:
      throw new Error(`unsupported command: ${command}`);
  }
}

export async function runRaw(command, request, runtimeOptions = {}) {
  let resolvedRuntime;
  try {
    resolvedRuntime = resolveRuntime(runtimeOptions);
  } catch (error) {
    throw new RuntimeInvocationError(error.message, {
      command,
      exitStatus: null,
      stdout: '',
      stderr: '',
      resolvedRuntime: null,
      failurePhase: 'runtime-resolution',
    });
  }

  const argv = buildArgs(command, request, resolvedRuntime);

  return await new Promise((resolve, reject) => {
    const child = spawn(resolvedRuntime.launcherExecutable, argv, {
      cwd: resolvedRuntime.mode === 'workspace' ? path.dirname(path.dirname(resolvedRuntime.manifestPath)) : undefined,
      stdio: ['ignore', 'pipe', 'pipe'],
    });

    let stdout = '';
    let stderr = '';
    child.stdout.on('data', (chunk) => {
      stdout += chunk.toString();
    });
    child.stderr.on('data', (chunk) => {
      stderr += chunk.toString();
    });
    child.on('error', (error) => {
      reject(
        new RuntimeInvocationError(error.message, {
          command,
          exitStatus: null,
          stdout,
          stderr,
          resolvedRuntime,
          failurePhase: 'spawn',
        }),
      );
    });
    child.on('close', (code) => {
      resolve({
        command,
        argv: [resolvedRuntime.launcherExecutable, ...argv],
        exitStatus: code ?? -1,
        stdout,
        stderr,
        resolvedRuntime,
      });
    });
  });
}
```

```js
// bindings/node/src/index.js
export { WRAPPER_API_VERSION } from './version.js';
export { ProtocolParseError, RuntimeInvocationError } from './errors.js';
export { resolveRuntime } from './runtime.js';
export { runRaw } from './raw.js';
```

- [ ] **Step 4: Run the Node raw/runtime tests to verify they pass**

Run: `node --test bindings/node/test/raw.test.js`
Expected: PASS with workspace discovery, raw normalize, missing-launcher coverage, and wrapped runtime-discovery failure coverage.

- [ ] **Step 5: Commit**

```bash
git add bindings/node/package.json bindings/node/src/version.js bindings/node/src/errors.js bindings/node/src/runtime.js bindings/node/src/raw.js bindings/node/src/index.js bindings/node/test/raw.test.js
git commit -m "feat: add node raw bindings for contract tools"
```

### Task 5: Add the Node structured layer, helper/view layer, and Node example/docs

**Files:**
- Modify: `bindings/node/src/index.js`
- Create: `bindings/node/src/contracts.js`
- Create: `bindings/node/src/helpers.js`
- Create: `bindings/node/src/structured.js`
- Create: `bindings/node/README.md`
- Create: `bindings/node/examples/minimal-flow.mjs`
- Create: `bindings/node/test/structured.test.js`

- [ ] **Step 1: Write the failing Node structured-layer tests**

```js
// bindings/node/test/structured.test.js
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { build, diff, inspect, normalize, ProtocolParseError } from '../src/index.js';

const bindingsNodeRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const repoRoot = path.resolve(bindingsNodeRoot, '../..');
const validAuthoringInput = path.join(repoRoot, 'contracts/fixtures/valid/minimal-authoring-ir.json');
const invalidAuthoringInput = path.join(repoRoot, 'contracts/fixtures/invalid/missing-document-id.json');
const validNormalizedInput = path.join(repoRoot, 'contracts/fixtures/phase3/inputs/basic-normalized-ir.json');

function fakeLauncherScript(source) {
  const fakeDir = fs.mkdtempSync(path.join(os.tmpdir(), 'anki-forge-node-fake-'));
  const fakeScript = path.join(fakeDir, 'fake.js');
  fs.writeFileSync(fakeScript, source);
  return fakeScript;
}

test('structured normalize returns invalid result without throwing on contract-invalid input', async () => {
  const result = await normalize({ inputPath: invalidAuthoringInput }, { cwd: bindingsNodeRoot });

  assert.equal(result.kind, 'normalization-result');
  assert.equal(result.result_status, 'invalid');
  assert.equal(result.helper.isInvalid, true);
  assert.equal(result.helper.warningCount >= 0, true);
});

test('structured build derives helper artifact paths from returned refs', async () => {
  const fakeScript = fakeLauncherScript(`
    process.stdout.write(JSON.stringify({
      kind: 'package-build-result',
      result_status: 'success',
      tool_contract_version: 'phase3-v1',
      writer_policy_ref: 'writer-policy.default@1.0.0',
      build_context_ref: 'build-context.default@1.0.0',
      staging_ref: 'artifacts/alt/staging/manifest.json',
      artifact_fingerprint: 'artifact:demo',
      apkg_ref: 'artifacts/alt/package.apkg',
      diagnostics: { kind: 'build-diagnostics', items: [] }
    }));
  `);
  const artifactsDir = fs.mkdtempSync(path.join(os.tmpdir(), 'anki-forge-node-build-'));
  const result = await build(
    { inputPath: validNormalizedInput, artifactsDir },
    {
      mode: 'installed',
      manifestPath: path.join(repoRoot, 'contracts/manifest.yaml'),
      bundleRoot: path.join(repoRoot, 'contracts'),
      launcherExecutable: process.execPath,
      launcherPrefix: [fakeScript],
    },
  );

  assert.equal(result.kind, 'package-build-result');
  assert.equal(result.result_status, 'success');
  assert.equal(typeof result.resolvedRuntime.bundleVersion, 'string');
  assert.match(result.helper.artifactPaths.stagingManifest, /alt\/staging\/manifest\.json$/);
  assert.match(result.helper.artifactPaths.apkg, /alt\/package\.apkg$/);
});

test('structured normalize raises ProtocolParseError for invalid json stdout', async () => {
  const fakeScript = fakeLauncherScript("process.stdout.write('{broken');");

  await assert.rejects(
    () =>
      normalize(
        { inputPath: validAuthoringInput },
        {
          mode: 'installed',
          manifestPath: path.join(repoRoot, 'contracts/manifest.yaml'),
          bundleRoot: path.join(repoRoot, 'contracts'),
          launcherExecutable: process.execPath,
          launcherPrefix: [fakeScript],
        },
      ),
    (error) => error instanceof ProtocolParseError && error.parsePhase === 'json',
  );
});

test('structured normalize raises ProtocolParseError for contract-shape mismatch', async () => {
  const fakeScript = fakeLauncherScript("process.stdout.write(JSON.stringify({ kind: 'normalization-result' }));");

  await assert.rejects(
    () =>
      normalize(
        { inputPath: validAuthoringInput },
        {
          mode: 'installed',
          manifestPath: path.join(repoRoot, 'contracts/manifest.yaml'),
          bundleRoot: path.join(repoRoot, 'contracts'),
          launcherExecutable: process.execPath,
          launcherPrefix: [fakeScript],
        },
      ),
    (error) => error instanceof ProtocolParseError && error.parsePhase === 'contract-shape',
  );
});

test('structured build raises ProtocolParseError for contract-version mismatch', async () => {
  const fakeScript = fakeLauncherScript(`
    process.stdout.write(JSON.stringify({
      kind: 'package-build-result',
      result_status: 'success',
      tool_contract_version: 'phase3-v999',
      writer_policy_ref: 'writer-policy.default@1.0.0',
      build_context_ref: 'build-context.default@1.0.0',
      staging_ref: 'artifacts/staging/manifest.json',
      artifact_fingerprint: 'artifact:demo',
      diagnostics: { kind: 'build-diagnostics', items: [] }
    }));
  `);

  await assert.rejects(
    () =>
      build(
        { inputPath: validNormalizedInput, artifactsDir: fs.mkdtempSync(path.join(os.tmpdir(), 'anki-forge-node-version-')) },
        {
          mode: 'installed',
          manifestPath: path.join(repoRoot, 'contracts/manifest.yaml'),
          bundleRoot: path.join(repoRoot, 'contracts'),
          launcherExecutable: process.execPath,
          launcherPrefix: [fakeScript],
        },
      ),
    (error) => error instanceof ProtocolParseError && error.parsePhase === 'contract-version',
  );
});

test('structured inspect returns degraded result without throwing', async () => {
  const fakeScript = fakeLauncherScript(`
    process.stdout.write(JSON.stringify({
      kind: 'inspect-report',
      observation_model_version: 'phase3-inspect-v1',
      source_kind: 'apkg',
      source_ref: 'artifacts/package-no-media.apkg',
      artifact_fingerprint: 'artifact:demo',
      observation_status: 'degraded',
      missing_domains: ['media'],
      degradation_reasons: ['media map unavailable'],
      observations: { notetypes: [], templates: [], fields: [], media: [], metadata: [], references: [] }
    }));
  `);

  const result = await inspect(
    { apkgPath: path.join(repoRoot, 'tmp/fake.apkg') },
    {
      mode: 'installed',
      manifestPath: path.join(repoRoot, 'contracts/manifest.yaml'),
      bundleRoot: path.join(repoRoot, 'contracts'),
      launcherExecutable: process.execPath,
      launcherPrefix: [fakeScript],
    },
  );

  assert.equal(result.observation_status, 'degraded');
  assert.equal(result.helper.isDegraded, true);
});

test('structured diff returns partial result without throwing', async () => {
  const fakeScript = fakeLauncherScript(`
    process.stdout.write(JSON.stringify({
      kind: 'diff-report',
      comparison_status: 'partial',
      left_fingerprint: 'artifact:left',
      right_fingerprint: 'artifact:right',
      left_observation_model_version: 'phase3-inspect-v1',
      right_observation_model_version: 'phase3-inspect-v1',
      summary: 'reference coverage reduced',
      uncompared_domains: ['references'],
      comparison_limitations: ['right report is degraded'],
      changes: []
    }));
  `);

  const result = await diff(
    {
      leftPath: path.join(repoRoot, 'tmp/left.inspect.json'),
      rightPath: path.join(repoRoot, 'tmp/right.inspect.json'),
    },
    {
      mode: 'installed',
      manifestPath: path.join(repoRoot, 'contracts/manifest.yaml'),
      bundleRoot: path.join(repoRoot, 'contracts'),
      launcherExecutable: process.execPath,
      launcherPrefix: [fakeScript],
    },
  );

  assert.equal(result.comparison_status, 'partial');
  assert.equal(result.helper.isPartial, true);
});
```

- [ ] **Step 2: Run the Node structured-layer tests to verify they fail**

Run: `node --test bindings/node/test/structured.test.js`
Expected: FAIL with missing exports for `normalize`, `build`, helper data, or `ProtocolParseError` handling.

- [ ] **Step 3: Implement Node structured operations, helper projections, and docs/example**

```js
// bindings/node/src/contracts.js
function fail(parsePhase, message) {
  const error = new Error(message);
  error.parsePhase = parsePhase;
  throw error;
}

const CONTRACT_RULES = {
  normalize: {
    kind: 'normalization-result',
    required: ['kind', 'result_status', 'tool_contract_version', 'diagnostics'],
    versionFields: [['tool_contract_version', 'phase2-v1']],
  },
  build: {
    kind: 'package-build-result',
    required: ['kind', 'result_status', 'tool_contract_version', 'writer_policy_ref', 'build_context_ref', 'diagnostics'],
    versionFields: [['tool_contract_version', 'phase3-v1']],
  },
  inspect: {
    kind: 'inspect-report',
    required: [
      'kind',
      'observation_model_version',
      'source_kind',
      'source_ref',
      'artifact_fingerprint',
      'observation_status',
      'missing_domains',
      'degradation_reasons',
      'observations',
    ],
    versionFields: [['observation_model_version', 'phase3-inspect-v1']],
  },
  diff: {
    kind: 'diff-report',
    required: [
      'kind',
      'comparison_status',
      'left_fingerprint',
      'right_fingerprint',
      'left_observation_model_version',
      'right_observation_model_version',
      'summary',
      'uncompared_domains',
      'comparison_limitations',
      'changes',
    ],
    versionFields: [
      ['left_observation_model_version', 'phase3-inspect-v1'],
      ['right_observation_model_version', 'phase3-inspect-v1'],
    ],
  },
};

export function validateContractPayload(command, payload) {
  const rules = CONTRACT_RULES[command];
  if (!payload || typeof payload !== 'object' || Array.isArray(payload)) {
    fail('contract-shape', `${command} contract payload must be an object`);
  }
  if (payload.kind !== rules.kind) {
    fail('contract-shape', `${command} contract kind must be ${rules.kind}`);
  }
  for (const field of rules.required) {
    if (!(field in payload)) {
      fail('contract-shape', `${command} contract payload missing required field ${field}`);
    }
  }
  for (const [field, expected] of rules.versionFields) {
    if (payload[field] !== expected) {
      fail('contract-version', `${command} contract field ${field} must be ${expected}`);
    }
  }
}
```

```js
// bindings/node/src/helpers.js
import path from 'node:path';

export function warningCount(result) {
  const diagnostics = result.diagnostics?.items ?? [];
  return diagnostics.filter((item) => item.level === 'warning').length;
}

function artifactPathFromRef(artifactsDir, ref) {
  if (!artifactsDir || !ref) {
    return null;
  }
  const normalizedRef = ref.replace(/^artifacts\//, '');
  return path.join(artifactsDir, ...normalizedRef.split('/'));
}

export function helperView(command, result, request) {
  return {
    isInvalid: result.result_status === 'invalid',
    isDegraded: result.observation_status === 'degraded',
    isPartial: result.comparison_status === 'partial',
    warningCount: warningCount(result),
    artifactPaths:
      command === 'build'
        ? {
            stagingManifest: artifactPathFromRef(request.artifactsDir, result.staging_ref ?? null),
            apkg: artifactPathFromRef(request.artifactsDir, result.apkg_ref ?? null),
          }
        : null,
  };
}
```

```js
// bindings/node/src/structured.js
import { ProtocolParseError, RuntimeInvocationError } from './errors.js';
import { validateContractPayload } from './contracts.js';
import { helperView } from './helpers.js';
import { runRaw } from './raw.js';

async function runStructured(command, request, runtimeOptions) {
  const raw = await runRaw(command, request, runtimeOptions);

  if (raw.exitStatus !== 0) {
    throw new RuntimeInvocationError(`${command} exited with status ${raw.exitStatus}`, {
      command,
      exitStatus: raw.exitStatus,
      stdout: raw.stdout,
      stderr: raw.stderr,
      resolvedRuntime: raw.resolvedRuntime,
      failurePhase: 'process-exit',
    });
  }

  let parsed;
  try {
    parsed = JSON.parse(raw.stdout);
  } catch (error) {
    throw new ProtocolParseError(error.message, {
      command,
      exitStatus: raw.exitStatus,
      stdout: raw.stdout,
      stderr: raw.stderr,
      resolvedRuntime: raw.resolvedRuntime,
      parsePhase: 'json',
    });
  }

  try {
    validateContractPayload(command, parsed);
  } catch (error) {
    throw new ProtocolParseError(error.message, {
      command,
      exitStatus: raw.exitStatus,
      stdout: raw.stdout,
      stderr: raw.stderr,
      resolvedRuntime: raw.resolvedRuntime,
      parsePhase: error.parsePhase ?? 'contract-shape',
    });
  }

  return {
    ...parsed,
    resolvedRuntime: raw.resolvedRuntime,
    rawCommand: {
      command: raw.command,
      argv: raw.argv,
      exitStatus: raw.exitStatus,
    },
    helper: helperView(command, parsed, request),
  };
}

export function normalize(request, runtimeOptions = {}) {
  return runStructured('normalize', request, runtimeOptions);
}

export function build(request, runtimeOptions = {}) {
  return runStructured('build', request, runtimeOptions);
}

export function inspect(request, runtimeOptions = {}) {
  return runStructured('inspect', request, runtimeOptions);
}

export function diff(request, runtimeOptions = {}) {
  return runStructured('diff', request, runtimeOptions);
}
```

```js
// bindings/node/src/index.js
export { WRAPPER_API_VERSION } from './version.js';
export { ProtocolParseError, RuntimeInvocationError } from './errors.js';
export { validateContractPayload } from './contracts.js';
export { resolveRuntime } from './runtime.js';
export { runRaw } from './raw.js';
export { build, diff, inspect, normalize } from './structured.js';
```

```md
<!-- bindings/node/README.md -->
# anki-forge Node Bindings

This wrapper exposes three layers:

1. `runRaw()` for argv/stdout/stderr/exit-status preservation
2. `normalize()/build()/inspect()/diff()` for structured `contract-json` plus command-specific shape/version validation
3. helper projections under `result.helper`

The default path is workspace-mode discovery from the current working directory.

Example:

~~~bash
npm --prefix bindings/node run example:minimal
~~~
```

```js
// bindings/node/examples/minimal-flow.mjs
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { build, diff, inspect, normalize, resolveRuntime } from '../src/index.js';

const bindingsNodeRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const repoRoot = path.resolve(bindingsNodeRoot, '../..');
const runtime = resolveRuntime({ cwd: bindingsNodeRoot });
console.log('resolved runtime =>', runtime);

const normalized = await normalize(
  { inputPath: path.join(repoRoot, 'contracts/fixtures/valid/minimal-authoring-ir.json') },
  { cwd: bindingsNodeRoot },
);
console.log('normalize status =>', normalized.result_status);

const artifactsDir = path.join(repoRoot, 'tmp/phase4-node-example/basic');
const buildResult = await build(
  {
    inputPath: path.join(repoRoot, 'contracts/fixtures/phase3/inputs/basic-normalized-ir.json'),
    artifactsDir,
  },
  { cwd: bindingsNodeRoot },
);
console.log('build status =>', buildResult.result_status);

const stagingReport = await inspect({ stagingPath: path.join(artifactsDir, 'staging/manifest.json') }, { cwd: bindingsNodeRoot });
const apkgReport = await inspect({ apkgPath: path.join(artifactsDir, 'package.apkg') }, { cwd: bindingsNodeRoot });
console.log('inspect statuses =>', stagingReport.observation_status, apkgReport.observation_status);

const diffResult = await diff(
  {
    leftPath: path.join(repoRoot, 'contracts/fixtures/phase3/expected/basic.inspect.json'),
    rightPath: path.join(repoRoot, 'contracts/fixtures/phase3/expected/basic.inspect.json'),
  },
  { cwd: bindingsNodeRoot },
);
console.log('diff status =>', diffResult.comparison_status);
```

- [ ] **Step 4: Run the Node structured-layer tests and example to verify they pass**

Run: `node --test bindings/node/test/structured.test.js`
Expected: PASS with invalid-result handling, ref-derived helper view, degraded/partial result coverage, and protocol parse/shape/version failure coverage.

Run: `npm --prefix bindings/node run example:minimal`
Expected: PASS with runtime metadata plus `normalize/build/inspect/diff` statuses.

- [ ] **Step 5: Commit**

```bash
git add bindings/node/src/contracts.js bindings/node/src/helpers.js bindings/node/src/structured.js bindings/node/src/index.js bindings/node/README.md bindings/node/examples/minimal-flow.mjs bindings/node/test/structured.test.js
git commit -m "feat: add node structured bindings and example"
```

### Task 6: Add the Python runtime locator and raw command layer

**Files:**
- Create: `bindings/python/pyproject.toml`
- Create: `bindings/python/src/anki_forge_python/version.py`
- Create: `bindings/python/src/anki_forge_python/errors.py`
- Create: `bindings/python/src/anki_forge_python/runtime.py`
- Create: `bindings/python/src/anki_forge_python/raw.py`
- Create: `bindings/python/src/anki_forge_python/__init__.py`
- Create: `bindings/python/tests/test_raw.py`

- [ ] **Step 1: Write the failing Python raw/runtime tests**

```python
# bindings/python/tests/test_raw.py
import os
import pathlib
import tempfile
import unittest

from anki_forge_python import WRAPPER_API_VERSION, RuntimeInvocationError, resolve_runtime, run_raw


REPO_ROOT = pathlib.Path(__file__).resolve().parents[3]
VALID_AUTHORING_INPUT = REPO_ROOT / "contracts/fixtures/valid/minimal-authoring-ir.json"


class RawBindingsTests(unittest.TestCase):
    def test_resolve_runtime_discovers_workspace_metadata(self) -> None:
        runtime = resolve_runtime(cwd=pathlib.Path(__file__).resolve().parents[1])

        self.assertEqual(runtime.mode, "workspace")
        self.assertTrue(str(runtime.manifest_path).endswith("contracts/manifest.yaml"))
        self.assertTrue(str(runtime.bundle_root).endswith("contracts"))
        self.assertEqual(runtime.bundle_version, "0.1.0")
        self.assertIsInstance(WRAPPER_API_VERSION, str)

    def test_run_raw_normalize_preserves_process_result(self) -> None:
        result = run_raw("normalize", {"input_path": str(VALID_AUTHORING_INPUT)}, cwd=pathlib.Path(__file__).resolve().parents[1])

        self.assertEqual(result.command, "normalize")
        self.assertEqual(result.exit_status, 0)
        self.assertIsInstance(result.stdout, str)
        self.assertIsInstance(result.stderr, str)
        self.assertGreaterEqual(len(result.argv), 1)

    def test_run_raw_raises_runtime_invocation_error_for_missing_launcher(self) -> None:
        with self.assertRaises(RuntimeInvocationError) as context:
            run_raw(
                "normalize",
                {"input_path": str(VALID_AUTHORING_INPUT)},
                cwd=pathlib.Path(__file__).resolve().parents[1],
                launcher_executable="/definitely-missing-anki-forge-python-binary",
            )

        self.assertEqual(context.exception.command, "normalize")
        self.assertEqual(context.exception.resolved_runtime.mode, "workspace")

    def test_run_raw_wraps_runtime_discovery_failure(self) -> None:
        detached_dir = pathlib.Path(tempfile.mkdtemp(prefix="anki-forge-python-detached-"))

        with self.assertRaises(RuntimeInvocationError) as context:
            run_raw("normalize", {"input_path": str(VALID_AUTHORING_INPUT)}, cwd=detached_dir)

        self.assertEqual(context.exception.command, "normalize")
        self.assertEqual(context.exception.failure_phase, "runtime-resolution")
        self.assertIsNone(context.exception.resolved_runtime)


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: Run the Python raw/runtime tests to verify they fail**

Run: `PYTHONPATH="$(pwd)/bindings/python/src" python3 -m unittest discover -s bindings/python/tests -p 'test_raw.py' -v`
Expected: FAIL with missing module/package files under `bindings/python/src/anki_forge_python`.

- [ ] **Step 3: Add the Python package metadata, runtime locator, raw launcher, and runtime errors**

```toml
# bindings/python/pyproject.toml
[project]
name = "anki-forge-python"
version = "0.1.0"
description = "Local Phase 4 Python bindings for anki-forge"
requires-python = ">=3.11"

[build-system]
requires = ["setuptools>=68"]
build-backend = "setuptools.build_meta"
```

```python
# bindings/python/src/anki_forge_python/version.py
WRAPPER_API_VERSION = "0.1.0"
```

```python
# bindings/python/src/anki_forge_python/errors.py
class RuntimeInvocationError(Exception):
    def __init__(self, message, *, command, exit_status=None, stdout="", stderr="", resolved_runtime=None, failure_phase=None, parse_phase=None):
        super().__init__(message)
        self.command = command
        self.exit_status = exit_status
        self.stdout = stdout
        self.stderr = stderr
        self.resolved_runtime = resolved_runtime
        self.failure_phase = failure_phase
        self.parse_phase = parse_phase


class ProtocolParseError(Exception):
    def __init__(self, message, *, command, exit_status=None, stdout="", stderr="", resolved_runtime=None, parse_phase="json"):
        super().__init__(message)
        self.command = command
        self.exit_status = exit_status
        self.stdout = stdout
        self.stderr = stderr
        self.resolved_runtime = resolved_runtime
        self.parse_phase = parse_phase
```

```python
# bindings/python/src/anki_forge_python/runtime.py
from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path


@dataclass(frozen=True)
class ResolvedRuntime:
    mode: str
    manifest_path: Path
    bundle_root: Path
    bundle_version: str
    launcher_executable: str
    launcher_prefix: tuple[str, ...]


def _read_bundle_version(manifest_path: Path) -> str:
    for line in manifest_path.read_text().splitlines():
        if line.startswith('bundle_version:'):
            return line.split(':', 1)[1].strip().strip('"')
    return "unknown"


def resolve_runtime(
    *,
    cwd: Path | None = None,
    mode: str | None = None,
    manifest_path: str | None = None,
    bundle_root: str | None = None,
    launcher_executable: str | None = None,
    launcher_prefix: list[str] | None = None,
) -> ResolvedRuntime:
    if mode == "installed":
        manifest = Path(manifest_path).resolve()
        bundle = Path(bundle_root).resolve()
        return ResolvedRuntime(
            mode="installed",
            manifest_path=manifest,
            bundle_root=bundle,
            bundle_version=_read_bundle_version(manifest),
            launcher_executable=launcher_executable or "contract_tools",
            launcher_prefix=tuple(launcher_prefix or []),
        )

    current = Path(cwd or Path.cwd()).resolve()
    while True:
        manifest = current / "contracts" / "manifest.yaml"
        if manifest.is_file():
            return ResolvedRuntime(
                mode="workspace",
                manifest_path=manifest,
                bundle_root=manifest.parent,
                bundle_version=_read_bundle_version(manifest),
                launcher_executable=launcher_executable or "cargo",
                launcher_prefix=tuple(launcher_prefix or ["run", "-q", "-p", "contract_tools", "--"]),
            )
        if current.parent == current:
            raise RuntimeError("failed to discover contracts/manifest.yaml from workspace path")
        current = current.parent
```

```python
# bindings/python/src/anki_forge_python/raw.py
from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import subprocess

from .errors import RuntimeInvocationError
from .runtime import ResolvedRuntime, resolve_runtime


@dataclass(frozen=True)
class RawCommandResult:
    command: str
    argv: tuple[str, ...]
    exit_status: int
    stdout: str
    stderr: str
    resolved_runtime: ResolvedRuntime


def _build_args(command: str, request: dict, runtime: ResolvedRuntime) -> list[str]:
    if command == "normalize":
        return [
            *runtime.launcher_prefix,
            "normalize",
            "--manifest",
            str(runtime.manifest_path),
            "--input",
            request["input_path"],
            "--output",
            request.get("output", "contract-json"),
        ]
    if command == "build":
        return [
            *runtime.launcher_prefix,
            "build",
            "--manifest",
            str(runtime.manifest_path),
            "--input",
            request["input_path"],
            "--writer-policy",
            request.get("writer_policy", "default"),
            "--build-context",
            request.get("build_context", "default"),
            "--artifacts-dir",
            request["artifacts_dir"],
            "--output",
            request.get("output", "contract-json"),
        ]
    if command == "inspect":
        if "staging_path" in request:
            return [*runtime.launcher_prefix, "inspect", "--staging", request["staging_path"], "--output", request.get("output", "contract-json")]
        return [*runtime.launcher_prefix, "inspect", "--apkg", request["apkg_path"], "--output", request.get("output", "contract-json")]
    if command == "diff":
        return [
            *runtime.launcher_prefix,
            "diff",
            "--left",
            request["left_path"],
            "--right",
            request["right_path"],
            "--output",
            request.get("output", "contract-json"),
        ]
    raise ValueError(f"unsupported command: {command}")


def run_raw(command: str, request: dict, **runtime_kwargs) -> RawCommandResult:
    try:
        runtime = resolve_runtime(**runtime_kwargs)
    except Exception as error:
        raise RuntimeInvocationError(
            str(error),
            command=command,
            stdout="",
            stderr="",
            resolved_runtime=None,
            failure_phase="runtime-resolution",
        ) from error

    argv = [runtime.launcher_executable, *_build_args(command, request, runtime)]
    try:
        completed = subprocess.run(
            argv,
            cwd=str(runtime.manifest_path.parent.parent) if runtime.mode == "workspace" else None,
            check=False,
            capture_output=True,
            text=True,
        )
    except OSError as error:
        raise RuntimeInvocationError(
            str(error),
            command=command,
            stdout="",
            stderr="",
            resolved_runtime=runtime,
            failure_phase="spawn",
        ) from error

    return RawCommandResult(
        command=command,
        argv=tuple(argv),
        exit_status=completed.returncode,
        stdout=completed.stdout,
        stderr=completed.stderr,
        resolved_runtime=runtime,
    )
```

```python
# bindings/python/src/anki_forge_python/__init__.py
from .errors import ProtocolParseError, RuntimeInvocationError
from .raw import RawCommandResult, run_raw
from .runtime import ResolvedRuntime, resolve_runtime
from .version import WRAPPER_API_VERSION

__all__ = [
    "ProtocolParseError",
    "RawCommandResult",
    "ResolvedRuntime",
    "RuntimeInvocationError",
    "WRAPPER_API_VERSION",
    "resolve_runtime",
    "run_raw",
]
```

- [ ] **Step 4: Run the Python raw/runtime tests to verify they pass**

Run: `PYTHONPATH="$(pwd)/bindings/python/src" python3 -m unittest discover -s bindings/python/tests -p 'test_raw.py' -v`
Expected: PASS with workspace discovery, raw normalize, missing-launcher coverage, and wrapped runtime-discovery failure coverage.

- [ ] **Step 5: Commit**

```bash
git add bindings/python/pyproject.toml bindings/python/src/anki_forge_python/version.py bindings/python/src/anki_forge_python/errors.py bindings/python/src/anki_forge_python/runtime.py bindings/python/src/anki_forge_python/raw.py bindings/python/src/anki_forge_python/__init__.py bindings/python/tests/test_raw.py
git commit -m "feat: add python raw bindings for contract tools"
```

### Task 7: Add the Python structured layer, helper/view layer, and Python example/docs

**Files:**
- Modify: `bindings/python/src/anki_forge_python/__init__.py`
- Create: `bindings/python/src/anki_forge_python/contracts.py`
- Create: `bindings/python/src/anki_forge_python/helpers.py`
- Create: `bindings/python/src/anki_forge_python/structured.py`
- Create: `bindings/python/README.md`
- Create: `bindings/python/examples/minimal_flow.py`
- Create: `bindings/python/tests/test_structured.py`

- [ ] **Step 1: Write the failing Python structured-layer tests**

```python
# bindings/python/tests/test_structured.py
import pathlib
import tempfile
import unittest

from anki_forge_python import ProtocolParseError, build, diff, inspect, normalize


REPO_ROOT = pathlib.Path(__file__).resolve().parents[3]
VALID_AUTHORING_INPUT = REPO_ROOT / "contracts/fixtures/valid/minimal-authoring-ir.json"
INVALID_AUTHORING_INPUT = REPO_ROOT / "contracts/fixtures/invalid/missing-document-id.json"
VALID_NORMALIZED_INPUT = REPO_ROOT / "contracts/fixtures/phase3/inputs/basic-normalized-ir.json"


def fake_launcher_script(source: str) -> str:
    fake_dir = pathlib.Path(tempfile.mkdtemp(prefix="anki-forge-python-fake-"))
    fake_script = fake_dir / "fake.py"
    fake_script.write_text(source)
    return str(fake_script)


class StructuredBindingsTests(unittest.TestCase):
    def test_structured_normalize_returns_invalid_result_without_throwing(self) -> None:
        result = normalize({"input_path": str(INVALID_AUTHORING_INPUT)}, cwd=pathlib.Path(__file__).resolve().parents[1])

        self.assertEqual(result["kind"], "normalization-result")
        self.assertEqual(result["result_status"], "invalid")
        self.assertTrue(result["helper"]["isInvalid"])

    def test_structured_build_derives_helper_artifact_paths_from_returned_refs(self) -> None:
        fake_script = fake_launcher_script(
            """import json
print(json.dumps({
  "kind": "package-build-result",
  "result_status": "success",
  "tool_contract_version": "phase3-v1",
  "writer_policy_ref": "writer-policy.default@1.0.0",
  "build_context_ref": "build-context.default@1.0.0",
  "staging_ref": "artifacts/alt/staging/manifest.json",
  "artifact_fingerprint": "artifact:demo",
  "apkg_ref": "artifacts/alt/package.apkg",
  "diagnostics": {"kind": "build-diagnostics", "items": []}
}))"""
        )
        artifacts_dir = tempfile.mkdtemp(prefix="anki-forge-python-build-")
        result = build(
            {"input_path": str(VALID_NORMALIZED_INPUT), "artifacts_dir": artifacts_dir},
            mode="installed",
            manifest_path=str(REPO_ROOT / "contracts/manifest.yaml"),
            bundle_root=str(REPO_ROOT / "contracts"),
            launcher_executable="python3",
            launcher_prefix=[fake_script],
        )

        self.assertEqual(result["kind"], "package-build-result")
        self.assertEqual(result["result_status"], "success")
        self.assertTrue(result["helper"]["artifactPaths"]["stagingManifest"].endswith("alt/staging/manifest.json"))
        self.assertTrue(result["helper"]["artifactPaths"]["apkg"].endswith("alt/package.apkg"))

    def test_structured_normalize_raises_protocol_parse_error_for_invalid_json_stdout(self) -> None:
        fake_script = fake_launcher_script("import sys\nsys.stdout.write('{broken')\n")

        with self.assertRaises(ProtocolParseError) as context:
            normalize(
                {"input_path": str(VALID_AUTHORING_INPUT)},
                mode="installed",
                manifest_path=str(REPO_ROOT / "contracts/manifest.yaml"),
                bundle_root=str(REPO_ROOT / "contracts"),
                launcher_executable="python3",
                launcher_prefix=[str(fake_script)],
            )

        self.assertEqual(context.exception.parse_phase, "json")

    def test_structured_normalize_raises_protocol_parse_error_for_contract_shape_mismatch(self) -> None:
        fake_script = fake_launcher_script("import json\nprint(json.dumps({'kind': 'normalization-result'}))\n")

        with self.assertRaises(ProtocolParseError) as context:
            normalize(
                {"input_path": str(VALID_AUTHORING_INPUT)},
                mode="installed",
                manifest_path=str(REPO_ROOT / "contracts/manifest.yaml"),
                bundle_root=str(REPO_ROOT / "contracts"),
                launcher_executable="python3",
                launcher_prefix=[fake_script],
            )

        self.assertEqual(context.exception.parse_phase, "contract-shape")

    def test_structured_build_raises_protocol_parse_error_for_contract_version_mismatch(self) -> None:
        fake_script = fake_launcher_script(
            """import json
print(json.dumps({
  "kind": "package-build-result",
  "result_status": "success",
  "tool_contract_version": "phase3-v999",
  "writer_policy_ref": "writer-policy.default@1.0.0",
  "build_context_ref": "build-context.default@1.0.0",
  "staging_ref": "artifacts/staging/manifest.json",
  "artifact_fingerprint": "artifact:demo",
  "diagnostics": {"kind": "build-diagnostics", "items": []}
}))"""
        )

        with self.assertRaises(ProtocolParseError) as context:
            build(
                {"input_path": str(VALID_NORMALIZED_INPUT), "artifacts_dir": tempfile.mkdtemp(prefix="anki-forge-python-version-")},
                mode="installed",
                manifest_path=str(REPO_ROOT / "contracts/manifest.yaml"),
                bundle_root=str(REPO_ROOT / "contracts"),
                launcher_executable="python3",
                launcher_prefix=[fake_script],
            )

        self.assertEqual(context.exception.parse_phase, "contract-version")

    def test_structured_inspect_returns_degraded_result_without_throwing(self) -> None:
        fake_script = fake_launcher_script(
            """import json
print(json.dumps({
  "kind": "inspect-report",
  "observation_model_version": "phase3-inspect-v1",
  "source_kind": "apkg",
  "source_ref": "artifacts/package-no-media.apkg",
  "artifact_fingerprint": "artifact:demo",
  "observation_status": "degraded",
  "missing_domains": ["media"],
  "degradation_reasons": ["media map unavailable"],
  "observations": {"notetypes": [], "templates": [], "fields": [], "media": [], "metadata": [], "references": []}
}))"""
        )

        result = inspect(
            {"apkg_path": str(REPO_ROOT / "tmp/fake.apkg")},
            mode="installed",
            manifest_path=str(REPO_ROOT / "contracts/manifest.yaml"),
            bundle_root=str(REPO_ROOT / "contracts"),
            launcher_executable="python3",
            launcher_prefix=[fake_script],
        )

        self.assertEqual(result["observation_status"], "degraded")
        self.assertTrue(result["helper"]["isDegraded"])

    def test_structured_diff_returns_partial_result_without_throwing(self) -> None:
        fake_script = fake_launcher_script(
            """import json
print(json.dumps({
  "kind": "diff-report",
  "comparison_status": "partial",
  "left_fingerprint": "artifact:left",
  "right_fingerprint": "artifact:right",
  "left_observation_model_version": "phase3-inspect-v1",
  "right_observation_model_version": "phase3-inspect-v1",
  "summary": "reference coverage reduced",
  "uncompared_domains": ["references"],
  "comparison_limitations": ["right report is degraded"],
  "changes": []
}))"""
        )

        result = diff(
            {
                "left_path": str(REPO_ROOT / "tmp/left.inspect.json"),
                "right_path": str(REPO_ROOT / "tmp/right.inspect.json"),
            },
            mode="installed",
            manifest_path=str(REPO_ROOT / "contracts/manifest.yaml"),
            bundle_root=str(REPO_ROOT / "contracts"),
            launcher_executable="python3",
            launcher_prefix=[fake_script],
        )

        self.assertEqual(result["comparison_status"], "partial")
        self.assertTrue(result["helper"]["isPartial"])


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: Run the Python structured-layer tests to verify they fail**

Run: `PYTHONPATH="$(pwd)/bindings/python/src" python3 -m unittest discover -s bindings/python/tests -p 'test_structured.py' -v`
Expected: FAIL with missing structured exports such as `normalize` and `build`.

- [ ] **Step 3: Implement Python structured operations, helper projections, and docs/example**

```python
from __future__ import annotations

# bindings/python/src/anki_forge_python/contracts.py
def _fail(parse_phase: str, message: str) -> None:
    error = ValueError(message)
    error.parse_phase = parse_phase
    raise error


CONTRACT_RULES = {
    "normalize": {
        "kind": "normalization-result",
        "required": ["kind", "result_status", "tool_contract_version", "diagnostics"],
        "version_fields": [("tool_contract_version", "phase2-v1")],
    },
    "build": {
        "kind": "package-build-result",
        "required": ["kind", "result_status", "tool_contract_version", "writer_policy_ref", "build_context_ref", "diagnostics"],
        "version_fields": [("tool_contract_version", "phase3-v1")],
    },
    "inspect": {
        "kind": "inspect-report",
        "required": [
            "kind",
            "observation_model_version",
            "source_kind",
            "source_ref",
            "artifact_fingerprint",
            "observation_status",
            "missing_domains",
            "degradation_reasons",
            "observations",
        ],
        "version_fields": [("observation_model_version", "phase3-inspect-v1")],
    },
    "diff": {
        "kind": "diff-report",
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
            "changes",
        ],
        "version_fields": [
            ("left_observation_model_version", "phase3-inspect-v1"),
            ("right_observation_model_version", "phase3-inspect-v1"),
        ],
    },
}


def validate_contract_payload(command: str, payload: dict) -> None:
    rules = CONTRACT_RULES[command]
    if not isinstance(payload, dict):
        _fail("contract-shape", f"{command} contract payload must be an object")
    if payload.get("kind") != rules["kind"]:
        _fail("contract-shape", f"{command} contract kind must be {rules['kind']}")
    for field in rules["required"]:
        if field not in payload:
            _fail("contract-shape", f"{command} contract payload missing required field {field}")
    for field, expected in rules["version_fields"]:
        if payload.get(field) != expected:
            _fail("contract-version", f"{command} contract field {field} must be {expected}")
```

```python
# bindings/python/src/anki_forge_python/helpers.py
from __future__ import annotations

import os


def warning_count(result: dict) -> int:
    diagnostics = result.get("diagnostics", {}).get("items", [])
    return sum(1 for item in diagnostics if item.get("level") == "warning")


def artifact_path_from_ref(artifacts_dir: str | None, ref: str | None) -> str | None:
    if not artifacts_dir or not ref:
        return None
    normalized_ref = ref.removeprefix("artifacts/")
    return os.path.join(artifacts_dir, *normalized_ref.split("/"))


def helper_view(command: str, result: dict, request: dict) -> dict:
    return {
        "isInvalid": result.get("result_status") == "invalid",
        "isDegraded": result.get("observation_status") == "degraded",
        "isPartial": result.get("comparison_status") == "partial",
        "warningCount": warning_count(result),
        "artifactPaths": {
            "stagingManifest": artifact_path_from_ref(request.get("artifacts_dir"), result.get("staging_ref")),
            "apkg": artifact_path_from_ref(request.get("artifacts_dir"), result.get("apkg_ref")),
        }
        if command == "build"
        else None,
    }
```

```python
# bindings/python/src/anki_forge_python/structured.py
from __future__ import annotations

import json

from .contracts import validate_contract_payload
from .errors import ProtocolParseError, RuntimeInvocationError
from .helpers import helper_view
from .raw import run_raw


def _run_structured(command: str, request: dict, **runtime_kwargs) -> dict:
    raw = run_raw(command, request, **runtime_kwargs)

    if raw.exit_status != 0:
        raise RuntimeInvocationError(
            f"{command} exited with status {raw.exit_status}",
            command=command,
            exit_status=raw.exit_status,
            stdout=raw.stdout,
            stderr=raw.stderr,
            resolved_runtime=raw.resolved_runtime,
            failure_phase="process-exit",
        )

    try:
        parsed = json.loads(raw.stdout)
    except json.JSONDecodeError as error:
        raise ProtocolParseError(
            str(error),
            command=command,
            exit_status=raw.exit_status,
            stdout=raw.stdout,
            stderr=raw.stderr,
            resolved_runtime=raw.resolved_runtime,
            parse_phase="json",
        ) from error

    try:
        validate_contract_payload(command, parsed)
    except ValueError as error:
        raise ProtocolParseError(
            str(error),
            command=command,
            exit_status=raw.exit_status,
            stdout=raw.stdout,
            stderr=raw.stderr,
            resolved_runtime=raw.resolved_runtime,
            parse_phase=getattr(error, "parse_phase", "contract-shape"),
        ) from error

    return {
        **parsed,
        "resolvedRuntime": raw.resolved_runtime,
        "rawCommand": {
            "command": raw.command,
            "argv": raw.argv,
            "exitStatus": raw.exit_status,
        },
        "helper": helper_view(command, parsed, request),
    }


def normalize(request: dict, **runtime_kwargs) -> dict:
    return _run_structured("normalize", request, **runtime_kwargs)


def build(request: dict, **runtime_kwargs) -> dict:
    return _run_structured("build", request, **runtime_kwargs)


def inspect(request: dict, **runtime_kwargs) -> dict:
    return _run_structured("inspect", request, **runtime_kwargs)


def diff(request: dict, **runtime_kwargs) -> dict:
    return _run_structured("diff", request, **runtime_kwargs)
```

```python
# bindings/python/src/anki_forge_python/__init__.py
from .contracts import validate_contract_payload
from .errors import ProtocolParseError, RuntimeInvocationError
from .helpers import artifact_path_from_ref, helper_view, warning_count
from .raw import RawCommandResult, run_raw
from .runtime import ResolvedRuntime, resolve_runtime
from .structured import build, diff, inspect, normalize
from .version import WRAPPER_API_VERSION

__all__ = [
    "ProtocolParseError",
    "RawCommandResult",
    "ResolvedRuntime",
    "RuntimeInvocationError",
    "WRAPPER_API_VERSION",
    "artifact_path_from_ref",
    "build",
    "diff",
    "helper_view",
    "inspect",
    "normalize",
    "resolve_runtime",
    "run_raw",
    "validate_contract_payload",
    "warning_count",
]
```

```md
<!-- bindings/python/README.md -->
# anki-forge Python Bindings

This wrapper exposes three layers:

1. `run_raw()` for launcher/argv/stdout/stderr preservation
2. `normalize()/build()/inspect()/diff()` for structured `contract-json` plus command-specific shape/version validation
3. helper projections under `result["helper"]`

The default path is workspace-mode discovery from the current working directory.

Example:

~~~bash
PYTHONPATH="$(pwd)/bindings/python/src" python3 bindings/python/examples/minimal_flow.py
~~~
```

```python
# bindings/python/examples/minimal_flow.py
from __future__ import annotations

from pathlib import Path

from anki_forge_python import build, diff, inspect, normalize, resolve_runtime


repo_root = Path(__file__).resolve().parents[3]
runtime = resolve_runtime(cwd=Path(__file__).resolve().parents[1])
print("resolved runtime =>", runtime)

normalized = normalize(
    {"input_path": str(repo_root / "contracts/fixtures/valid/minimal-authoring-ir.json")},
    cwd=Path(__file__).resolve().parents[1],
)
print("normalize status =>", normalized["result_status"])

artifacts_dir = repo_root / "tmp/phase4-python-example/basic"
build_result = build(
    {
        "input_path": str(repo_root / "contracts/fixtures/phase3/inputs/basic-normalized-ir.json"),
        "artifacts_dir": str(artifacts_dir),
    },
    cwd=Path(__file__).resolve().parents[1],
)
print("build status =>", build_result["result_status"])

staging_report = inspect({"staging_path": str(artifacts_dir / "staging/manifest.json")}, cwd=Path(__file__).resolve().parents[1])
apkg_report = inspect({"apkg_path": str(artifacts_dir / "package.apkg")}, cwd=Path(__file__).resolve().parents[1])
print("inspect statuses =>", staging_report["observation_status"], apkg_report["observation_status"])

diff_result = diff(
    {
        "left_path": str(repo_root / "contracts/fixtures/phase3/expected/basic.inspect.json"),
        "right_path": str(repo_root / "contracts/fixtures/phase3/expected/basic.inspect.json"),
    },
    cwd=Path(__file__).resolve().parents[1],
)
print("diff status =>", diff_result["comparison_status"])
```

- [ ] **Step 4: Run the Python structured-layer tests and example to verify they pass**

Run: `PYTHONPATH="$(pwd)/bindings/python/src" python3 -m unittest discover -s bindings/python/tests -p 'test_structured.py' -v`
Expected: PASS with invalid-result handling, ref-derived helper view, degraded/partial result coverage, and protocol parse/shape/version failure coverage.

Run: `PYTHONPATH="$(pwd)/bindings/python/src" python3 bindings/python/examples/minimal_flow.py`
Expected: PASS with runtime metadata plus `normalize/build/inspect/diff` statuses.

- [ ] **Step 5: Commit**

```bash
git add bindings/python/src/anki_forge_python/contracts.py bindings/python/src/anki_forge_python/helpers.py bindings/python/src/anki_forge_python/structured.py bindings/python/src/anki_forge_python/__init__.py bindings/python/README.md bindings/python/examples/minimal_flow.py bindings/python/tests/test_structured.py
git commit -m "feat: add python structured bindings and example"
```

### Task 8: Add the shared conformance runner, CI hook, root docs, and Phase 4 exit checklist

**Files:**
- Create: `tests/conformance/README.md`
- Create: `tests/conformance/run_phase4_suite.py`
- Create: `tests/conformance/node_surface.mjs`
- Create: `tests/conformance/python_surface.py`
- Modify: `.github/workflows/contract-ci.yml`
- Modify: `README.md`
- Create: `docs/superpowers/checklists/phase-4-exit-evidence.md`

- [ ] **Step 1: Define the failing shared-runner entrypoint and checklist target**

```md
<!-- tests/conformance/README.md -->
# Phase 4 Shared Conformance Suite

Canonical runner identity:

~~~bash
python3 tests/conformance/run_phase4_suite.py
~~~

Primary objects:

- Rust facade
- CLI contract-json surface
- Node structured wrapper
- Python structured wrapper

Secondary objects:

- Node raw wrapper
- Python raw wrapper

The runner is not just a meta-test launcher. Its primary responsibility is to execute a shared case corpus across the four primary surfaces and compare canonical contract payloads for the same input. Secondary raw-layer conformance stays in language-native tests and focuses on runtime metadata plus invocation-failure classification.
```

- [ ] **Step 2: Run the canonical shared-runner command to verify it fails**

Run: `python3 tests/conformance/run_phase4_suite.py`
Expected: FAIL with `No such file or directory`.

- [ ] **Step 3: Add the runner, CI hook, root docs, and exit-evidence checklist**

```python
# tests/conformance/run_phase4_suite.py
from __future__ import annotations

import json
import os
import pathlib
import subprocess
import tempfile
import zipfile


REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]
PYTHONPATH = str(REPO_ROOT / "bindings/python/src")


def run(command: list[str], *, env: dict[str, str] | None = None, capture: bool = False) -> str | None:
    print("$", " ".join(command))
    merged_env = os.environ.copy()
    if env:
        merged_env.update(env)
    completed = subprocess.run(
        command,
        cwd=REPO_ROOT,
        check=True,
        env=merged_env,
        capture_output=capture,
        text=True,
    )
    return completed.stdout if capture else None


def canonicalize(payload: dict) -> str:
    return json.dumps(payload, sort_keys=True, separators=(",", ":"))


def write_request(tmp_root: pathlib.Path, case_id: str, command: str, request: dict, runtime_options: dict) -> pathlib.Path:
    path = tmp_root / f"{case_id}.{command}.request.json"
    path.write_text(json.dumps({"command": command, "request": request, "runtimeOptions": runtime_options}))
    return path


def cli_payload(command: str, args: list[str]) -> dict:
    stdout = run(
        ["cargo", "run", "-q", "-p", "contract_tools", "--", command, *args, "--output", "contract-json"],
        capture=True,
    )
    return json.loads(stdout)


def rust_payload(request_path: pathlib.Path) -> dict:
    stdout = run(
        ["cargo", "run", "-q", "-p", "anki_forge", "--example", "conformance_surface", "--", str(request_path)],
        capture=True,
    )
    return json.loads(stdout)


def node_payload(request_path: pathlib.Path) -> dict:
    stdout = run(["node", "tests/conformance/node_surface.mjs", str(request_path)], capture=True)
    return json.loads(stdout)


def python_payload(request_path: pathlib.Path) -> dict:
    stdout = run(
        ["python3", "tests/conformance/python_surface.py", str(request_path)],
        env={"PYTHONPATH": PYTHONPATH},
        capture=True,
    )
    return json.loads(stdout)


def strip_zip_entry(source: pathlib.Path, destination: pathlib.Path, entry_name: str) -> None:
    with zipfile.ZipFile(source) as src, zipfile.ZipFile(destination, "w") as dst:
        for info in src.infolist():
            if info.filename == entry_name:
                continue
            dst.writestr(info, src.read(info.filename))


def prepare_degraded_apkg(tmp_root: pathlib.Path) -> pathlib.Path:
    artifacts_dir = tmp_root / "inspect-degraded"
    cli_payload(
        "build",
        [
            "--manifest",
            str(REPO_ROOT / "contracts/manifest.yaml"),
            "--input",
            str(REPO_ROOT / "contracts/fixtures/phase3/inputs/basic-normalized-ir.json"),
            "--writer-policy",
            "default",
            "--build-context",
            "default",
            "--artifacts-dir",
            str(artifacts_dir),
        ],
    )
    degraded_apkg = artifacts_dir / "package-no-media.apkg"
    strip_zip_entry(artifacts_dir / "package.apkg", degraded_apkg, "media")
    return degraded_apkg


def prepare_partial_diff_inputs(tmp_root: pathlib.Path) -> tuple[pathlib.Path, pathlib.Path]:
    left_path = tmp_root / "left.inspect.json"
    right_path = tmp_root / "right.inspect.json"
    left_path.write_text(
        json.dumps(
            {
                "kind": "inspect-report",
                "observation_model_version": "phase3-inspect-v1",
                "source_kind": "staging",
                "source_ref": "artifacts/left/staging/manifest.json",
                "artifact_fingerprint": "artifact:left",
                "observation_status": "complete",
                "missing_domains": [],
                "degradation_reasons": [],
                "observations": {"notetypes": [], "templates": [], "fields": [], "media": [], "metadata": [], "references": []},
            }
        )
    )
    right_path.write_text(
        json.dumps(
            {
                "kind": "inspect-report",
                "observation_model_version": "phase3-inspect-v1",
                "source_kind": "apkg",
                "source_ref": "artifacts/right/package.apkg",
                "artifact_fingerprint": "artifact:right",
                "observation_status": "degraded",
                "missing_domains": ["references"],
                "degradation_reasons": ["reference coverage reduced"],
                "observations": {"notetypes": [], "templates": [], "fields": [], "media": [], "metadata": [], "references": []},
            }
        )
    )
    return left_path, right_path


def assert_same_payload(case_id: str, payloads: dict[str, dict]) -> None:
    canonical = {surface: canonicalize(payload) for surface, payload in payloads.items()}
    first_surface, first_payload = next(iter(canonical.items()))
    for surface, payload in canonical.items():
        if payload != first_payload:
            raise AssertionError(
                f"{case_id} mismatch between {first_surface} and {surface}\n"
                f"{first_surface}={payloads[first_surface]}\n{surface}={payloads[surface]}"
            )


def main() -> int:
    run(["cargo", "test", "-p", "anki_forge", "--test", "typed_core_tests", "-v"])
    run(["cargo", "test", "-p", "anki_forge", "--test", "runtime_facade_tests", "-v"])
    run(["cargo", "test", "-p", "contract_tools", "--test", "cli_tests", "-v"])
    run(["node", "--test", "bindings/node/test/raw.test.js", "bindings/node/test/structured.test.js"])
    run(
        ["python3", "-m", "unittest", "discover", "-s", "bindings/python/tests", "-v"],
        env={"PYTHONPATH": PYTHONPATH},
    )

    with tempfile.TemporaryDirectory(prefix="anki-forge-phase4-suite-") as tmp_dir:
        tmp_root = pathlib.Path(tmp_dir)
        degraded_apkg = prepare_degraded_apkg(tmp_root)
        left_report, right_report = prepare_partial_diff_inputs(tmp_root)

        cases = [
            {
                "id": "normalize-invalid",
                "command": "normalize",
                "request": {"inputPath": str(REPO_ROOT / "contracts/fixtures/invalid/missing-document-id.json")},
                "cli_args": [
                    "--manifest",
                    str(REPO_ROOT / "contracts/manifest.yaml"),
                    "--input",
                    str(REPO_ROOT / "contracts/fixtures/invalid/missing-document-id.json"),
                ],
            },
            {
                "id": "build-success",
                "command": "build",
                "request": {
                    "inputPath": str(REPO_ROOT / "contracts/fixtures/phase3/inputs/basic-normalized-ir.json"),
                    "artifactsDir": str(tmp_root / "build-success"),
                },
                "cli_args": [
                    "--manifest",
                    str(REPO_ROOT / "contracts/manifest.yaml"),
                    "--input",
                    str(REPO_ROOT / "contracts/fixtures/phase3/inputs/basic-normalized-ir.json"),
                    "--writer-policy",
                    "default",
                    "--build-context",
                    "default",
                    "--artifacts-dir",
                    str(tmp_root / "build-success"),
                ],
            },
            {
                "id": "inspect-degraded",
                "command": "inspect",
                "request": {"apkgPath": str(degraded_apkg)},
                "cli_args": ["--apkg", str(degraded_apkg)],
            },
            {
                "id": "diff-partial",
                "command": "diff",
                "request": {"leftPath": str(left_report), "rightPath": str(right_report)},
                "cli_args": ["--left", str(left_report), "--right", str(right_report)],
            },
        ]

        for case in cases:
            request_path = write_request(
                tmp_root,
                case["id"],
                case["command"],
                case["request"],
                {"cwd": str(REPO_ROOT / "bindings/node")},
            )
            payloads = {
                "rust": rust_payload(request_path),
                "cli": cli_payload(case["command"], case["cli_args"]),
                "node": node_payload(request_path),
                "python": python_payload(request_path),
            }
            assert_same_payload(case["id"], payloads)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
```

```js
// tests/conformance/node_surface.mjs
import fs from 'node:fs';

import { build, diff, inspect, normalize } from '../../bindings/node/src/index.js';

const requestEnvelope = JSON.parse(fs.readFileSync(process.argv[2], 'utf8'));
const handlers = { normalize, build, inspect, diff };
const result = await handlers[requestEnvelope.command](
  requestEnvelope.request,
  requestEnvelope.runtimeOptions,
);

delete result.helper;
delete result.rawCommand;
delete result.resolvedRuntime;
process.stdout.write(JSON.stringify(result));
```

```python
# tests/conformance/python_surface.py
from __future__ import annotations

import json
import pathlib
import sys

from anki_forge_python import build, diff, inspect, normalize


def normalize_request(command: str, request: dict) -> dict:
    if command == "normalize":
        return {"input_path": request["inputPath"]}
    if command == "build":
        return {
            "input_path": request["inputPath"],
            "artifacts_dir": request["artifactsDir"],
        }
    if command == "inspect":
        if "stagingPath" in request:
            return {"staging_path": request["stagingPath"]}
        return {"apkg_path": request["apkgPath"]}
    if command == "diff":
        return {"left_path": request["leftPath"], "right_path": request["rightPath"]}
    raise ValueError(f"unsupported conformance command: {command}")


def main() -> int:
    envelope = json.loads(pathlib.Path(sys.argv[1]).read_text())
    handlers = {
        "normalize": normalize,
        "build": build,
        "inspect": inspect,
        "diff": diff,
    }
    result = handlers[envelope["command"]](
        normalize_request(envelope["command"], envelope["request"]),
        **envelope["runtimeOptions"],
    )
    result.pop("helper", None)
    result.pop("rawCommand", None)
    result.pop("resolvedRuntime", None)
    print(json.dumps(result))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
```

```yaml
# .github/workflows/contract-ci.yml
name: contract-ci

on:
  pull_request:
  push:
    branches:
      - main

jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: actions/setup-node@v4
        with:
          node-version: "20"
      - uses: actions/setup-python@v5
        with:
          python-version: "3.11"
      - run: cargo fmt --all -- --check
      - run: cargo clippy -p contract_tools --all-targets -- -D warnings
      - run: cargo test -p contract_tools -v
      - run: cargo test -p anki_forge -v
      - run: cargo run -p contract_tools -- verify --manifest "$GITHUB_WORKSPACE/contracts/manifest.yaml"
      - run: cargo run -p contract_tools -- summary --manifest "$GITHUB_WORKSPACE/contracts/manifest.yaml"
      - run: cargo run -p contract_tools -- package --manifest "$GITHUB_WORKSPACE/contracts/manifest.yaml" --out-dir "$GITHUB_WORKSPACE/dist"
      - run: python3 tests/conformance/run_phase4_suite.py
```

```md
<!-- README.md -->
# anki-forge

`anki-forge` is a contract-first repository.
`contracts/` is the normative source of truth.
`anki_forge/` is the Rust facade surface for Phase 4.
`contract_tools/` is the CLI protocol and gate surface.
`bindings/node/` and `bindings/python/` are the first Phase 4 wrapper surfaces.

## Verification and release readiness

Use the contract tooling and the shared conformance runner from the repository root:

~~~bash
cargo run -p contract_tools -- verify --manifest "$(pwd)/contracts/manifest.yaml"
cargo run -p contract_tools -- summary --manifest "$(pwd)/contracts/manifest.yaml"
cargo run -p contract_tools -- package --manifest "$(pwd)/contracts/manifest.yaml" --out-dir dist
python3 tests/conformance/run_phase4_suite.py
~~~

For language-surface examples:

~~~bash
cargo run -p anki_forge --example minimal_flow
npm --prefix bindings/node run example:minimal
PYTHONPATH="$(pwd)/bindings/python/src" python3 bindings/python/examples/minimal_flow.py
~~~
```

```md
<!-- docs/superpowers/checklists/phase-4-exit-evidence.md -->
# Phase 4 Exit Evidence

- [ ] `cargo test -p anki_forge -v`
- [ ] `cargo test -p contract_tools --test cli_tests -v`
- [ ] `node --test bindings/node/test/raw.test.js bindings/node/test/structured.test.js`
- [ ] `PYTHONPATH="$(pwd)/bindings/python/src" python3 -m unittest discover -s bindings/python/tests -v`
- [ ] `python3 tests/conformance/run_phase4_suite.py`
- [ ] `cargo run -p anki_forge --example minimal_flow`
- [ ] `npm --prefix bindings/node run example:minimal`
- [ ] `PYTHONPATH="$(pwd)/bindings/python/src" python3 bindings/python/examples/minimal_flow.py`
```

- [ ] **Step 4: Run the shared conformance runner and key example commands to verify they pass**

Run: `python3 tests/conformance/run_phase4_suite.py`
Expected: PASS with shared `normalize-invalid`, `build-success`, `inspect-degraded`, and `diff-partial` cases producing canonical-equal payloads across Rust facade, CLI, Node structured, and Python structured.

Run: `cargo run -p anki_forge --example minimal_flow`
Expected: PASS with Rust example statuses.

Run: `npm --prefix bindings/node run example:minimal`
Expected: PASS with Node example statuses.

Run: `PYTHONPATH="$(pwd)/bindings/python/src" python3 bindings/python/examples/minimal_flow.py`
Expected: PASS with Python example statuses.

- [ ] **Step 5: Commit**

```bash
git add tests/conformance/README.md tests/conformance/run_phase4_suite.py tests/conformance/node_surface.mjs tests/conformance/python_surface.py .github/workflows/contract-ci.yml README.md docs/superpowers/checklists/phase-4-exit-evidence.md
git commit -m "docs: add phase4 conformance runner and exit evidence"
```
