use std::path::{Path, PathBuf};

use anki_forge::build::{BuildError, BuildReport};
use anki_forge::prelude::*;
use anki_forge::writer::{inspect_apkg, InspectReport};
use anki_forge::Deck;
use serde_json::Value;

fn scenario_dir() -> PathBuf {
    let value = std::env::var("ANKI_FORGE_CAPABILITY_ARTIFACT_DIR").expect(
        "set ANKI_FORGE_CAPABILITY_ARTIFACT_DIR by running scripts/run_rust_user_capabilities.sh",
    );
    assert!(
        !value.trim().is_empty(),
        "ANKI_FORGE_CAPABILITY_ARTIFACT_DIR must not be empty; run scripts/run_rust_user_capabilities.sh"
    );
    let path = PathBuf::from(value);
    std::fs::create_dir_all(&path).expect("create scenario artifact dir");
    path
}

#[allow(dead_code)]
fn normalize_report(result: Result<BuildReport, BuildError>) -> BuildReport {
    match result {
        Ok(report) => report,
        Err(error) => *error.report,
    }
}

fn inspect_complete(path: &Path) -> InspectReport {
    let report = inspect_apkg(path).expect("inspect generated APKG");
    assert_eq!(report.source_kind, "apkg");
    assert_eq!(report.observation_status, "complete");
    assert!(
        report.missing_domains.is_empty(),
        "inspect missing domains: {:?}",
        report.missing_domains
    );
    assert!(
        report.degradation_reasons.is_empty(),
        "inspect degradation reasons: {:?}",
        report.degradation_reasons
    );
    report
}

fn counts(report: &InspectReport) -> &Value {
    report
        .observations
        .metadata
        .iter()
        .find(|value| value["selector"] == "counts")
        .expect("counts metadata observation")
}

fn has_observation(values: &[Value], key: &str, expected: &str) -> bool {
    values
        .iter()
        .any(|value| value[key].as_str() == Some(expected))
}

const PNG_1X1: &[u8] = &[
    137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1, 8, 6, 0,
    0, 0, 31, 21, 196, 137, 0, 0, 0, 12, 73, 68, 65, 84, 120, 156, 99, 248, 15, 4, 0, 9, 251, 3,
    253, 167, 102, 129, 94, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
];

const MP3_BYTES: &[u8] = b"fake-mp3-bytes-for-capability-test";

fn io_fixture_image_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../contracts/fixtures/phase3/manual-desktop-v1/S03_io_minimal/assets/occlusion-heart.png",
    )
}

fn has_selector(values: &[Value], expected: &str) -> bool {
    values
        .iter()
        .any(|value| value["selector"].as_str() == Some(expected))
}

fn vocab_notetype() -> NoteType {
    NoteType::custom("jp-vocab")
        .field(Field::new("Expression").key("expr"))
        .field(Field::new("Meaning").key("meaning"))
        .template(
            Template::new("Recognition")
                .key("recognition")
                .front("{{Expression}}")
                .back("{{Meaning}}")
                .generate_when(GenerationRule::all(["expr"])),
        )
}

#[ignore]
#[test]
fn deck_basic_apkg() {
    let root = scenario_dir();
    let apkg = root.join("package.apkg");
    let mut deck = Deck::builder("Spanish").stable_id("cap-deck-basic").build();

    deck.basic()
        .note("hola", "hello")
        .stable_id("es:hola")
        .add()
        .expect("add basic note");

    let report = deck.write_apkg(&apkg).expect("write apkg");
    report.ensure_success().expect("successful report");
    assert_eq!(report.counts.notes, 1);
    assert_eq!(report.counts.cards, 1);
    assert_eq!(report.counts.media, 0);
    assert!(apkg.is_file());

    let inspected = inspect_complete(&apkg);
    let counts = counts(&inspected);
    assert_eq!(counts["note_count"], 1);
    assert_eq!(counts["card_count"], 1);
    assert!(has_observation(
        &inspected.observations.notetypes,
        "name",
        "Basic"
    ));
}

#[ignore]
#[test]
fn deck_cloze_apkg() {
    let root = scenario_dir();
    let apkg = root.join("package.apkg");
    let mut deck = Deck::builder("Cloze").stable_id("cap-deck-cloze").build();
    deck.cloze()
        .note("A {{c1::cloze}} fact")
        .stable_id("cloze:one")
        .add()
        .expect("add cloze");

    let report = deck.write_apkg(&apkg).expect("write apkg");
    report.ensure_success().expect("successful report");
    assert_eq!(report.counts.notes, 1);
    assert_eq!(report.counts.cards, 1);
    let inspected = inspect_complete(&apkg);
    assert_eq!(counts(&inspected)["note_count"], 1);
    assert_eq!(counts(&inspected)["card_count"], 1);
    assert!(inspected
        .observations
        .notetypes
        .iter()
        .any(|value| value["name"]
            .as_str()
            .is_some_and(|name| name.contains("Cloze"))));
}

