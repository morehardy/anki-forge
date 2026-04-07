use anyhow::bail;

pub fn run(manifest: &str, input: &str, output: &str) -> anyhow::Result<String> {
    let runtime = anki_forge::runtime::load_bundle_from_manifest(manifest)?.runtime;
    let result = anki_forge::runtime::normalize_from_path(&runtime, input)?;

    match output {
        "contract-json" => anki_forge::to_authoring_canonical_json(&result),
        "human" => Ok(format!("status: {}", result.result_status)),
        other => bail!("unsupported normalize output mode: {other}"),
    }
}
