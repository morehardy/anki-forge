# ADR 0001: Contract Bundle Is the Source of Truth

## Context

The repository keeps executable tools, schemas, and governance text together so
that the contract bundle can be validated as a single unit.

## Decision

`contracts/manifest.yaml` is the authoritative index for bundled assets. Any
contract asset that is part of the public bundle must be referenced from the
manifest and validated through the bundle tooling.

## Consequences

- Contract changes stay reviewable from one entry point.
- Bundle consumers do not need to infer asset locations from convention alone.
- Missing or stale bundle entries become validation failures instead of hidden
  drift.
