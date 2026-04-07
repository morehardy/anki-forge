# anki-forge Node Bindings

This wrapper exposes three layers:

1. `runRaw()` for argv/stdout/stderr/exit-status preservation
2. `normalize()/build()/inspect()/diff()` for structured `contract-json` plus command-specific shape/version validation
3. helper projections under `result.helper`

The default path is workspace-mode discovery from the current working directory.

Example:

```bash
npm --prefix bindings/node run example:minimal
```
