# anki-forge

`anki-forge` Phase 1 is a contract-first repository.
`contracts/` is the normative source of truth.
`contract_tools/` provides internal verification tooling only.

## Phase 1 verification and release readiness

Use the contract tooling from the repository root with the bundled manifest:

```bash
cargo run -p contract_tools -- verify --manifest "$(pwd)/contracts/manifest.yaml"
cargo run -p contract_tools -- summary --manifest "$(pwd)/contracts/manifest.yaml"
cargo run -p contract_tools -- package --manifest "$(pwd)/contracts/manifest.yaml" --out-dir dist
```

`verify` checks the contract bundle and all executable gates.
`summary` prints the release-readiness smoke view of the bundle version, public axis, component versions, and asset inventory.
`package` writes the versioned contract artifact into `dist/` for release validation.

Before a Phase 1 release or merge that affects contracts, capture the checklist in `docs/superpowers/checklists/phase-1-exit-evidence.md` and make sure the same commands pass locally and in CI.

<!-- smoke-test: final codex trigger validation -->
