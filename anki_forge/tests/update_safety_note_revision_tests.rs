#![cfg(feature = "internal-tools")]

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::Path;

use anki_forge::build::{RiskLevel, UpdateSafetyMode};
use anki_forge::prelude::*;
use rusqlite::Connection;

fn project(answer: &str, tags: &[&str]) -> Project {
    let mut project = Project::new("Revision test").stable_id("revision-test");
    let mut note = Note::basic("Question", answer).stable_id("changed");
    for tag in tags {
        note = note.tag(*tag);
    }
    project.add_note(note).unwrap();
    project
        .add_note(Note::basic("Unchanged question", "Unchanged answer").stable_id("unchanged"))
        .unwrap();
    project
}

fn mtimes(path: &Path) -> BTreeMap<String, i64> {
    let mut archive = zip::ZipArchive::new(std::fs::File::open(path).unwrap()).unwrap();
    let mut collection = archive.by_name("collection.anki21b").unwrap();
    let decoded = zstd::stream::decode_all(&mut collection).unwrap();
    let root = tempfile::tempdir().unwrap();
    let db = root.path().join("collection.sqlite");
    std::fs::write(&db, decoded).unwrap();
    let conn = Connection::open(db).unwrap();
    let values = conn
        .prepare("select guid, mod from notes")
        .unwrap()
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    values
}

#[test]
fn apkg_updates_advance_only_changed_notes_and_rebuild_deterministically() {
    let root = tempfile::tempdir().unwrap();
    let first = root.path().join("first.apkg");
    project("first answer", &[]).write_apkg(&first).unwrap();
    let second = root.path().join("second.apkg");
    let repeat = root.path().join("repeat.apkg");
    let updated = project("second answer", &[]);
    for output in [&second, &repeat] {
        updated
            .build(BuildOptions::new().output(output).compare_to(&first))
            .unwrap();
    }
    let before = mtimes(&first);
    let after = mtimes(&second);
    assert_eq!(after["changed"], before["changed"] + 1);
    assert_eq!(after["unchanged"], before["unchanged"]);
    assert_eq!(
        std::fs::read(&second).unwrap(),
        std::fs::read(&repeat).unwrap()
    );
    let third = root.path().join("third.apkg");
    updated
        .build(BuildOptions::new().output(&third).compare_to(&second))
        .unwrap();
    assert_eq!(mtimes(&third), after);
}

#[test]
fn lockfile_persists_full_content_revision_and_handles_reverts() {
    let root = tempfile::tempdir().unwrap();
    let lock = root.path().join("identity.json");
    let first = root.path().join("first.apkg");
    project("A", &[])
        .build(
            BuildOptions::new()
                .output(&first)
                .first_update_safe_build(&lock),
        )
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&std::fs::read(&lock).unwrap()).unwrap();
    let revision = &json["identity_index"]["notes"][0]["revision"];
    assert!(
        revision["content_hash"].as_str().is_some(),
        "missing full-content evidence: {json}"
    );
    assert_eq!(revision["mtime_secs"], mtimes(&first)["changed"]);
    let mut last = mtimes(&first);
    for (index, (answer, tags)) in [("B", vec![]), ("A", vec![]), ("A", vec!["new-tag"])]
        .into_iter()
        .enumerate()
    {
        let output = root.path().join(format!("update-{index}.apkg"));
        project(answer, &tags)
            .build(
                BuildOptions::new()
                    .output(&output)
                    .update_safe(&lock)
                    .write_identity_lockfile(true),
            )
            .unwrap();
        let current = mtimes(&output);
        assert_eq!(current["changed"], last["changed"] + 1);
        assert_eq!(current["unchanged"], last["unchanged"]);
        last = current;
    }
}

