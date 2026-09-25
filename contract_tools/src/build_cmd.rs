use anyhow::bail;

pub fn run(
    manifest: &str,
    input: &str,
    writer_policy: &str,
    build_context: &str,
    artifacts_dir: &str,
    output: &str,
) -> anyhow::Result<String> {
    let runtime = ankiforge::tools::load_runtime(manifest)?;
    let result = ankiforge::tools::build_from_path(
        &runtime,
        input,
        writer_policy,
        build_context,
        artifacts_dir,
    )?;

    match output {
        "contract-json" => ankiforge::tools::canonical_json(&result),
        "human" => Ok(format!("status: {}", result.result_status)),
        other => bail!("unsupported build output mode: {other}"),
    }
}
