use std::path::Path;

use crate::writer::{inspect_apkg, inspect_staging, InspectReport};

pub fn inspect_staging_path(path: impl AsRef<Path>) -> anyhow::Result<InspectReport> {
    inspect_staging(path)
}

pub fn inspect_apkg_path(path: impl AsRef<Path>) -> anyhow::Result<InspectReport> {
    inspect_apkg(path)
}
