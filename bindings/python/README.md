# anki-forge Python Bindings

This wrapper exposes three layers:

1. `run_raw()` for argv/stdout/stderr/exit-status preservation
2. `normalize()/build()/inspect()/diff()` for structured `contract-json` plus command-specific shape/version validation
3. helper projections under `result["helper"]`

The default path is workspace-mode discovery from the current working directory.

Example:

```bash
PYTHONPATH=bindings/python/src python3.11 bindings/python/examples/minimal_flow.py
```
