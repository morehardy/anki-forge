use ankiforge::{BuildOptions, Content, Media as OwnedMedia, Note, Project};
use anyhow::{bail, ensure, Context};
use serde::Deserialize;
use std::{collections::BTreeMap, path::Path};

#[cfg(feature = "mimalloc")]
#[global_allocator]
static ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[derive(Deserialize)]
struct Workload {
    schema: String,
    deck_name: String,
    note_count: usize,
    notes: Vec<Record>,
    #[serde(default)]
    media: Vec<Media>,
}

#[derive(Deserialize)]
struct Record {
    front: String,
    back: String,
    #[serde(default)]
    front_media: Vec<String>,
    #[serde(default)]
    back_media: Vec<String>,
}

#[derive(Deserialize)]
struct Media {
    id: String,
    kind: String,
    path: String,
    filename: String,
}

fn render_field(
    text: &str,
    references: &[String],
    media: &BTreeMap<String, Media>,
) -> anyhow::Result<String> {
    let mut field = html_escape::encode_safe(text).into_owned();
    for id in references {
        let item = media
            .get(id)
            .with_context(|| format!("unknown media: {id}"))?;
        field.push('\n');
        match item.kind.as_str() {
            "image" => field.push_str(&format!(
                "<img src=\"{}\">",
                html_escape::encode_double_quoted_attribute(&item.filename)
            )),
            "audio" => field.push_str(&format!("[sound:{}]", item.filename)),
            _ => bail!("unsupported media kind: {}", item.kind),
        }
    }
    Ok(field)
}

fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.as_slice() == ["--metadata"] {
        let adapter_features: &[&str] = if cfg!(feature = "mimalloc") {
            &["mimalloc"]
        } else {
            &[]
        };
        println!(
            "{}",
            serde_json::json!({
                "protocol": "basic-apkg-v1", "adapter": "anki-forge/rust",
                "protocols": ["basic-apkg-v1", "basic-media-apkg-v1"],
                "media_registration": "individual",
                "crate_version": ankiforge::facade_api_version(),
                "bundle_version": ankiforge::embedded_contract_version(),
                "features": "default", "adapter_features": adapter_features,
                "allocator": if cfg!(feature = "mimalloc") { "mimalloc" } else { "system" },
                "allocator_version": if cfg!(feature = "mimalloc") { Some("0.1.52") } else { None },
                "process_scope": "single_process"
            })
        );
        return Ok(());
    }
    let [input, output, rest @ ..] = args.as_slice() else {
        bail!("usage: anki-forge-benchmark INPUT OUTPUT");
    };
    let mode = rest.first().map(String::as_str).unwrap_or("build");
    let baseline = rest.get(1).filter(|s| !s.is_empty());
    let repeats: usize = rest.get(2).map(|s| s.parse()).transpose()?.unwrap_or(1);
    let input_started = std::time::Instant::now();
    let workload: Workload = serde_json::from_slice(&std::fs::read(input)?)?;
    ensure!(
        matches!(
            workload.schema.as_str(),
            "basic-apkg-v1" | "basic-media-apkg-v1"
        ),
        "unsupported workload"
    );
    ensure!(workload.notes.len() == workload.note_count, "wrong count");
    let mut project = Project::new("benchmark.basic-v1")?.default_deck(workload.deck_name);
    let parent = Path::new(input).parent().context("input parent")?;
    let mut media_by_id = BTreeMap::new();
    let mut retained_media = Vec::new();
    for media in workload.media {
        let mut snapshot = None;
        for _ in 0..repeats {
            let owned = if mode == "bytes" {
                OwnedMedia::bytes(std::fs::read(parent.join(&media.path))?, "audio/wav")?
            } else {
                OwnedMedia::file(parent.join(&media.path))?
            }
            .with_export_name(&media.filename)?;
            retained_media.push(owned.clone());
            snapshot = Some(owned);
        }
        let snapshot = snapshot.context("at least one import")?;
        ensure!(
            matches!(media.kind.as_str(), "image" | "audio"),
            "unsupported media kind"
        );
        project.add_asset(snapshot)?;
        ensure!(
            media_by_id.insert(media.id.clone(), media).is_none(),
            "duplicate media id"
        );
    }
    for (index, note) in workload.notes.into_iter().enumerate() {
        // Both adapters render the same escaped text and media markup inside
        // the measured process; the native API makes the HTML intent explicit.
        project.add(
            format!("record.{index}"),
            Note::basic(
                Content::html(render_field(&note.front, &note.front_media, &media_by_id)?),
                Content::html(render_field(&note.back, &note.back_media, &media_by_id)?),
            ),
        )?;
    }
    let input_ms = input_started.elapsed().as_secs_f64() * 1000.0;
    let operation_started = std::time::Instant::now();
    let options = baseline.map_or_else(
        || BuildOptions::to(output),
        |p| BuildOptions::to(output).update_from(p),
    );
    let mut comparison = None;
    if matches!(mode, "compare-build" | "compare" | "compare-blocked") {
        comparison = Some(
            project
                .compare(ankiforge::update::CompareOptions::against(
                    baseline.context("baseline")?,
                ))?
                .snapshot(),
        );
    }
    #[allow(unused_mut)]
    let mut retained_candidate_bytes = 0;
    let attempt = if matches!(mode, "prepare-publish" | "prepare-blocked") {
        #[cfg(feature = "prepared-publication")]
        {
            let prepared = project.prepare_publication(options)?;
            comparison = prepared.report().comparison().map(|c| c.snapshot());
            // The runner gives this process an otherwise empty temporary root.
            retained_candidate_bytes = std::fs::read_dir(std::env::temp_dir())?
                .filter_map(Result::ok)
                .filter_map(|e| e.metadata().ok())
                .filter(|m| m.is_file())
                .map(|m| m.len())
                .sum::<u64>();
            prepared.publish().map(Some)
        }
        #[cfg(not(feature = "prepared-publication"))]
        {
            bail!("this benchmark binary predates prepared publication")
        }
    } else if mode == "compare" {
        Ok(None)
    } else {
        project.build(options).map(Some)
    };
    let (output, failure) = match attempt {
        Ok(output) => (output, None),
        Err(error) if mode.ends_with("blocked") && error.code() == "UPDATE.POLICY_BLOCKED" => {
            (None, Some(error.snapshot()))
        }
        Err(error) => return Err(error.into()),
    };
    println!(
        "{}",
        serde_json::json!({"input_ms":input_ms,
        "operation_ms": operation_started.elapsed().as_secs_f64() * 1000.0,
        "comparison":comparison, "report":output.as_ref().map(|o| o.report().snapshot()),
        "failure":failure, "retained_candidate_bytes":retained_candidate_bytes, "retained_media":retained_media.len()})
    );
    Ok(())
}
