use std::{fs, io::Read};
use ankiforge::{BuildOptions, Media, Note, Project};
use prost::Message;

#[derive(Clone, PartialEq, Message)]
struct MediaIndex { #[prost(message, repeated, tag="1")] entries: Vec<MediaEntry> }
#[derive(Clone, PartialEq, Message)]
struct MediaEntry { #[prost(string, tag="1")] name: String }
fn entry(archive: &mut zip::ZipArchive<fs::File>, name: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    archive.by_name(name).unwrap().read_to_end(&mut bytes).unwrap();
    zstd::decode_all(bytes.as_slice()).unwrap()
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let image = Media::file("assets/pixel.png")?.with_export_name("图 50%#&'.png")?;
    let mut project = Project::new("portable-references")?;
    project.add("image", Note::basic(image.image(), "image with a portable, URL-sensitive filename"))?;
    let output = project.build(BuildOptions::temporary())?;
    let mut archive = zip::ZipArchive::new(fs::File::open(output.artifact().path())?)?;
    let index = MediaIndex::decode(entry(&mut archive, "media").as_slice())?;
    assert_eq!(index.entries[0].name, "图 50%#&'.png");
    assert_eq!(entry(&mut archive, "0"), fs::read("assets/pixel.png")?);
    fs::write("references.sqlite", entry(&mut archive, "collection.anki21b"))?;
    let db = rusqlite::Connection::open("references.sqlite")?;
    let fields: String = db.query_row("select flds from notes", [], |row| row.get(0))?;
    assert!(fields.starts_with("<img src=\"%E5%9B%BE%2050%25%23%26%27.png\">\u{1f}"), "{fields}");
    Ok(())
}
