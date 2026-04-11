use anki_forge::{Deck, IoMode, MediaSource};

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

    let heart = deck.media().add(MediaSource::from_bytes(
        "heart.png",
        std::fs::read("heart.png")?,
    ))?;

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
