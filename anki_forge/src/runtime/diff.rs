use std::{fs, path::Path};

use anyhow::Context;

use crate::{diff_reports, DiffReport, InspectReport};

pub fn diff_from_paths(
    left_path: impl AsRef<Path>,
    right_path: impl AsRef<Path>,
) -> anyhow::Result<DiffReport> {
    let left_path = left_path.as_ref();
    let right_path = right_path.as_ref();

    let left: InspectReport = serde_json::from_str(
        &fs::read_to_string(left_path)
            .with_context(|| format!("failed to read left report: {}", left_path.display()))?,
    )
    .context("left report must be valid inspect-report JSON")?;
    let right: InspectReport = serde_json::from_str(
        &fs::read_to_string(right_path)
            .with_context(|| format!("failed to read right report: {}", right_path.display()))?,
    )
    .context("right report must be valid inspect-report JSON")?;

    diff_reports(&left, &right)
}
