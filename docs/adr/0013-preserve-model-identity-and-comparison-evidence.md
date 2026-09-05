# ADR 0013: Preserve model identity and compare all observed domains

## Decision

Numeric Anki notetype IDs are persistent identities. New IDs are derived by one
internal writer Module from normalized document ID and logical note type ID;
existing APKG or lockfile IDs take precedence. The selected IDs cross staging,
writer, inspect, and lockfile persistence without independent reassignment.
Unavailable or conflicting baseline evidence must be diagnosed before strict
publication. Removed note types retain their reserved IDs in rewritten lockfiles.

Field set differences are explicit update-safety events. Field removal blocks
strict builds and remains high risk in report-only mode. Inspect owns the domain
inventory consumed by diff; incomplete evidence always produces partial or
unavailable comparison status.

## Consequences

- Reordering or adding note types no longer renumbers existing types.
- Old APKGs migrate by preserving their actual IDs. Legacy lockfiles with null
  model IDs require an APKG baseline for a strict migration.
- Contract bundle 0.4.0 documents the changed pre-1.0 derivation and stricter
  validation while retaining existing note GUID and wire schema versions.
- Detailed rationale, migration requirements, and regression coverage are in
  RFC 0003.
- Review follow-up: bundle 0.5.0 advances the observation model to
  `phase3-inspect-v2`; saved v1 reports remain readable but cross-version diffs
  are partial. Unverified report-only baselines must not rewrite lockfiles.
