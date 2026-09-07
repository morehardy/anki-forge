use anki_forge::prelude::*;
use anyhow::{bail, ensure};
use serde::Deserialize;

#[cfg(feature = "mimalloc")]
#[global_allocator]
static ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[derive(Deserialize)]
struct Workload {
    schema: String,
    deck_name: String,
    note_count: usize,
    notes: Vec<Record>,
}

#[derive(Deserialize)]
struct Record {
    front: String,
    back: String,
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
                "crate_version": anki_forge::facade_api_version(),
                "bundle_version": anki_forge::embedded_contract_version(),
                "features": "default", "adapter_features": adapter_features,
                "allocator": if cfg!(feature = "mimalloc") { "mimalloc" } else { "system" },
                "allocator_version": if cfg!(feature = "mimalloc") { Some("0.1.52") } else { None },
                "process_scope": "single_process"
            })
        );
        return Ok(());
    }
    let [input, output] = args.as_slice() else {
        bail!("usage: anki-forge-benchmark INPUT OUTPUT");
    };
    let workload: Workload = serde_json::from_slice(&std::fs::read(input)?)?;
    ensure!(workload.schema == "basic-apkg-v1", "unsupported workload");
    ensure!(workload.notes.len() == workload.note_count, "wrong count");
    let mut deck = Deck::new(workload.deck_name);
    for note in workload.notes {
        // Deck fields accept HTML, just like genanki. Escape plain text here,
        // inside the measured invocation, without changing the shared fixture.
        deck.basic()
            .note(
                html_escape::encode_safe(&note.front).as_ref(),
                html_escape::encode_safe(&note.back).as_ref(),
            )
            .add()?;
    }
    deck.write_apkg(output)?.ensure_success()?;
    Ok(())
}
