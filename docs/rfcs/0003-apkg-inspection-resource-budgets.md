# RFC 0003: APKG inspection resource budgets

Status: open for PR review

## Proposal

Adopt [ADR 0013](../adr/0013-bounded-apkg-inspection.md) and the limits in
[inspect semantics](../../contracts/semantics/inspect.md). Defaults are deliberate
initial policy, not measurements of the largest legitimate Anki collections.
Callers handling trusted large decks can choose larger finite Rust limits.
The CLI uses the same safe defaults; this proposal adds no CLI bypass flag.

Keep the complexity inside `writer_core` (`apkg_index`, `apkg_reader`, and
`inspect_limits`); callers select a policy and handle a typed failure. Do not
spread ZIP/zstd accounting into build, comparison, or identity logic.

## Compatibility and failure policy

Successful supported legacy and modern APKG inspection keeps the same hashes,
observations, and fingerprints. Oversized or ambiguous archives previously
accepted may now fail. Bundle 0.4.0 records this security-related tightening and
the new `INSPECT.RESOURCE_LIMIT_EXCEEDED` diagnostic; schema versions do not
change. The unsupported low-level inspector returns a typed `InspectError`.

Strict baseline failures are fatal. Report-only/disabled modes can publish a
valid current package while reporting the baseline unavailable; they cannot
claim a complete diff or silently erase the resource diagnostic. A current
inspection resource failure is fatal regardless of update-safety mode. Published
APKG bytes and identity lockfiles remain untouched on fatal inspection failure.

## Validation and rollout

Use small configurable limits to test exact boundaries, one-byte excesses,
outer ZIP expansion, forged sizes, nested/concatenated zstd, skippable frames,
oversized windows, ZIP64 counts, repeated media entries, and malformed streams.
Use public build/diff/runtime/CLI seams to verify error propagation and temporary
file cleanup. Retain all existing inspection/identity/publication regression
tests. Run workspace tests, clippy, docs, contract verify/summary/package,
embedded-bundle comparison, and package payload validation before opening PR.

No new dependency or APKG output-format change is needed. Heap/RSS and CPU
quotas, cancellation, SQLite query limits, and staging quotas are explicitly
outside this change; do not claim a general-purpose sandbox.

## Local verification evidence

Verified on macOS ARM64 with Rust 1.92.0; the full CLI/bindings gate used Node
24 and an isolated Python 3.12 environment with pytest:

- `make verify-ci`: passed, including all 23 Rust capability scenarios,
  conformance example, Node/Python tests, and contract verify/summary/package.
- `cargo test --workspace --all-features --locked`: passed; 14 new resource,
  build, and diff tests plus 2 new runtime/CLI/cleanup tests passed.
- All-features clippy with `-D warnings`, Rust doctests, and warning-free
  rustdoc: passed.
- Embedded bundle comparison, package allowlist, release metadata, dependency
  exception policy, and fresh packaged-consumer smoke test: passed.
- Contract summary/package identify bundle 0.4.0 and error registry 0.4.0;
  report schemas and fixture component versions remain 0.3.0.

Cross-platform execution is delegated to the PR's existing CI matrix. No
crates.io publication or release tag is part of this change.
