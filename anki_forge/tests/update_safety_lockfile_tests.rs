use anki_forge::update_safety::lockfile::{read_lockfile, write_lockfile_atomic};
use anki_forge::update_safety::model::{
    GeneratedBy, IdentityIndex, IdentityLockfile, NoteIdentityEntry,
};

#[test]
fn lockfile_roundtrip_uses_canonical_json_and_generated_by() {
    let root = tempfile::tempdir().expect("tempdir");
    let path = root.path().join("anki-forge.lock.json");
    let lockfile = IdentityLockfile {
        schema_version: "identity-lockfile-v1".into(),
        project_stable_id: "project-a".into(),
        writer_policy_ref: "writer-policy.default@1.0.0".into(),
        identity_index: IdentityIndex::empty_lockfile("project-a", "writer-policy.default@1.0.0"),
        generated_by: GeneratedBy {
            tool: "anki-forge".into(),
            tool_version: env!("CARGO_PKG_VERSION").into(),
            writer_policy_ref: "writer-policy.default@1.0.0".into(),
        },
    };

    write_lockfile_atomic(&path, &lockfile).expect("write lockfile");
    let raw = std::fs::read_to_string(&path).expect("read raw");
    assert!(raw.starts_with("{\"generated_by\""));

    let loaded = read_lockfile(&path).expect("read lockfile");
    assert_eq!(loaded.project_stable_id, "project-a");
    assert_eq!(loaded.generated_by.tool, "anki-forge");
}

#[test]
fn lockfile_rejects_unknown_schema_version() {
    let root = tempfile::tempdir().expect("tempdir");
    let path = root.path().join("anki-forge.lock.json");
    std::fs::write(
        &path,
        r#"{"schema_version":"identity-lockfile-v99","project_stable_id":"p","writer_policy_ref":"writer-policy.default@1.0.0","identity_index":{"schema_version":"identity-index-v1","source_kind":"lockfile","source_ref":"baseline.identity_lockfile.primary","writer_policy_ref":"writer-policy.default@1.0.0","project_stable_id":"p","notes":[],"notetypes":[],"limitations":[]},"generated_by":{"tool":"anki-forge","tool_version":"0.0.0","writer_policy_ref":"writer-policy.default@1.0.0"}}"#,
    )
    .expect("write invalid lockfile");

    let err = read_lockfile(&path).expect_err("schema should fail");
    assert!(err
        .to_string()
        .contains("UPDATE.BASELINE_SCHEMA_UNSUPPORTED"));
}

#[test]
fn lockfile_rejects_duplicate_stable_id_and_guid() {
    let mut lockfile = IdentityLockfile {
        schema_version: "identity-lockfile-v1".into(),
        project_stable_id: "project-a".into(),
        writer_policy_ref: "writer-policy.default@1.0.0".into(),
        identity_index: IdentityIndex::empty_lockfile("project-a", "writer-policy.default@1.0.0"),
        generated_by: GeneratedBy {
            tool: "anki-forge".into(),
            tool_version: env!("CARGO_PKG_VERSION").into(),
            writer_policy_ref: "writer-policy.default@1.0.0".into(),
        },
    };
    lockfile
        .identity_index
        .notes
        .push(note_entry("stable-a", "guid-a"));
    lockfile
        .identity_index
        .notes
        .push(note_entry("stable-a", "guid-b"));

    let root = tempfile::tempdir().expect("tempdir");
    let path = root.path().join("duplicate-stable.lock.json");
    let err = write_lockfile_atomic(&path, &lockfile).expect_err("duplicate stable id");
    assert!(err
        .to_string()
        .contains("UPDATE.STABLE_ID_DUPLICATE_IN_BASELINE"));

    lockfile.identity_index.notes.clear();
    lockfile
        .identity_index
        .notes
        .push(note_entry("stable-a", "guid-a"));
    lockfile
        .identity_index
        .notes
        .push(note_entry("stable-b", "guid-a"));
    let err = write_lockfile_atomic(&path, &lockfile).expect_err("duplicate guid");
    assert!(err
        .to_string()
        .contains("UPDATE.GUID_DUPLICATE_IN_BASELINE"));
}

