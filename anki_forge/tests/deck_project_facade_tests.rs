use std::path::PathBuf;

use anki_forge::prelude::*;
use anki_forge::{IoMode, MediaSource};

const PNG: &[u8] = &[
    137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1, 8, 6, 0,
    0, 0, 31, 21, 196, 137, 0, 0, 0, 12, 73, 68, 65, 84, 120, 156, 99, 248, 15, 4, 0, 9, 251, 3,
    253, 167, 102, 129, 94, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
];

#[test]
fn deck_build_matches_project_from_deck_for_stock_notes() {
    let root = unique_artifacts_dir("deck-project-stock");
    let mut deck = Deck::builder("Spanish").stable_id("spanish-v1").build();
    deck.basic()
        .note("hola", "hello")
        .stable_id("es-hola")
        .add()
        .expect("add basic");
    deck.cloze()
        .note("La capital de Espana es {{c1::Madrid}}")
        .stable_id("geo-es-capital")
        .add()
        .expect("add cloze");

    let deck_report = deck
        .build(BuildOptions::new().output(root.join("deck.apkg")))
        .expect("deck build");
    let project_report = Project::from(deck.clone())
        .build(BuildOptions::new().output(root.join("project.apkg")))
        .expect("project build");

    assert_eq!(deck_report.counts, project_report.counts);
    assert_eq!(
        deck_report.diagnostic_codes(),
        project_report.diagnostic_codes()
    );
    assert_eq!(
        deck_report
            .inspect
            .as_ref()
            .map(|summary| summary.observation_status.as_str()),
        project_report
            .inspect
            .as_ref()
            .map(|summary| summary.observation_status.as_str())
    );
}

#[test]
fn project_from_deck_preserves_existing_image_occlusion_support() {
    let root = unique_artifacts_dir("deck-project-io");
    let mut deck = Deck::builder("Anatomy").stable_id("anatomy-v1").build();
    let image = deck
        .media()
        .add(MediaSource::from_bytes("heart.png", PNG.to_vec()))
        .expect("add image");
    deck.image_occlusion()
        .note(image)
        .mode(IoMode::HideAllGuessOne)
        .rect(10, 20, 80, 40)
        .stable_id("heart-io-1")
        .add()
        .expect("add io");

    let report = Project::from(deck)
        .build(BuildOptions::new().output(root.join("io.apkg")))
        .expect("project from deck build");

    assert_eq!(report.counts.notes, 1);
    assert_eq!(report.counts.media, 1);
    assert!(report.counts.cards >= 1);
}

#[test]
fn project_from_deck_rejects_extra_project_authoring_state() {
    let root = unique_artifacts_dir("deck-project-extra-state");
    let mut deck = Deck::builder("Spanish").stable_id("spanish-v1").build();
    deck.basic()
        .note("hola", "hello")
        .stable_id("es-hola")
        .add()
        .expect("add deck note");
    let mut project = Project::from(deck);
    project
        .add_note(Note::basic("adios", "goodbye").stable_id("es-adios"))
        .expect("add project note");

    let err = project
        .build(BuildOptions::new().output(root.join("deck.apkg")))
        .expect_err("deck-backed project must not silently drop extra project notes");

    assert!(err
        .report
        .diagnostic_codes()
        .iter()
        .any(|code| code == "PROJECT.DECK_SOURCE_AUTHORING_STATE_UNSUPPORTED"));
}

fn unique_artifacts_dir(label: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "anki-forge-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("create temp artifacts dir");
    dir
}
