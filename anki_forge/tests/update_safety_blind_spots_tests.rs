#![cfg(feature = "internal-tools")]

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::Path;

use anki_forge::build::{BuildOptions, UpdateSafetyMode};
use anki_forge::prelude::*;
use anki_forge::update_safety::{baseline::load_previous_apkg_identity_index, lockfile};
use rusqlite::Connection;

#[test]
fn notetype_model_ids_survive_declaration_reordering_and_insertion() {
    let root = tempfile::tempdir().unwrap();
    let previous = root.path().join("previous.apkg");
    let standalone = root.path().join("standalone.apkg");
    let updated = root.path().join("updated.apkg");
    project_with_types(&["alpha", "beta"], false)
        .write_apkg(&previous)
        .unwrap();
    let next = project_with_types(&["gamma", "beta", "alpha"], false);
    next.write_apkg(&standalone).unwrap();
    next.build(BuildOptions::new().output(&updated).compare_to(&previous))
        .unwrap();

    let original_ids = model_ids(&previous);
    for path in [&standalone, &updated] {
        let current_ids = model_ids(path);
        for name in ["alpha", "beta"] {
            assert_eq!(current_ids[name], original_ids[name], "{name}: {path:?}");
        }
        assert!(!original_ids.values().any(|id| *id == current_ids["gamma"]));
    }
}

#[test]
fn apkg_baseline_and_lockfile_record_actual_model_ids() {
    let root = tempfile::tempdir().unwrap();
    let previous = root.path().join("previous.apkg");
    let lock = root.path().join("identity.json");
    project_with_types(&["alpha", "beta"], false)
        .build(
            BuildOptions::new()
                .output(&previous)
                .first_update_safe_build(&lock),
        )
        .unwrap();

    let actual = model_ids(&previous);
    let indexes = [
        load_previous_apkg_identity_index(&previous, None, None).unwrap(),
        lockfile::read_lockfile(&lock).unwrap().identity_index,
    ];
    for index in indexes {
        for notetype in index.notetypes {
            assert_eq!(notetype.anki_model_id, Some(actual[&notetype.name]));
        }
    }
}

#[test]
fn lockfile_only_updates_preserve_model_ids_and_reintroduced_types() {
    let root = tempfile::tempdir().unwrap();
    let lock = root.path().join("identity.json");
    let first = root.path().join("first.apkg");
    project_with_types(&["alpha", "beta"], false)
        .build(
            BuildOptions::new()
                .output(&first)
                .first_update_safe_build(&lock),
        )
        .unwrap();
    let mut previous = lockfile::read_lockfile(&lock).unwrap();
    // Simulate a baseline imported with pre-existing Anki model IDs.
    for notetype in &mut previous.identity_index.notetypes {
        notetype.anki_model_id = Some(if notetype.name == "alpha" { 101 } else { 202 });
    }
    lockfile::write_lockfile_atomic(&lock, &previous).unwrap();
    project_with_types(&["beta", "gamma"], false)
        .build(
            BuildOptions::new()
                .output(root.path().join("second.apkg"))
                .update_safe(&lock)
                .write_identity_lockfile(true),
        )
        .unwrap();
    let last = root.path().join("last.apkg");
    project_with_types(&["gamma", "alpha", "beta"], false)
        .build(BuildOptions::new().output(&last).update_safe(&lock))
        .unwrap();
    let actual = model_ids(&last);
    assert_eq!(actual["alpha"], 101);
    assert_eq!(actual["beta"], 202);
}

