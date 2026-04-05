---
asset_refs:
  - schema/package-build-result.schema.json
  - schema/writer-policy.schema.json
  - schema/build-context.schema.json
---

# Build

Source anchors:

- `docs/source/rslib/src/import_export/package/apkg/export.rs`
- `docs/source/rslib/src/import_export/package/colpkg/export.rs`
- `docs/source/rslib/src/import_export/package/meta.rs`
- `docs/source/rslib/src/import_export/package/media.rs`

Build first materializes a staging collection and then packages `.apkg` output.

The source shows two packaging paths:

- `export_apkg` writes a temporary collection, gathers media, and then packages
  the collection and media into an archive before atomically renaming the
  temporary package into place.
- `export_colpkg` closes the collection at the appropriate schema version,
  writes the collection file into a zip archive, includes the package metadata,
  writes a dummy legacy collection, and emits the media payload and media map.

The meta helpers show the package distinguishes legacy and latest modes through
its versioned metadata, including whether the media list is encoded as a
hashmap or as structured media entries and whether the archive is zstd
compressed.

The media helpers show filenames are normalized and validated during import,
and that a missing media map in older archives is treated as a legacy-compatible
case rather than an error.