#[test]
fn legacy_lockfile_needs_actual_apkg_revision_evidence_in_strict_mode() {
    let root = tempfile::tempdir().unwrap();
    let lock = root.path().join("identity.json");
    let first = root.path().join("first.apkg");
    project("A", &[])
        .build(
            BuildOptions::new()
                .output(&first)
                .first_update_safe_build(&lock),
        )
        .unwrap();
    let mut legacy: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&lock).unwrap()).unwrap();
    for note in legacy["identity_index"]["notes"].as_array_mut().unwrap() {
        note.as_object_mut().unwrap().remove("revision");
    }
    std::fs::write(&lock, serde_json::to_vec(&legacy).unwrap()).unwrap();
    let unchanged_lock = std::fs::read(&lock).unwrap();
    let blocked = root.path().join("blocked.apkg");
    let error = project("B", &[])
        .build(
            BuildOptions::new()
                .output(&blocked)
                .update_safe(&lock)
                .write_identity_lockfile(true),
        )
        .unwrap_err();
    assert!(error
        .report
        .diagnostic_codes()
        .contains(&"UPDATE.NOTE_REVISION_MISSING".into()));
    assert!(!blocked.exists());
    assert_eq!(std::fs::read(&lock).unwrap(), unchanged_lock);
    let migrated = root.path().join("migrated.apkg");
    project("B", &[])
        .build(
            BuildOptions::new()
                .output(&migrated)
                .update_safe(&lock)
                .compare_to(&first)
                .write_identity_lockfile(true),
        )
        .unwrap();
    assert_eq!(mtimes(&migrated)["changed"], mtimes(&first)["changed"] + 1);
}

#[test]
fn report_only_missing_revision_is_explicit_and_high_risk() {
    let root = tempfile::tempdir().unwrap();
    let lock = root.path().join("identity.json");
    project("A", &[])
        .build(
            BuildOptions::new()
                .output(root.path().join("first.apkg"))
                .first_update_safe_build(&lock),
        )
        .unwrap();
    let mut legacy: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&lock).unwrap()).unwrap();
    for note in legacy["identity_index"]["notes"].as_array_mut().unwrap() {
        note.as_object_mut().unwrap().remove("revision");
    }
    std::fs::write(&lock, serde_json::to_vec(&legacy).unwrap()).unwrap();
    let report = project("B", &[])
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
        .any(|d| d.code.as_str() == "UPDATE.NOTE_REVISION_MISSING"
            && d.severity == Severity::Warning));
    assert_eq!(report.risk.unwrap().highest_level, Some(RiskLevel::High));
}

fn assert_report_only_cannot_launder_missing_evidence(missing_model_id: bool) {
    let root = tempfile::tempdir().unwrap();
    let lock = root.path().join("identity.json");
    let first = root.path().join("first.apkg");
    project("A", &[])
        .build(
            BuildOptions::new()
                .output(&first)
                .first_update_safe_build(&lock),
        )
        .unwrap();
    let mut legacy: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&lock).unwrap()).unwrap();
    let missing_code = if missing_model_id {
        legacy["identity_index"]["notetypes"][0]["anki_model_id"] = serde_json::Value::Null;
        "UPDATE.NOTETYPE_MODEL_ID_MISSING"
    } else {
        for note in legacy["identity_index"]["notes"].as_array_mut().unwrap() {
            note.as_object_mut().unwrap().remove("revision");
        }
        "UPDATE.NOTE_REVISION_MISSING"
    };
    std::fs::write(&lock, serde_json::to_vec(&legacy).unwrap()).unwrap();
    let before = std::fs::read(&lock).unwrap();
    let report = project("B", &[])
        .build(
            BuildOptions::new()
                .output(root.path().join("report-only.apkg"))
                .identity_lockfile(&lock)
                .write_identity_lockfile(true)
                .update_safety(UpdateSafetyMode::ReportOnly),
        )
        .unwrap();
    assert!(report.diagnostic_codes().contains(&missing_code.into()));
    assert!(
        !report.update_safety.as_ref().unwrap().lockfile_written,
        "report-only must not turn missing evidence into trusted baseline facts"
    );
    assert_eq!(std::fs::read(&lock).unwrap(), before);
    assert!(report
        .diagnostic_codes()
        .contains(&"UPDATE.LOCKFILE_WRITE_SKIPPED_UNVERIFIED".into()));
    let strict = project("B", &[])
        .build(
            BuildOptions::new()
                .output(root.path().join("strict.apkg"))
                .update_safe(&lock),
        )
        .unwrap_err();
    assert!(strict
        .report
        .diagnostic_codes()
        .contains(&missing_code.into()));
    assert!(!root.path().join("strict.apkg").exists());

    let migrated = project("B", &[])
        .build(
            BuildOptions::new()
                .output(root.path().join("migrated.apkg"))
                .identity_lockfile(&lock)
                .compare_to(&first)
                .write_identity_lockfile(true)
                .update_safety(UpdateSafetyMode::ReportOnly),
        )
        .unwrap();
    assert!(migrated.update_safety.unwrap().lockfile_written);
    project("B", &[])
        .build(
            BuildOptions::new()
                .output(root.path().join("verified.apkg"))
                .update_safe(&lock),
        )
        .unwrap();
}

