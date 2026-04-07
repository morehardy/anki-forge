use anyhow::bail;

pub fn run(
    manifest: &str,
    input: &str,
    writer_policy: &str,
    build_context: &str,
    artifacts_dir: &str,
    output: &str,
) -> anyhow::Result<String> {
    let runtime = anki_forge::runtime::load_bundle_from_manifest(manifest)?.runtime;
    let result = anki_forge::runtime::build_from_path(
        &runtime,
        input,
        writer_policy,
        build_context,
        artifacts_dir,
    )?;

    match output {
        "contract-json" => anki_forge::to_writer_canonical_json(&result),
        "human" => Ok(format!("status: {}", result.result_status)),
        other => bail!("unsupported build output mode: {other}"),
    }
}
