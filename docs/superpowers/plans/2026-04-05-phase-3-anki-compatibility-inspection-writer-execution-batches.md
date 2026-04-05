# Phase 3 Anki Compatibility, Inspection, and Writer Execution Batch Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Break the Phase 3 implementation work into execution-safe batches with explicit checkpoints, review stops, and verification evidence.

**Architecture:** Treat the main implementation plan as the normative task-level source of truth, and use this document as the orchestration layer for batching, sequencing, and review. Keep early batches strictly sequential while contracts, artifact lifetimes, and observation models are still moving; only parallelize after the writer artifact boundary and inspect/diff shape are stable.

**Tech Stack:** Rust workspace (`cargo`), JSON Schema, YAML contract assets, `rusqlite`, `zip`, `zstd`, `prost`, local `docs/source/rslib` compatibility anchors

---

## Source Of Truth

- Main implementation plan: `docs/superpowers/plans/2026-04-04-phase-3-anki-compatibility-inspection-writer-implementation-plan.md`
- Main spec: `docs/superpowers/specs/2026-04-04-phase-3-anki-compatibility-inspection-writer-design.md`
- Local Anki anchors that must be consulted before package-layout or compatibility decisions:
  - `docs/source/rslib/src/import_export/package/apkg/export.rs`
  - `docs/source/rslib/src/import_export/package/colpkg/export.rs`
  - `docs/source/rslib/src/import_export/package/meta.rs`
  - `docs/source/rslib/src/import_export/package/media.rs`
  - `docs/source/rslib/src/import_export/package/apkg/tests.rs`
  - `docs/source/rslib/src/import_export/package/apkg/import/notes.rs`
  - `docs/source/rslib/src/notetype/stock.rs`
  - `docs/source/rslib/src/image_occlusion/notetype.rs`

## Batch Rules

- Do not merge code from a later batch into an earlier checkpoint just because it is convenient.
- Do not start writer artifact work until the build-context asset is contract-driven and the Phase 3 schemas are registered.
- Do not start `.apkg` emission until staging materialization is real and caller-owned.
- Do not generate or review goldens until staging refs, `.apkg` refs, and fingerprints are deterministic.
- At every checkpoint, stop and review before opening the next batch.

## Checkpoint Packet

Every checkpoint should produce the same review packet:

- one fresh commit or a tight commit stack for that batch only
- exact verification commands run
- whether each command passed
- the key artifact paths created in that batch
- one short note on what is now stable enough for the next batch to depend on

## Batch Map

1. Batch 1: Workspace bootstrap
2. Batch 2: Writer-ready authoring and normalized contracts
3. Batch 3: Phase 3 report contracts and policy/context assets
4. Batch 4: `writer_core` DTOs and canonical serialization
5. Batch 5: Deterministic staging build
6. Batch 6: `.apkg` emission and Image Occlusion lane
7. Batch 7: Inspect/diff engines
8. Batch 8: CLI orchestration
9. Batch 9: Fixtures, gates, compat oracle, and exit evidence

### Batch 1: Workspace Bootstrap

**Includes:** Task 1

**Primary files:**
- `Cargo.toml`
- `Cargo.lock`
- `contract_tools/Cargo.toml`
- `contract_tools/tests/workspace_smoke_tests.rs`
- `writer_core/Cargo.toml`
- `writer_core/src/lib.rs`

- [ ] Land Task 1 exactly as written in the main plan.
- [ ] Run `cargo test -p contract_tools --test workspace_smoke_tests -v`
- [ ] Verify the resulting `Cargo.lock` delta only reflects the new `writer_core` workspace member and its direct dependency graph.
- [ ] Verify `writer_core::tool_contract_version()` is the only new public surface.
- [ ] Stop for review.

**Checkpoint outcome:**
- The workspace recognizes `writer_core`.
- No later module declarations exist yet.
- The repo is safe to start adding writer-ready contracts.

**Do not start Batch 2 until:**
- the smoke test passes
- `writer_core/src/lib.rs` is still minimal

