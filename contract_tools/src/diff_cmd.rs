pub fn run(left: &str, right: &str, output: &str) -> anyhow::Result<String> {
    let diff = anki_forge::runtime::diff_from_paths(left, right)?;

    match output {
        "contract-json" => anki_forge::to_writer_canonical_json(&diff),
        "human" => Ok(format!("status: {}", diff.comparison_status)),
        other => anyhow::bail!("unsupported diff output mode: {other}"),
    }
}