#[test]
fn report_only_lockfile_rewrite_does_not_manufacture_model_ids() {
    assert_report_only_cannot_launder_missing_evidence(true);
}

#[test]
fn report_only_lockfile_rewrite_does_not_manufacture_note_revisions() {
    assert_report_only_cannot_launder_missing_evidence(false);
}

fn assert_rejected_lockfile_is_high_risk(invalid_model_id: bool) {
    let root = tempfile::tempdir().unwrap();
    let lock = root.path().join("identity.json");
    project("A", &[])
        .build(
            BuildOptions::new()
                .output(root.path().join("first.apkg"))
                .first_update_safe_build(&lock),
        )
        .unwrap();
    let mut invalid: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&lock).unwrap()).unwrap();
    let cause = if invalid_model_id {
        invalid["identity_index"]["notetypes"][0]["anki_model_id"] = 0.into();
        "UPDATE.NOTETYPE_MODEL_ID_INVALID"
    } else {
        invalid["identity_index"]["notes"][0]["revision"]["content_hash"] = "bad".into();
        "UPDATE.NOTE_REVISION_INVALID"
    };
    std::fs::write(&lock, serde_json::to_vec(&invalid).unwrap()).unwrap();
    let before = std::fs::read(&lock).unwrap();
    let output = root.path().join("report-only.apkg");
    let options = || {
        BuildOptions::new()
            .output(&output)
            .identity_lockfile(&lock)
            .write_identity_lockfile(true)
            .update_safety(UpdateSafetyMode::ReportOnly)
    };
    let report = project("B", &[]).build(options()).unwrap();
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.code.as_str() == "UPDATE.BASELINE_LOCKFILE_UNREADABLE"
            && diagnostic.severity == Severity::Warning
            && diagnostic.message.contains(cause)
    }));
    assert_eq!(
        report.risk.as_ref().unwrap().highest_level,
        Some(RiskLevel::High),
        "rejected requested baseline must remain high risk: {report:?}"
    );
    assert!(report
        .risk
        .as_ref()
        .unwrap()
        .findings
        .iter()
        .any(|finding| {
            finding.code == "RISK.BASELINE_UNAVAILABLE"
                && finding.source.as_ref().unwrap().as_str() == lock.to_str().unwrap()
                && finding.evidence_refs.iter().any(|evidence| {
                    evidence
                        .ref_id
                        .contains("UPDATE.BASELINE_LOCKFILE_UNREADABLE")
                })
        }));
    assert!(!report.update_safety.unwrap().lockfile_written);
    assert_eq!(std::fs::read(&lock).unwrap(), before);
    let published = std::fs::read(&output).unwrap();
    let blocked = project("C", &[])
        .build(options().fail_on(RiskLevel::High))
        .unwrap_err();
    assert_eq!(
        blocked.cause,
        anki_forge::build::BuildFailureCause::PolicyBlocked
    );
    assert!(blocked.report.artifact.is_none());
    assert!(!blocked.report.update_safety.unwrap().lockfile_written);
    assert_eq!(std::fs::read(&output).unwrap(), published);
    assert_eq!(std::fs::read(&lock).unwrap(), before);
}

#[test]
fn report_only_rejected_model_id_baseline_is_high_risk() {
    assert_rejected_lockfile_is_high_risk(true);
}

#[test]
fn report_only_rejected_revision_baseline_is_high_risk() {
    assert_rejected_lockfile_is_high_risk(false);
}

#[test]
fn report_only_first_build_and_verified_updates_can_write_lockfiles() {
    let root = tempfile::tempdir().unwrap();
    let lock = root.path().join("identity.json");
    for (index, answer) in ["A", "B"].into_iter().enumerate() {
        let output = root.path().join(format!("release-{index}.apkg"));
        let report = project(answer, &[])
            .build(
                BuildOptions::new()
                    .output(&output)
                    .identity_lockfile(&lock)
                    .write_identity_lockfile(true)
                    .update_safety(UpdateSafetyMode::ReportOnly),
            )
            .unwrap();
        assert!(report.update_safety.unwrap().lockfile_written);
        assert_eq!(report.risk.unwrap().highest_level, None);
        let saved = anki_forge::update_safety::lockfile::read_lockfile(&lock).unwrap();
        let revision = saved.identity_index.notes[0].revision.as_ref().unwrap();
        assert_eq!(revision.mtime_secs, index as i64 + 1);
        assert_eq!(revision.mtime_secs, mtimes(&output)["changed"]);
    }
}

