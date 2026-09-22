# Node parity final review

Review base: `c8754ad3b83d2f8991a7c0d4f5f61ff50877c6c1`.
Scope: Node implementation, named tests, CI minimum-version entry, parity plan,
ADRs and evidence. Unrelated README/site/Python documentation work was excluded.
The only Python test edit removes one pre-existing trailing blank line that
blocked the repository's branch whitespace gate.

Spec: [Node parity plan](2026-09-21-node-api-parity-plan.md).
Standards: AGENT.md, CONTEXT.md, development guide and ADRs 0012/0017/0022.
Two independent review agents reviewed Standards and Spec; they did not edit
files or repeat the full test suites.

## Standards

No actionable hard standard violations or concrete defects found. Ownership,
private native adoption, independent state copying, Rust-authored snapshots and
binding protocol checks conform to the documented decisions. Thin copy-task
adapters have distinct N-API return types and were not considered pointless
indirection. Local resource measurements and external release gates are clearly
separated.

Outstanding findings: 0.

## Spec

One initial P2 acceptance gap: Worker termination tests covered completed
artifact owners but lacked unsettled temporary build/persist ownership. Added
an isolated-temp-root test that witnesses native build activity, terminates
before JS settlement, observes eventual cleanup, and separately preserves a
persistent copy with its original checksum. The reviewer confirmed the gap
closed. The capability matrix now links that case.

The final root audit also caught writeApkg(undefined) accidentally becoming a
temporary build after output became optional. A public red/green regression now
requires a filename for writeApkg while preserving build() temporary output.
The Spec reviewer rechecked this fix and the explicit identity recipe used by
both new independent parity constructors: no additional concerns.

Outstanding findings: 0. R01 and remote/platform/Desktop/publishing gates remain
explicitly separate; no unsupported completeness claim is made.

Final review totals: Standards 0 outstanding; Spec 0 outstanding.