#[ignore]
#[test]
fn deck_image_occlusion_apkg() {
    let root = scenario_dir();
    let apkg = root.join("package.apkg");
    let mut deck = Deck::builder("IO").stable_id("cap-deck-io").build();
    let image = deck
        .media()
        .add(anki_forge::MediaSource::from_file(io_fixture_image_path()))
        .expect("image media");
    deck.image_occlusion()
        .note(image)
        .mode(anki_forge::IoMode::HideAllGuessOne)
        .rect(0, 0, 50, 50)
        .stable_id("io:one")
        .add()
        .expect("add image occlusion");

    let report = deck.write_apkg(&apkg).expect("write apkg");
    report.ensure_success().expect("successful report");
    assert_eq!(report.counts.notes, 1);
    assert_eq!(report.counts.cards, 1);
    assert_eq!(report.counts.media, 1);
    let inspected = inspect_complete(&apkg);
    assert_eq!(counts(&inspected)["card_count"], 1);
    assert!(has_selector(
        &inspected.observations.notetypes,
        "notetype[id='image_occlusion']"
    ));
}

#[ignore]
#[test]
fn deck_bytes_export() {
    let root = scenario_dir();
    let apkg = root.join("package.apkg");
    let mut deck = Deck::builder("Bytes").stable_id("cap-deck-bytes").build();
    deck.basic()
        .note("front", "back")
        .stable_id("bytes:one")
        .add()
        .expect("add basic");
    let bytes = deck.to_apkg_bytes().expect("apkg bytes");
    assert!(!bytes.is_empty());
    std::fs::write(&apkg, bytes).expect("write bytes");
    let inspected = inspect_complete(&apkg);
    assert_eq!(counts(&inspected)["note_count"], 1);
    assert_eq!(counts(&inspected)["card_count"], 1);
}

#[ignore]
#[test]
fn project_stock_notes_apkg() {
    let root = scenario_dir();
    let apkg = root.join("package.apkg");
    let mut project = Project::new("Stock")
        .stable_id("cap-project-stock")
        .default_deck("Stock");
    project
        .add_note(Note::basic("front", "back").stable_id("stock:basic"))
        .expect("add basic");
    project
        .add_note(Note::cloze("A {{c1::cloze}} fact").stable_id("stock:cloze"))
        .expect("add cloze");
    let report = project.write_apkg(&apkg).expect("write apkg");
    report.ensure_success().expect("successful report");
    assert_eq!(report.counts.notes, 2);
    assert_eq!(report.counts.cards, 2);
    let inspected = inspect_complete(&apkg);
    assert_eq!(counts(&inspected)["note_count"], 2);
    assert_eq!(counts(&inspected)["card_count"], 2);
}

#[ignore]
#[test]
fn project_custom_notetype_apkg() {
    let root = scenario_dir();
    let apkg = root.join("package.apkg");
    let mut project = Project::new("Custom")
        .stable_id("cap-project-custom")
        .default_deck("Custom");
    project
        .add_notetype(vocab_notetype())
        .expect("add notetype");
    project
        .add_note(
            Note::new("jp-vocab")
                .stable_id("jp:taberu")
                .text("expr", "taberu")
                .text("meaning", "to eat"),
        )
        .expect("add note");
    let report = project.write_apkg(&apkg).expect("write apkg");
    report.ensure_success().expect("successful report");
    assert_eq!(report.counts.notes, 1);
    assert_eq!(report.counts.cards, 1);
    let inspected = inspect_complete(&apkg);
    assert!(has_observation(
        &inspected.observations.notetypes,
        "id",
        "jp-vocab"
    ));
}

#[ignore]
#[test]
fn project_media_references_apkg() {
    let root = scenario_dir();
    let apkg = root.join("package.apkg");
    let mut project = Project::new("Media")
        .stable_id("cap-project-media")
        .default_deck("Media");
    let audio = project
        .media_mut()
        .add_bytes("raw-audio.bin", MP3_BYTES.to_vec())
        .expect("audio bytes")
        .export_as("voice.mp3")
        .expect("audio export");
    let image = project
        .media_mut()
        .add_bytes("raw-image.bin", PNG_1X1.to_vec())
        .expect("image bytes")
        .export_as("chart.png")
        .expect("image export");
    let back = format!("{}{}", audio.sound().render(), image.image().render());
    project
        .add_note(
            Note::basic("media", "")
                .stable_id("media:one")
                .html("Back", back),
        )
        .expect("add media note");
    let report = project.write_apkg(&apkg).expect("write apkg");
    report.ensure_success().expect("successful report");
    assert_eq!(report.counts.notes, 1);
    assert_eq!(report.counts.cards, 1);
    assert_eq!(report.counts.media, 2);
    assert_eq!(report.media.objects, 2);
    assert_eq!(report.media.bindings, 2);
    assert_eq!(report.media.references, 2);
    assert_eq!(report.media.missing_references, 0);
    assert_eq!(report.media.unsafe_references, 0);
    assert_eq!(report.media.unused_bindings, 0);
    let inspected = inspect_complete(&apkg);
    assert!(inspected
        .observations
        .media
        .iter()
        .any(|value| value["filename"].as_str() == Some("voice.mp3")));
    assert!(inspected
        .observations
        .media
        .iter()
        .any(|value| value["filename"].as_str() == Some("chart.png")));
    let note = inspected
        .observations
        .references
        .iter()
        .find(|value| value["selector"].as_str() == Some("note[id='media:one']"))
        .expect("media note observation");
    let back = note["fields"]["Back"]
        .as_str()
        .expect("media note Back field");
    assert!(back.contains("[sound:voice.mp3]"));
    assert!(back.contains("src=\"chart.png\""));
}
