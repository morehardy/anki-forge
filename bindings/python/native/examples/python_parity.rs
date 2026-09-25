//! Independent default-API producer used by Python package parity tests.
use ankiforge::{BuildOptions, Content, Field, Note, NoteType, Project, Template};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("extract") {
        use std::io::Read;
        let mut archive = zip::ZipArchive::new(std::fs::File::open(&args[2])?)?;
        let destination = std::path::Path::new(&args[3]);
        std::fs::create_dir_all(destination)?;
        for index in 0..archive.len() {
            let mut entry = archive.by_index(index)?;
            let name = entry.name().to_owned();
            if name != "collection.anki21b"
                && name != "media"
                && !name.chars().all(|c| c.is_ascii_digit())
            {
                continue;
            }
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes)?;
            let decoded = zstd::stream::decode_all(bytes.as_slice())?;
            std::fs::write(destination.join(name), decoded)?;
        }
        return Ok(());
    }
    let mut project = Project::new("python-parity")?;
    match args.get(1).map(String::as_str) {
        Some("basic") => {
            project.add("term", Note::basic("<front>", Content::html("<b>back</b>")))?
        }
        Some("custom") => {
            let model = NoteType::builder("vocab")
                .name("词汇")
                .field(Field::new("front").name("正面"))
                .field(Field::new("back").name("背面"))
                .template(
                    Template::new("recognition")
                        .name("识别")
                        .front("{{front}}")
                        .back("{{FrontSide}}<hr>{{back}}"),
                )
                .build()?;
            project.add(
                "term",
                model
                    .note()
                    .field("front", "<front>")
                    .field("back", Content::html("<b>back</b>")),
            )?;
        }
        _ => return Err("expected basic or custom, followed by output path".into()),
    }
    let output = project.build(BuildOptions::to(args.get(2).ok_or("missing output path")?))?;
    println!("{}", serde_json::to_string(&output.snapshot())?);
    Ok(())
}
