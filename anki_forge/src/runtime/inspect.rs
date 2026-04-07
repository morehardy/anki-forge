use std::path::Path;

use crate::InspectReport;

pub fn inspect_staging_path(path: impl AsRef<Path>) -> anyhow::Result<InspectReport> {
    crate::inspect_staging(path)
}

pub fn inspect_apkg_path(path: impl AsRef<Path>) -> anyhow::Result<InspectReport> {
    crate::inspect_apkg(path)
}
