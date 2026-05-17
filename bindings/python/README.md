# anki-forge Python Bindings

This wrapper currently exposes three contract-runtime layers:

1. `run_raw()` for argv/stdout/stderr/exit-status preservation
2. `normalize()/build()/inspect()/diff()` for structured `contract-json` plus command-specific shape/version validation
3. helper projections under `result["helper"]`

The default path is workspace-mode discovery from the current working directory.

Example:

```bash
PYTHONPATH=bindings/python/src python3.11 bindings/python/examples/minimal_flow.py
```

## Target Product API Shape

The Phase 1 Rust MVP also documents the intended high-level Python Product API
shape for future bindings. Read this file as an API sketch, not as a currently
executable example:

- `bindings/python/examples/target_api_custom.py`

That example mirrors the Rust `Project`, `NoteType`, `Field`, `Template`,
`GenerationRule`, `IdentityRecipe`, and `Note` APIs. It is a product-facing API
sketch; the implemented Python runtime module remains `anki_forge_python`.
