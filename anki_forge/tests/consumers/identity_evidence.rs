use ankiforge::{BuildOptions, Field, Media, Note, NoteType, Project, Template};
use ankiforge::note::Mask;
use std::{error::Error, fs, io::Read};

fn project(namespace: &str) -> Result<Project, Box<dyn Error>> {
    let mut project = Project::new(namespace)?;
    let model = NoteType::builder("vocab")
        .field(Field::new("front").name("正面"))
        .field(Field::new("back").name("背面"))
        .template(Template::new("recognition").front("{{front}}").back("{{back}}"))
        .build()?;
    project.add("term", model.note().field("front", "cell").field("back", "细胞"))?;
    project.add("diagram", Note::image_occlusion(Media::file("assets/occlusion.png")?)
        .mask(Mask::rect("nucleus", 10, 10, 20, 20))
        .mask(Mask::rect("wall", 50, 20, 20, 20)).build()?)?;
    Ok(project)
}

fn inspect(path: &std::path::Path, db_path: &str) -> (serde_json::Value, String) {
    let mut archive = zip::ZipArchive::new(fs::File::open(path).unwrap()).unwrap();
    let mut evidence = String::new();
    archive.by_name("ankiforge-identity.json").expect("complete update evidence must travel with the APKG")
        .read_to_string(&mut evidence).unwrap();
    let evidence: serde_json::Value = serde_json::from_str(&evidence).unwrap();
    assert_eq!(evidence["format_version"], "ankiforge-identity-v1");
    assert_eq!(evidence["collection_blake3"].as_str().unwrap().len(), 64);
    let mut bytes = Vec::new();
    archive.by_name("collection.anki21b").unwrap().read_to_end(&mut bytes).unwrap();
    fs::write(db_path, zstd::decode_all(bytes.as_slice()).unwrap()).unwrap();
    let db = rusqlite::Connection::open(db_path).unwrap();
    let guid: String = db.query_row("select guid from notes where flds like 'cell%'", [], |r| r.get(0)).unwrap();
    let model: i64 = db.query_row("select mid from notes where guid = ?1", [&guid], |r| r.get(0)).unwrap();
    assert_eq!(evidence["identity"]["models"]["vocab"]["id"], model);
    assert_eq!(evidence["identity"]["notes"]["term"]["guid"], guid);
    assert_eq!(evidence["identity"]["models"]["vocab"]["fields"]["front"]["ordinal"], 0);
    assert_eq!(evidence["identity"]["models"]["vocab"]["templates"]["recognition"]["ordinal"], 0);
    let masks = &evidence["identity"]["notes"]["diagram"]["masks"];
    assert_eq!(masks["nucleus"]["ordinal"], 0);
    assert_eq!(masks["wall"]["ordinal"], 1);
    assert_eq!(evidence["identity"]["notes"]["diagram"]["mask_high_water"], 2);
    let mtime: i64 = db.query_row("select mtime_secs from notetypes where id = ?1", [model], |r| r.get(0)).unwrap();
    assert!(mtime > 0, "Anki needs a real model modification time to accept changes");
    (evidence, guid)
}

fn main() -> Result<(), Box<dyn Error>> {
    let first = project("biology")?.build(BuildOptions::temporary())?;
    let (evidence, guid) = inspect(first.artifact().path(), "first.sqlite");
    assert_eq!(evidence["identity"]["namespace"], "biology");
    let other = project("chemistry")?.build(BuildOptions::temporary())?;
    let (_, other_guid) = inspect(other.artifact().path(), "other.sqlite");
    assert_ne!(guid, other_guid, "equal note keys in different namespaces are independent notes");
    let repeat = project("biology")?.build(BuildOptions::temporary())?;
    let (_, repeated_guid) = inspect(repeat.artifact().path(), "repeat.sqlite");
    assert_eq!(guid, repeated_guid);
    Ok(())
}
