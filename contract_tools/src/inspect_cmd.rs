use anyhow::bail;

pub fn run(staging: Option<&str>, apkg: Option<&str>, output: &str) -> anyhow::Result<String> {
    let report = match (staging, apkg) {
        (Some(path), None) => ankiforge::tools::inspect_staging_path(path)?,
        (None, Some(path)) => ankiforge::tools::inspect_apkg_path(path)?,
        _ => bail!("inspect requires exactly one of --staging or --apkg"),
    };

    match output {
        "contract-json" => ankiforge::tools::canonical_json(&report),
        "human" => Ok(format!("status: {}", report.observation_status)),
        other => bail!("unsupported inspect output mode: {other}"),
    }
}
