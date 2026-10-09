//! Internal-tools and concurrent-miss controls; never included in SDK builds.
use ankiforge::{BuildOptions, Content, Media, Note, Project};
use anyhow::{ensure, Context};
use std::{
    path::Path,
    sync::{Arc, Barrier},
    time::Instant,
};

fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let [input, output, mode, ..] = args.as_slice() else {
        anyhow::bail!("usage: publication-control INPUT OUTPUT MODE");
    };
    let started = Instant::now();
    let document: serde_json::Value = serde_json::from_slice(&std::fs::read(input)?)?;
    let notes = document["notes"].as_array().context("notes")?;
    if mode == "staging" {
        use ankiforge::tools::{BuildArtifactTarget, BuildContext, NormalizedIr, WriterPolicy};
        let mut ir: NormalizedIr = serde_json::from_str(include_str!(
            "../../../../contracts/fixtures/phase3/inputs/basic-normalized-ir.json"
        ))?;
        let template = ir.notes.remove(0);
        ir.notes = notes
            .iter()
            .enumerate()
            .map(|(index, source)| {
                let mut note = template.clone();
                note.id = format!("record.{index}");
                note.deck_name = document["deck_name"].as_str().unwrap().into();
                note.tags.clear();
                for (field, name) in [("front", "Front"), ("back", "Back")] {
                    note.fields.insert(
                        name.into(),
                        html_escape::encode_safe(source[field].as_str().unwrap()).into_owned(),
                    );
                }
                note
            })
            .collect();
        let policy: WriterPolicy = serde_yaml::from_str(include_str!(
            "../../../../contracts/policies/writer-policy.default.yaml"
        ))?;
        let context: BuildContext = serde_yaml::from_str(include_str!(
            "../../../../contracts/contexts/build-context.default.yaml"
        ))?;
        let target = BuildArtifactTarget::new(
            Path::new(output).parent().unwrap().join("staging-output"),
            "control",
        );
        let input_ms = started.elapsed().as_secs_f64() * 1000.0;
        let operation = Instant::now();
        let result = ankiforge::tools::build_contract(&ir, &policy, &context, &target)?;
        ensure!(result.result_status == "success", "{result:?}");
        let operation_ms = operation.elapsed().as_secs_f64() * 1000.0;
        let manifest: serde_json::Value =
            serde_json::from_slice(&std::fs::read(target.staging_manifest_path())?)?;
        ensure!(manifest["normalized_ir"] == serde_json::to_value(&ir)?);
        ensure!(result.staging_ref.is_some() && result.artifact_fingerprint.is_some());
        std::fs::copy(target.root_dir.join("package.apkg"), output)?;
        std::fs::remove_dir_all(&target.root_dir)?;
        println!(
            "{}",
            serde_json::json!({"input_ms":input_ms, "operation_ms":operation_ms,
            "comparison":null, "manifest_fingerprint":result.artifact_fingerprint,
            "manifest_verified":true})
        );
    } else {
        ensure!(mode == "bytes-concurrent");
        let media = &document["media"][0];
        let path = Path::new(input)
            .parent()
            .unwrap()
            .join(media["path"].as_str().unwrap());
        let barrier = Arc::new(Barrier::new(8));
        let owners = std::thread::scope(|scope| -> anyhow::Result<Vec<Media>> {
            let handles: Vec<_> = (0..8)
                .map(|_| {
                    let barrier = barrier.clone();
                    let path = &path;
                    scope.spawn(move || -> anyhow::Result<Media> {
                        let bytes = std::fs::read(path)?;
                        barrier.wait();
                        Ok(Media::bytes(bytes, "audio/wav")?)
                    })
                })
                .collect();
            handles.into_iter().map(|h| h.join().unwrap()).collect()
        })?;
        let name = media["filename"].as_str().unwrap();
        let mut project = Project::new("benchmark.basic-v1")?
            .default_deck(document["deck_name"].as_str().unwrap());
        project.add_asset(owners[0].clone().with_export_name(name)?)?;
        for (index, note) in notes.iter().enumerate() {
            let front = html_escape::encode_safe(note["front"].as_str().unwrap()).into_owned();
            let mut back = html_escape::encode_safe(note["back"].as_str().unwrap()).into_owned();
            if note.get("back_media").is_some() {
                back.push_str(&format!("\n[sound:{name}]"));
            }
            project.add(
                format!("record.{index}"),
                Note::basic(Content::html(front), Content::html(back)),
            )?;
        }
        let input_ms = started.elapsed().as_secs_f64() * 1000.0;
        let operation = Instant::now();
        let result = project.build(BuildOptions::to(output))?;
        println!(
            "{}",
            serde_json::json!({"input_ms":input_ms, "operation_ms":operation.elapsed().as_secs_f64()*1000.0,
            "comparison":null, "report":result.report().snapshot(), "owners":owners.len()})
        );
    }
    Ok(())
}
