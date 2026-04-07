use std::{env, fs, path::PathBuf};

use anyhow::{bail, Context};
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
struct ConformanceEnvelope {
    command: String,
    request: Value,
    #[serde(rename = "runtimeOptions", default)]
    runtime_options: RuntimeOptions,
}

#[derive(Debug, Default, Deserialize)]
struct RuntimeOptions {
    #[serde(rename = "manifestPath")]
    manifest_path: Option<PathBuf>,
    #[serde(rename = "workspaceStart")]
    workspace_start: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct NormalizeRequest {
    input: PathBuf,
}

#[derive(Debug, Deserialize)]
struct BuildRequest {
    input: PathBuf,
    #[serde(rename = "writerPolicy", default = "default_selector")]
    writer_policy: String,
    #[serde(rename = "buildContext", default = "default_selector")]
    build_context: String,
    #[serde(rename = "artifactsDir")]
    artifacts_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
struct InspectRequest {
    staging: Option<PathBuf>,
    apkg: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct DiffRequest {
    left: PathBuf,
    right: PathBuf,
}

fn default_selector() -> String {
    "default".into()
}

fn default_workspace_start() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn resolve_runtime(options: &RuntimeOptions) -> anyhow::Result<anki_forge::runtime::ResolvedRuntime> {
    if let Some(manifest_path) = &options.manifest_path {
        return anki_forge::runtime::load_bundle_from_manifest(manifest_path).map(|bundle| bundle.runtime);
    }

    let workspace_start = options
        .workspace_start
        .clone()
        .unwrap_or_else(default_workspace_start);
    anki_forge::runtime::discover_workspace_runtime(workspace_start)
}

fn print_json(text: String) {
    println!("{text}");
}

fn main() -> anyhow::Result<()> {
    let path = env::args()
        .nth(1)
        .context("usage: cargo run -p anki_forge --example conformance_surface -- <request.json>")?;
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("failed to read request file: {path}"))?;
    let envelope: ConformanceEnvelope =
        serde_json::from_str(&raw).context("request file must be valid JSON")?;

    match envelope.command.as_str() {
        "normalize" => {
            let request: NormalizeRequest = serde_json::from_value(envelope.request)
                .context("normalize request is invalid")?;
            let runtime = resolve_runtime(&envelope.runtime_options)?;
            let result = anki_forge::runtime::normalize_from_path(&runtime, &request.input)?;
            print_json(anki_forge::to_authoring_canonical_json(&result)?);
        }
        "build" => {
            let request: BuildRequest = serde_json::from_value(envelope.request)
                .context("build request is invalid")?;
            let runtime = resolve_runtime(&envelope.runtime_options)?;
            let result = anki_forge::runtime::build_from_path(
                &runtime,
                &request.input,
                &request.writer_policy,
                &request.build_context,
                &request.artifacts_dir,
            )?;
            print_json(anki_forge::to_writer_canonical_json(&result)?);
        }
        "inspect" => {
            let request: InspectRequest = serde_json::from_value(envelope.request)
                .context("inspect request is invalid")?;
            let result = match (request.staging, request.apkg) {
                (Some(path), None) => anki_forge::runtime::inspect_staging_path(path)?,
                (None, Some(path)) => anki_forge::runtime::inspect_apkg_path(path)?,
                _ => bail!("inspect request requires exactly one of staging or apkg"),
            };
            print_json(anki_forge::to_writer_canonical_json(&result)?);
        }
        "diff" => {
            let request: DiffRequest = serde_json::from_value(envelope.request)
                .context("diff request is invalid")?;
            let result = anki_forge::runtime::diff_from_paths(&request.left, &request.right)?;
            print_json(anki_forge::to_writer_canonical_json(&result)?);
        }
        other => bail!("unsupported command: {other}"),
    }

    Ok(())
}
