#![cfg(feature = "internal-tools")]

#[test]
fn update_safety_diagnostic_matrix_lists_every_update_code() {
    let matrix = include_str!("../../docs/manual-validation/phase3-update-safety-oracle.md");
    for code in [
        "UPDATE.BASELINE_APKG_UNREADABLE",
        "UPDATE.BASELINE_LOCKFILE_UNREADABLE",
        "UPDATE.BASELINE_SCHEMA_UNSUPPORTED",
        "UPDATE.BASELINE_CONFLICT_GUID",
        "UPDATE.GUID_PRESERVED_FROM_PREVIOUS",
        "UPDATE.GUID_DERIVATION_DRIFT",
        "UPDATE.FIELD_MERGE_ID_CHANGED",
        "UPDATE.TEMPLATE_MERGE_ID_CHANGED",
        "UPDATE.NOTE_DATA_METADATA_UNMERGEABLE",
    ] {
        assert!(matrix.contains(code), "matrix missing {code}");
    }
}