#[test]
fn strict_lockfile_only_field_removal_blocks_before_output() {
    let root = tempfile::tempdir().unwrap();
    let lock = root.path().join("identity.json");
    project_with_types(&["alpha"], true)
        .build(
            BuildOptions::new()
                .output(root.path().join("previous.apkg"))
                .first_update_safe_build(&lock),
        )
        .unwrap();
    let before = std::fs::read(&lock).unwrap();
    let output = root.path().join("updated.apkg");
    let error = project_with_types(&["alpha"], false)
        .build(
            BuildOptions::new()
                .output(&output)
                .update_safe(&lock)
                .write_identity_lockfile(true),
        )
        .expect_err("removing a field must block strict lockfile-only updates");
    assert!(error
        .report
        .diagnostic_codes()
        .contains(&"UPDATE.FIELD_REMOVED".into()));
    assert!(error
        .report
        .risk
        .unwrap()
        .findings
        .iter()
        .any(|finding| { finding.code == "RISK.FIELD_REMOVED_OR_RENAMED" }));
    assert!(!output.exists());
    assert_eq!(std::fs::read(&lock).unwrap(), before);
}

#[test]
fn field_addition_and_report_only_removal_are_visible() {
    let root = tempfile::tempdir().unwrap();
    let lock = root.path().join("identity.json");
    project_with_types(&["alpha"], false)
        .build(
            BuildOptions::new()
                .output(root.path().join("first.apkg"))
                .first_update_safe_build(&lock),
        )
        .unwrap();
    let added = project_with_types(&["alpha"], true)
        .build(
            BuildOptions::new()
                .output(root.path().join("added.apkg"))
                .update_safe(&lock)
                .write_identity_lockfile(true),
        )
        .unwrap();
    assert!(added
        .diagnostic_codes()
        .contains(&"UPDATE.FIELD_ADDED".into()));
    let removed = project_with_types(&["alpha"], false)
        .build(
            BuildOptions::new()
                .output(root.path().join("removed.apkg"))
                .identity_lockfile(&lock)
                .update_safety(UpdateSafetyMode::ReportOnly),
        )
        .unwrap();
    assert!(removed.diagnostics.iter().any(|diagnostic| {
        diagnostic.code.as_str() == "UPDATE.FIELD_REMOVED"
            && diagnostic.severity == Severity::Warning
    }));
    assert!(removed
        .risk
        .unwrap()
        .findings
        .iter()
        .any(|finding| { finding.code == "RISK.FIELD_REMOVED_OR_RENAMED" }));
}

#[test]
fn legacy_lockfile_requires_apkg_in_strict_mode_and_can_be_migrated() {
    let root = tempfile::tempdir().unwrap();
    let lock = root.path().join("identity.json");
    let previous = root.path().join("legacy.apkg");
    let project = project_with_types(&["alpha"], false);
    project
        .build(
            BuildOptions::new()
                .output(&previous)
                .first_update_safe_build(&lock),
        )
        .unwrap();
    rewrite_apkg_model_id(&previous, 101);
    let mut legacy = lockfile::read_lockfile(&lock).unwrap();
    legacy.identity_index.notetypes[0].anki_model_id = None;
    lockfile::write_lockfile_atomic(&lock, &legacy).unwrap();
    let before = std::fs::read(&lock).unwrap();
    let blocked = root.path().join("blocked.apkg");
    let error = project
        .build(BuildOptions::new().output(&blocked).update_safe(&lock))
        .unwrap_err();
    assert!(error
        .report
        .diagnostic_codes()
        .contains(&"UPDATE.NOTETYPE_MODEL_ID_MISSING".into()));
    assert!(!blocked.exists());
    assert_eq!(std::fs::read(&lock).unwrap(), before);
    let report = project
        .build(
            BuildOptions::new()
                .output(root.path().join("report-only.apkg"))
                .identity_lockfile(&lock)
                .update_safety(UpdateSafetyMode::ReportOnly),
        )
        .unwrap();
    assert!(report
        .diagnostics
        .iter()
        .any(
            |diagnostic| diagnostic.code.as_str() == "UPDATE.NOTETYPE_MODEL_ID_MISSING"
                && diagnostic.severity == Severity::Warning
        ));
    assert_eq!(std::fs::read(&lock).unwrap(), before);
    let updated = root.path().join("migrated.apkg");
    project_with_types(&["beta", "alpha"], false)
        .build(
            BuildOptions::new()
                .output(&updated)
                .update_safe(&lock)
                .compare_to(&previous)
                .write_identity_lockfile(true),
        )
        .unwrap();
    assert_eq!(model_ids(&updated)["alpha"], 101);
    assert_eq!(
        lockfile::read_lockfile(&lock)
            .unwrap()
            .identity_index
            .notetypes
            .iter()
            .find(|notetype| notetype.name == "alpha")
            .unwrap()
            .anki_model_id,
        Some(101)
    );
}

