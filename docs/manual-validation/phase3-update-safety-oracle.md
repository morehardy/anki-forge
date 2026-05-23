# Phase 3 Update Safety Oracle

Required evidence for each scenario:

- Date
- Platform
- Anki version
- anki-forge commit
- Scenario input APKG SHA-256
- Updated APKG SHA-256
- Note count before and after import
- Card count before and after import
- GUID comparison result
- Duplicate note outcome
- Relevant diagnostics

Diagnostic coverage matrix:

- `UPDATE.BASELINE_APKG_UNREADABLE`: unit or integration test
- `UPDATE.BASELINE_LOCKFILE_UNREADABLE`: unit or integration test
- `UPDATE.BASELINE_SCHEMA_UNSUPPORTED`: lockfile schema test
- `UPDATE.BASELINE_CONFLICT_GUID`: reconcile test
- `UPDATE.GUID_PRESERVED_FROM_PREVIOUS`: compare_to integration test
- `UPDATE.GUID_DERIVATION_DRIFT`: reconcile test with legacy GUID
- `UPDATE.FIELD_MERGE_ID_CHANGED`: merge safety test
- `UPDATE.TEMPLATE_MERGE_ID_CHANGED`: merge safety test
- `UPDATE.NOTE_DATA_METADATA_UNMERGEABLE`: writer APKG test

Manual scenarios:

1. First import then update with same stable ids updates existing notes.
2. Adding a new note inserts only the new note.
3. Field rename with stable field key/config id remains update-safe.
4. Field config id drift is caught before import.
5. Template reorder is visible as a scheduling risk signal.
