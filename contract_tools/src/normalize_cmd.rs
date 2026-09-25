use anyhow::bail;

pub fn run(manifest: &str, input: &str, output: &str) -> anyhow::Result<String> {
    let runtime = ankiforge::tools::load_runtime(manifest)?;
    let result = ankiforge::tools::normalize_from_path(&runtime, input)?;

    match output {
        "contract-json" => ankiforge::tools::canonical_json(&result),
        "human" => Ok(format!("status: {}", result.result_status)),
        other => bail!("unsupported normalize output mode: {other}"),
    }
}
