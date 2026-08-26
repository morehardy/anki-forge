use std::collections::BTreeSet;
use std::path::Path;

use anyhow::{Context, Result};

use super::model::IdentityLockfile;

pub fn read_lockfile(path: impl AsRef<Path>) -> Result<IdentityLockfile> {
    let path = path.as_ref();
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("read identity lockfile {}", path.display()))?;
    let lockfile: IdentityLockfile = serde_json::from_str(&raw)
        .with_context(|| format!("parse identity lockfile {}", path.display()))?;
    validate_lockfile(&lockfile)?;
    Ok(lockfile)
}

pub fn write_lockfile_atomic(path: impl AsRef<Path>, lockfile: &IdentityLockfile) -> Result<()> {
    let path = path.as_ref();
    validate_lockfile(lockfile)?;
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(parent)
        .with_context(|| format!("create lockfile directory {}", parent.display()))?;
    let tmp = parent.join(format!(
        ".{}.tmp-{}",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("anki-forge.lock.json"),
        std::process::id()
    ));
    let bytes = crate::writer_core::to_canonical_json(lockfile)
        .context("serialize canonical identity lockfile")?;
    std::fs::write(&tmp, bytes)
        .with_context(|| format!("write temporary lockfile {}", tmp.display()))?;
    std::fs::rename(&tmp, path)
        .with_context(|| format!("replace identity lockfile {}", path.display()))?;
    Ok(())
}

fn validate_lockfile(lockfile: &IdentityLockfile) -> Result<()> {
    anyhow::ensure!(
        lockfile.schema_version == "identity-lockfile-v1",
        "UPDATE.BASELINE_SCHEMA_UNSUPPORTED: {}",
        lockfile.schema_version
    );
    anyhow::ensure!(
        !lockfile.project_stable_id.trim().is_empty(),
        "UPDATE.PROJECT_STABLE_ID_MISSING"
    );
    anyhow::ensure!(
        lockfile.generated_by.tool == "anki-forge",
        "invalid generated_by.tool"
    );
    let mut stable_ids = BTreeSet::new();
    let mut anki_guids = BTreeSet::new();
    for note in &lockfile.identity_index.notes {
        if note.entry_lifecycle == "active" {
            anyhow::ensure!(
                note.normalized_note_id.as_deref() == Some(note.stable_id.as_str()),
                "UPDATE.NORMALIZED_NOTE_ID_MISMATCH: stable_id={} normalized_note_id={:?}",
                note.stable_id,
                note.normalized_note_id
            );
        }
        anyhow::ensure!(
            stable_ids.insert(note.stable_id.as_str()),
            "UPDATE.STABLE_ID_DUPLICATE_IN_BASELINE: {}",
            note.stable_id
        );
        anyhow::ensure!(
            anki_guids.insert(note.anki_guid.as_str()),
            "UPDATE.GUID_DUPLICATE_IN_BASELINE: {}",
            note.anki_guid
        );
    }
    Ok(())
}