### Batch 2: Writer-Ready Authoring And Normalized Contracts

**Includes:** Task 2 and Task 3

**Primary files:**
- `contracts/manifest.yaml`
- `contracts/schema/authoring-ir.schema.json`
- `contracts/schema/normalized-ir.schema.json`
- `contracts/schema/normalization-result.schema.json`
- `contracts/semantics/normalization.md`
- `authoring_core/src/model.rs`
- `authoring_core/src/stock.rs`
- `authoring_core/src/normalize.rs`
- `authoring_core/src/lib.rs`
- `authoring_core/tests/normalization_pipeline_tests.rs`
- `contract_tools/src/normalize_cmd.rs`
- `contract_tools/tests/schema_gate_tests.rs`

- [ ] Land Task 2 first, keeping manifest edits additive.
- [ ] Land Task 3 immediately after Task 2 in the same batch.
- [ ] Run `cargo test -p contract_tools --test schema_gate_tests -v`
- [ ] Run `cargo test -p authoring_core --test normalization_pipeline_tests -v`
- [ ] Capture one sample normalized output each for Basic, Cloze, and the scoped Image Occlusion lane.
- [ ] Stop for review.

**Checkpoint outcome:**
- `authoring_core` can emit writer-ready normalized data for the supported stock lanes.
- The normalized contract is stable enough for downstream writer work.

**Do not start Batch 3 until:**
- schema tests pass
- normalization pipeline tests pass
- Basic/Cloze/Image Occlusion normalized outputs look structurally correct

### Batch 3: Phase 3 Report Contracts And Policy Assets

**Includes:** Task 4

**Primary files:**
- `contracts/manifest.yaml`
- `contracts/schema/package-build-result.schema.json`
- `contracts/schema/inspect-report.schema.json`
- `contracts/schema/diff-report.schema.json`
- `contracts/schema/writer-policy.schema.json`
- `contracts/schema/verification-policy.schema.json`
- `contracts/schema/build-context.schema.json`
- `contracts/policies/writer-policy.default.yaml`
- `contracts/policies/verification-policy.default.yaml`
- `contracts/contexts/build-context.default.yaml`
- `contracts/semantics/build.md`
- `contracts/semantics/inspect.md`
- `contracts/semantics/diff.md`
- `contracts/semantics/golden-regression.md`
- `contract_tools/src/policies.rs`
- `contract_tools/src/semantics.rs`
- `contract_tools/tests/schema_gate_tests.rs`
- `contract_tools/tests/policy_gate_tests.rs`

- [ ] Land Task 4 exactly as written in the main plan.
- [ ] Re-check that `build_context_default` is loaded from `contracts/`, not command defaults.
- [ ] Run `cargo test -p contract_tools --test schema_gate_tests --test policy_gate_tests -v`
- [ ] Stop for review.

**Checkpoint outcome:**
- Phase 3 report schemas, policies, semantics, and build-context assets exist and validate.
- Downstream code can now rely on stable contract keys and asset lookup paths.

**Do not start Batch 4 until:**
- schema and policy tests pass
- manifest additions preserve all existing asset keys

### Batch 4: `writer_core` DTOs And Canonical Serialization

**Includes:** Task 5

**Primary files:**
- `writer_core/src/model.rs`
- `writer_core/src/policy.rs`
- `writer_core/src/canonical_json.rs`
- `writer_core/src/lib.rs`
- `writer_core/tests/build_tests.rs`

- [ ] Land Task 5 exactly as written in the main plan.
- [ ] Confirm `writer_core/src/lib.rs` still exports only the modules that exist at this point.
- [ ] Run `cargo test -p writer_core --test build_tests -v`
- [ ] Stop for review.

**Checkpoint outcome:**
- DTO names and references are stable enough for build/inspect/diff implementation.
- Canonical serialization works before any artifact logic is layered on top.

**Do not start Batch 5 until:**
- build tests pass
- no phantom module exports remain in `writer_core/src/lib.rs`