#[test]
fn lockfile_rejects_active_normalized_note_id_mismatch() {
    let mut lockfile = IdentityLockfile {
        schema_version: "identity-lockfile-v1".into(),
        project_stable_id: "project-a".into(),
        writer_policy_ref: "writer-policy.default@1.0.0".into(),
        identity_index: IdentityIndex::empty_lockfile("project-a", "writer-policy.default@1.0.0"),
        generated_by: GeneratedBy {
            tool: "anki-forge".into(),
            tool_version: env!("CARGO_PKG_VERSION").into(),
            writer_policy_ref: "writer-policy.default@1.0.0".into(),
        },
    };
    let mut note = note_entry("stable-a", "guid-a");
    note.normalized_note_id = Some("different-normalized-id".into());
    lockfile.identity_index.notes.push(note);

    let root = tempfile::tempdir().expect("tempdir");
    let path = root.path().join("mismatch.lock.json");
    let err = write_lockfile_atomic(&path, &lockfile).expect_err("mismatch");
    assert!(err
        .to_string()
        .contains("UPDATE.NORMALIZED_NOTE_ID_MISMATCH"));
}

fn note_entry(stable_id: &str, guid: &str) -> NoteIdentityEntry {
    NoteIdentityEntry {
        stable_id: stable_id.into(),
        normalized_note_id: Some(stable_id.into()),
        anki_guid: guid.into(),
        current_guid_candidate: stable_id.into(),
        guid_derivation_version: "guid.raw-stable-id.v1".into(),
        note_type_id: "basic".into(),
        recipe_id: "product.explicit-or-normalized.v1".into(),
        canonical_payload_hash: None,
        provenance: "ExplicitStableId".into(),
        used_override: false,
        entry_lifecycle: "active".into(),
        source_path: "test".into(),
        recovery_method: "current_resolution".into(),
    }
}

#[test]
#[ignore = "manual performance boundary check"]
fn lockfile_parse_scale_100k_entries() {
    let root = tempfile::tempdir().expect("tempdir");
    let path = root.path().join("large.lock.json");
    let mut lockfile = sample_lockfile_with_entries(100_000);
    let start = std::time::Instant::now();
    write_lockfile_atomic(&path, &lockfile).expect("write large lockfile");
    let write_elapsed = start.elapsed();
    let start = std::time::Instant::now();
    lockfile = read_lockfile(&path).expect("read large lockfile");
    let read_elapsed = start.elapsed();
    assert_eq!(lockfile.identity_index.notes.len(), 100_000);
    eprintln!("write={write_elapsed:?} read={read_elapsed:?}");
}

fn sample_lockfile_with_entries(count: usize) -> IdentityLockfile {
    let mut lockfile = IdentityLockfile {
        schema_version: "identity-lockfile-v1".into(),
        project_stable_id: "scale-project".into(),
        writer_policy_ref: "writer-policy.default@1.0.0".into(),
        identity_index: IdentityIndex::empty_lockfile(
            "scale-project",
            "writer-policy.default@1.0.0",
        ),
        generated_by: GeneratedBy {
            tool: "anki-forge".into(),
            tool_version: env!("CARGO_PKG_VERSION").into(),
            writer_policy_ref: "writer-policy.default@1.0.0".into(),
        },
    };
    lockfile.identity_index.notes = (0..count)
        .map(|index| bench_note_entry(&format!("note-{index:06}"), &format!("guid-{index:06}")))
        .collect();
    lockfile
}

fn bench_note_entry(stable_id: &str, guid: &str) -> NoteIdentityEntry {
    NoteIdentityEntry {
        stable_id: stable_id.into(),
        normalized_note_id: Some(stable_id.into()),
        anki_guid: guid.into(),
        current_guid_candidate: stable_id.into(),
        guid_derivation_version: "guid.raw-stable-id.v1".into(),
        note_type_id: "basic".into(),
        recipe_id: "product.explicit-or-normalized.v1".into(),
        canonical_payload_hash: None,
        provenance: "ExplicitStableId".into(),
        used_override: false,
        entry_lifecycle: "active".into(),
        source_path: "benchmark".into(),
        recovery_method: "current_resolution".into(),
    }
}
