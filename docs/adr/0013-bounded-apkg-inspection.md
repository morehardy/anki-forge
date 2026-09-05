# ADR 0013: Bound APKG inspection at the archive boundary

Status: proposed for merge

## Context

APKG input can arrive through standalone inspection, a comparison baseline, or
identity recovery. ZIP expansion and nested zstd expansion previously used
unbounded reads. Checking declared archive sizes after constructing the ZIP
index or after decoding would not protect allocation and temporary storage.

## Decision

Use one bounded reader for all APKG inspection, with finite defaults and explicit
Rust overrides. Bound the ZIP index before library allocation, actual outer ZIP
output, decoded per-entry/cumulative output, media-map entry count, and every
zstd frame window. Stream collection content to a private temporary SQLite file
and media payloads directly to hashes.

Resource-limit failures are typed and terminal for inspection. Keep the existing
strict/report-only build policy, while preserving a dedicated diagnostic and an
unavailable comparison. A failed current inspection cannot publish a candidate.

Record the new diagnostic and security-related acceptance tightening in bundle
0.4.0, without changing successful inspect report shapes or fingerprints.

## Consequences

Legitimate very large decks may require explicit larger limits. Ambiguous,
prefixed, multi-disk, and trailing-data ZIP layouts are rejected. No compression
ratio heuristic or automatic retry makes high-compression valid content fail or
silently disables the protection. This is an expansion budget, not a total heap
or execution-time sandbox; SQLite and observation construction need separate
quotas if used in an adversarial service.

See [RFC 0003](../rfcs/0003-apkg-inspection-resource-budgets.md) and the
[normative inspection semantics](../../contracts/semantics/inspect.md).