### Batch 5: Deterministic Staging Build

**Includes:** Task 6

**Primary files:**
- `writer_core/src/staging.rs`
- `writer_core/src/build.rs`
- `writer_core/src/lib.rs`
- `writer_core/tests/build_tests.rs`

- [ ] Land Task 6 exactly as written in the main plan.
- [ ] Verify `BuildArtifactTarget` owns artifact lifetime.
- [ ] Verify `staging_ref` points at a real materialized `staging/manifest.json`.
- [ ] Run `cargo test -p writer_core --test build_tests -v`
- [ ] Inspect one generated staging tree for Basic and one for Cloze.
- [ ] Stop for review.

**Checkpoint outcome:**
- Writer fast gate now has a real artifact to observe.
- `artifact_fingerprint` comes from canonical staging content, not paths.

**Do not start Batch 6 until:**
- build tests pass
- materialized staging trees exist on disk
- invalid builds carry selector/path-level diagnostics

### Batch 6: `.apkg` Emission And Image Occlusion Lane

**Includes:** Task 7

**Primary files:**
- `writer_core/Cargo.toml`
- `writer_core/src/apkg.rs`
- `writer_core/src/build.rs`
- `writer_core/src/lib.rs`
- `writer_core/tests/build_tests.rs`

- [ ] Re-read the local `rslib` package/export anchors before coding this batch.
- [ ] Land Task 7 exactly as written in the main plan.
- [ ] Verify `.apkg` artifacts are written into the caller-owned artifact root.
- [ ] Verify the latest lane uses `meta`, `collection.anki21b`, `collection.anki2`, and `media` according to the source-grounded rules.
- [ ] Run `cargo test -p writer_core --test build_tests -v`
- [ ] Inspect one generated `.apkg` layout for Basic and one for Image Occlusion.
- [ ] Stop for review.

**Checkpoint outcome:**
- `.apkg` materialization exists and survives process exit.
- The scoped Image Occlusion lane is supported at the writer layer.

**Do not start Batch 7 until:**
- build tests pass
- `.apkg` paths are stable
- media-map encoding and dummy collection behavior are aligned with local `rslib` anchors

### Batch 7: Inspect And Diff Engines

**Includes:** Task 8

**Primary files:**
- `writer_core/src/inspect.rs`
- `writer_core/src/diff.rs`
- `writer_core/src/lib.rs`
- `writer_core/tests/inspect_tests.rs`
- `writer_core/tests/diff_tests.rs`

- [ ] Re-read the local `rslib` `meta.rs`, `media.rs`, and relevant package tests before coding this batch.
- [ ] Land Task 8 exactly as written in the main plan.
- [ ] Verify `inspect_build_result()` is staging-first.
- [ ] Verify `inspect_apkg()` reads real archive semantics, not just entry presence.
- [ ] Run `cargo test -p writer_core --test inspect_tests --test diff_tests -v`
- [ ] Capture one staging inspect report, one `.apkg` inspect report, and one semantic-consistency diff.
- [ ] Stop for review.

**Checkpoint outcome:**
- The observation model is now a real regression surface.
- Diff output is structured enough to drive verification policy later.

**Do not start Batch 8 until:**
- inspect/diff tests pass
- staging/apkg semantic consistency test passes on a supported fixture

### Batch 8: CLI Orchestration

**Includes:** Task 9

**Primary files:**
- `contract_tools/src/main.rs`
- `contract_tools/src/lib.rs`
- `contract_tools/src/policies.rs`
- `contract_tools/src/build_cmd.rs`
- `contract_tools/src/inspect_cmd.rs`
- `contract_tools/src/diff_cmd.rs`
- `contract_tools/tests/cli_tests.rs`

