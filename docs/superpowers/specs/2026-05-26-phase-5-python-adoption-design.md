# Phase 5 Python Adoption Design

- Date: 2026-05-26
- Status: Approved design draft
- Source: `docs/api-design.md` Phase 5
- Scope: Python Product API adoption, packaging, diagnostics, docs, and release readiness

## Confirmed Decisions

1. Phase 5 uses a pure Python Product API wrapper with a bundled Rust CLI runtime.
2. The first release does not include CSV or pandas helpers.
3. Python users import `anki_forge`; the existing `anki_forge_python` package remains a lower-level runtime wrapper or compatibility layer.
4. Phase 5 does not introduce PyO3, maturin, native Python extension modules, or generated Rust source execution.
5. Python Product API must lower into the same Rust Product/IR/build/report pipeline as the Rust API.

## Goal

Phase 5 makes `anki-forge` adoptable by Python users, especially users who know genanki and want a safer build system. A user should be able to install a wheel, write a Python `Project`, build an `.apkg`, inspect structured diagnostics, and migrate basic/custom/media decks without installing Rust.

Phase 5 is not the first Python technology spike. The repository already has low-level Python contract wrappers, target API sketches, Rust Product APIs, `BuildReport`, and Phase 4 diff/risk behavior. This phase turns those pieces into a product-grade Python package.

## Non-Goals

- CSV or pandas helpers.
- PyO3 or native Python extension bindings.
- A Python API that mirrors Rust ownership or chaining patterns.
- Exposing Authoring IR, Normalized IR, or contract tooling as the first user-facing mental model.
- A genanki-compatible API clone. Migration is concept-oriented, not drop-in compatibility.

## Recommended Approach

Use a Python Product API plus a bundled Rust CLI.

Python owns the user-facing object model and serialization. Rust owns lowering, normalization, writer behavior, inspection, diff, risk, and report generation.

```text
Python Product objects
  -> ProductDocument JSON
  -> bundled contract_tools product-build
  -> Rust ProductDocument -> Project::from_product_document(...)
  -> existing lowering / normalize / writer / inspect / diff / risk
  -> BuildReport JSON
  -> Python BuildReport / DiagnosticsError
```

This approach avoids PyO3 release complexity while preserving one semantic pipeline. It also aligns with the existing `contract_tools product-build` command and `anki_forge::runtime::build_product_document_with_writer_stack(...)`.

## Alternatives Considered

### Current Contract Tools Only

Python could generate Authoring IR and call the existing `normalize`, `build`, `inspect`, and `diff` wrapper APIs.

This has the smallest code delta, but it exposes low-level contract concepts as the main Python experience. It also risks diverging from the Rust Product API, especially for media, identity, diagnostics, and `BuildReport`.

### Temporary Rust Runner

Python could generate temporary Rust code or a config consumed by a Rust runner.

This reuses Rust builders quickly, but it is brittle for packaging, debugging, error locations, and cross-platform wheels. It is not suitable as the adoption path.

## Package Layout

Add a new public package under `bindings/python/src/anki_forge`:

```text
anki_forge/
  __init__.py
  project.py        # Project, build/write_apkg
  notetype.py       # NoteType, Field, Template, GenerationRule
  note.py           # Note and Content helpers
  media.py          # MediaRegistry and MediaRef
  report.py         # BuildReport projections
  diagnostics.py    # Diagnostic and DiagnosticsError
  runtime.py        # bundled/workspace runtime discovery and product-build invocation
  product_json.py   # ProductDocument transport serialization
```

Keep `anki_forge_python` as the low-level wrapper for existing `normalize`, `build`, `inspect`, and `diff` workflows. The main README and Python guide should use `anki_forge`, not `anki_forge_python`.

## Public Python API Shape

The main entry points are:

- `Project(name, stable_id=None, default_deck=None)`
- `NoteType.custom(id, name=None)`
- `Field(name, key=None, identity=False, sort=False, required=False, optional=False)`
- `Template(name, key=None, front=..., back=..., generate_when=None)`
- `GenerationRule.anki_default()`, `.all(fields)`, `.any(fields)`, `.cloze(field)`
- `Note.basic(front, back, stable_id=None)`
- `Note.cloze(text, back_extra="", stable_id=None)`
- `Note(note_type_id, stable_id=None).text(...).html(...).sound(...).image(...).tag(...)`
- `Project.media.add_file(path, export_as=...)`
- `Project.media.add_bytes(source_label=..., data=..., export_as=...)`
- `Project.write_apkg(path, compare_to=None, fail_on=None, report_json=None)`

`text()` is safe text by default. `html()` is explicit raw HTML. Python should prefer mutable object style with optional fluent helpers where they read naturally.

## Transport Schema

Python serializes Product objects to `ProductDocument` JSON and calls `contract_tools product-build`.

The current Rust `ProductDocument` transport is narrower than the target Python API. Phase 5 must extend that transport before relying on it for the Python API. Required additions include:

