# Node API parity implementation record

- Plan: [2026-09-21 Node gap analysis](2026-09-21-node-api-parity-plan.md).
- Implementation starting point: `c8754ad3b83d2f8991a7c0d4f5f61ff50877c6c1`.
- Status: N01–N08/S01/V01 implemented and locally verified; both review axes have no outstanding findings.
- Scope: Rust Supported Consumer Interface. Core IO repair and external release
  gates are separate from this binding implementation; no publication occurred.

## Delivered interfaces

| Item | Implementation and observable evidence |
| --- | --- |
| N01 | Real native ApkgArtifact owner survives successful/failed report projection. build() and artifactsDir-only work. Clone/persist/close, GC, aliases, chdir and Worker teardown have public tests. Manual report JSON remains detached. |
| N02 | Async Project.fromDeck creates independent editable state and inherits baseDir. Mixed Basic/Cloze/IO, raw HTML, tags, IDs, custom additions, template bundles and media are compared with an independent Rust producer. Changed source fingerprints remain changed. |
| N03 | writeTo completes generation, reads 64 KiB chunks and waits for writes/drain. It preserves target openness and errors while closing source files and temporary handles. |
| N04 | Field/Template/NoteType/IdentityRecipe/GenerationRule/Note.describe and async Deck.describe return typed, deeply frozen Rust projections. Incomplete authoring input is not subjected to Project.add validation. |
| N05 | Deck.media.get returns a genuine branded reference or undefined; lookup preserves fingerprints and follows Busy/Failed state machinery. |
| N06 | Project/Deck.clone copy core state into independent state machines. Arc-owned spooled media survives original collection and is removed after the last copy. |
| N07 | All 11 inspection budgets accept safe numbers or exact u64 bigints. Native JSON parsing never uses f64. Build/diff share encoding and defaults remain checked safe numbers. |
| N08 | Deck builds dispatch directly to Rust Deck.build. Validate preserves existing Project-level checks; validate/diff snapshots no longer remain in the context. Direct Rust comparison and repeat-build allocation measurements are recorded. |
| S01 | ADR 0012 and Node README acknowledge public lower() while excluding its IR from product compatibility. |
| V01 | capability-matrix.json maps C01–C18/N01–N08 to tests; npm test discovers added files and rejects stale evidence references. Installed tests exercise new APIs and readonly/opaque types. |

Design decisions: [ADR 0022](../adr/0022-node-artifact-and-state-snapshots.md).
The candidate and all platform manifest versions remain synchronized at 0.2.0;
new native protocol 2 rejects stale same-version development binaries. There is
no claim that this candidate is published.

## Verification evidence

Targeted red/green checks passed for artifacts, conversion/query, cloning and
spooled-resource lifetime, readonly snapshots, bigint budgets and stream cleanup.
The Rust budget protocol tests validate exact 2^53+1/u64::MAX parsing and invalid
representations. Independent parity includes direct Deck.build and explicit
Deck.validate_report warning comparisons; mixed import cases are included in the
final suite.

| Verification | Result |
| --- | --- |
| Node 24.19.0 product suite | 42/42 passed |
| Independent Rust/Node parity | 7/7 tests, including 16 independent constructor scenarios, passed |
| Node 22.17.0 artifact/clone/snapshot/stream checks | 17/17 passed; this is not the exact minimum-patch CI gate |
| TypeScript source checks | passed |
| Real host tarball installation | ESM/CJS classes, new APIs, readonly/opaque declarations, five README examples, read-only node_modules, missing/mismatched runtime and old protocol rejection passed |
| Legacy Node tests | raw 7/7 and structured 16/16 passed; legacy example passed |
| Rust verification | fmt, contract governance, warning-free workspace Clippy, workspace tests (including artifact lifecycle and native JSON/u64 budgets), conformance example and 23/23 Rust consumer capabilities passed |
| Python regression checks | mypy passed; 190 tests and 5 subtests passed with the verify-ci import-isolation exclusion; both examples passed |
| Contract tooling | verify, summary and package passed |
| Host package metadata | matched versions, protocol 2, notices and binary passed |
| Review | Standards 0 outstanding; Spec 0 outstanding ([review record](2026-09-22-node-api-parity-review.md)) |

The verify-ci workflow was completed in segments after interruptions: an
N-API test-build lint was fixed; a new parity fixture was given its required
explicit identity recipe on both sides; an existing trailing blank line in
`bindings/python/tests/test_report.py` was removed to satisfy the branch diff
check. The sandbox denied loopback listening and dependency DNS, so the local
registry test and pinned Python test-tool installation were rerun with tool
approval. Remaining gates resumed from their interruption point rather than
repeating already successful suites. No skipped gate is reported as passed.
The final writeApkg filename regression was followed by source typechecking,
the artifact suite and the full 42-test Node suite.

[Resource evidence](2026-09-22-node-api-parity-resources.md) includes reproducible
private probes and raw JSON. On the measured workload, the direct Deck path
retains 10,900,241 fewer live Rust bytes after each build than a controlled version
of the previous dispatch. 1/16/64 MiB WAV exports use maximum 65,536-byte writes;
all target streams remain open. These are local observations, not general RSS or
performance guarantees.

## Remaining independent gates

### PR isolation verification

The Node changes were extracted from local commit `1e2e3bd` onto remote main
`8a44a94` for the independent `codex/node-api-parity` PR. Unmerged Python SDK
commits, the README rewrite and the Python-only whitespace cleanup described
above are excluded. The earlier full-workflow results describe the original
implementation checkout; they are not a full-workflow run on this isolated base.

On the isolated base, native build, TypeScript checks, all 42 product tests,
all 7 Rust/Node parity tests, real installed-package checks (including five
README examples), Rust formatting and native all-target Clippy passed with
Node 24.19.0. The Node implementation therefore does not require the unmerged
Python PR. Full remote CI remains a PR gate.

### Outstanding gates

- R01: Rust hide-one grouped cloze markup is still rejected by the writer. This
  work preserves the core error and does not rewrite markers in JavaScript.
- V02: CI now includes exact Node 22.13.0 plus rolling 22/24/26 on four targets;
  this local run is not remote candidate-commit evidence. Windows deletion,
  symlink/ACL and platform dynamic dependencies still need their real jobs.
- V03: Anki Desktop import/render/update-history checks remain pending. Automated
  Rust inspection of APKGs does not stand in for a GUI import.
- V04: npm name control, publishing credentials, full-platform tarballs,
  prerelease/recovery and public-registry checks follow RELEASING.md separately.

No native-panic fault injection was added. Ordinary failures, collection and
worker teardown are exercised through public interfaces; existing Failed state
handling remains in place. No public lower/raw writer/contract-loading API was
added, no core output semantics changed, and no release gate is silently closed.
