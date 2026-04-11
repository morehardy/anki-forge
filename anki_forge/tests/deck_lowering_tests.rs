use anki_forge::product::ProductNote;
use anki_forge::{Deck, IoMode, MediaSource};

#[test]
fn deck_lowers_notes_in_original_mixed_order() {
    let mut deck = Deck::builder("Mixed").stable_id("mixed-v1").build();

    let heart = deck
        .media()
        .add(MediaSource::from_bytes("heart.png", vec![0x89, 0x50]))
        .expect("register media");

    deck.cloze()
        .note("A {{c1::cloze}} card")
        .stable_id("cloze-1")
        .extra("extra")
        .add()
        .expect("add cloze");
    deck.basic()
        .note("front", "back")
        .stable_id("basic-1")
        .tags(["demo"])
        .add()
        .expect("add basic");
    deck.image_occlusion()
        .note(heart)
        .mode(IoMode::HideAllGuessOne)
        .rect(10, 20, 80, 40)
        .header("Heart")
        .back_extra("Identify the chamber")
        .comments("Left ventricle")
        .stable_id("io-1")
        .add()
        .expect("add io");

    let product = deck.clone().into_product_document().expect("product bridge");
    let lowered = deck.lower_authoring().expect("authoring lowering");

    assert!(matches!(&product.notes()[0], ProductNote::Cloze(_)));
    assert!(matches!(&product.notes()[1], ProductNote::Basic(_)));
    assert!(matches!(&product.notes()[2], ProductNote::ImageOcclusion(_)));

    assert_eq!(lowered.notes[0].id, "cloze-1");
    assert_eq!(lowered.notes[1].id, "basic-1");
    assert_eq!(lowered.notes[2].id, "io-1");
    assert_eq!(lowered.notes[1].tags, vec!["demo"]);
    assert_eq!(lowered.media.len(), 1);
}
