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
                .extra("extra"),
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
fn project_stock_multi_cloze_writes_one_card_per_distinct_ordinal() {
    let root = tempfile::tempdir().expect("tempdir");
    let apkg = root.path().join("project-multi-cloze.apkg");
    let mut project = Project::new("Multi Cloze")
        .stable_id("multi-cloze")
        .default_deck("Multi Cloze");

    project
        .add_note(
            Note::cloze("{{c1::Madrid}} is in {{c2::Spain}} and {{c1::Europe}}")
                .stable_id("stock:multi-cloze"),
        )
        .expect("add multi-cloze note");

    let report = project.write_apkg(&apkg).expect("write apkg");
    report.ensure_success().expect("successful report");
    assert_eq!(report.counts.notes, 1);
    assert_eq!(report.counts.cards, 2);

    let inspected = inspect_apkg(&apkg).expect("inspect apkg");
    let counts = inspected
        .observations
        .metadata
        .iter()
        .find(|value| value["selector"] == "counts")
        .expect("counts observation");
    assert_eq!(counts["card_count"], 2);

    let card_ords = inspected
        .observations
        .references
        .iter()
        .filter(|value| {
            value["selector"]
                .as_str()
                .is_some_and(|selector| selector.starts_with("card["))
        })
        .filter_map(|value| value["ord"].as_u64())
        .collect::<Vec<_>>();
    assert_eq!(card_ords, vec![0, 1]);
}

#[test]
fn project_many_templates_assigns_unique_card_ids_across_notes() {
    let root = tempfile::tempdir().expect("tempdir");
    let apkg = root.path().join("project-many-templates.apkg");
    let mut note_type = NoteType::custom("many-templates")
        .field(Field::new("Front").key("front").identity().sort())
        .identity(IdentityRecipe::fields(["front"]));
    for index in 0..12 {
        note_type = note_type.template(
            Template::new(format!("Card {}", index + 1))
                .key(format!("card-{}", index + 1))
                .front("{{Front}}")
                .back("{{Front}}"),
        );
    }

    let mut project = Project::new("Many Templates")
        .stable_id("many-templates")
        .default_deck("Many Templates");
    project.add_notetype(note_type).expect("add note type");
    for index in 0..2 {
        project
            .add_note(
                Note::new("many-templates")
                    .stable_id(format!("note-{index}"))
                    .text("front", format!("front {index}")),
            )
            .expect("add note");
    }

    let report = project.write_apkg(&apkg).expect("write apkg");
    report.ensure_success().expect("successful report");
    assert_eq!(report.counts.notes, 2);
    assert_eq!(report.counts.cards, 24);

    let inspected = inspect_apkg(&apkg).expect("inspect apkg");
    let counts = inspected
        .observations
        .metadata
        .iter()
        .find(|value| value["selector"] == "counts")
        .expect("counts observation");
    assert_eq!(counts["card_count"], 24);
}

#[test]
fn project_custom_cloze_writes_one_card_per_distinct_ordinal() {
    let root = tempfile::tempdir().expect("tempdir");
    let apkg = root.path().join("project-custom-cloze.apkg");
    let note_type = NoteType::custom_cloze("language-cloze", "text")
        .field(Field::new("Sentence").key("text").identity().sort())
        .field(Field::new("Extra").key("extra").optional())
        .template(
            Template::new("Cloze")
                .key("cloze")
                .front("{{cloze:Sentence}}")
                .back("{{cloze:Sentence}}<br>{{Extra}}"),
        )
        .identity(IdentityRecipe::fields(["text"]));

    let mut project = Project::new("Custom Cloze")
        .stable_id("custom-cloze")
        .default_deck("Custom Cloze");
    project.add_notetype(note_type).expect("add note type");
    project
        .add_note(
            Note::new("language-cloze")
                .stable_id("custom:cloze:1")
                .text("text", "{{c1::Madrid}} is in {{c2::Spain}}")
                .text("extra", "geography"),
        )
        .expect("add note");

    let report = project.write_apkg(&apkg).expect("write apkg");
    report.ensure_success().expect("successful report");
    assert_eq!(report.counts.cards, 2);

    let inspected = inspect_apkg(&apkg).expect("inspect apkg");
    assert!(inspected
        .observations
        .notetypes
        .iter()
        .any(|value| { value["id"] == "language-cloze" && value["kind"] == "cloze" }));
    let card_ords = inspected
        .observations
        .references
        .iter()
        .filter_map(|value| {
            value["selector"]
                .as_str()
                .filter(|selector| selector.starts_with("card["))
                .and_then(|_| value["ord"].as_u64())
        })
        .collect::<Vec<_>>();
    assert_eq!(card_ords, vec![0, 1]);
}

#[test]
fn project_custom_cloze_rejects_malformed_marker() {
    let root = tempfile::tempdir().expect("tempdir");
    let apkg = root.path().join("malformed-custom-cloze.apkg");
    let note_type = NoteType::custom_cloze("language-cloze", "text")
        .field(Field::new("Sentence").key("text").identity().sort())
        .template(
            Template::new("Cloze")
                .key("cloze")
                .front("{{cloze:Sentence}}")
                .back("{{cloze:Sentence}}"),
        )
        .identity(IdentityRecipe::fields(["text"]));
    let mut project = Project::new("Malformed Custom Cloze")
        .stable_id("malformed-custom-cloze")
        .default_deck("Malformed Custom Cloze");
    project.add_notetype(note_type).expect("add note type");
    project
        .add_note(
            Note::new("language-cloze")
                .stable_id("custom:cloze:malformed")
                .text("text", "{{c1::unclosed"),
        )
        .expect("add note");

    let error = project
        .write_apkg(&apkg)
        .expect_err("malformed cloze must fail");

    assert!(error
        .report
        .diagnostic_codes()
        .contains(&"PRODUCT.CLOZE_MARKER_MALFORMED".to_string()));
    assert!(!apkg.exists());
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
