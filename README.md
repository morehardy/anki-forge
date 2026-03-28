# anki-forge

`anki-forge` Phase 1 is a contract-first repository.
`contracts/` is the normative source of truth.
`contract_tools/` provides internal verification tooling only.

## Phase 1 verification and packaging

Run `cargo run -p contract_tools -- verify --manifest "$(pwd)/contracts/manifest.yaml"` to validate the bundle locally.
Run `cargo run -p contract_tools -- package --manifest "$(pwd)/contracts/manifest.yaml" --out-dir dist` to build the versioned contract artifact.
