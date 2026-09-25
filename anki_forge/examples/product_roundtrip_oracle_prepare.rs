//! Prepares native Project distributions for the real Anki roundtrip oracle.
use ankiforge::{
    tools::{inspect_apkg, load_project},
    BuildOptions,
};
use anyhow::{bail, Context};
use serde::Serialize;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Serialize)]
struct PreparedPackage {
    apkg_path: PathBuf,
    notetype_ids_by_name: BTreeMap<String, String>,
}
#[derive(Serialize)]
struct RoundtripOracleInput {
    label: String,
    first_case: PathBuf,
    second_case: PathBuf,
    first_package: PreparedPackage,
    second_package: PreparedPackage,
}

fn main() -> anyhow::Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let (output, supplied) = match args.as_slice() {
        [output] => (PathBuf::from(output), None),
        [output, first, second, label] => (PathBuf::from(output), Some((PathBuf::from(first), PathBuf::from(second), label.clone()))),
        _ => bail!("usage: product_roundtrip_oracle_prepare <output.json> [first-project.json second-project.json label]"),
    };
    let root = output.parent().context("output needs a parent")?;
    fs::create_dir_all(root)?;
    let (first_case, second_case, label) = if let Some(cases) = supplied {
        cases
    } else {
        let first = root.join("native-v1.json");
        let second = root.join("native-v2.json");
        fs::write(&first, serde_json::to_vec_pretty(&default_input(false))?)?;
        fs::write(&second, serde_json::to_vec_pretty(&default_input(true))?)?;
        (first, second, "native-io-font-roundtrip".into())
    };
    let first_package = prepare(&first_case, &root.join("first.apkg"), None)?;
    let second_package = prepare(
        &second_case,
        &root.join("second.apkg"),
        Some(&first_package.apkg_path),
    )?;
    fs::write(
        &output,
        serde_json::to_vec_pretty(&RoundtripOracleInput {
            label,
            first_case,
            second_case,
            first_package,
            second_package,
        })?,
    )?;
    println!("{}", output.display());
    Ok(())
}

fn prepare(
    input: &Path,
    destination: &Path,
    baseline: Option<&Path>,
) -> anyhow::Result<PreparedPackage> {
    let project = load_project(input, None)?;
    let mut options = BuildOptions::to(destination);
    if let Some(baseline) = baseline {
        options = options.update_from(baseline);
    }
    let output = project.build(options)?;
    let inspected = inspect_apkg(output.artifact().path())?;
    let notetype_ids_by_name = inspected
        .observations
        .notetypes
        .iter()
        .map(|model| {
            Ok((
                model["name"].as_str().context("model name")?.into(),
                model["id"].as_str().context("model id")?.into(),
            ))
        })
        .collect::<anyhow::Result<_>>()?;
    Ok(PreparedPackage {
        apkg_path: output.artifact().path().to_owned(),
        notetype_ids_by_name,
    })
}

fn default_input(updated: bool) -> serde_json::Value {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/public-api");
    serde_json::json!({
        "format_version": "ankiforge-project-v1", "namespace": "native-io-roundtrip",
        "default_deck": "Oracle::Native",
        "assets": [
            {"key":"image", "source":{"kind":"file", "path":fixtures.join("occlusion.png")}},
            {"key":"font", "source":{"kind":"file", "path":fixtures.join("labels.woff")}, "export_as":"labels.woff"}
        ],
        "notes":[{"key":"diagram", "content":{
            "kind":"image_occlusion", "image":"image",
            "masks":[
                {"key":"nucleus", "x":if updated {20} else {10}, "y":10,"width":10,"height":10},
                {"key":"wall", "x":60,"y":10,"width":10,"height":10}
            ],
            "fields":{"header":if updated {"Updated diagram"} else {"Diagram"},
                "back_extra":{"kind":"html", "value":"<style>@font-face{font-family:Labels;src:url(labels.woff)}</style><span style='font-family:Labels'>Cell</span>"}}
        }}]
    })
}
