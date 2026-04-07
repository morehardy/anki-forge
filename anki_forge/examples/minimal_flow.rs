use std::{fs, path::PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn main() -> anyhow::Result<()> {
    let repo_root = repo_root();
    let runtime = anki_forge::runtime::discover_workspace_runtime(&repo_root)?;

    let authoring_input = repo_root.join("contracts/fixtures/phase3/inputs/basic-authoring-ir.json");
    let normalized_output = repo_root.join("tmp/phase4-examples/minimal-flow/normalized-ir.json");
    let artifacts_dir = repo_root.join("tmp/phase4-examples/minimal-flow/artifacts");

    if let Some(parent) = normalized_output.parent() {
        fs::create_dir_all(parent)?;
    }
    if artifacts_dir.exists() {
        fs::remove_dir_all(&artifacts_dir)?;
    }

    let normalized =
        anki_forge::runtime::normalize_from_path(&runtime, &authoring_input)?;
    let normalized_ir = normalized
        .normalized_ir
        .as_ref()
        .expect("basic authoring fixture should normalize");
    fs::write(
        &normalized_output,
        serde_json::to_string_pretty(normalized_ir)?,
    )?;

    let build = anki_forge::runtime::build_from_path(
        &runtime,
        &normalized_output,
        "default",
        "default",
        &artifacts_dir,
    )?;
    let inspect = anki_forge::runtime::inspect_apkg_path(artifacts_dir.join("package.apkg"))?;

    println!("runtime.mode={:?}", runtime.mode);
    println!("runtime.bundle_version={}", runtime.bundle_version);
    println!("runtime.manifest_path={}", runtime.manifest_path.display());
    println!("normalize.status={}", normalized.result_status);
    println!("build.status={}", build.result_status);
    println!("inspect.status={}", inspect.observation_status);

    Ok(())
}
