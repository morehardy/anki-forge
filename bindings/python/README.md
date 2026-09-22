# anki-forge Python 0.2

[Website](https://ankiforge.dev/) · [Python documentation](https://ankiforge.dev/docs/python-quickstart/) ·
[GitHub](https://github.com/morehardy/anki-forge) · [Issues](https://github.com/morehardy/anki-forge/issues)

The Python SDK owns Rust Project, Deck, media and Artifact objects through a
private native extension. Validation, identity derivation, template loading,
media verification, comparison and APKG writing run in the same core as Rust.
Consumers need a matching wheel, not Cargo, a CLI executable or this checkout.

This branch prepares 0.2; it does not publish a PyPI release. The supported
validation matrix is ordinary CPython **3.11 and 3.12**, Linux x86_64,
Windows x86_64, macOS x86_64 and macOS ARM64. See the
[coverage and evidence index](COVERAGE.md) for scope and verification results.
Free-threaded Python, subinterpreters and other Python versions are not claimed.

## From a source checkout

Use Rust 1.92.0 and a Python 3.11/3.12 virtual environment:

```sh
python3 -m venv .venv
. .venv/bin/activate
python -m pip install maturin==1.15.0 pytest==9.1.1 mypy==2.3.1
maturin develop --manifest-path bindings/python/native/Cargo.toml --locked
python bindings/python/examples/native_workflow.py target/python-example
```

On Windows, activate `.venv\Scripts\Activate.ps1` instead. Merely adding `src`
to PYTHONPATH does not build the extension.

## Author and build

```python
from anki_forge import Note, Project

project = Project("Spanish", stable_id="spanish", default_deck="Spanish")
project.add_note(Note.basic("hola", "hello", stable_id="es:hola"))
project.validate().ensure_success()
project.write_apkg("spanish.apkg").ensure_success()
```

Inputs remain mutable until added. Project additions validate and take a
snapshot; later changes to the input or a `project.notes` observation do not
change the Project. Construction settings are read-only. Relative paths use
`base_dir`, fixed at construction, including after a working-directory change.

Custom types use `Field`, `Template`, `GenerationRule` and `IdentityRecipe`.
`Field(identity=True)` remains the fallback recipe; explicit type and per-note
recipes are supported. `Content.text/html` and `MediaRef.image/sound` use core
rendering. Field/template default keys now use Rust rules. Preserve explicit
old keys when upgrading an existing deck.

## Media and templates

`project.media.add_file(path, export_as=...)` registers source fingerprints
immediately. Keep that file available and unchanged through build. `add_bytes`
accepts bytes/bytearray, snapshots the input and enforces the core 64 KiB limit.
References use export filenames; another project's reference resolves only if
the destination registers that filename. `report.media` describes the build's
media entries and counts.

`project.import_template_bundle(directory)` loads `anki-template.yaml`, template
files, CSS and assets through Rust. Failed imports leave no partial note type or
media registrations. `TemplateBundleError` retains `code`, `path` and UTF-8
`byte_offset`. [Template bundle format](../../docs/template-bundles.md).

## Build, compare and keep outputs

```python
from anki_forge import BuildOptions, InspectLimits

options = BuildOptions(output="baseline.apkg").first_update_safe_build("identity.lock.json")
project.build(options).ensure_success()
project.diff_against_apkg("baseline.apkg", inspect_limits=InspectLimits()).ensure_success()
project.build(BuildOptions(output="next.apkg", compare_to="baseline.apkg", fail_on="high")
    .update_safe("identity.lock.json")).ensure_success()

with project.build() as report:
    report.ensure_success()
    assert report.artifact is not None
    with report.artifact.persist_to("saved.apkg") as saved:
        print(saved.path)
```

BuildOptions covers output/staging/report paths, all 11 inspection budgets,
comparison/risk, lockfiles/update safety and media modes/policies. `None` keeps
Rust defaults. `inspect=False` still performs final package checks. Advanced
media policies are SDK extensions over the core's internal-tools surface.

`build` returns complete failure reports; `ensure_success` raises `BuildError`
retaining the report and any recoverable artifact. Diff returns a
`ProjectDiffReport` without publishing APKGs or advancing lockfiles. Both report
types support lossless `raw` / `to_json()` snapshots, including unknown fields.
`BuildReport.from_json()` returns path metadata, never an owning Artifact.

Retain an Artifact handle to keep its temporary file alive. `copy.copy` creates
an independent owner; `close` releases it. `report.close()` releases the report's
reference, so a separately retained Artifact remains usable. Persistent copies
and explicit outputs survive close. `to_apkg_bytes()` returns bytes;
`write_to(binary_file)` copies the completed package in bounded chunks, handles
short writes, returns a byte count and keeps the caller's stream open.

## Deck and concurrency

`Deck(name, stable_id=..., basic_identity=["front"])` exposes `add_basic`,
`add_cloze`, `add_image_occlusion`, media registration, validate, build and the
same output helpers. `BasicIdentityOverride(fields, reason_code)` supports
per-note Basic identity choices. Deck Basic/Cloze strings follow Rust Deck's
HTML semantics. Deck IO derives identity without an explicit stable ID and
checks image dimensions and rectangles in Rust. Its separate `DeckMediaRef`
and media registration follow Rust Deck rules; the Project inline limit is not
silently imposed on Deck's distinct API.

`Project.from_deck(deck)` takes a core snapshot, preserving media and identity
evidence while retaining the original Deck. The returned Project can import
bundles and add custom types and notes. The shared core currently rejects
`hide_one_guess_one` grouped cloze output with `PRODUCT.CLOZE_MARKER_MALFORMED`;
this limitation applies across bindings.

One operation may run per Project/Deck at a time. Concurrent use of the same
object raises `BINDING.PROJECT_BUSY` / `BINDING.DECK_BUSY`; independent objects
can work concurrently while Rust releases the GIL. Mutable input builders need
caller synchronization. Reusing native state or Artifact handles after fork is
rejected. Create new objects in the child. No cancellation guarantee is made.

## Install, verify and troubleshoot

```sh
maturin build --manifest-path bindings/python/native/Cargo.toml --release --locked --out bindings/python/dist
python -m pip install bindings/python/dist/<matching-wheel>.whl
cargo build -p anki_forge_python_native --example python_parity --locked
python -m pytest bindings/python/tests -q
python -m mypy --config-file bindings/python/pyproject.toml bindings/python/src/anki_forge
python bindings/python/scripts/check_native_wheel.py bindings/python/dist/<matching-wheel>.whl --observer target/debug/examples/python_parity
```

Use `python_parity.exe` on Windows. The installed check creates a clean venv
outside the checkout, removes compiler/CLI paths, runs the example and positive /
negative typing consumers, and optionally runs the full public suite against
an independently built Rust producer. `py.typed`, the private extension stub,
MIT license and dependency notices are included in the wheel. The abi3 wheel is
built once per platform and tested on both supported Python versions.

`versions()` reports binding, linked core API and embedded contract versions.
`BINDING.EXTENSION_UNAVAILABLE` means the extension could not load: install a
matching wheel or build the checkout with Maturin. `BINDING.VERSION_MISMATCH`
means Python and native package files are incompatible; reinstall the package.
There is no automatic Cargo invocation or CLI fallback at import or build time.

See [0.1 migration](MIGRATION.md), [quick start](../../docs/python/quick-start.md),
[diagnostics](../../docs/python/diagnostics.md), [coverage](COVERAGE.md), and the
[installed runnable example](examples/native_workflow.py). The dev-only
`anki_forge_python` low-level CLI wrapper remains outside the public wheel.
