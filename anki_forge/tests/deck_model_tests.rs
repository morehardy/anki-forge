use anki_forge::{BasicNote, ClozeNote, Deck, DeckNote, Package};

#[test]
fn deck_add_preserves_mixed_note_order() {
    let mut deck = Deck::builder("Mixed")
        .stable_id("mixed-v1")
        .build();

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
    assert_eq!(deck.stable_id().as_deref(), Some("mixed-v1"));
}

#[test]
fn package_single_can_override_package_stable_id_without_changing_root_deck() {
    let deck = Deck::builder("Mixed")
        .stable_id("mixed-v1")
        .build();

    let package = Package::single(deck).with_stable_id("package-v1");

    assert_eq!(package.stable_id().as_deref(), Some("package-v1"));
    assert_eq!(package.root_deck().stable_id().as_deref(), Some("mixed-v1"));
}

#[test]
fn deck_add_generated_id_skips_existing_explicit_stable_id() {
    let mut deck = Deck::builder("Mixed").build();

    deck.add(BasicNote::new("front 1", "back 1").stable_id("generated:Mixed:1"))
        .expect("add explicit stable id");
    deck.add(BasicNote::new("front 2", "back 2"))
        .expect("add generated note");

    assert_eq!(deck.notes().len(), 2);
    assert_eq!(deck.notes()[0].id(), "generated:Mixed:1");
    assert_eq!(deck.notes()[1].id(), "generated:Mixed:2");
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

    assert!(error.to_string().contains("duplicate stable_id: basic-1"));
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
    assert_eq!(package.root_deck().stable_id().as_deref(), Some("deck-v1"));
}
