use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use anki_forge::{normalize, product::ProductDocument, NormalizationRequest};
use anyhow::{bail, ensure, Context};
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize)]
struct RoundtripOracleInput {
    label: String,
    first_case: PathBuf,
    second_case: PathBuf,
    first_package: PreparedPackage,
    second_package: PreparedPackage,
}

#[derive(Debug, Serialize)]
struct PreparedPackage {
    apkg_path: PathBuf,
    notetype_ids_by_name: BTreeMap<String, String>,
}

fn main() -> anyhow::Result<()> {
    let (output_path, first_case, second_case, label) = parse_args()?;
    let work_root = output_path
        .parent()
        .context("output path must have a parent directory")?;
    fs::create_dir_all(work_root)
        .with_context(|| format!("create oracle work root {}", work_root.display()))?;

    let first_package =
        build_phase5a_case_apkg(&first_case, &label, &work_root.join("first-package"), "v1")?;
    let second_package = build_phase5a_case_apkg(
        &second_case,
        &label,
        &work_root.join("second-package"),
        "v2",
    )?;

    let input = RoundtripOracleInput {
        label,
        first_case,
        second_case,
        first_package,
        second_package,
    };

    fs::write(&output_path, serde_json::to_string_pretty(&input)?)
        .with_context(|| format!("write oracle input {}", output_path.display()))?;

    println!("{}", output_path.display());

    Ok(())
}

fn parse_args() -> anyhow::Result<(PathBuf, PathBuf, PathBuf, String)> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    match args.as_slice() {
        [output_path] => Ok((
            PathBuf::from(output_path),
            repo_root().join("anki_forge/tests/fixtures/product/io_font_bundle_roundtrip_v1.case.json"),
            repo_root().join("anki_forge/tests/fixtures/product/io_font_bundle_roundtrip_v2.case.json"),
            "phase5a-io-font-roundtrip".into(),
        )),
        [output_path, first_case, second_case, label] => Ok((
            PathBuf::from(output_path),
            resolve_repo_relative(first_case),
            resolve_repo_relative(second_case),
            label.clone(),
        )),
        _ => bail!(
            "usage: cargo run -p anki_forge --example product_roundtrip_oracle_prepare -- <output.json> [first_case second_case label]"
        ),
    }
}

fn build_phase5a_case_apkg(
    case_path: &Path,
    label: &str,
    artifact_root: &Path,
    suffix: &str,
) -> anyhow::Result<PreparedPackage> {
    let raw = fs::read_to_string(case_path)
        .with_context(|| format!("read phase5a product fixture {}", case_path.display()))?;
    let document: ProductDocument = serde_json::from_str(&raw)
        .with_context(|| format!("decode phase5a product fixture {}", case_path.display()))?;

    let lowering = document.lower().map_err(|err| {
        anyhow::anyhow!(
            "lower phase5a product fixture {}: {:?}",
            case_path.display(),
            err
        )
    })?;
    let normalized = normalize(NormalizationRequest::new(lowering.authoring_document));
    let normalized_ir = normalized
        .normalized_ir
        .context("phase5a product fixture must normalize successfully")?;

    if artifact_root.exists() {
        fs::remove_dir_all(artifact_root)
            .with_context(|| format!("remove old artifact root {}", artifact_root.display()))?;
    }

    let target = writer_core::BuildArtifactTarget::new(
        artifact_root.to_path_buf(),
        format!("artifacts/phase5a-roundtrip/{label}-{suffix}"),
    );
    let build_result = writer_core::build(
        &normalized_ir,
        &phase5a_writer_policy(),
        &phase5a_build_context(),
        &target,
    )
    .with_context(|| format!("build phase5a product fixture {}", case_path.display()))?;
    ensure!(
        build_result.result_status == "success",
        "phase5a roundtrip build must succeed for {}: {:?}",
        case_path.display(),
        build_result.diagnostics
    );
    let apkg_ref = build_result
        .apkg_ref
        .as_deref()
        .context("phase5a roundtrip build must emit an apkg_ref")?;
    let apkg_path = artifact_path_from_ref(&target, apkg_ref);
    let inspect_report = writer_core::inspect_apkg(&apkg_path)
        .with_context(|| format!("inspect phase5a apkg {}", apkg_path.display()))?;

    Ok(PreparedPackage {
        apkg_path,
        notetype_ids_by_name: inspect_notetype_ids_by_name(&inspect_report)?,
    })
}

fn inspect_notetype_ids_by_name(
    inspect_report: &writer_core::InspectReport,
) -> anyhow::Result<BTreeMap<String, String>> {
    let mut ids_by_name = BTreeMap::new();
    for notetype in &inspect_report.observations.notetypes {
        let name = required_str_field(notetype, "name")?.to_string();
        let id = required_str_field(notetype, "id")?.to_string();
        ensure!(
            ids_by_name.insert(name.clone(), id).is_none(),
            "phase5a oracle requires unique notetype names, found duplicate '{}'",
            name
        );
    }
    Ok(ids_by_name)
}

fn artifact_path_from_ref(target: &writer_core::BuildArtifactTarget, reference: &str) -> PathBuf {
    let prefix = target.stable_ref_prefix.trim_end_matches('/');
    let trimmed = reference
        .strip_prefix(prefix)
        .unwrap_or(reference)
        .trim_start_matches('/');
    if trimmed.is_empty() {
        target.root_dir.clone()
    } else {
        target.root_dir.join(trimmed)
    }
}

fn required_str_field<'a>(value: &'a Value, field: &str) -> anyhow::Result<&'a str> {
    value
        .get(field)
        .and_then(Value::as_str)
        .with_context(|| format!("missing string field {}", field))
}

fn resolve_repo_relative(path: &str) -> PathBuf {
    let candidate = PathBuf::from(path);
    if candidate.is_absolute() {
        candidate
    } else {
        repo_root().join(candidate)
    }
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("repo root")
        .to_path_buf()
}

fn phase5a_writer_policy() -> writer_core::WriterPolicy {
    writer_core::WriterPolicy {
        id: "writer-policy.default".into(),
        version: "1.0.0".into(),
        compatibility_target: "latest-only".into(),
        stock_notetype_mode: "source-grounded".into(),
        media_entry_mode: "inline".into(),
        apkg_version: "latest".into(),
    }
}

fn phase5a_build_context() -> writer_core::BuildContext {
    writer_core::BuildContext {
        id: "build-context.default".into(),
        version: "1.0.0".into(),
        emit_apkg: true,
        materialize_staging: true,
        media_resolution_mode: "inline-only".into(),
        unresolved_asset_behavior: "fail".into(),
        fingerprint_mode: "canonical".into(),
    }
}