#[test]
fn actual_apkg_model_id_overrides_conflicting_lockfile_evidence() {
    let root = tempfile::tempdir().unwrap();
    let lock = root.path().join("identity.json");
    let previous = root.path().join("legacy.apkg");
    let project = project_with_types(&["alpha"], false);
    project
        .build(
            BuildOptions::new()
                .output(&previous)
                .first_update_safe_build(&lock),
        )
        .unwrap();
    rewrite_apkg_model_id(&previous, 101);
    let updated = root.path().join("updated.apkg");
    let report = project
        .build(
            BuildOptions::new()
                .output(&updated)
                .update_safe(&lock)
                .compare_to(&previous)
                .write_identity_lockfile(true),
        )
        .unwrap();
    assert!(report
        .diagnostic_codes()
        .contains(&"UPDATE.NOTETYPE_MODEL_ID_CONFLICT".into()));
    assert_eq!(model_ids(&updated)["alpha"], 101);
    assert_eq!(
        lockfile::read_lockfile(&lock)
            .unwrap()
            .identity_index
            .notetypes[0]
            .anki_model_id,
        Some(101)
    );
}

#[test]
fn reserved_absent_model_id_collision_is_fatal_even_in_report_only() {
    let root = tempfile::tempdir().unwrap();
    let lock = root.path().join("identity.json");
    let first = root.path().join("first.apkg");
    project_with_types(&["alpha"], false)
        .build(
            BuildOptions::new()
                .output(&first)
                .first_update_safe_build(&lock),
        )
        .unwrap();
    let candidate = root.path().join("candidate.apkg");
    let project = project_with_types(&["beta"], false);
    project.write_apkg(&candidate).unwrap();
    let mut baseline = lockfile::read_lockfile(&lock).unwrap();
    baseline.identity_index.notetypes[0].anki_model_id = Some(model_ids(&candidate)["beta"]);
    lockfile::write_lockfile_atomic(&lock, &baseline).unwrap();
    let before = std::fs::read(&lock).unwrap();
    for mode in [UpdateSafetyMode::Strict, UpdateSafetyMode::ReportOnly] {
        let output = root.path().join("blocked.apkg");
        let error = project
            .build(
                BuildOptions::new()
                    .output(&output)
                    .identity_lockfile(&lock)
                    .update_safety(mode)
                    .write_identity_lockfile(true),
            )
            .unwrap_err();
        assert!(error
            .report
            .diagnostic_codes()
            .contains(&"UPDATE.NOTETYPE_MODEL_ID_COLLISION".into()));
        assert!(!output.exists());
        assert_eq!(std::fs::read(&lock).unwrap(), before);
    }
}

#[test]
fn lockfile_rejects_duplicate_and_nonpositive_model_ids() {
    let root = tempfile::tempdir().unwrap();
    let lock = root.path().join("identity.json");
    project_with_types(&["alpha", "beta"], false)
        .build(
            BuildOptions::new()
                .output(root.path().join("first.apkg"))
                .first_update_safe_build(&lock),
        )
        .unwrap();
    let original: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&lock).unwrap()).unwrap();
    for bad_id in [0, -1, 101] {
        let mut invalid = original.clone();
        invalid["identity_index"]["notetypes"][0]["anki_model_id"] = 101.into();
        invalid["identity_index"]["notetypes"][1]["anki_model_id"] = bad_id.into();
        std::fs::write(&lock, serde_json::to_vec(&invalid).unwrap()).unwrap();
        assert!(lockfile::read_lockfile(&lock).is_err());
    }
}

