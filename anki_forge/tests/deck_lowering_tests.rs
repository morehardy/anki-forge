use anki_forge::product::ProductNote;
use anki_forge::{AuthoringMediaSource, Deck, IoMode, MediaSource};

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

    let product = deck
        .clone()
        .into_product_document()
        .expect("product bridge");
    let lowered = deck.lower_authoring().expect("authoring lowering");

    assert!(matches!(&product.notes()[0], ProductNote::Cloze(_)));
    assert!(matches!(&product.notes()[1], ProductNote::Basic(_)));
    assert!(matches!(
        &product.notes()[2],
        ProductNote::ImageOcclusion(_)
    ));

    assert_eq!(lowered.notes[0].id, "cloze-1");
    assert_eq!(lowered.notes[1].id, "basic-1");
    assert_eq!(lowered.notes[2].id, "io-1");
    assert_eq!(lowered.notes[1].tags, vec!["demo"]);
    assert_eq!(lowered.media.len(), 1);
}

#[test]
fn public_lower_authoring_inlines_file_media_without_hidden_base_dir() {
    let root = unique_test_dir("deck-lowering-file-media");
    let source_path = root.join("hello.txt");
    std::fs::write(&source_path, b"hello").expect("write source media");

    let mut deck = Deck::builder("Media Deck").stable_id("media-deck").build();
    deck.media()
        .add(MediaSource::from_file(&source_path))
        .expect("register file media");

    let lowered = deck.lower_authoring().expect("lower authoring");
    let media = lowered.media.first().expect("lowered media");

    assert_eq!(media.desired_filename, "hello.txt");
    assert!(matches!(
        &media.source,
        AuthoringMediaSource::InlineBytes { data_base64 } if data_base64 == "aGVsbG8="
    ));
    let serialized = serde_json::to_string(&lowered).expect("serialize lowered authoring");
    assert!(serialized.contains("data_base64"));
    assert!(!serialized.contains("\"path\""));
}

fn unique_test_dir(label: &str) -> std::path::PathBuf {
    let mut dir = std::env::temp_dir();
    let nonce = format!(
        "anki-forge-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    );
    dir.push(nonce);
    std::fs::create_dir_all(&dir).expect("create test dir");
    dir
}
