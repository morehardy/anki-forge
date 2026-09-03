use anki_forge::prelude::*;

#[test]
fn packaged_crate_runs_the_supported_facade_with_embedded_contracts() {
    assert_eq!(anki_forge::facade_api_version(), "0.1.0");
    assert!(!anki_forge::embedded_contract_version().is_empty());

    let root = tempfile::tempdir().expect("create package test directory");
    let apkg = root.path().join("packaged-facade.apkg");
    let mut deck = Deck::new("Packaged Facade");
    deck.basic()
        .note("front", "back")
        .stable_id("packaged:facade")
        .add()
        .expect("add packaged note");
    deck.write_apkg(&apkg)
        .expect("build through embedded contracts")
        .ensure_success()
        .expect("successful packaged build");
    assert!(apkg.is_file());
}
