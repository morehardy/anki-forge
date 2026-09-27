//! Independent public-API producer; optional APKG observation uses generic ZIP/SQLite APIs.
use ankiforge::{BuildOptions, Content, Note, Project};
use serde_json::json;
fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("produce") => {
            let mut p = Project::new("parity")?;
            p.add(
                "hello",
                Note::basic("<hello>", Content::html("<b>world</b>")),
            )?;
            let out = p.build(BuildOptions::to(&args[2]))?;
            println!("{}", serde_json::to_string(&out.snapshot())?);
        }
        Some("inspect") => {
            let file = std::fs::File::open(&args[2])?;
            let mut zip = zip::ZipArchive::new(file)?;
            let mut identity = String::new();
            use std::io::Read;
            zip.by_name("ankiforge-identity.json")?
                .read_to_string(&mut identity)?;
            let mut data = Vec::new();
            zip.by_name("collection.anki21b")?.read_to_end(&mut data)?;
            let collection = zstd::stream::decode_all(data.as_slice())?;
            let file = tempfile::NamedTempFile::new()?;
            std::fs::write(file.path(), collection)?;
            let db = rusqlite::Connection::open(file.path())?;
            let mut stmt = db.prepare("SELECT guid, flds FROM notes ORDER BY guid")?;
            let notes: Vec<_> = stmt
                .query_map([], |r| {
                    Ok(json!({"guid":r.get::<_,String>(0)?,"fields":r.get::<_,String>(1)?}))
                })?
                .collect::<Result<_, _>>()?;
            let cards: i64 = db.query_row("SELECT count(*) FROM cards", [], |r| r.get(0))?;
            let mut media = Vec::new();
            for n in 0..zip.len() {
                let mut entry = zip.by_index(n)?;
                if entry.name().parse::<usize>().is_ok() {
                    let name = entry.name().to_owned();
                    let mut b = Vec::new();
                    entry.read_to_end(&mut b)?;
                    let decoded = zstd::stream::decode_all(b.as_slice())?;
                    media.push(json!({"entry":name,"bytes":decoded}));
                }
            }
            println!(
                "{}",
                json!({"identity":serde_json::from_str::<serde_json::Value>(&identity)?,"notes":notes,"cards":cards,"media":media})
            );
        }
        _ => anyhow::bail!("usage: sdk_parity produce|inspect PATH"),
    }
    Ok(())
}