#[test]
fn report_only_unreadable_apkg_does_not_create_a_trusted_lockfile() {
    let root = tempfile::tempdir().unwrap();
    let baseline = root.path().join("invalid.apkg");
    std::fs::write(&baseline, b"unreadable baseline").unwrap();
    let lock = root.path().join("identity.json");
    let report = project("A", &[])
        .build(
            BuildOptions::new()
                .output(root.path().join("report-only.apkg"))
                .compare_to(&baseline)
                .identity_lockfile(&lock)
                .write_identity_lockfile(true)
                .update_safety(UpdateSafetyMode::ReportOnly),
        )
        .unwrap();
    assert!(report
        .diagnostic_codes()
        .contains(&"UPDATE.BASELINE_APKG_UNREADABLE".into()));
    assert!(report
        .diagnostic_codes()
        .contains(&"UPDATE.LOCKFILE_WRITE_SKIPPED_UNVERIFIED".into()));
    assert!(!report.update_safety.unwrap().lockfile_written);
    assert!(!lock.exists());
}

#[test]
fn tag_order_duplicates_and_storage_whitespace_do_not_advance_revision() {
    let root = tempfile::tempdir().unwrap();
    let first = root.path().join("first.apkg");
    project("A", &["b", "a", "a", "c d"])
        .write_apkg(&first)
        .unwrap();
    let second = root.path().join("second.apkg");
    project("A", &["d", "c", "a", "b"])
        .build(BuildOptions::new().output(&second).compare_to(&first))
        .unwrap();
    assert_eq!(mtimes(&first), mtimes(&second));
}

#[test]
fn actual_apkg_revision_overrides_stale_lockfile() {
    let root = tempfile::tempdir().unwrap();
    let lock = root.path().join("identity.json");
    let first = root.path().join("first.apkg");
    project("A", &[])
        .build(
            BuildOptions::new()
                .output(&first)
                .first_update_safe_build(&lock),
        )
        .unwrap();
    let mut stale: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&lock).unwrap()).unwrap();
    stale["identity_index"]["notes"][0]["revision"]["mtime_secs"] = 99.into();
    std::fs::write(&lock, serde_json::to_vec(&stale).unwrap()).unwrap();
    let second = root.path().join("second.apkg");
    let report = project("B", &[])
        .build(
            BuildOptions::new()
                .output(&second)
                .update_safe(&lock)
                .compare_to(&first),
        )
        .unwrap();
    assert!(report
        .diagnostic_codes()
        .contains(&"UPDATE.NOTE_REVISION_CONFLICT".into()));
    assert_eq!(mtimes(&second)["changed"], mtimes(&first)["changed"] + 1);
}

#[test]
fn baseline_recovers_actual_stored_content_and_real_timestamp() {
    let root = tempfile::tempdir().unwrap();
    let first = root.path().join("first.apkg");
    project("A", &[]).write_apkg(&first).unwrap();
    let db = root.path().join("collection.sqlite");
    let entries = {
        let mut archive = zip::ZipArchive::new(std::fs::File::open(&first).unwrap()).unwrap();
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
                    conn.execute(
                        "update notes set mod = 1700000000, flds = ?1 where guid = 'changed'",
                        ["Question\u{1f}B"],
                    )
                    .unwrap();
                    drop(conn);
                    bytes = zstd::stream::encode_all(std::fs::read(&db).unwrap().as_slice(), 0)
                        .unwrap();
                }
                (name, bytes)
            })
            .collect::<Vec<_>>()
    };
    let mut archive = zip::ZipWriter::new(std::fs::File::create(&first).unwrap());
    for (name, bytes) in entries {
        archive
            .start_file(name, zip::write::SimpleFileOptions::default())
            .unwrap();
        archive.write_all(&bytes).unwrap();
    }
    archive.finish().unwrap();
    let unchanged = root.path().join("unchanged.apkg");
    project("B", &[])
        .build(BuildOptions::new().output(&unchanged).compare_to(&first))
        .unwrap();
    assert_eq!(mtimes(&unchanged)["changed"], 1_700_000_000);
    let updated = root.path().join("updated.apkg");
    project("C", &[])
        .build(BuildOptions::new().output(&updated).compare_to(&first))
        .unwrap();
    assert_eq!(mtimes(&updated)["changed"], 1_700_000_001);
}

