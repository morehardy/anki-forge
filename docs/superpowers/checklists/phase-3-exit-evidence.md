# Phase 3 Exit Evidence

Use this checklist to confirm `anki-forge` Phase 3 compatibility exits with executable oracle evidence and stable machine interfaces.

Recorded on `2026-04-06` in worktree `phase3-compat-writer`.

## Exit Checklist

- [ ] `cargo test -p contract_tools --test compat_oracle_tests -v` passes.
- [ ] `cargo test -p contract_tools --test compat_oracle_tests --test cli_tests --test fixture_gate_tests -v` passes.
- [ ] `cargo test -p contract_tools -v` passes.
- [ ] `cargo run -p contract_tools -- build ... --output contract-json` returns valid package build result JSON.
- [ ] `cargo run -p contract_tools -- inspect ... --output contract-json` returns valid inspect report JSON for both staging and apkg artifacts.
- [ ] `cargo run -p contract_tools -- diff ... --output contract-json` returns valid diff report JSON.
- [ ] `cargo run -p contract_tools -- verify --manifest "$(pwd)/contracts/manifest.yaml"` passes.
- [ ] `git diff --check` reports no whitespace errors.

## Command Evidence

### 1. Phase 3 compatibility oracle test

Command:

```bash
cargo test -p contract_tools --test compat_oracle_tests -v
```

Evidence:

- `test compat_oracle_gates_accept_bundled_writer_phase3_fixtures ... ok`

### 2. Focused gate regression suite

Command:

```bash
cargo test -p contract_tools --test compat_oracle_tests --test cli_tests --test fixture_gate_tests -v
```

Evidence:

- Record pass/fail status and key failing test names if any regress.

### 3. Full `contract_tools` suite

Command:

```bash
cargo test -p contract_tools -v
```

Evidence:

- Record pass/fail status for each test target.

### 4. Build contract-json interface

Command:

```bash
cargo run -p contract_tools -- build \
  --manifest "$(pwd)/contracts/manifest.yaml" \
  --input "$(pwd)/contracts/fixtures/phase3/inputs/basic-normalized-ir.json" \
  --writer-policy default \
  --build-context default \
  --artifacts-dir "$(pwd)/contracts/artifacts/phase3-checklist" \
  --output contract-json
```

Evidence:

- Output JSON includes `kind=package-build-result`, `result_status`, `writer_policy_ref`, `build_context_ref`, and artifact refs/fingerprints.

### 5. Inspect contract-json interface (staging/apkg)

Commands:

```bash
cargo run -p contract_tools -- inspect \
  --staging "$(pwd)/contracts/artifacts/phase3-checklist/staging/manifest.json" \
  --output contract-json > "$(pwd)/contracts/artifacts/phase3-checklist/staging.inspect.json"

cargo run -p contract_tools -- inspect \
  --apkg "$(pwd)/contracts/artifacts/phase3-checklist/package.apkg" \
  --output contract-json > "$(pwd)/contracts/artifacts/phase3-checklist/apkg.inspect.json"
```

Evidence:

- Both commands emit `kind=inspect-report` with `observation_status=complete` for the fixture run.

### 6. Diff contract-json interface

Command:

```bash
cargo run -p contract_tools -- diff \
  --left "$(pwd)/contracts/artifacts/phase3-checklist/staging.inspect.json" \
  --right "$(pwd)/contracts/artifacts/phase3-checklist/apkg.inspect.json" \
  --output contract-json
```

Evidence:

- Output JSON includes `kind=diff-report` and expected comparison status for the staged/apkg pair.

### 7. Full contract verification

Command:

```bash
cargo run -p contract_tools -- verify --manifest "$(pwd)/contracts/manifest.yaml"
```

Evidence:

- `verification passed`

### 8. Whitespace/lint hygiene

Command:

```bash
git diff --check
```

Evidence:

- No whitespace errors reported.
