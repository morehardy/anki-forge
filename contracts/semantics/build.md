---
asset_refs:
  - schema/package-build-result.schema.json
  - schema/writer-policy.schema.json
  - schema/build-context.schema.json
---

# Build

Local reference source anchors under `docs/source/rslib`:

- `docs/source/rslib/src/import_export/package/apkg/export.rs`
- `docs/source/rslib/src/import_export/package/colpkg/export.rs`
- `docs/source/rslib/src/import_export/package/meta.rs`
- `docs/source/rslib/src/import_export/package/media.rs`

Phase 3 build semantics use a staging-first materialization flow before optional
`.apkg` packaging. The reference source below constrains how package metadata,
collection payloads, and media payloads should align with modern Anki package
format behavior.

The reference source defines two closely related package export paths:

- `export_apkg` creates a temporary collection file, gathers media filenames,
  packages the collection and media through `export_collection()`, and
  atomically renames the temporary archive into place.
- `export_colpkg` closes the collection at the legacy or latest schema version,
  then calls `export_collection_file()`, which writes `meta`, the versioned
  collection payload, a dummy legacy `collection.anki2`, media files, and the
  `media` map into the archive.

In `meta.rs`, package version determines the collection filename, supported
schema version, zstd compression usage, and whether the `media` listing is
encoded as a legacy JSON hashmap or as structured media entries.

In `media.rs`, imported media filenames are safety-checked and normalized, and
archives that omit the `media` entry are treated as a legacy-compatible empty
media map during import rather than as an immediate error.

For `Phase 5A`, the writer also preserves product-layer template metadata:

- field-label metadata lowers into authoring field metadata and is carried into
  staged and packaged output
- browser appearance declarations are carried onto matching templates during
  lowering and preserved through build materialization
- template target deck names are resolved to stable deck ids during staging and
  reused when writing template configs and card rows
