use anki_forge::{Deck, ValidationCode};

#[test]
fn add_basic_generates_non_empty_id_and_validate_report_warns() {
    let mut deck = Deck::new("Spanish");
    deck.add_basic("hola", "hello").expect("add basic note");

    assert!(deck.notes()[0].id().starts_with("generated:"));

    let report = deck.validate_report().expect("validation report");
    assert!(report
        .diagnostics()
        .iter()
        .any(|item| item.code == ValidationCode::MissingStableId));
}

#[test]
fn blank_explicit_stable_id_fails_at_add_time() {
    let mut deck = Deck::new("Spanish");

    let err = deck
        .basic()
        .note("hola", "hello")
        .stable_id("   ")
        .add()
        .expect_err("blank stable id must fail");

    assert!(err.to_string().contains("stable_id"));
}
