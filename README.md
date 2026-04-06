# anki-forge

`anki-forge` Phase 1 is a contract-first repository.
`contracts/` is the normative source of truth.
`contract_tools/` provides internal verification tooling only.

## Verification and release readiness

Use the contract tooling from the repository root with the bundled manifest:

```bash
cargo run -p contract_tools -- verify --manifest "$(pwd)/contracts/manifest.yaml"
cargo run -p contract_tools -- summary --manifest "$(pwd)/contracts/manifest.yaml"
cargo run -p contract_tools -- package --manifest "$(pwd)/contracts/manifest.yaml" --out-dir dist
cargo run -p contract_tools -- normalize --manifest "$(pwd)/contracts/manifest.yaml" --input "$(pwd)/contracts/fixtures/phase2/inputs/minimal-authoring-ir.json" --output contract-json
cargo run -p contract_tools -- build --manifest "$(pwd)/contracts/manifest.yaml" --input "$(pwd)/contracts/fixtures/phase3/inputs/basic-normalized-ir.json" --writer-policy default --build-context default --artifacts-dir "$(pwd)/contracts/artifacts/readme-phase3" --output contract-json
cargo run -p contract_tools -- inspect --staging "$(pwd)/contracts/artifacts/readme-phase3/staging/manifest.json" --output contract-json > "$(pwd)/contracts/artifacts/readme-phase3/staging.inspect.json"
cargo run -p contract_tools -- inspect --apkg "$(pwd)/contracts/artifacts/readme-phase3/package.apkg" --output contract-json > "$(pwd)/contracts/artifacts/readme-phase3/apkg.inspect.json"
cargo run -p contract_tools -- diff --left "$(pwd)/contracts/artifacts/readme-phase3/staging.inspect.json" --right "$(pwd)/contracts/artifacts/readme-phase3/apkg.inspect.json" --output contract-json
```

`verify` checks the contract bundle and all executable gates.
`summary` prints the release-readiness smoke view of the bundle version, public axis, component versions, and asset inventory.
`package` writes the versioned contract artifact into `dist/` for release validation.
`normalize --output contract-json` runs Phase 2 authoring normalization and emits contract-stable machine output for CI/fixtures.
`build`, `inspect`, and `diff` with `--output contract-json` are stable machine interfaces for Phase 3 compatibility and fixture assertions.

Before a Phase 1 release or merge that affects contracts, capture the checklist in `docs/superpowers/checklists/phase-1-exit-evidence.md` and make sure the same commands pass locally and in CI.
For Phase 2 core authoring readiness, capture and update `docs/superpowers/checklists/phase-2-exit-evidence.md`.
For Phase 3 compatibility readiness, capture and update `docs/superpowers/checklists/phase-3-exit-evidence.md`.
