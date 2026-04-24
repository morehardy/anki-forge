use anki_forge::{
    BasicIdentityField, BasicIdentityOverride, BasicIdentitySelection, BasicNote, ClozeNote, Deck,
    DeckNote, IoMode, MediaSource, Package,
};
use serde_json::json;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn io_fixture_image_path() -> PathBuf {
    repo_root().join(
        "contracts/fixtures/phase3/manual-desktop-v1/S03_io_minimal/assets/occlusion-heart.png",
    )
}

#[test]
fn deck_add_preserves_mixed_note_order() {
    let mut deck = Deck::builder("Mixed").stable_id("mixed-v1").build();

    deck.add(BasicNote::new("front 1", "back 1").stable_id("basic-1"))
        .expect("add first basic");
    deck.add(ClozeNote::new("A {{c1::cloze}} card").stable_id("cloze-1"))
        .expect("add cloze");
    deck.add(BasicNote::new("front 2", "back 2").stable_id("basic-2"))
        .expect("add second basic");

    assert_eq!(deck.notes().len(), 3);
    assert!(matches!(&deck.notes()[0], DeckNote::Basic(_)));
    assert!(matches!(&deck.notes()[1], DeckNote::Cloze(_)));
    assert!(matches!(&deck.notes()[2], DeckNote::Basic(_)));
    assert_eq!(deck.stable_id(), Some("mixed-v1"));
}

#[test]
fn package_single_can_override_package_stable_id_without_changing_root_deck() {
    let deck = Deck::builder("Mixed").stable_id("mixed-v1").build();

    let package = Package::single(deck).with_stable_id("package-v1");

    assert_eq!(package.stable_id(), Some("package-v1"));
    assert_eq!(package.root_deck().stable_id(), Some("mixed-v1"));
}

#[test]
fn deck_add_infers_image_occlusion_afid_even_when_generated_like_id_exists() {
    let mut deck = Deck::builder("Mixed").build();

    deck.add(BasicNote::new("front 1", "back 1").stable_id("generated:Mixed:1"))
        .expect("add explicit stable id");
    let image = deck
        .media()
        .add(MediaSource::from_file(io_fixture_image_path()))
        .expect("register image");
    deck.image_occlusion()
        .note(image)
        .mode(IoMode::HideAllGuessOne)
        .rect(10, 20, 30, 40)
        .add()
        .expect("add inferred image occlusion note");

    assert_eq!(deck.notes().len(), 2);
    assert_eq!(deck.notes()[0].id(), "generated:Mixed:1");
    assert!(deck.notes()[1].id().starts_with("afid:v1:"));
}

