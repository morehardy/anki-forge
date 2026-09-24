mod common;
use ankiforge::{BuildOptions, Media, Note, Project};
use std::path::Path;

fn media_project(root: &Path) -> Project {
    let mut project = Project::new("prepared-media").unwrap();
    for index in 0..24 {
        let name = format!("media-{index}.bin");
        let path = root.join(&name);
        let bytes = if index == 0 {
            let mut state = 17u32;
            (0..1_500_000)
                .map(|_| {
                    state ^= state << 13;
                    state ^= state >> 17;
                    state ^= state << 5;
                    state as u8
                })
                .collect()
        } else {
            format!("shared content {}", index % 12)
                .repeat(5000)
                .into_bytes()
        };
        std::fs::write(&path, bytes).unwrap();
        project
            .add_asset(Media::file(&path).unwrap().with_export_name(name).unwrap())
            .unwrap();
        project
            .add(
                format!("note-{index}"),
                Note::basic(format!("front {index}"), "back"),
            )
            .unwrap();
    }
    project
}

#[test]
fn temporary_and_persistent_exports_contain_identical_decoded_assets() {
    let root = tempfile::tempdir().unwrap();
    let project = media_project(root.path());
    let temporary = project.build(BuildOptions::temporary()).unwrap();
    let persistent = project
        .build(
            BuildOptions::to(root.path().join("saved.apkg"))
                .update_from(temporary.artifact().path()),
        )
        .unwrap();
    assert_eq!(
        common::entries(temporary.artifact().path()),
        common::entries(persistent.artifact().path())
    );
    assert_eq!(persistent.report().counts().media, 24);
}

#[test]
fn concurrent_builds_keep_media_and_output_independent() {
    let root = tempfile::tempdir().unwrap();
    let project = media_project(root.path());
    let initial = project.build(BuildOptions::temporary()).unwrap();
    let outputs = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..3)
            .map(|_| {
                scope.spawn(|| {
                    let output = project
                        .build(BuildOptions::temporary().update_from(initial.artifact().path()))
                        .unwrap();
                    (
                        output.artifact().path().to_owned(),
                        common::entries(output.artifact().path()),
                    )
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect::<Vec<_>>()
    });
    assert!(outputs.windows(2).all(|pair| pair[0].1 == pair[1].1));
    assert!(outputs.iter().all(|(path, _)| !path.exists()));
}

#[test]
fn same_name_changed_snapshot_conflict_is_atomic() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("same.txt");
    std::fs::write(&source, b"original").unwrap();
    let mut project = Project::new("individual-media").unwrap();
    project
        .add_asset(
            Media::file(&source)
                .unwrap()
                .with_export_name("same.txt")
                .unwrap(),
        )
        .unwrap();
    project.add("one", Note::basic("front", "back")).unwrap();
    std::fs::write(&source, b"modified").unwrap();
    let error = project
        .add_asset(
            Media::file(&source)
                .unwrap()
                .with_export_name("same.txt")
                .unwrap(),
        )
        .unwrap_err();
    assert_eq!(error.code(), "MEDIA.DUPLICATE_FILENAME_CONFLICT");
    let output = project.build(BuildOptions::temporary()).unwrap();
    let evidence = common::evidence(output.artifact().path());
    assert_eq!(evidence["media"]["same.txt"]["size"], 8);
    assert_eq!(output.report().counts().media, 1);
}

#[test]
fn deleting_or_replacing_source_paths_after_import_does_not_change_snapshots() {
    let root = tempfile::tempdir().unwrap();
    let project = media_project(root.path());
    let first = project.build(BuildOptions::temporary()).unwrap();
    for index in 0..24 {
        let path = root.path().join(format!("media-{index}.bin"));
        if index % 2 == 0 {
            std::fs::remove_file(path).unwrap();
        } else {
            std::fs::write(path, b"changed after import").unwrap();
        }
    }
    let next = project
        .build(BuildOptions::temporary().update_from(first.artifact().path()))
        .unwrap();
    assert_eq!(
        common::entries(first.artifact().path()),
        common::entries(next.artifact().path())
    );
}
