# Phase 2 Exit Evidence

Use this checklist to confirm `anki-forge` Phase 2 core authoring model exits with executable contract evidence.

Recorded on `2026-04-04` in worktree `codex/phase2-core-authoring`.

## Exit Checklist

- [x] `cargo test -p authoring_core -v` passes.
- [x] `cargo test -p contract_tools -v` passes.
- [x] `cargo run -p contract_tools -- verify --manifest "$(pwd)/contracts/manifest.yaml"` passes.
- [x] `cargo run -p contract_tools -- normalize --manifest "$(pwd)/contracts/manifest.yaml" --input "$(pwd)/contracts/fixtures/phase2/inputs/minimal-authoring-ir.json" --output contract-json` returns valid contract JSON with required top-level fields.

## Command Evidence

### 1. `authoring_core` crate tests

Command:

```bash
cargo test -p authoring_core -v
```

Evidence:

- `test result: ok. 8 passed; 0 failed; ...` (`normalization_pipeline_tests`)
- `test result: ok. 5 passed; 0 failed; ...` (`risk_tests`)
- `test result: ok. 7 passed; 0 failed; ...` (`selector_tests`)

### 2. `contract_tools` crate tests

Command:

```bash
cargo test -p contract_tools -v
```

Evidence:

- `test result: ok. 4 passed; 0 failed; ...` (`cli_tests`)
- `test result: ok. 5 passed; 0 failed; ...` (`fixture_gate_tests`)
- `test result: ok. 10 passed; 0 failed; ...` (`schema_gate_tests`)
- `test result: ok. 1 passed; 0 failed; ...` (`package_tests`)

### 3. Contract gate verification

Command:

```bash
cargo run -p contract_tools -- verify --manifest "$(pwd)/contracts/manifest.yaml"
```

Evidence:

- `verification passed`

### 4. Phase 2 contract-json normalization

Command:

```bash
cargo run -p contract_tools -- normalize --manifest "$(pwd)/contracts/manifest.yaml" --input "$(pwd)/contracts/fixtures/phase2/inputs/minimal-authoring-ir.json" --output contract-json
```

Evidence:

- Command returns one JSON object with required top-level fields:
  - `kind`
  - `result_status`
  - `tool_contract_version`
  - `policy_refs`
  - `comparison_context`
  - `diagnostics`
- Captured output (2026-04-04):

```json
{"comparison_context":null,"diagnostics":{"items":[],"kind":"normalization-diagnostics","status":"valid"},"kind":"normalization-result","merge_risk_report":null,"normalized_ir":{"document_id":"demo-doc","kind":"normalized-ir","resolved_identity":"det:demo-doc","schema_version":"0.1.0"},"policy_refs":{"identity_policy_ref":"identity-policy.default@1.0.0","risk_policy_ref":null},"result_status":"success","tool_contract_version":"phase2-v1"}
```