#[test]
fn deck_add_rejects_reserved_afid_explicit_stable_id() {
    let mut deck = Deck::builder("Reserved").build();

    let error = deck
        .add(
            BasicNote::new("front", "back").stable_id(
                "afid:v1:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            ),
        )
        .expect_err("reserved AFID explicit stable id should be rejected");

    assert!(error
        .to_string()
        .contains("AFID.IDENTITY_SNAPSHOT_INCOMPLETE"));
    assert!(deck.notes().is_empty());
}

#[test]
fn deck_add_rejects_blank_stable_id() {
    let mut deck = Deck::builder("Blank").build();

    let error = deck
        .add(BasicNote::new("front", "back").stable_id("   "))
        .expect_err("blank stable id should be rejected");

    assert!(error.to_string().contains("stable_id must not be blank"));
    assert!(deck.notes().is_empty());
}

#[test]
fn deck_add_rejects_duplicate_stable_id() {
    let mut deck = Deck::builder("Duplicates").build();

    deck.add(BasicNote::new("front 1", "back 1").stable_id("basic-1"))
        .expect("add first note");

    let error = deck
        .add(ClozeNote::new("A {{c1::cloze}} card").stable_id("basic-1"))
        .expect_err("duplicate stable id should be rejected");

    assert!(error
        .to_string()
        .contains("AFID.STABLE_ID_DUPLICATE: basic-1"));
    assert_eq!(deck.notes().len(), 1);
}

#[test]
fn deck_builder_treats_blank_stable_id_as_none() {
    let deck = Deck::builder("Blank").stable_id("   ").build();

    assert_eq!(deck.stable_id(), None);
}

#[test]
fn package_with_stable_id_treats_blank_stable_id_as_none() {
    let deck = Deck::builder("Blank").stable_id("deck-v1").build();
    let package = Package::single(deck).with_stable_id("   ");

    assert_eq!(package.stable_id(), None);
    assert_eq!(package.root_deck().stable_id(), Some("deck-v1"));
}

#[test]
fn deck_builder_stores_canonicalized_typed_identity_fields() {
    let deck = Deck::builder("Spanish")
        .basic_identity(
            BasicIdentitySelection::new([
                BasicIdentityField::Back,
                BasicIdentityField::Front,
                BasicIdentityField::Back,
            ])
            .expect("selection"),
        )
        .build();

    let policy = deck.identity_policy();
    assert_eq!(
        policy.basic.as_ref().expect("basic policy").as_slice(),
        &[BasicIdentityField::Front, BasicIdentityField::Back]
    );
}

#[test]
fn note_level_identity_override_is_constructed_atomically() {
    let override_cfg =
        BasicIdentityOverride::new([BasicIdentityField::Front], "homonym-disambiguation")
            .expect("override");

    let note = BasicNote::new("hola", "hello").identity_override(override_cfg.clone());
    assert_eq!(note.identity_override_config(), Some(&override_cfg));
}

#[test]
fn basic_lane_sets_identity_override() {
    let override_cfg =
        BasicIdentityOverride::new([BasicIdentityField::Back], "translation-disambiguation")
            .expect("override");
    let mut deck = Deck::builder("Spanish").build();

    deck.basic()
        .note("hola", "hello")
        .identity_override(override_cfg.clone())
        .add()
        .expect("add note");

    match &deck.notes()[0] {
        DeckNote::Basic(note) => {
            assert_eq!(note.identity_override_config(), Some(&override_cfg));
        }
        other => panic!("expected basic note, got {other:?}"),
    }
}

#[test]
fn typed_identity_override_uses_stable_wire_names() {
    let override_cfg = BasicIdentityOverride::new(
        [BasicIdentityField::Back, BasicIdentityField::Front],
        "sense-disambiguation",
    )
    .expect("override");

    let json_value = serde_json::to_value(&override_cfg).expect("serialize override");
    assert_eq!(
        json_value,
        json!({
            "fields": ["front", "back"],
            "reason_code": "sense-disambiguation"
        })
    );
}

#[test]
fn empty_basic_identity_selection_returns_coded_error() {
    let error =
        BasicIdentitySelection::new(std::iter::empty::<BasicIdentityField>()).expect_err("error");

    assert!(error.to_string().contains("AFID.IDENTITY_FIELDS_EMPTY"));
}

#[test]
fn blank_basic_identity_override_reason_returns_coded_error() {
    let error = BasicIdentityOverride::new([BasicIdentityField::Front], "   ").expect_err("error");

    assert!(error
        .to_string()
        .contains("AFID.NOTE_LEVEL_IDENTITY_OVERRIDE_REASON_REQUIRED"));
}

#[test]
fn deserializing_empty_basic_identity_selection_returns_coded_error() {
    let error = serde_json::from_value::<BasicIdentitySelection>(json!({ "fields": [] }))
        .expect_err("empty fields should be rejected");

    assert!(error.to_string().contains("AFID.IDENTITY_FIELDS_EMPTY"));
}

#[test]
fn deserializing_basic_identity_selection_canonicalizes_fields() {
    let selection = serde_json::from_value::<BasicIdentitySelection>(json!({
        "fields": ["back", "front", "back"]
    }))
    .expect("deserialize selection");

    assert_eq!(
        selection.as_slice(),
        &[BasicIdentityField::Front, BasicIdentityField::Back]
    );
}

#[test]
fn deserializing_blank_basic_identity_override_reason_returns_coded_error() {
    let error = serde_json::from_value::<BasicIdentityOverride>(json!({
        "fields": ["front"],
        "reason_code": "   "
    }))
    .expect_err("blank reason should be rejected");

    assert!(error
        .to_string()
        .contains("AFID.NOTE_LEVEL_IDENTITY_OVERRIDE_REASON_REQUIRED"));
}

#[test]
fn default_deck_identity_policy_is_not_serialized() {
    let deck = Deck::builder("Spanish").build();

    let json_value = serde_json::to_value(&deck).expect("serialize deck");

    assert!(json_value.get("identity_policy").is_none());
}
