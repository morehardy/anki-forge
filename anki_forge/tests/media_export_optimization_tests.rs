use anki_forge::prelude::*;
use std::io::Read;
use std::path::Path;

fn package_entries(path: &Path) -> std::collections::BTreeMap<String, Vec<u8>> {
    let mut zip = zip::ZipArchive::new(std::fs::File::open(path).unwrap()).unwrap();
    (0..zip.len())
        .map(|index| {
            let mut member = zip.by_index(index).unwrap();
            let name = member.name().to_owned();
            let mut bytes = Vec::new();
            member.read_to_end(&mut bytes).unwrap();
            (name, bytes)
        })
        .collect()
}

fn media_project(root: &Path) -> Project {
    let mut project = Project::new("Prepared media").stable_id("prepared-media");
    for index in 0..24 {
        let name = format!("media-{index}.txt");
        let path = root.join(&name);
        // Different names include both shared and distinct content.
        std::fs::write(&path, format!("shared content {}", index % 12).repeat(5000)).unwrap();
        if index == 0 {
            // Incompressible data forces the bounded encoded payload to spill.
            let mut state = 17u32;
            let bytes = (0..1_500_000)
                .map(|_| {
                    state ^= state << 13;
                    state ^= state >> 17;
                    state ^= state << 5;
                    state as u8
                })
                .collect::<Vec<_>>();
            std::fs::write(&path, bytes).unwrap();
        }
        let media = project
            .media_mut()
            .add_file(&path)
            .unwrap()
            .export_as(name)
            .unwrap();
        project
            .add_note(
                Note::basic(format!("front {index}"), "back")
                    .stable_id(format!("note-{index}"))
                    .sound("Back", media),
            )
            .unwrap();
    }
    project
}

#[test]
fn direct_media_matches_persistent_cas_and_keeps_inspect_artifacts() {
    let root = tempfile::tempdir().unwrap();
    let project = media_project(root.path());
    let direct = project.build(BuildOptions::new()).unwrap();
    let retained = root.path().join("artifacts");
    let persistent = project
        .build(BuildOptions::new().artifacts_dir(&retained))
        .unwrap();
    assert_eq!(
        package_entries(direct.artifact.as_ref().unwrap().path()),
        package_entries(persistent.artifact.as_ref().unwrap().path())
    );
    assert_eq!(direct.counts.media, 24);
    assert!(retained.join("staging/media/media-0.txt").is_file());
    assert_eq!(
        std::fs::read(retained.join("staging/media/media-0.txt")).unwrap(),
        std::fs::read(root.path().join("media-0.txt")).unwrap()
    );
}

#[test]
fn concurrent_builds_keep_their_media_and_output_independent() {
    let root = tempfile::tempdir().unwrap();
    let project = media_project(root.path());
    let outputs = std::thread::scope(|scope| {
        let handles = (0..3)
            .map(|_| {
                scope.spawn(|| {
                    let report = project.build(BuildOptions::new()).unwrap();
                    std::fs::read(report.artifact.as_ref().unwrap().path()).unwrap()
                })
            })
            .collect::<Vec<_>>();
        handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect::<Vec<_>>()
    });
    assert!(outputs.windows(2).all(|pair| pair[0] == pair[1]));
}

#[test]
fn individual_file_registration_rejects_content_conflicts_immediately() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("same.txt");
    std::fs::write(&source, b"original").unwrap();
    let mut deck = Deck::new("Individual media");
    deck.media().add(MediaSource::from_file(&source)).unwrap();
    let registered = serde_json::to_value(&deck).unwrap();
    // Same-length replacement must be checked by content during add().
    std::fs::write(&source, b"modified").unwrap();
    let error = deck
        .media()
        .add(MediaSource::from_file(&source))
        .unwrap_err();
    assert!(error
        .to_string()
        .contains("MEDIA.DUPLICATE_FILENAME_CONFLICT"));
    assert_eq!(serde_json::to_value(&deck).unwrap(), registered);
}

#[test]
fn parallel_source_failure_preserves_output_and_allows_retry() {
    let root = tempfile::tempdir().unwrap();
    let project = media_project(root.path());
    let output = root.path().join("output.apkg");
    std::fs::write(&output, b"previous package").unwrap();
    let source = root.path().join("media-23.txt");
    let original = std::fs::read(&source).unwrap();
    std::fs::write(&source, b"changed after registration").unwrap();
    let error = project.write_apkg(&output).unwrap_err();
    assert!(error
        .report
        .diagnostics
        .iter()
        .any(|item| item.code.as_str() == "MEDIA.SOURCE_CHANGED"));
    assert_eq!(std::fs::read(&output).unwrap(), b"previous package");
    std::fs::write(&source, original).unwrap();
    let report = project.write_apkg(&output).unwrap();
    assert_eq!(report.counts.media, 24);
    assert!(zip::ZipArchive::new(std::fs::File::open(&output).unwrap()).is_ok());
}
