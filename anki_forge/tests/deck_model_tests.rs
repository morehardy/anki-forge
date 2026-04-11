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
