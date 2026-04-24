use serde_json::Value;
use serde_yaml::Value as YamlValue;
use std::collections::BTreeSet;
use std::{fs, path::PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn contracts_root() -> PathBuf {
    repo_root().join("contracts")
}

fn note_identity_catalog_cases() -> Vec<YamlValue> {
    let raw = fs::read_to_string(contracts_root().join("fixtures/index.yaml"))
        .expect("read fixture catalog");
    let catalog: YamlValue = serde_yaml::from_str(&raw).expect("parse fixture catalog");
    let cases = catalog["cases"].as_sequence().expect("catalog cases");

    cases
        .iter()
        .filter(|case| case["category"].as_str() == Some("note-identity"))
        .cloned()
        .collect()
}

fn note_identity_catalog_inputs() -> Vec<String> {
    note_identity_catalog_cases()
        .iter()
        .map(|case| {
            case["input"]
                .as_str()
                .expect("note-identity case input")
                .to_string()
        })
        .collect()
}

#[test]
fn bundled_catalog_declares_all_note_identity_cases() {
    let ids: BTreeSet<String> = note_identity_catalog_cases()
        .iter()
        .map(|case| {
            case["id"]
                .as_str()
                .expect("note-identity case id")
                .to_string()
        })
        .collect();
    let expected = BTreeSet::from([
        "note-identity-basic-front-only".to_string(),
        "note-identity-cloze-hint-ignored".to_string(),
        "note-identity-cloze-whitespace-significant".to_string(),
        "note-identity-cloze-malformed".to_string(),
        "note-identity-io-order-insensitive".to_string(),
        "note-identity-io-translation-different".to_string(),
    ]);

    assert_eq!(
        ids, expected,
        "expected the bundled catalog to declare exactly the complete note-identity golden set"
    );
}

#[test]
fn all_cataloged_note_identity_fixtures_exist_and_parse() {
    for rel in note_identity_catalog_inputs() {
        let raw = fs::read_to_string(contracts_root().join(&rel))
            .unwrap_or_else(|err| panic!("missing fixture {rel}: {err}"));
        let _: Value = serde_json::from_str(&raw)
            .unwrap_or_else(|err| panic!("invalid JSON fixture {rel}: {err}"));
    }
}

#[test]
fn io_order_insensitive_fixture_input_is_not_already_canonical() {
    let raw = fs::read_to_string(
        contracts_root().join("fixtures/note-identity/io-order-insensitive.case.json"),
    )
    .expect("read io fixture");
    let fixture: Value = serde_json::from_str(&raw).expect("parse io fixture");
    let rects = fixture["input"]["rects"]
        .as_array()
        .expect("io fixture rects");

    assert_eq!(rects[0]["x"].as_i64(), Some(100));
    assert_eq!(rects[0]["y"].as_i64(), Some(40));
    assert_eq!(rects[1]["x"].as_i64(), Some(10));
    assert_eq!(rects[1]["y"].as_i64(), Some(20));
}
