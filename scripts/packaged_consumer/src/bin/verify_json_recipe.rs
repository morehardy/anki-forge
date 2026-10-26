#[path = "../inspection.rs"]
mod inspection;

use anyhow::{ensure, Context};
use serde_json::Value;
use std::{collections::BTreeMap, fs, path::PathBuf};

fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let output = PathBuf::from(args.get(1).context("output directory")?);
    let input = PathBuf::from(args.get(2).context("fixture directory")?);
    let first = inspection::inspect(&output.join("v1.apkg"))?;
    let next = inspection::inspect(&output.join("v2.apkg"))?;
    for observed in [&first, &next] {
        ensure!(observed.fields.len() == 3, "expected three actual notes");
        ensure!(
            observed.ordinals == vec![0, 0, 0],
            "expected three Basic cards"
        );
        ensure!(
            observed.assets.len() == 2,
            "expected two actual media entries"
        );
        let expected: Vec<_> = ["cell.png", "tone.wav"]
            .iter()
            .map(|name| fs::read(input.join(name)))
            .collect::<Result<_, _>>()?;
        for bytes in expected {
            ensure!(
                observed.assets.values().any(|asset| *asset == bytes),
                "packaged asset differs from fixture"
            );
        }
    }
    ensure!(first.guids == next.guids, "note GUIDs changed");
    ensure!(
        first.identity["identity"]["namespace"] == next.identity["identity"]["namespace"],
        "namespace changed"
    );
    ensure!(
        first.identity["identity"]["models"] == next.identity["identity"]["models"],
        "model identity changed"
    );
    ensure!(first.models == next.models, "models changed");
    ensure!(first.assets == next.assets, "media changed");
    for (version, observed) in [("v1.json", &first), ("v2.json", &next)] {
        let data: Value = serde_json::from_slice(&fs::read(input.join(version))?)?;
        for row in data["notes"].as_array().context("notes array")? {
            let front = row["front"].as_str().context("front")?;
            let back = row["back"].as_str().context("back")?;
            let fields = observed
                .fields
                .iter()
                .find(|fields| fields.starts_with(front))
                .context("missing question")?;
            ensure!(
                fields.split('\u{1f}').nth(1) == Some(back),
                "incorrect answer"
            );
            ensure!(
                fields.contains("<img src=") && fields.contains("[sound:"),
                "missing typed media references"
            );
            let key = row["id"].as_str().context("business id")?;
            ensure!(
                !observed.identity["identity"]["notes"][key].is_null(),
                "missing stable business ID"
            );
            // Content hashes/timestamps must change for an answer edit; identity must not.
            for property in ["guid", "model", "active", "cards", "masks"] {
                ensure!(
                    first.identity["identity"]["notes"][key][property]
                        == next.identity["identity"]["notes"][key][property],
                    "stable note mapping changed"
                );
            }
        }
    }
    let before: BTreeMap<_, _> = first.guids.iter().zip(&first.fields).collect();
    let after: BTreeMap<_, _> = next.guids.iter().zip(&next.fields).collect();
    ensure!(
        before
            .iter()
            .filter(|(guid, fields)| after.get(*guid) != Some(fields))
            .count()
            == 1,
        "expected exactly one changed answer"
    );
    for file in ["comparison.json", "review.json"] {
        let report: Value = serde_json::from_slice(&fs::read(output.join(file))?)?;
        ensure!(
            report["policy"]["allows_publication"] == true,
            "policy blocked"
        );
        let codes: Vec<_> = report["findings"]
            .as_array()
            .context("findings")?
            .iter()
            .map(|f| f["code"].as_str().unwrap_or(""))
            .collect();
        ensure!(
            codes.contains(&"RISK.NOTE_CHANGED"),
            "answer change not reported"
        );
        ensure!(
            !codes.contains(&"RISK.NOTE_ADDED") && !codes.contains(&"RISK.NOTE_REMOVED"),
            "identity changed"
        );
    }
    println!("Verified 3 notes, 3 cards, 2 exact media assets; stable identities; one answer change; compare and publication allowed.");
    Ok(())
}
