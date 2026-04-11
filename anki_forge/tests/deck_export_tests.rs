use anki_forge::{Deck, Package};
use serde_json::json;

#[test]
fn deck_export_surfaces_use_runtime_defaults_and_real_artifact_paths() {
    let mut deck = Deck::builder("Spanish").stable_id("spanish-v1").build();
    deck.basic()
        .note("hola", "hello")
        .stable_id("es-hola")
        .add()
        .expect("add basic note");

    let artifacts_dir = unique_artifacts_dir("deck-export");
    let build = deck.build(&artifacts_dir).expect("build facade");

    assert!(build.apkg_path().exists());
    assert!(build.staging_manifest_path().exists());
    assert_eq!(build.package_build_result().result_status, "success");

    let bytes = deck.to_apkg_bytes().expect("apkg bytes");
    assert!(!bytes.is_empty());
}

#[test]
fn deck_basic_flow_example_shape_matches_the_public_happy_path() {
    let mut deck = Deck::builder("Spanish").stable_id("spanish-v1").build();

    deck.basic()
        .note("hola", "hello")
        .stable_id("es-hola")
        .add()
        .expect("add basic");

    assert_eq!(deck.notes().len(), 1);
    assert_eq!(deck.notes()[0].id(), "es-hola");
}

#[test]
fn package_single_export_surfaces_support_bytes_and_package_stable_id() {
    let mut deck = Deck::builder("Spanish").stable_id("spanish-v1").build();
    deck.basic()
        .note("hola", "hello")
        .stable_id("es-hola")
        .add()
        .expect("add basic note");

    let base_package = Package::single(deck.clone());
    let overridden_package = Package::single(deck.clone()).with_stable_id("package-v1");

    let deck_bytes = deck.to_apkg_bytes().expect("deck bytes");
    let base_bytes = base_package.to_apkg_bytes().expect("base package bytes");
    let overridden_bytes = overridden_package
        .to_apkg_bytes()
        .expect("overridden package bytes");

    assert_eq!(deck_bytes, base_bytes);
    assert_eq!(base_bytes, overridden_bytes);

    let base_build = base_package
        .build(unique_artifacts_dir("deck-export-base-build"))
        .expect("base build");
    let overridden_build = overridden_package
        .build(unique_artifacts_dir("deck-export-overridden-build"))
        .expect("overridden build");
    assert_eq!(
        base_build.package_build_result().apkg_ref.as_deref(),
        Some("artifacts/spanish-v1/package.apkg")
    );
    assert_eq!(
        base_build.package_build_result().staging_ref.as_deref(),
        Some("artifacts/spanish-v1/staging/manifest.json")
    );
    assert_eq!(
        overridden_build.package_build_result().apkg_ref.as_deref(),
        Some("artifacts/package-v1/package.apkg")
    );
    assert_eq!(
        overridden_build
            .package_build_result()
            .staging_ref
            .as_deref(),
        Some("artifacts/package-v1/staging/manifest.json")
    );

    let mut written = Vec::new();
    overridden_package
        .write_to(&mut written)
        .expect("write package bytes");
    assert_eq!(written, overridden_bytes);

    let write_path = unique_artifacts_dir("deck-export-write-apkg").join("export.apkg");
    overridden_package
        .write_apkg(&write_path)
        .expect("write apkg to path");
    assert_eq!(
        std::fs::read(&write_path).expect("read written apkg"),
        overridden_bytes
    );
}

#[test]
fn deck_export_rejects_invalid_deserialized_deck_during_bytes_export() {
    let deck: Deck = serde_json::from_value(json!({
        "name": "Spanish",
        "stable_id": null,
        "notes": [
            {
                "Basic": {
                    "id": "",
                    "stable_id": "   ",
                    "front": "hola",
                    "back": "hello",
                    "tags": [],
                    "generated": false
                }
            }
        ],
        "next_generated_note_id": 1,
        "media": {}
    }))
    .expect("deserialize invalid deck");

    let err = deck
        .to_apkg_bytes()
        .expect_err("invalid deck should fail export");
    assert!(err.to_string().contains("deck validation failed"));
}

#[test]
fn package_single_with_stable_id_keeps_root_deck_and_changes_export_identity() {
    let deck = Deck::builder("Spanish").stable_id("spanish-v1").build();

    let package = Package::single(deck.clone()).with_stable_id("package-v1");

    assert_eq!(
        package.root_deck().stable_id().as_deref(),
        Some("spanish-v1")
    );
    assert_eq!(
        deck.to_apkg_bytes().expect("deck bytes"),
        Package::single(deck.clone())
            .to_apkg_bytes()
            .expect("base package bytes")
    );
    assert_eq!(
        Package::single(deck.clone())
            .to_apkg_bytes()
            .expect("base package bytes"),
        package.to_apkg_bytes().expect("override package bytes")
    );
}

fn unique_artifacts_dir(label: &str) -> std::path::PathBuf {
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
    std::fs::create_dir_all(&dir).expect("create artifacts dir");
    dir
}
