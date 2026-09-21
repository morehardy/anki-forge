# anki-forge Python

Use the public `anki_forge` package to build Anki `.apkg` files with `Project`,
`Note`, custom note types, and media. Python 3.11+ is required. Exports run
through the Rust `contract_tools` runtime.

## From a source checkout

From the repository root, build the runtime with Rust 1.92.0:

```sh
cargo build -p contract_tools --release
```

Save the following as `example.py` in the repository root:

```python
from anki_forge import Note, Project

project = Project("Spanish", stable_id="spanish", default_deck="Spanish")
project.add_note(Note.basic("hola", "hello", stable_id="es:hola"))
report = project.write_apkg("spanish.apkg")
report.ensure_success()
```

Run it with the source package on the import path:

```sh
PYTHONPATH=bindings/python/src python3 example.py
```

This writes `spanish.apkg` for import into Anki Desktop. The source setup finds
`contracts/manifest.yaml` and the built runtime from the checkout. Run it from
the repository root, or configure `RuntimeOverride` explicitly.

## Runtime packaging

A wheel with a staged runtime includes its executable and contract files;
consumers of that wheel do not need Rust or this checkout. Maintainers stage
those resources after building the release executable:

```sh
python3 bindings/python/scripts/stage_runtime.py
```

This prepares files for packaging; it does not build or publish a wheel. The
[wheel CI job](../../.github/workflows/contract-ci.yml) contains the complete
platform-specific packaging and installed-consumer checks.

## Guides

- [Quick start, custom note types, and media](../../docs/python/quick-start.md)
- [Migration from genanki](../../docs/python/genanki-migration.md)
- [Diagnostics and build errors](../../docs/python/diagnostics.md)
- [Development and verification](../../docs/development.md#node-and-python-development)

`anki_forge_python` remains the legacy low-level runtime for earlier integration
code in this repository; it is excluded from the public wheel. New applications
should import `anki_forge`.
