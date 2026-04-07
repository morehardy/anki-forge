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

    let input_path = repo_root.join("contracts/fixtures/phase3/inputs/basic-authoring-ir.json");
    let normalized_ir_path = repo_root.join("tmp/phase4-examples/minimal-flow/normalized-ir.json");
    let artifacts_dir = repo_root.join("tmp/phase4-examples/minimal-flow/artifacts");

    if let Some(parent) = normalized_ir_path.parent() {
        fs::create_dir_all(parent)?;
    }
    if artifacts_dir.exists() {
        fs::remove_dir_all(&artifacts_dir)?;
    }

    let normalized = anki_forge::runtime::normalize_from_path(&runtime, &input_path)?;
    let normalized_ir = normalized
        .normalized_ir
        .as_ref()
        .expect("basic authoring fixture should normalize");
    fs::write(&normalized_ir_path, serde_json::to_string_pretty(normalized_ir)?)?;

    let build = anki_forge::runtime::build_from_path(
        &runtime,
        &normalized_ir_path,
        "default",
        "default",
        &artifacts_dir,
    )?;
    let staging_path = artifacts_dir.join("staging/manifest.json");
    let apkg_path = artifacts_dir.join("package.apkg");
    let inspect = anki_forge::runtime::inspect_apkg_path(&apkg_path)?;

    println!("runtime.mode={:?}", runtime.mode);
    println!("runtime.bundle_version={}", runtime.bundle_version);
    println!("runtime.manifest_path={}", runtime.manifest_path.display());
    println!("input_path={}", input_path.display());
    println!("normalized_ir_path={}", normalized_ir_path.display());
    println!("artifacts_dir={}", artifacts_dir.display());
    println!("staging_path={}", staging_path.display());
    println!("apkg_path={}", apkg_path.display());
    println!("normalize.status={}", normalized.result_status);
    println!("build.status={}", build.result_status);
    println!("inspect.status={}", inspect.observation_status);

    Ok(())
}
