use anyhow::bail;

pub fn run(staging: Option<&str>, apkg: Option<&str>, output: &str) -> anyhow::Result<String> {
    let report = match (staging, apkg) {
        (Some(path), None) => writer_core::inspect_staging(path)?,
        (None, Some(path)) => writer_core::inspect_apkg(path)?,
        _ => bail!("inspect requires exactly one of --staging or --apkg"),
    };

    match output {
        "contract-json" => writer_core::to_canonical_json(&report),
        "human" => Ok(format!("status: {}", report.observation_status)),
        other => bail!("unsupported inspect output mode: {other}"),
    }
}
