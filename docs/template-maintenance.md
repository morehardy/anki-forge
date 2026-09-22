# Maintaining template contracts

This page is for contributors changing contract fixtures and packaging. For
authoring a deck, use the [template bundle guide](template-bundles.md).

## Verify and package fixtures

A `template-bundle` entry in `contracts/fixtures/index.yaml` must point to the
bundle's `anki-template.yaml`. Contract verification checks its schema and imports
the directory through `Project::import_template_bundle`. Missing files, invalid
UTF-8 templates, unsafe paths, invalid template field references, and invalid
media registrations fail verification with the fixture ID and loader diagnostic.

The contract packager uses the Rust template loader's input list to include the
manifest and every declared front, back, browser template, stylesheet, and asset.
It preserves their relative paths, sorts and deduplicates the archive entries,
and excludes unreferenced files. A missing dependency fails before an existing
archive is opened for replacement. Symlink targets must stay inside the template
bundle; allowed aliases are packaged as regular files at the declared paths.

Package regression tests unpack the archive into a fresh directory, import both
normal and Cloze fixtures, build APKGs, and inspect card counts, templates, CSS,
browser appearance, target decks, and media. They also verify that removing any
declared fixture dependency makes both verification and packaging fail.

Run the focused checks after changing template fixtures or their packaging:

```bash
cargo test -p contract_tools --test fixture_gate_tests --test package_tests
cargo run -p contract_tools -- verify --manifest contracts/manifest.yaml
cargo run -p contract_tools -- summary --manifest contracts/manifest.yaml
cargo run -p contract_tools -- package --manifest contracts/manifest.yaml --out-dir dist
```

The Rust Distribution carries a committed copy of the bundle. Regenerate it
with the contract packager when the package payload changes, then check that it
matches a fresh deterministic package:

```bash
cargo run -p contract_tools -- package --manifest contracts/manifest.yaml --out-dir anki_forge/assets/contracts
bash scripts/check_embedded_contract_bundle.sh
```
