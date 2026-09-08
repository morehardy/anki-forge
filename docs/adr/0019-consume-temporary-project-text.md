# ADR 0019: Consume temporary Project text during Deck export

## Decision

The build pipeline accepts either a borrowed authoring input or ownership of the
temporary Project created by Deck/Package export. Preparation consumes the input;
later stages retain only normalized content, identity evidence and the project
stable ID. Public Deck and Project methods keep their borrowed signatures.

For an owned Project, resolve every note identity before draining any fields.
Move HTML strings through the existing product and authoring lowering into the
normalized document. Text escaping, nested content rendering, stock defaults and
custom field alias precedence use the existing semantics. Keep field names in
the temporary note shells until diagnostic source paths have been mapped. Reuse
the resolved identities for source mapping and reconciliation, then release the
temporary Project before media normalization and writing.

Borrowed Project builds continue to render from the caller's editable state.
`Project::lower()` still returns a self-contained plan. No public representation,
serialization schema, global allocator or update-safety rule changes.

## Consequences

- Large Deck fields no longer require a second full copy in the temporary Project
  alongside normalized content. The caller's original Deck remains reusable.
- Identity-dependent rendering must happen before values are taken. Retained
  field names are diagnostic metadata, not editable notes exposed to callers.
- Duplicate identities and skipped notes use the same source mapping rules as
  borrowed lowering. Imported identity snapshots retain their provenance.
- Media ingestion still owns its temporary files and prepared sources; releasing
  the input never changes external-file verification or cleanup responsibilities.

## Validation

Tests compare consuming and borrowed lowering, including partial invalid plans,
field aliases, duplicate IDs and source paths. A storage-identity test protects
long HTML transfers. Facade tests compare exact APKG and identity-lockfile bytes,
repeat exports, and repair missing media after failure. Native paired benchmarks
and a separate allocation probe measure the memory effect without adding
instrumentation to the published crate.

The [paired benchmark report](../../benchmarks/results/20260907-text-ownership/README.md)
records all 29 cases, separate RSS and allocation probes, exact artifact checks,
Anki imports and the excluded power-transition run.
