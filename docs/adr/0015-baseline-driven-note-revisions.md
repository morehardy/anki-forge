# ADR 0015: Baseline-driven deterministic note revisions

## Decision

Note identity and content revision are separate. A full-content digest plus the
actual numeric modification time is persistent baseline evidence. Unchanged notes
retain their baseline time; changed notes advance by one with checked arithmetic.
APKG inspection is authoritative over lockfile evidence. Missing evidence blocks
strict updates and remains explicit high risk in report-only mode.

## Consequences

- Answer and tag changes can trigger Anki's normal newer-note import condition.
- Same input plus same baseline remains reproducible, including content reverts.
- No wall clock or new public Interface is introduced. Baseline-free exports are
  initial releases, not a proof that an existing collection will be updated.
- Legacy lockfiles need a previous APKG to recover revision evidence in strict mode.
- Report-only never persists fallback revisions/model IDs as recovered baseline
  evidence: incomplete or rejected baselines suppress lockfile writes. Rejected
  requested lockfiles remain high risk even when their diagnostics are warnings.
- See RFC 0005 for the content hash, migration rules, and real-import verification.
