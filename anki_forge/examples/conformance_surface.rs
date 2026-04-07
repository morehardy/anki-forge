use std::{
    env, fs,
    path::{Path, PathBuf},
};

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
    cwd: Option<PathBuf>,
    #[serde(rename = "manifestPath")]
    manifest_path: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct NormalizeRequest {
    #[serde(rename = "inputPath")]
    input_path: PathBuf,
}

#[derive(Debug, Deserialize)]
struct BuildRequest {
    #[serde(rename = "inputPath")]
    input_path: PathBuf,
    #[serde(rename = "writerPolicy", default = "default_selector")]
    writer_policy: String,
    #[serde(rename = "buildContext", default = "default_selector")]
    build_context: String,
    #[serde(rename = "artifactsDir")]
    artifacts_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
struct InspectRequest {
    #[serde(rename = "stagingPath")]
    staging_path: Option<PathBuf>,
    #[serde(rename = "apkgPath")]
    apkg_path: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct DiffRequest {
    #[serde(rename = "leftPath")]
    left_path: PathBuf,
    #[serde(rename = "rightPath")]
    right_path: PathBuf,
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

fn resolved_cwd(options: &RuntimeOptions) -> anyhow::Result<PathBuf> {
    let current_dir = env::current_dir().context("resolve current working directory")?;
    Ok(match &options.cwd {
        Some(cwd) if cwd.is_absolute() => cwd.clone(),
        Some(cwd) => current_dir.join(cwd),
        None => default_workspace_start(),
    })
}

fn resolve_request_path(cwd: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    }
}

fn resolve_runtime(
    options: &RuntimeOptions,
) -> anyhow::Result<anki_forge::runtime::ResolvedRuntime> {
    let cwd = resolved_cwd(options)?;
    if let Some(manifest_path) = &options.manifest_path {
        let manifest_path = resolve_request_path(&cwd, manifest_path);
        return anki_forge::runtime::load_bundle_from_manifest(manifest_path)
            .map(|bundle| bundle.runtime);
    }

    anki_forge::runtime::discover_workspace_runtime(cwd)
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
    let cwd = resolved_cwd(&envelope.runtime_options)?;

    match envelope.command.as_str() {
        "normalize" => {
            let request: NormalizeRequest = serde_json::from_value(envelope.request)
                .context("normalize request is invalid")?;
            let runtime = resolve_runtime(&envelope.runtime_options)?;
            let input_path = resolve_request_path(&cwd, &request.input_path);
            let result = anki_forge::runtime::normalize_from_path(&runtime, &input_path)?;
            print_json(anki_forge::to_authoring_canonical_json(&result)?);
        }
        "build" => {
            let request: BuildRequest = serde_json::from_value(envelope.request)
                .context("build request is invalid")?;
            let runtime = resolve_runtime(&envelope.runtime_options)?;
            let input_path = resolve_request_path(&cwd, &request.input_path);
            let artifacts_dir = resolve_request_path(&cwd, &request.artifacts_dir);
            let result = anki_forge::runtime::build_from_path(
                &runtime,
                &input_path,
                &request.writer_policy,
                &request.build_context,
                &artifacts_dir,
            )?;
            print_json(anki_forge::to_writer_canonical_json(&result)?);
        }
        "inspect" => {
            let request: InspectRequest = serde_json::from_value(envelope.request)
                .context("inspect request is invalid")?;
            let result = match (request.staging_path, request.apkg_path) {
                (Some(path), None) => {
                    anki_forge::runtime::inspect_staging_path(resolve_request_path(&cwd, &path))?
                }
                (None, Some(path)) => {
                    anki_forge::runtime::inspect_apkg_path(resolve_request_path(&cwd, &path))?
                }
                _ => bail!("inspect request requires exactly one of staging or apkg"),
            };
            print_json(anki_forge::to_writer_canonical_json(&result)?);
        }
        "diff" => {
            let request: DiffRequest = serde_json::from_value(envelope.request)
                .context("diff request is invalid")?;
            let left_path = resolve_request_path(&cwd, &request.left_path);
            let right_path = resolve_request_path(&cwd, &request.right_path);
            let result = anki_forge::runtime::diff_from_paths(&left_path, &right_path)?;
            print_json(anki_forge::to_writer_canonical_json(&result)?);
        }
        other => bail!("unsupported command: {other}"),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_envelope_accepts_planned_task8_field_names() {
        let envelope: ConformanceEnvelope = serde_json::from_value(serde_json::json!({
            "command": "build",
            "request": {
                "inputPath": "fixtures/input.json",
                "artifactsDir": "tmp/artifacts",
                "writerPolicy": "default",
                "buildContext": "default"
            },
            "runtimeOptions": {
                "cwd": "/tmp/workspace"
            }
        }))
        .expect("decode conformance envelope");

        let request: BuildRequest =
            serde_json::from_value(envelope.request).expect("decode build request");

        assert_eq!(request.input_path, PathBuf::from("fixtures/input.json"));
        assert_eq!(request.artifacts_dir, PathBuf::from("tmp/artifacts"));
        assert_eq!(
            envelope.runtime_options.cwd,
            Some(PathBuf::from("/tmp/workspace"))
        );
    }

    #[test]
    fn relative_request_paths_resolve_against_runtime_cwd() {
        let cwd = PathBuf::from("/tmp/runtime-root");

        assert_eq!(
            resolve_request_path(&cwd, Path::new("fixtures/input.json")),
            PathBuf::from("/tmp/runtime-root/fixtures/input.json")
        );
        assert_eq!(
            resolve_request_path(&cwd, Path::new("/tmp/absolute.json")),
            PathBuf::from("/tmp/absolute.json")
        );
    }
}
