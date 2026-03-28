# Phase 1 Exit Evidence

Use this checklist to confirm `anki-forge` Phase 1 is ready to exit the foundation stage. The readiness bar is intentionally narrow: `contracts/` stays normative, `contract_tools/` stays verification-only, and the bundled manifest is the source of truth for the smoke checks.

Recorded against the current worktree on `2026-03-28`.

## Exit Checklist

- [ ] Repository workspace is clean enough to evaluate release readiness.
- [ ] `cargo fmt --all -- --check` passes.
- [ ] `cargo clippy -p contract_tools --all-targets -- -D warnings` passes.
- [ ] `cargo test -p contract_tools -v` passes.
- [ ] `cargo run -p contract_tools -- verify --manifest "$(pwd)/contracts/manifest.yaml"` passes.
- [ ] `cargo run -p contract_tools -- summary --manifest "$(pwd)/contracts/manifest.yaml"` prints the expected readiness smoke view.
- [ ] `cargo run -p contract_tools -- package --manifest "$(pwd)/contracts/manifest.yaml" --out-dir dist` produces a versioned bundle artifact.

## Command Evidence

Record the exact command output from the most recent readiness run.

### 1. Format check

Command:

```bash
cargo fmt --all -- --check
```

Evidence:

- `FAIL` in this worktree because `cargo fmt --all -- --check` reported pre-existing formatting drift in `contract_tools/src/fixtures.rs`, `contract_tools/src/registry.rs`, `contract_tools/src/versioning.rs`, and several test files outside the Task 8 write scope.
- The Task 8 docs/CI changes do not introduce new formatting issues.

### 2. Lint check

Command:

```bash
cargo clippy -p contract_tools --all-targets -- -D warnings
```

Evidence:

- `PASS`

### 3. Test suite

Command:

```bash
cargo test -p contract_tools -v
```

Evidence:

- `PASS`

### 4. Contract verification

Command:

```bash
cargo run -p contract_tools -- verify --manifest "$(pwd)/contracts/manifest.yaml"
```

Evidence:

- `verification passed`

### 5. Release-readiness summary

Command:

```bash
cargo run -p contract_tools -- summary --manifest "$(pwd)/contracts/manifest.yaml"
```

Evidence:

- Includes `bundle_version`, `public_axis`, `component_versions`, and `assets`
- Matches the bundled manifest in `contracts/manifest.yaml`

### 6. Package smoke

Command:

```bash
cargo run -p contract_tools -- package --manifest "$(pwd)/contracts/manifest.yaml" --out-dir dist
```

Evidence:

- Writes a tarball named `anki-forge-contract-bundle-0.1.0.tar.gz`
- Places the artifact under `dist/`

## Release Note

If any item fails, stop the release-readiness review and fix the contract bundle or tooling first. The exit gate is only satisfied when the checklist above is fully green and the same commands are reflected in CI.
