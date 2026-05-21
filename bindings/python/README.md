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

The high-level Python Product API is a target shape sketch for future bindings,
not a complete binding and not currently executable. The implemented Python
runtime module remains `anki_forge_python`.

- `bindings/python/examples/target_api_custom.py`
- `bindings/python/examples/target_api_media.py`

These examples mirror the product-facing `Project`, `NoteType`, `Field`,
`Template`, `GenerationRule`, `IdentityRecipe`, `Note`, media, and report APIs.
Slice 7's shape check is that media registration, media refs, template/CSS
references, unused binding warnings, and `pretty_report()` can be expressed with
plain Python variables and registry methods. The sketch deliberately avoids
Rust-only ownership patterns such as `media_mut()` borrowing or a required
`PendingMedia.export_as(...)` chain.