- Field metadata needed by Python `Field`: identity, sort, required, optional.
- Template generation rules using stable field/template keys.
- Typed content for notes: safe text, raw HTML, sound media, image media.
- Media asset declarations that support `add_file`, `add_bytes`, and `export_as`.
- Stable source paths that let Rust diagnostics point back to Python project objects.

Python must not silently degrade typed content into plain strings when that would change escaping, media handling, identity behavior, or diagnostics.

## Runtime Behavior

`anki_forge.runtime` locates a runtime in this order:

1. Bundled wheel runtime: platform-specific `contract_tools` executable plus contract assets.
2. Workspace runtime: repository checkout with `contracts/manifest.yaml`, used by development and CI.
3. Explicit runtime override for tests and advanced users.

`Project.write_apkg()` writes a temporary ProductDocument JSON file and invokes:

```bash
contract_tools product-build \
  --manifest <runtime>/contracts/manifest.yaml \
  --product-input <tmp/project.json> \
  --apkg-out <target.apkg> \
  --output contract-json
```

`compare_to`, `fail_on`, and `report_json` are passed through to `product-build`.

## Reports And Exceptions

Python exposes `BuildReport` as a typed projection over `BuildReportJson`, including:

- artifact path
- counts
- media summary
- diagnostics
- inspect summary
- update safety summary
- diff summary
- risk summary
- status and comparison status

`DiagnosticsError` is the primary Product API failure exception. It includes:

- `message`
- `report`
- `diagnostics`
- `status`
- `exit_status`
- `stdout`
- `stderr`

If `product-build` exits non-zero but stdout contains valid report JSON, Python raises `DiagnosticsError` with the parsed report. If stdout cannot be parsed as a report, Python raises a runtime/protocol error that preserves argv, stdout, stderr, exit status, and runtime details.

`BuildReport.ensure_success()` raises `DiagnosticsError` when report status is invalid, blocked, error, missing an artifact, or contains error diagnostics.

## Packaging And Release

The publish package should be named `anki-forge`, with import path `anki_forge`.

Wheel contents must include:

- Python `anki_forge` package.
- Existing low-level `anki_forge_python` package if retained for compatibility.
- Platform-matching `contract_tools` executable.
- `contracts/manifest.yaml` and required bundle assets.

Release automation should build wheels for Linux, macOS, and Windows. A packaging smoke test must install the wheel into a clean virtual environment and build a basic deck without a Rust toolchain.

The current `bindings/python/pyproject.toml` package name `anki-forge-python` should not be the final user-facing package name for Phase 5.

## Documentation

Python docs should start with Product API examples:

1. Basic deck quick start.
2. Long-term `Project` with stable IDs.
3. Custom note type with fields and templates.
4. Media registration with `sound()` and `image()`.
5. Diagnostics and `BuildReport`.
6. genanki concept migration guide.

The low-level contract wrapper remains documented separately for advanced workflows.

## Testing

Phase 5 needs these test layers:

- Python unit tests for object modeling, ProductDocument dict serialization, and report/diagnostic projections.
- Golden tests comparing Python-generated ProductDocument JSON with Rust-side expected snapshots.
- End-to-end Python tests for basic, custom note type, and media projects that produce real `.apkg` artifacts.
- Failure tests for duplicate stable IDs, missing media, `fail_on` policy, and unwritable `report_json`.
- Runtime tests for bundled, workspace, and explicit override discovery.
- Wheel smoke tests in a clean environment without Rust.
- Existing `anki_forge_python` raw/structured tests kept green.

## Implementation Slices

### Phase 5A: Product API And Transport

- Add public `anki_forge` Python package.
- Implement Product object model and `to_product_document()`.
- Extend Rust `ProductDocument` transport to cover Python target API semantics.
- Add `product-build` invocation and Python `BuildReport`/`DiagnosticsError`.
- Ship runnable Python examples for basic, custom note type, and media.

### Phase 5B: Packaging And Adoption

- Bundle `contract_tools` and contract assets into wheels.
- Add clean-venv wheel smoke tests.
- Write Python quick start and genanki migration guide.
- Add release workflow artifacts for Linux, macOS, and Windows.
- Preserve low-level wrapper documentation for advanced users.

## Acceptance Criteria

1. `pip install anki-forge` provides `import anki_forge`.
2. Basic, custom note type, and media Python examples run without a local Rust toolchain.
3. Python Product API builds through the Rust Product/IR/build/report pipeline.
4. Python exposes structured diagnostics exceptions, not only strings.
5. `BuildReport` exposes counts, media summary, artifact path, diagnostics, inspect, diff/risk where available, and status.
6. Documentation guides genanki users by concept migration.
7. CI validates Linux, macOS, and Windows wheel artifacts or equivalent release candidates.
8. Existing low-level Python wrapper tests continue to pass.
