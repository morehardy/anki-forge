#[path = "../inspection.rs"]
mod inspection;
use anyhow::{ensure, Context};
use std::path::{Path, PathBuf};

fn verify(path: &Path) -> anyhow::Result<()> {
    let seen = inspection::inspect(path)?;
    ensure!(
        !seen.fields.is_empty() && !seen.ordinals.is_empty(),
        "empty documented package {}",
        path.display()
    );
    let filename = path.file_name().context("filename")?.to_string_lossy();
    match filename.as_ref() {
        "spanish.apkg" => ensure!(seen.fields == ["hola\u{1f}hello"], "Basic example contents"),
        "jp-core.apkg" => {
            ensure!(
                seen.fields.len() == 1 && seen.fields[0].starts_with("食べる\u{1f}"),
                "Japanese example contents"
            );
            ensure!(
                seen.fields[0].ends_with("to eat") || seen.fields[0].ends_with("<b>to eat</b>"),
                "Japanese answer"
            );
        }
        "spanish-media.apkg" => {
            ensure!(
                seen.assets.len() == 3
                    && seen.assets.contains_key("hola.png")
                    && seen.assets.contains_key("hola.wav")
                    && seen.assets.contains_key("hint.wav"),
                "media example closure"
            );
            ensure!(
                seen.fields[0].contains("[sound:hola.wav]")
                    && seen.fields[0].contains("<img src=\"hola.png\">"),
                "typed media references"
            );
        }
        "cards.apkg" => ensure!(
            seen.fields.len() == 2
                && seen.ordinals.len() == 3
                && seen.fields.iter().any(|v| v.contains("{{c2::Spain}}")),
            "Basic/Cloze workflow"
        ),
        "cell-diagram.apkg" | "image-occlusion.apkg" => {
            ensure!(seen.ordinals == [0, 1], "two independent IO cards");
            ensure!(
                seen.fields[0].contains("{{c1::image-occlusion")
                    && seen.fields[0].contains("{{c2::image-occlusion"),
                "IO content syntax"
            );
        }
        "bundle-example.apkg" | "template.apkg" => {
            ensure!(
                seen.ordinals == [0, 1] && seen.fields[0].contains("{{c2::Spain}}"),
                "bundle Cloze contents"
            );
            ensure!(
                seen.models.values().any(|m| m["templates"][0]["front"]
                    .as_str()
                    .is_some_and(|f| f.contains("{{cloze:Sentence}}"))),
                "bundle compiled display name"
            );
        }
        "media-guide.apkg" => {
            ensure!(
                seen.assets.len() == 3
                    && seen.assets.contains_key("badge.png")
                    && seen.assets.contains_key("labels.woff"),
                "raw template/CSS assets"
            );
            ensure!(
                seen.fields[0].contains("<img src=\"badge.png\">")
                    && seen.fields[0].contains("[sound:"),
                "typed image/sound content"
            );
        }
        "biology.apkg" => ensure!(
            seen.assets.len() == 1 && seen.fields[0].contains("<img src="),
            "README media snapshot"
        ),
        "policy-v1.apkg" => ensure!(seen.fields == ["Cell?\u{1f}Unit of life"], "baseline body"),
        "policy-v2.apkg" => {
            ensure!(
                seen.fields == ["Cell?\u{1f}The basic unit of life"],
                "updated body"
            );
            let previous = inspection::inspect(&path.with_file_name("policy-v1.apkg"))?;
            ensure!(seen.guids == previous.guids, "update kept GUID");
            ensure!(
                seen.identity["identity"]["models"] == previous.identity["identity"]["models"],
                "content update kept model identity"
            );
        }
        _ => {}
    }
    println!(
        "Verified {}: {} notes, {} cards, {} media",
        path.display(),
        seen.fields.len(),
        seen.ordinals.len(),
        seen.assets.len()
    );
    Ok(())
}
fn main() -> anyhow::Result<()> {
    let directory = PathBuf::from(std::env::args().nth(1).context("directory required")?);
    let mut files = std::fs::read_dir(directory)?
        .map(|e| e.map(|e| e.path()))
        .collect::<Result<Vec<_>, _>>()?;
    files.sort();
    for path in files
        .into_iter()
        .filter(|p| p.extension().is_some_and(|e| e == "apkg"))
    {
        verify(&path)?;
    }
    Ok(())
}