#[test]
fn overflow_blocks_both_modes_without_publishing_or_replacing_lockfile() {
    let root = tempfile::tempdir().unwrap();
    let lock = root.path().join("identity.json");
    project("A", &[])
        .build(
            BuildOptions::new()
                .output(root.path().join("first.apkg"))
                .first_update_safe_build(&lock),
        )
        .unwrap();
    let mut baseline: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&lock).unwrap()).unwrap();
    baseline["identity_index"]["notes"][0]["revision"]["mtime_secs"] = i64::MAX.into();
    std::fs::write(&lock, serde_json::to_vec(&baseline).unwrap()).unwrap();
    let original = std::fs::read(&lock).unwrap();
    for mode in [UpdateSafetyMode::Strict, UpdateSafetyMode::ReportOnly] {
        let output = root.path().join("blocked.apkg");
        let error = project("B", &[])
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
            .contains(&"UPDATE.NOTE_MTIME_OVERFLOW".into()));
        assert!(!output.exists());
        assert_eq!(std::fs::read(&lock).unwrap(), original);
    }
}

#[test]
fn invalid_revision_is_rejected_by_lockfile_reader() {
    let root = tempfile::tempdir().unwrap();
    let lock = root.path().join("identity.json");
    project("A", &[])
        .build(
            BuildOptions::new()
                .output(root.path().join("first.apkg"))
                .first_update_safe_build(&lock),
        )
        .unwrap();
    let original: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&lock).unwrap()).unwrap();
    for (key, value) in [
        ("mtime_secs", serde_json::json!(0)),
        ("content_hash", serde_json::json!("unsupported:hash")),
    ] {
        let mut invalid = original.clone();
        invalid["identity_index"]["notes"][0]["revision"][key] = value;
        std::fs::write(&lock, serde_json::to_vec(&invalid).unwrap()).unwrap();
        let error = anki_forge::update_safety::lockfile::read_lockfile(&lock).unwrap_err();
        assert!(error.to_string().contains("UPDATE.NOTE_REVISION_INVALID"));
    }
}

#[test]
fn temporarily_absent_note_keeps_revision_history() {
    let root = tempfile::tempdir().unwrap();
    let lock = root.path().join("identity.json");
    let first = root.path().join("first.apkg");
    project("A", &[])
        .build(
            BuildOptions::new()
                .output(&first)
                .first_update_safe_build(&lock),
        )
        .unwrap();
    let second = root.path().join("second.apkg");
    project("B", &[])
        .build(
            BuildOptions::new()
                .output(&second)
                .update_safe(&lock)
                .write_identity_lockfile(true),
        )
        .unwrap();
    let mut missing = Project::new("Revision test").stable_id("revision-test");
    missing
        .add_note(Note::basic("Unchanged question", "Unchanged answer").stable_id("unchanged"))
        .unwrap();
    let third = root.path().join("third.apkg");
    missing
        .build(
            BuildOptions::new()
                .output(&third)
                .update_safe(&lock)
                .write_identity_lockfile(true),
        )
        .unwrap();
    let fourth = root.path().join("fourth.apkg");
    project("C", &[])
        .build(
            BuildOptions::new()
                .output(&fourth)
                .update_safe(&lock)
                .compare_to(&third),
        )
        .unwrap();
    assert_eq!(mtimes(&fourth)["changed"], mtimes(&second)["changed"] + 1);
}

#[test]
#[ignore = "requires ANKI_FORGE_ANKI_PYTHON pointing to an installed Anki Python environment"]
fn real_anki_applies_content_updates_without_changing_identity_or_review_state() {
    let python = std::env::var_os("ANKI_FORGE_ANKI_PYTHON").expect("set ANKI_FORGE_ANKI_PYTHON");
    let root = tempfile::tempdir().unwrap();
    let first = root.path().join("first.apkg");
    let second = root.path().join("second.apkg");
    project("A", &["old-tag"]).write_apkg(&first).unwrap();
    project("B", &["new-tag"])
        .build(BuildOptions::new().output(&second).compare_to(&first))
        .unwrap();
    let script =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/support/note_revision_import_oracle.py");
    let output = std::process::Command::new(python)
        .arg(script)
        .arg(first)
        .arg(second)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "Anki oracle failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    println!("{}", String::from_utf8_lossy(&output.stdout));
}
