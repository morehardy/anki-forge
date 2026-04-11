use anki_forge::{Deck, IoMode, MediaSource};

const HEART_PNG: &[u8] = &[
    137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1, 8, 6, 0,
    0, 0, 31, 21, 196, 137, 0, 0, 0, 12, 73, 68, 65, 84, 120, 156, 99, 248, 15, 4, 0, 9, 251, 3,
    253, 167, 102, 129, 94, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
];

fn main() -> anyhow::Result<()> {
    let mut deck = Deck::builder("Spanish").stable_id("spanish-v1").build();

    deck.basic()
        .note("hola", "hello")
        .stable_id("es-hola")
        .tags(["vocab", "a1"])
        .add()?;

    deck.cloze()
        .note("La capital de Espana es {{c1::Madrid}}")
        .extra("Europe")
        .stable_id("geo-es-capital")
        .add()?;

    let heart = deck
        .media()
        .add(MediaSource::from_bytes("heart.png", HEART_PNG.to_vec()))?;

    deck.image_occlusion()
        .note(heart)
        .mode(IoMode::HideAllGuessOne)
        .rect(10, 20, 80, 40)
        .header("Heart")
        .back_extra("Identify the chamber")
        .comments("Left ventricle")
        .stable_id("anatomy-heart-1")
        .add()?;

    deck.write_apkg("spanish.apkg")?;
    Ok(())
}
