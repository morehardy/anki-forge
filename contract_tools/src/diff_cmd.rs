use anyhow::Context;
use std::fs;

pub fn run(left: &str, right: &str, output: &str) -> anyhow::Result<String> {
    let left: writer_core::InspectReport = serde_json::from_str(
        &fs::read_to_string(left).with_context(|| format!("failed to read left report: {left}"))?,
    )
    .context("left report must be valid inspect-report JSON")?;
    let right: writer_core::InspectReport = serde_json::from_str(
        &fs::read_to_string(right)
            .with_context(|| format!("failed to read right report: {right}"))?,
    )
    .context("right report must be valid inspect-report JSON")?;
    let diff = writer_core::diff_reports(&left, &right)?;

    match output {
        "contract-json" => writer_core::to_canonical_json(&diff),
        "human" => Ok(format!("status: {}", diff.comparison_status)),
        other => anyhow::bail!("unsupported diff output mode: {other}"),
    }
}
