use anki_forge::{Deck, Package};

#[test]
fn deck_export_surfaces_use_runtime_defaults_and_real_artifact_paths() {
    let mut deck = Deck::builder("Spanish")
        .stable_id("spanish-v1")
        .build();
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
fn package_single_matches_deck_export_surface() {
    let mut deck = Deck::builder("Spanish")
        .stable_id("spanish-v1")
        .build();
    deck.basic()
        .note("hola", "hello")
        .stable_id("es-hola")
        .add()
        .expect("add basic note");

    let package = Package::single(deck.clone()).with_stable_id("package-v1");

    assert_eq!(
        deck.to_apkg_bytes().expect("deck bytes"),
        package.to_apkg_bytes().expect("package bytes"),
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
