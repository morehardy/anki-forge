use anki_forge::prelude::*;
use anki_forge::writer::inspect_apkg;
use anki_forge::Deck;

#[test]
fn deck_basic_write_apkg_smoke() {
    let root = tempfile::tempdir().expect("tempdir");
    let apkg = root.path().join("deck-basic.apkg");
    let mut deck = Deck::builder("Spanish").stable_id("spanish-smoke").build();

    deck.basic()
        .note("hola", "hello")
        .stable_id("es:hola")
        .add()
        .expect("add basic note");

    let report = deck.write_apkg(&apkg).expect("write apkg");
    report.ensure_success().expect("successful report");
    assert_eq!(report.counts.notes, 1);
    assert_eq!(report.counts.cards, 1);
    assert!(apkg.is_file());

    let inspected = inspect_apkg(&apkg).expect("inspect apkg");
    assert_eq!(inspected.source_kind, "apkg");
    assert_eq!(inspected.observation_status, "complete");
    assert!(inspected.observations.metadata.iter().any(|value| {
        value["selector"] == "counts" && value["note_count"] == 1 && value["card_count"] == 1
    }));
}

#[test]
fn project_stock_write_apkg_smoke() {
    let root = tempfile::tempdir().expect("tempdir");
    let apkg = root.path().join("project-stock.apkg");
    let mut project = Project::new("Stock Smoke")
        .stable_id("stock-smoke")
        .default_deck("Stock Smoke");

    project
        .add_note(Note::basic("front", "back").stable_id("stock:basic"))
        .expect("add basic note");
    project
        .add_note(
            Note::cloze("A {{c1::cloze}} fact")
                .stable_id("stock:cloze")
                .text("Extra", "extra"),
        )
        .expect("add cloze note");

    let report = project.write_apkg(&apkg).expect("write apkg");
    report.ensure_success().expect("successful report");
    assert_eq!(report.counts.notes, 2);
    assert_eq!(report.counts.cards, 2);

    let inspected = inspect_apkg(&apkg).expect("inspect apkg");
    assert_eq!(inspected.observation_status, "complete");
    assert!(inspected.observations.metadata.iter().any(|value| {
        value["selector"] == "counts" && value["note_count"] == 2 && value["card_count"] == 2
    }));
}

#[test]
fn deck_to_apkg_bytes_smoke() {
    let root = tempfile::tempdir().expect("tempdir");
    let apkg = root.path().join("bytes.apkg");
    let mut deck = Deck::builder("Bytes Smoke")
        .stable_id("bytes-smoke")
        .build();

    deck.basic()
        .note("front", "back")
        .stable_id("bytes:basic")
        .add()
        .expect("add basic note");

    let bytes = deck.to_apkg_bytes().expect("apkg bytes");
    assert!(!bytes.is_empty());
    std::fs::write(&apkg, bytes).expect("write bytes for inspection");

    let inspected = inspect_apkg(&apkg).expect("inspect apkg");
    assert_eq!(inspected.observation_status, "complete");
    assert!(inspected.observations.metadata.iter().any(|value| {
        value["selector"] == "counts" && value["note_count"] == 1 && value["card_count"] == 1
    }));
}
