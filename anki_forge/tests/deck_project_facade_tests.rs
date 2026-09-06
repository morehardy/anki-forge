use std::path::PathBuf;

use anki_forge::prelude::*;

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
fn project_from_deck_can_append_project_notes() {
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

    let report = project
        .build(BuildOptions::new().output(root.join("deck.apkg")))
        .expect("an imported Deck is an editable Project");

    assert_eq!(report.counts.notes, 2);
    assert_eq!(report.counts.cards, 2);
}

#[test]
fn imported_deck_keeps_duplicate_id_locations_after_clone() {
    let mut deck = Deck::new("Imported IDs");
    deck.basic()
        .note("original", "answer")
        .stable_id("existing")
        .add()
        .unwrap();
    let project = Project::from(deck);
    let mut cloned = project.clone();
    cloned
        .add_note(Note::basic("new", "answer").stable_id("new"))
        .unwrap();
    let error = cloned
        .add_note(Note::basic("duplicate", "answer").stable_id("existing"))
        .expect_err("an imported note must reserve its stable ID");
    assert_eq!(error.code(), ErrorCode::StableIdDuplicate);
    assert_eq!(
        error.diagnostic().message,
        "duplicate stable_id 'existing' at project.notes[2]; first definition is project.notes[0]"
    );
    assert_eq!(cloned.build(BuildOptions::new()).unwrap().counts.notes, 2);
    assert_eq!(project.build(BuildOptions::new()).unwrap().counts.notes, 1);
}

#[test]
fn imported_deck_media_and_new_project_media_share_one_registry() {
    let mut deck = Deck::builder("Media").stable_id("imported-media").build();
    deck.media()
        .add(MediaSource::from_bytes("original.png", PNG.to_vec()))
        .unwrap();
    deck.basic()
        .note("<b>front</b>", "<img src=\"original.png\">")
        .stable_id("original")
        .add()
        .unwrap();
    let mut project = Project::from(deck);
    let image = project
        .media_mut()
        .add_bytes("extra", PNG.to_vec())
        .unwrap()
        .export_as("extra.png")
        .unwrap();
    project
        .add_notetype(
            NoteType::custom("custom")
                .field(Field::new("Question").key("question"))
                .template(
                    Template::new("Card")
                        .key("card")
                        .front("{{Question}}")
                        .back("{{FrontSide}}"),
                ),
        )
        .unwrap();
    project
        .add_note(
            Note::new("custom")
                .stable_id("extra")
                .image("question", image),
        )
        .unwrap();
    let plan = project.lower().unwrap();
    assert_eq!(
        plan.authoring_document.notes[0].fields["Front"],
        "<b>front</b>"
    );
    assert_eq!(plan.authoring_document.notes[0].id, "original");
    let report = project.build(BuildOptions::new()).unwrap();
    assert_eq!(report.counts.notes, 2);
    assert_eq!(report.media.bindings, 2);
    assert_eq!(report.media.references, 2);
    assert_eq!(report.media.missing_references, 0);
    assert!(project
        .media_mut()
        .add_bytes("conflict", b"different".to_vec())
        .unwrap()
        .export_as("original.png")
        .is_err());
}

#[test]
fn imported_deck_identity_provenance_survives_project_edits() {
    let root = tempfile::tempdir().unwrap();
    let mut deck = Deck::builder("Identity")
        .stable_id("imported-identity")
        .build();
    deck.basic().note("<b>front</b>", "back").add().unwrap();
    let snapshot = deck.notes()[0].resolved_identity().unwrap().clone();
    let mut project = Project::from(deck);
    project
        .add_note(Note::basic("other", "answer").stable_id("other"))
        .unwrap();
    let lockfile = root.path().join("identity.json");
    let report = project
        .build(BuildOptions::new().first_update_safe_build(&lockfile))
        .unwrap();
    assert_eq!(report.counts.notes, 2);
    let json: serde_json::Value =
        serde_json::from_slice(&std::fs::read(lockfile).unwrap()).unwrap();
    let note = json["identity_index"]["notes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|note| note["stable_id"] == snapshot.stable_id)
        .unwrap();
    assert_eq!(note["recipe_id"], snapshot.recipe_id.unwrap());
    assert_eq!(note["provenance"], "InferredFromStockRecipe");
    assert_eq!(
        note["canonical_payload_hash"],
        format!(
            "blake3:{}",
            blake3::hash(snapshot.canonical_payload.unwrap().as_bytes())
        )
    );
}

#[test]
fn imported_deck_keeps_legacy_stock_declarations_and_honors_project_metadata() {
    let mut deck = Deck::builder("Original").stable_id("original").build();
    deck.basic()
        .note("front", "back")
        .stable_id("note-1")
        .add()
        .unwrap();
    let legacy = deck
        .clone()
        .into_product_document()
        .unwrap()
        .lower()
        .unwrap();
    let imported = Project::from(deck)
        .stable_id("renamed")
        .default_deck("Renamed")
        .lower()
        .unwrap();
    assert_eq!(imported.authoring_document.metadata_document_id, "renamed");
    assert_eq!(imported.authoring_document.notes[0].deck_name, "Renamed");
    assert_eq!(imported.authoring_document.notetypes.len(), 3);
    for (before, after) in legacy
        .authoring_document
        .notetypes
        .iter()
        .zip(&imported.authoring_document.notetypes)
    {
        assert_eq!(before.id, after.id);
        assert_eq!(before.original_id, after.original_id);
        assert_eq!(
            before.fields, after.fields,
            "stock field identities and order"
        );
        assert_eq!(
            before.templates, after.templates,
            "stock template identities and order"
        );
        assert_eq!(before.css, after.css);
    }
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
