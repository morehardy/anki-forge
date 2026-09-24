use ankiforge::update::CompareOptions;
use ankiforge::{BuildOptions, Note, Project};

#[test]
fn packaged_crate_creates_compares_and_updates_with_embedded_contracts() {
    assert!(!ankiforge::facade_api_version().is_empty());
    assert!(!ankiforge::embedded_contract_version().is_empty());
    let root = tempfile::tempdir().unwrap();
    let apkg = root.path().join("v1.apkg");
    let mut project = Project::new("packaged-consumer").unwrap();
    project.add("hello", Note::basic("front", "back")).unwrap();
    let output = project.build(BuildOptions::to(&apkg)).unwrap();
    assert_eq!(output.report().counts().notes, 1);
    let mut next = Project::new("packaged-consumer").unwrap();
    next.add("hello", Note::basic("front", "new answer"))
        .unwrap();
    let report = next.compare(CompareOptions::against(&apkg)).unwrap();
    assert!(report.policy().allows_publication());
    let updated = next
        .build(BuildOptions::temporary().update_from(apkg))
        .unwrap();
    assert_eq!(updated.report().counts().cards, 1);
}
