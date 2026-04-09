use std::{fs, path::PathBuf};

use anki_forge::product::ProductDocument;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/product")
        .join(name)
}

#[test]
fn product_cases_can_be_loaded_from_data_fixtures_and_lowered() {
    let value =
        fs::read_to_string(fixture_path("basic_answer_divider.case.json")).expect("read fixture");
    let document: ProductDocument = serde_json::from_str(&value).expect("deserialize product case");
    let lowering = document.lower().expect("lower fixture");

    assert_eq!(
        lowering.authoring_document.metadata_document_id,
        "fixture-basic-doc"
    );
    assert!(lowering.authoring_document.notetypes[0]
        .templates
        .as_ref()
        .unwrap()[0]
        .answer_format
        .contains("Answer"));
}

#[test]
fn io_font_bundle_case_can_be_loaded_from_data_fixtures_and_lowered() {
    let value = fs::read_to_string(fixture_path("io_font_bundle.case.json")).expect("read fixture");
    let document: ProductDocument = serde_json::from_str(&value).expect("deserialize product case");
    let lowering = document.lower().expect("lower fixture");

    assert_eq!(
        lowering.authoring_document.metadata_document_id,
        "fixture-io-font-doc"
    );
    assert_eq!(lowering.authoring_document.media.len(), 1);
    assert!(lowering.authoring_document.media[0]
        .filename
        .starts_with("_io-main_"));
    assert_eq!(
        lowering.authoring_document.notetypes[0].field_metadata[0]
            .label
            .as_deref(),
        Some("Header")
    );
}
