#![cfg(feature = "internal-tools")]

use anki_forge::update_safety::model::{IdentityIndex, NoteIdentityEntry};
use anki_forge::update_safety::reconcile::{reconcile_guid_plan, GuidSource};

fn index_with_note(source_kind: &str, stable_id: &str, guid: &str) -> IdentityIndex {
    let mut index = IdentityIndex::empty_lockfile("project-a", "writer-policy.default@1.0.0");
    index.source_kind = source_kind.into();
    index.notes.push(NoteIdentityEntry {
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
    });
    index
}

#[test]
fn previous_apkg_wins_over_lockfile_for_same_stable_id() {
    let current = index_with_note("current", "note-a", "note-a");
    let previous = index_with_note("previous_apkg", "note-a", "guid-from-apkg");
    let lockfile = index_with_note("lockfile", "note-a", "guid-from-lockfile");

    let output =
        reconcile_guid_plan(&current, Some(&previous), Some(&lockfile)).expect("reconcile");

    assert_eq!(output.assignments[0].selected_anki_guid, "guid-from-apkg");
    assert_eq!(
        output.assignments[0].source,
        GuidSource::PreviousApkg.as_str()
    );
    assert!(output
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code.as_str() == "UPDATE.BASELINE_CONFLICT_GUID"));
}

#[test]
fn reconcile_rejects_duplicate_selected_guid() {
    let mut current = index_with_note("current", "note-a", "note-a");
    current.notes.push(NoteIdentityEntry {
        stable_id: "note-b".into(),
        normalized_note_id: Some("note-b".into()),
        anki_guid: "note-b".into(),
        current_guid_candidate: "same-guid".into(),
        guid_derivation_version: "guid.raw-stable-id.v1".into(),
        note_type_id: "basic".into(),
        recipe_id: "product.explicit-or-normalized.v1".into(),
        canonical_payload_hash: None,
        provenance: "ExplicitStableId".into(),
        used_override: false,
        entry_lifecycle: "active".into(),
        source_path: "test".into(),
        recovery_method: "current_resolution".into(),
    });
    current.notes[0].current_guid_candidate = "same-guid".into();

    let err = reconcile_guid_plan(&current, None, None).expect_err("duplicate guid");

    assert!(err
        .to_string()
        .contains("UPDATE.GUID_DUPLICATE_AT_RECONCILE"));
}

#[test]
fn reconcile_emits_info_diagnostics_for_guid_sources() {
    let current = index_with_note("current", "note-a", "note-a");
    let previous = index_with_note("previous_apkg", "note-a", "guid-from-apkg");

    let output = reconcile_guid_plan(&current, Some(&previous), None).expect("reconcile");

    assert!(output
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code.as_str() == "UPDATE.GUID_PRESERVED_FROM_PREVIOUS"));

    let output = reconcile_guid_plan(&current, None, None).expect("reconcile current only");
    assert!(output
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code.as_str() == "UPDATE.GUID_DERIVED_FOR_NEW_NOTE"));
}

#[test]
fn reconcile_warns_on_writer_policy_mismatch() {
    let current = index_with_note("current", "note-a", "note-a");
    let mut previous = index_with_note("previous_apkg", "note-a", "guid-from-apkg");
    previous.writer_policy_ref = "writer-policy.legacy@0.9.0".into();

    let output = reconcile_guid_plan(&current, Some(&previous), None).expect("reconcile");

    assert!(output
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code.as_str() == "UPDATE.WRITER_POLICY_MISMATCH"));
}
