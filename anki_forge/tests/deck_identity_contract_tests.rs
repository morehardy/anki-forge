use anki_forge::{
    BasicIdentityField, BasicIdentityOverride, BasicIdentitySelection, BasicNote, Deck,
    IdentityProvenance,
};
use serde::Deserialize;
use serde_json::Value;
use serde_yaml::Value as YamlValue;
use std::collections::BTreeSet;
use std::{fs, path::PathBuf};

#[derive(Debug, Deserialize)]
struct NoteIdentityFixture {
    recipe_id: String,
    note_kind: String,
    input: serde_json::Value,
    expected: serde_json::Value,
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn contracts_root() -> PathBuf {
    repo_root().join("contracts")
}

fn load_case(path: &str) -> NoteIdentityFixture {
    let raw = fs::read_to_string(repo_root().join(path)).expect("read fixture");
    serde_json::from_str(&raw).expect("parse fixture")
}

fn contract_provenance(provenance: &IdentityProvenance) -> &'static str {
    match provenance {
        IdentityProvenance::ExplicitStableId => "explicit_stable_id",
        IdentityProvenance::InferredFromNoteFields => "note_fields",
        IdentityProvenance::InferredFromNotetypeFields => "notetype_fields",
        IdentityProvenance::InferredFromStockRecipe => "stock_recipe",
    }
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

#[test]
fn basic_front_only_contract_case_matches_expected_output() {
    let fixture = load_case("contracts/fixtures/note-identity/basic-front-only.case.json");
    let front = fixture.input["front"].as_str().expect("fixture front");
    let back = fixture.input["back"].as_str().expect("fixture back");
    let mut deck = Deck::new("Spanish");
    deck.add(BasicNote::new(front, back))
        .expect("add inferred basic");

    assert_eq!(fixture.note_kind, "basic");

    let snapshot = deck.notes()[0]
        .resolved_identity()
        .expect("resolved identity snapshot");
    assert_eq!(
        snapshot.recipe_id.as_deref(),
        Some(fixture.recipe_id.as_str())
    );
    assert_eq!(
        snapshot.canonical_payload.as_deref(),
        fixture.expected["canonical_payload"].as_str()
    );
    assert_eq!(snapshot.stable_id, fixture.expected["stable_id"]);
    assert_eq!(
        contract_provenance(&snapshot.provenance),
        fixture.expected["provenance"]
            .as_str()
            .expect("fixture expected provenance")
    );
}

#[test]
fn inferred_basic_note_uses_afid_instead_of_generated_id() {
    let mut deck = Deck::new("Spanish");
    deck.add(BasicNote::new("hola", "hello"))
        .expect("add inferred note");

    assert!(deck.notes()[0].id().starts_with("afid:v1:"));
}

#[test]
fn duplicate_inferred_basic_identity_payload_fails_at_add_time() {
    let mut deck = Deck::new("Spanish");
    deck.add(BasicNote::new("hola", "hello"))
        .expect("add first inferred note");

    let err = deck
        .add(BasicNote::new("hola", "different back ignored by default"))
        .expect_err("duplicate inferred basic payload should fail");

    assert!(err.to_string().contains("AFID.IDENTITY_DUPLICATE_PAYLOAD"));
}

#[test]
fn basic_deck_policy_selecting_back_changes_canonical_payload() {
    let mut deck = Deck::builder("Spanish")
        .basic_identity(
            BasicIdentitySelection::new([BasicIdentityField::Back]).expect("basic selection"),
        )
        .build();
    deck.add(BasicNote::new("hola", "hello"))
        .expect("add inferred note");

    let snapshot = deck.notes()[0]
        .resolved_identity()
        .expect("resolved identity snapshot");
    assert_eq!(
        snapshot.provenance,
        IdentityProvenance::InferredFromNotetypeFields
    );
    assert!(!snapshot.used_override);
    assert_eq!(
        snapshot.canonical_payload.as_deref(),
        Some("{\"algo_version\":1,\"components\":{\"selected_fields\":[{\"name\":\"back\",\"value\":\"hello\"}]},\"notetype_family\":\"stock\",\"notetype_key\":\"basic\",\"recipe_id\":\"basic.core.v1\"}")
    );
}

#[test]
fn basic_note_override_selecting_front_and_back_sets_override_snapshot() {
    let override_cfg = BasicIdentityOverride::new(
        [BasicIdentityField::Front, BasicIdentityField::Back],
        "sense",
    )
    .expect("basic override");
    let mut deck = Deck::new("Spanish");
    deck.add(BasicNote::new("hola", "hello").identity_override(override_cfg))
        .expect("add inferred note");

    let snapshot = deck.notes()[0]
        .resolved_identity()
        .expect("resolved identity snapshot");
    assert_eq!(
        snapshot.provenance,
        IdentityProvenance::InferredFromNoteFields
    );
    assert!(snapshot.used_override);
    assert_eq!(
        snapshot.canonical_payload.as_deref(),
        Some("{\"algo_version\":1,\"components\":{\"selected_fields\":[{\"name\":\"front\",\"value\":\"hola\"},{\"name\":\"back\",\"value\":\"hello\"}]},\"notetype_family\":\"stock\",\"notetype_key\":\"basic\",\"recipe_id\":\"basic.core.v1\"}")
    );
}

#[test]
fn basic_identity_normalizes_unicode_and_newlines() {
    let mut decomposed = Deck::new("Spanish");
    decomposed
        .add(BasicNote::new("Cafe\u{301}\r\nline\rnext", "hello"))
        .expect("add decomposed note");

    let mut composed = Deck::new("Spanish");
    composed
        .add(BasicNote::new("Caf\u{e9}\nline\nnext", "hello"))
        .expect("add composed note");

    assert_eq!(decomposed.notes()[0].id(), composed.notes()[0].id());
}

#[test]
fn basic_identity_keeps_leading_and_trailing_whitespace_significant() {
    let mut plain = Deck::new("Spanish");
    plain
        .add(BasicNote::new("hola", "hello"))
        .expect("add plain note");

    let mut padded = Deck::new("Spanish");
    padded
        .add(BasicNote::new(" hola ", "hello"))
        .expect("add padded note");

    assert_ne!(plain.notes()[0].id(), padded.notes()[0].id());
}
