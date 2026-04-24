use anki_forge::{
    BasicIdentityField, BasicIdentityOverride, BasicIdentitySelection, BasicNote, ClozeNote, Deck,
    IdentityProvenance, IoMode, MediaSource,
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

fn io_fixture_image_path() -> PathBuf {
    repo_root().join(
        "contracts/fixtures/phase3/manual-desktop-v1/S03_io_minimal/assets/occlusion-heart.png",
    )
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
fn cloze_hint_ignored_contract_case_matches_expected_output() {
    let fixture = load_case("contracts/fixtures/note-identity/cloze-hint-ignored.case.json");
    let text = fixture.input["text"].as_str().expect("fixture text");
    let mut deck = Deck::new("Geo");
    deck.add(ClozeNote::new(text)).expect("add inferred cloze");

    assert_eq!(fixture.note_kind, "cloze");

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
fn cloze_whitespace_significant_contract_case_matches_expected_output() {
    let fixture =
        load_case("contracts/fixtures/note-identity/cloze-whitespace-significant.case.json");
    let text = fixture.input["text"].as_str().expect("fixture text");
    let mut deck = Deck::new("Geo");
    deck.add(ClozeNote::new(text)).expect("add inferred cloze");

    assert_eq!(fixture.note_kind, "cloze");

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
fn cloze_malformed_contract_case_reports_expected_error_code() {
    let fixture = load_case("contracts/fixtures/note-identity/cloze-malformed.case.json");
    let text = fixture.input["text"].as_str().expect("fixture text");
    let mut deck = Deck::new("Geo");
    let err = deck
        .add(ClozeNote::new(text))
        .expect_err("malformed cloze fixture must fail");

    assert_eq!(fixture.note_kind, "cloze");
    assert!(
        err.to_string().contains(
            fixture.expected["error_code"]
                .as_str()
                .expect("fixture expected error code")
        ),
        "{err}"
    );
}

fn assert_io_contract_case_matches_expected_output(path: &str) {
    let fixture = load_case(path);
    let image_path = repo_root().join(
        fixture.input["image_path"]
            .as_str()
            .expect("fixture image path"),
    );
    let mode = match fixture.input["mode"].as_str().expect("fixture io mode") {
        "hide_all_guess_one" => IoMode::HideAllGuessOne,
        "hide_one_guess_one" => IoMode::HideOneGuessOne,
        other => panic!("unknown fixture io mode: {other}"),
    };

    let mut deck = Deck::new("Anatomy");
    let image = deck
        .media()
        .add(MediaSource::from_file(&image_path))
        .expect("fixture image");
    let mut draft = deck.image_occlusion().note(image).mode(mode);
    for rect in fixture.input["rects"].as_array().expect("fixture io rects") {
        draft = draft.rect(
            rect["x"].as_u64().expect("rect x") as u32,
            rect["y"].as_u64().expect("rect y") as u32,
            rect["width"].as_u64().expect("rect width") as u32,
            rect["height"].as_u64().expect("rect height") as u32,
        );
    }
    draft.add().expect("add fixture io");

    assert_eq!(fixture.note_kind, "image_occlusion");

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

fn resolve_fixture_deck(fixture: &NoteIdentityFixture) -> anyhow::Result<Deck> {
    let mut deck = Deck::new("Fixture Deck");
    match fixture.note_kind.as_str() {
        "basic" => {
            deck.add(BasicNote::new(
                fixture.input["front"].as_str().expect("fixture front"),
                fixture.input["back"].as_str().expect("fixture back"),
            ))?;
        }
        "cloze" => {
            deck.add(ClozeNote::new(
                fixture.input["text"].as_str().expect("fixture text"),
            ))?;
        }
        "image_occlusion" => {
            let image_path = repo_root().join(
                fixture.input["image_path"]
                    .as_str()
                    .expect("fixture image path"),
            );
            let mode = match fixture.input["mode"].as_str().expect("fixture io mode") {
                "hide_all_guess_one" => IoMode::HideAllGuessOne,
                "hide_one_guess_one" => IoMode::HideOneGuessOne,
                other => panic!("unknown fixture io mode: {other}"),
            };

            let image = deck.media().add(MediaSource::from_file(&image_path))?;
            let mut draft = deck.image_occlusion().note(image).mode(mode);
            for rect in fixture.input["rects"].as_array().expect("fixture io rects") {
                draft = draft.rect(
                    rect["x"].as_u64().expect("rect x") as u32,
                    rect["y"].as_u64().expect("rect y") as u32,
                    rect["width"].as_u64().expect("rect width") as u32,
                    rect["height"].as_u64().expect("rect height") as u32,
                );
            }
            draft.add()?;
        }
        other => panic!("unknown fixture note kind: {other}"),
    }
    Ok(deck)
}

#[test]
fn all_cataloged_note_identity_fixtures_match_resolver_output() {
    for rel in note_identity_catalog_inputs() {
        let fixture = load_case(&format!("contracts/{rel}"));
        match fixture.expected["error_code"].as_str() {
            Some(error_code) => {
                let err = resolve_fixture_deck(&fixture)
                    .expect_err("error fixture must be rejected by resolver");
                assert!(
                    err.to_string().contains(error_code),
                    "{rel} expected {error_code}, got {err}"
                );
            }
            None => {
                let deck = resolve_fixture_deck(&fixture)
                    .unwrap_or_else(|err| panic!("{rel} should resolve: {err}"));
                let snapshot = deck.notes()[0]
                    .resolved_identity()
                    .expect("resolved identity snapshot");
                assert_eq!(
                    snapshot.canonical_payload.as_deref(),
                    fixture.expected["canonical_payload"].as_str(),
                    "{rel} canonical payload"
                );
                assert_eq!(
                    snapshot.stable_id, fixture.expected["stable_id"],
                    "{rel} stable id"
                );
                assert_eq!(
                    snapshot.recipe_id.as_deref(),
                    Some(fixture.recipe_id.as_str()),
                    "{rel} recipe id"
                );
            }
        }
    }
}

#[test]
fn io_order_insensitive_contract_case_matches_expected_output() {
    assert_io_contract_case_matches_expected_output(
        "contracts/fixtures/note-identity/io-order-insensitive.case.json",
    );
}

#[test]
fn io_translation_different_contract_case_matches_expected_output() {
    assert_io_contract_case_matches_expected_output(
        "contracts/fixtures/note-identity/io-translation-different.case.json",
    );
}

#[test]
fn io_mask_order_does_not_change_identity() {
    let image_path = io_fixture_image_path();

    let mut deck_a = Deck::new("Anatomy");
    let image_a = deck_a
        .media()
        .add(MediaSource::from_file(&image_path))
        .expect("image a");
    deck_a
        .image_occlusion()
        .note(image_a)
        .mode(IoMode::HideAllGuessOne)
        .rect(10, 20, 30, 40)
        .rect(100, 40, 30, 40)
        .add()
        .expect("io a");

    let mut deck_b = Deck::new("Anatomy");
    let image_b = deck_b
        .media()
        .add(MediaSource::from_file(&image_path))
        .expect("image b");
    deck_b
        .image_occlusion()
        .note(image_b)
        .mode(IoMode::HideAllGuessOne)
        .rect(100, 40, 30, 40)
        .rect(10, 20, 30, 40)
        .add()
        .expect("io b");

    assert_eq!(deck_a.notes()[0].id(), deck_b.notes()[0].id());
}

#[test]
fn io_translation_changes_identity() {
    let image_path = io_fixture_image_path();

    let mut deck_a = Deck::new("Anatomy");
    let image_a = deck_a
        .media()
        .add(MediaSource::from_file(&image_path))
        .expect("image a");
    deck_a
        .image_occlusion()
        .note(image_a)
        .mode(IoMode::HideAllGuessOne)
        .rect(10, 20, 30, 40)
        .add()
        .expect("io a");

    let mut deck_b = Deck::new("Anatomy");
    let image_b = deck_b
        .media()
        .add(MediaSource::from_file(&image_path))
        .expect("image b");
    deck_b
        .image_occlusion()
        .note(image_b)
        .mode(IoMode::HideAllGuessOne)
        .rect(11, 20, 30, 40)
        .add()
        .expect("io b");

    assert_ne!(deck_a.notes()[0].id(), deck_b.notes()[0].id());
}

#[test]
fn invalid_raster_without_dimensions_fails_identity_resolution() {
    let mut deck = Deck::new("Anatomy");
    let image = deck
        .media()
        .add(MediaSource::from_bytes("broken.png", vec![1, 2, 3]))
        .expect("register media");

    let err = deck
        .image_occlusion()
        .note(image)
        .mode(IoMode::HideAllGuessOne)
        .rect(1, 2, 3, 4)
        .add()
        .expect_err("missing dimensions must fail");
    assert!(err.to_string().contains("AFID.IO_IMAGE_DIMENSIONS_MISSING"));
}

#[test]
fn io_zero_sized_rect_fails_identity_resolution() {
    let image_path = io_fixture_image_path();
    let mut deck = Deck::new("Anatomy");
    let image = deck
        .media()
        .add(MediaSource::from_file(&image_path))
        .expect("image");

    let err = deck
        .image_occlusion()
        .note(image)
        .mode(IoMode::HideAllGuessOne)
        .rect(10, 20, 0, 40)
        .add()
        .expect_err("zero-sized rect must fail");
    assert!(err.to_string().contains("AFID.IO_RECT_EMPTY"));
}

#[test]
fn io_out_of_bounds_rect_fails_identity_resolution() {
    let image_path = io_fixture_image_path();
    let mut deck = Deck::new("Anatomy");
    let image = deck
        .media()
        .add(MediaSource::from_file(&image_path))
        .expect("image");

    let err = deck
        .image_occlusion()
        .note(image)
        .mode(IoMode::HideAllGuessOne)
        .rect(200, 20, 30, 40)
        .add()
        .expect_err("out-of-bounds rect must fail");
    assert!(err.to_string().contains("AFID.IO_RECT_OUT_OF_BOUNDS"));
}

#[test]
fn io_duplicate_rect_fails_identity_resolution() {
    let image_path = io_fixture_image_path();
    let mut deck = Deck::new("Anatomy");
    let image = deck
        .media()
        .add(MediaSource::from_file(&image_path))
        .expect("image");

    let err = deck
        .image_occlusion()
        .note(image)
        .mode(IoMode::HideAllGuessOne)
        .rect(10, 20, 30, 40)
        .rect(10, 20, 30, 40)
        .add()
        .expect_err("duplicate rect must fail");
    assert!(err.to_string().contains("AFID.IO_RECT_DUPLICATE"));
}

#[test]
fn io_deserialized_png_media_without_raster_metadata_backfills_dimensions() {
    let mut deck = Deck::new("Anatomy");
    deck.media()
        .add(MediaSource::from_file(io_fixture_image_path()))
        .expect("register image");

    let mut deck_json = serde_json::to_value(&deck).expect("serialize deck");
    deck_json["media"]["occlusion-heart.png"]
        .as_object_mut()
        .expect("serialized media object")
        .remove("raster_image");
    let mut restored: Deck = serde_json::from_value(deck_json).expect("deserialize deck");

    let image = restored
        .media()
        .get("occlusion-heart.png")
        .expect("restored media ref");
    restored
        .image_occlusion()
        .note(image)
        .mode(IoMode::HideAllGuessOne)
        .rect(10, 20, 30, 40)
        .add()
        .expect("io identity should use backfilled dimensions");

    assert!(restored.notes()[0].id().starts_with("afid:v1:"));
}

#[test]
fn io_rect_touching_image_right_and_bottom_bounds_is_accepted() {
    let mut deck = Deck::new("Anatomy");
    let image = deck
        .media()
        .add(MediaSource::from_file(io_fixture_image_path()))
        .expect("register image");

    deck.image_occlusion()
        .note(image)
        .mode(IoMode::HideAllGuessOne)
        .rect(198, 46, 30, 40)
        .add()
        .expect("rect touching right and bottom edge is in bounds");

    assert!(deck.notes()[0].id().starts_with("afid:v1:"));
}

#[test]
fn inferred_basic_note_uses_afid_instead_of_generated_id() {
    let mut deck = Deck::new("Spanish");
    deck.add(BasicNote::new("hola", "hello"))
        .expect("add inferred note");

    assert!(deck.notes()[0].id().starts_with("afid:v1:"));
}

#[test]
fn new_default_note_no_longer_uses_generated_id() {
    let mut deck = Deck::new("Spanish");
    deck.add(BasicNote::new("hola", "hello"))
        .expect("add default note");

    assert!(deck.notes()[0].id().starts_with("afid:v1:"));
    assert!(!deck.notes()[0].id().starts_with("generated:"));
}

#[test]
fn explicit_generated_prefix_is_preserved_as_explicit_stable_id() {
    let mut deck = Deck::new("Spanish");
    deck.add(BasicNote::new("hola", "hello").stable_id("generated:Spanish:1"))
        .expect("add explicit generated-prefixed id");

    let snapshot = deck.notes()[0]
        .resolved_identity()
        .expect("resolved identity snapshot");
    assert_eq!(deck.notes()[0].id(), "generated:Spanish:1");
    assert_eq!(snapshot.stable_id, "generated:Spanish:1");
    assert_eq!(snapshot.provenance, IdentityProvenance::ExplicitStableId);
    assert!(snapshot.canonical_payload.is_none());
}

#[test]
fn cross_notetype_same_visible_text_produces_different_afids() {
    let mut basic_deck = Deck::new("Shared Text");
    basic_deck
        .add(BasicNote::new("Paris", "France"))
        .expect("add basic note");

    let mut cloze_deck = Deck::new("Shared Text");
    cloze_deck
        .add(ClozeNote::new("{{c1::Paris}}"))
        .expect("add cloze note");

    assert_ne!(basic_deck.notes()[0].id(), cloze_deck.notes()[0].id());
}

#[test]
fn inferred_duplicate_payload_is_error() {
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

#[test]
fn cloze_hint_change_does_not_change_identity() {
    let mut deck_a = Deck::new("Geo");
    deck_a
        .add(ClozeNote::new(
            "Capital of {{c1::France::country}} is {{c2::Paris::city}}",
        ))
        .expect("deck a");

    let mut deck_b = Deck::new("Geo");
    deck_b
        .add(ClozeNote::new(
            "Capital of {{c1::France::nation}} is {{c2::Paris::place}}",
        ))
        .expect("deck b");

    assert_eq!(deck_a.notes()[0].id(), deck_b.notes()[0].id());
}

#[test]
fn cloze_boundary_whitespace_changes_identity() {
    let mut deck_a = Deck::new("Geo");
    deck_a.add(ClozeNote::new("A {{c1::B}} C")).expect("deck a");

    let mut deck_b = Deck::new("Geo");
    deck_b.add(ClozeNote::new("A{{c1::B}}C")).expect("deck b");

    assert_ne!(deck_a.notes()[0].id(), deck_b.notes()[0].id());
}

#[test]
fn literal_cloze_marker_positions_do_not_collide_with_deletion_positions() {
    let mut deck_a = Deck::new("Geo");
    deck_a
        .add(ClozeNote::new("A {{c1::B}} [[CLOZE]] C"))
        .expect("deck a");

    let mut deck_b = Deck::new("Geo");
    deck_b
        .add(ClozeNote::new("A [[CLOZE]] {{c1::B}} C"))
        .expect("deck b");

    let snapshot_a = deck_a.notes()[0]
        .resolved_identity()
        .expect("deck a resolved identity");
    let snapshot_b = deck_b.notes()[0]
        .resolved_identity()
        .expect("deck b resolved identity");

    assert_ne!(snapshot_a.stable_id, snapshot_b.stable_id);
    assert_ne!(snapshot_a.canonical_payload, snapshot_b.canonical_payload);
}

#[test]
fn duplicate_inferred_cloze_identity_payload_fails_at_add_time() {
    let mut deck = Deck::new("Geo");
    deck.add(ClozeNote::new("A {{c1::B}} C"))
        .expect("add first inferred cloze");

    let err = deck
        .add(ClozeNote::new("A {{c1::B::ignored hint}} C"))
        .expect_err("duplicate inferred cloze payload should fail");

    assert!(err.to_string().contains("AFID.IDENTITY_DUPLICATE_PAYLOAD"));
}

#[test]
fn malformed_cloze_reports_afid_error() {
    let mut deck = Deck::new("Geo");
    let err = deck
        .add(ClozeNote::new("Capital of {{c1::France is Paris"))
        .expect_err("malformed cloze must fail");

    assert!(err.to_string().contains("AFID.CLOZE_MALFORMED"));
}

#[test]
fn nested_cloze_reports_explicit_unsupported_error() {
    let mut deck = Deck::new("Geo");
    let err = deck
        .add(ClozeNote::new("{{c1::outer {{c2::inner}} body}}"))
        .expect_err("nested cloze must fail explicitly");

    assert!(err.to_string().contains("AFID.CLOZE_NESTED_UNSUPPORTED"));
}

#[test]
fn literal_c_like_braces_are_not_treated_as_malformed_cloze() {
    let mut deck = Deck::new("Geo");
    deck.add(ClozeNote::new("literal {{cat}} before {{c1::Paris}}"))
        .expect("literal c-like braces plus one valid cloze");
}

#[test]
fn overlapping_literal_c_prefix_does_not_hide_valid_cloze_start() {
    let mut deck = Deck::new("Geo");
    deck.add(ClozeNote::new("prefix {{c{{c1::Paris}} suffix"))
        .expect("literal c prefix plus overlapping valid cloze");

    assert!(deck.notes()[0].id().starts_with("afid:v1:"));
}

#[test]
fn cloze_ordinal_zero_reports_invalid_ordinal() {
    let mut deck = Deck::new("Geo");
    let err = deck
        .add(ClozeNote::new("{{c0::Paris}}"))
        .expect_err("ordinal zero must fail");

    assert!(err.to_string().contains("AFID.CLOZE_ORD_INVALID"));
}

#[test]
fn repeated_cloze_ordinals_are_allowed_and_slot_ordered() {
    let mut deck = Deck::new("Geo");
    deck.add(ClozeNote::new("{{c1::Paris}} and {{c1::Lyon}}"))
        .expect("same ordinal can produce multiple deletions");

    let snapshot = deck.notes()[0]
        .resolved_identity()
        .expect("resolved identity snapshot");
    assert!(snapshot.canonical_payload.as_deref().is_some_and(|payload| {
        payload.contains(
            "\"deletions\":[{\"body\":\"Paris\",\"ord\":1,\"slot\":0},{\"body\":\"Lyon\",\"ord\":1,\"slot\":1}]",
        )
    }));
}

#[test]
fn empty_cloze_body_reports_malformed() {
    let mut deck = Deck::new("Geo");
    let err = deck
        .add(ClozeNote::new("{{c1::}}"))
        .expect_err("empty cloze body must fail");

    assert!(err.to_string().contains("AFID.CLOZE_MALFORMED"));
}

#[test]
fn unicode_and_newline_normalization_are_stable() {
    let mut deck_a = Deck::new("Geo");
    deck_a
        .add(ClozeNote::new("{{c1::Cafe\u{301}\r\nParis}}"))
        .expect("decomposed");

    let mut deck_b = Deck::new("Geo");
    deck_b
        .add(ClozeNote::new("{{c1::Café\nParis}}"))
        .expect("composed");

    assert_eq!(deck_a.notes()[0].id(), deck_b.notes()[0].id());
}