- [ ] Land Task 9 exactly as written in the main plan.
- [ ] Verify `build` loads `build_context_default` from `contracts/`.
- [ ] Verify `build` requires `--artifacts-dir`.
- [ ] Run `cargo test -p contract_tools --test cli_tests -v`
- [ ] Run one manual CLI sequence:
  - `build`
  - `inspect --staging`
  - `inspect --apkg`
  - `diff`
- [ ] Stop for review.

**Checkpoint outcome:**
- `build`, `inspect`, and `diff` are now stable machine interfaces.
- Artifact roots and contract surfaces are wired end-to-end.

**Do not start Batch 9 until:**
- CLI tests pass
- manual contract-json commands work against one real fixture

### Batch 9: Fixtures, Gates, Compat Oracle, And Exit Evidence

**Includes:** Task 10 and Task 11

**Primary files:**
- `contracts/fixtures/index.yaml`
- `contracts/fixtures/phase3/**`
- `contract_tools/src/fixtures.rs`
- `contract_tools/src/gates.rs`
- `contract_tools/src/compat_oracle.rs`
- `contract_tools/tests/fixture_gate_tests.rs`
- `contract_tools/tests/compat_oracle_tests.rs`
- `README.md`
- `docs/superpowers/checklists/phase-3-exit-evidence.md`

- [ ] Land Task 10 first, including deterministic case-local `artifacts_dir` handling and golden capture from real outputs.
- [ ] Run `cargo test -p contract_tools --test fixture_gate_tests -v`
- [ ] Verify writer fixtures are staging-first and include staging/apkg semantic consistency checks.
- [ ] Land Task 11 second, using the local `rslib` anchors as the oracle baseline.
- [ ] Run `cargo test -p contract_tools --test compat_oracle_tests --test cli_tests --test fixture_gate_tests -v`
- [ ] Run `cargo run -p contract_tools -- verify --manifest "$(pwd)/contracts/manifest.yaml"`
- [ ] Update `docs/superpowers/checklists/phase-3-exit-evidence.md` with the exact final commands and artifact paths produced by the final implementation.
- [ ] Stop for final review.

**Checkpoint outcome:**
- Goldens are real, deterministic, and case-bound.
- Compatibility acceptance is stronger than a zip-entry smoke check.
- The final verify lane and operator evidence are complete.

**Do not mark Phase 3 execution complete until:**
- fixture gates pass
- compat oracle tests pass
- final `verify` passes
- exit evidence checklist is updated with the final command set

## Safe Parallelization Windows

Use these only after the entry criteria for the relevant batch are met:

1. After Batch 3:
   - one worker can start Batch 4 DTO/policy/canonical JSON work
   - another worker can prep the normalized fixture inputs that will later be used by Batch 9
   - do not parallelize writer artifact code before Batch 3 finishes

2. After Batch 7:
   - one worker can start CLI orchestration from Batch 8
   - another worker can draft case files and fixture catalog wiring for Batch 9
   - do not capture goldens until the CLI and inspect/diff surfaces stop moving

3. Inside Batch 9:
   - fixture catalog wiring and docs/checklist updates can run in parallel with compat-oracle implementation
   - final golden capture and final verify must still be serialized at the end

## Review Rhythm

- Review after Batch 2:
  - confirm normalized outputs are really writer-ready
  - confirm Image Occlusion is still scoped, not accidentally widened

- Review after Batch 5:
  - inspect the staging tree shape and diagnostic quality
  - confirm the writer fast gate is now truly staging-first

- Review after Batch 7:
  - inspect a real staging report, `.apkg` report, and diff report
  - confirm the evidence surface is rich enough for inspection-first goals

- Review after Batch 9:
  - inspect final verify output
  - inspect one compat oracle result
  - inspect final checked-in goldens

## Finish Line

The execution handoff is ready when all nine batches have a green checkpoint and the final review packet contains:

- passing test commands for `authoring_core`, `writer_core`, and `contract_tools`
- a passing `verify` command
- one real `package-build-result`
- one real staging `inspect-report`
- one real `.apkg` `inspect-report`
- one real `diff-report`
- one compat-oracle acceptance result for a supported core case