#[test]
fn apkg_only_migration_reserves_absent_types_in_new_lockfile() {
    let root = tempfile::tempdir().unwrap();
    let previous = root.path().join("legacy.apkg");
    project_with_types(&["alpha"], false)
        .write_apkg(&previous)
        .unwrap();
    rewrite_apkg_model_id(&previous, 101);
    let lock = root.path().join("identity.json");
    let middle = root.path().join("middle.apkg");
    project_with_types(&["beta"], false)
        .build(
            BuildOptions::new()
                .output(&middle)
                .compare_to(&previous)
                .first_update_safe_build(&lock),
        )
        .unwrap();
    let last = root.path().join("last.apkg");
    project_with_types(&["alpha", "beta"], false)
        .build(
            BuildOptions::new()
                .output(&last)
                .compare_to(&middle)
                .update_safe(&lock),
        )
        .unwrap();
    assert_eq!(model_ids(&last)["alpha"], 101);
}

fn rewrite_apkg_model_id(path: &Path, id: i64) {
    let root = tempfile::tempdir().unwrap();
    let db = root.path().join("collection.sqlite");
    let entries = {
        let mut archive = zip::ZipArchive::new(std::fs::File::open(path).unwrap()).unwrap();
        (0..archive.len())
            .map(|index| {
                let mut entry = archive.by_index(index).unwrap();
                let name = entry.name().to_owned();
                let mut bytes = Vec::new();
                entry.read_to_end(&mut bytes).unwrap();
                if name == "collection.anki21b" {
                    std::fs::write(&db, zstd::stream::decode_all(bytes.as_slice()).unwrap())
                        .unwrap();
                    let conn = Connection::open(&db).unwrap();
                    for sql in [
                        "update notetypes set id = ?1",
                        "update fields set ntid = ?1",
                        "update templates set ntid = ?1",
                        "update notes set mid = ?1",
                    ] {
                        conn.execute(sql, [id]).unwrap();
                    }
                    drop(conn);
                    bytes = zstd::stream::encode_all(std::fs::read(&db).unwrap().as_slice(), 0)
                        .unwrap();
                }
                (name, bytes)
            })
            .collect::<Vec<_>>()
    };
    let mut archive = zip::ZipWriter::new(std::fs::File::create(path).unwrap());
    for (name, bytes) in entries {
        archive
            .start_file(name, zip::write::SimpleFileOptions::default())
            .unwrap();
        archive.write_all(&bytes).unwrap();
    }
    archive.finish().unwrap();
}

fn project_with_types(ids: &[&str], extra_field: bool) -> Project {
    let mut project = Project::new("Identity regression").stable_id("identity-regression");
    for id in ids {
        let mut note_type = NoteType::custom(*id)
            .name(*id)
            .field(Field::new("Front").key("front"))
            .template(
                Template::new("Card")
                    .key("card")
                    .front("{{Front}}")
                    .back("{{Front}}"),
            );
        if extra_field {
            note_type = note_type.field(Field::new("Extra").key("extra").optional());
        }
        project.add_notetype(note_type).unwrap();
        project
            .add_note(
                Note::new(*id)
                    .stable_id(format!("note-{id}"))
                    .text("front", *id),
            )
            .unwrap();
    }
    project
}

fn model_ids(path: &Path) -> BTreeMap<String, i64> {
    let mut archive = zip::ZipArchive::new(std::fs::File::open(path).unwrap()).unwrap();
    let mut collection = archive.by_name("collection.anki21b").unwrap();
    let decoded = zstd::stream::decode_all(&mut collection).unwrap();
    let root = tempfile::tempdir().unwrap();
    let db = root.path().join("collection.sqlite");
    std::fs::write(&db, decoded).unwrap();
    let conn = Connection::open(db).unwrap();
    let ids = conn
        .prepare("select name, id from notetypes")
        .unwrap()
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    ids
}
