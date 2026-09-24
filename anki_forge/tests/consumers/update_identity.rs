use ankiforge::{BuildOptions, Field, Media, Note, NoteType, Project, Template};
use ankiforge::note::Mask;
use std::{error::Error, fs, io::Read};

fn project(updated: bool) -> Result<Project, Box<dyn Error>> {
    let fields = if updated { [Field::new("back").name("答案"), Field::new("front").name("题目")] }
        else { [Field::new("front").name("正面"), Field::new("back").name("背面")] };
    let cards = if updated { ["recall", "recognize"] } else { ["recognize", "recall"] };
    let mut builder = NoteType::builder("vocab").name(if updated { "新名称" } else { "旧名称" });
    for field in fields { builder = builder.field(field); }
    for card in cards { builder = builder.template(Template::new(card).name(format!("{card}-{updated}"))
        .front("{{front}}").back("{{back}}")); }
    let model = builder.build()?;
    let mut project = Project::new("updates")?;
    project.add("term", model.note().field("front", if updated { "updated" } else { "original" })
        .field("back", "answer"))?;
    let mut io = Note::image_occlusion(Media::file("assets/occlusion.png")?);
    for key in if updated { ["wall", "nucleus"] } else { ["nucleus", "wall"] } {
        io = io.mask(Mask::rect(key, if key == "wall" { 60 } else if updated { 20 } else { 10 }, 10, 10, 10));
    }
    project.add("diagram", io.build()?)?;
    Ok(project)
}
fn read(path: &std::path::Path, database: &str) -> (serde_json::Value, rusqlite::Connection) {
    let mut zip = zip::ZipArchive::new(fs::File::open(path).unwrap()).unwrap();
    let evidence: serde_json::Value = serde_json::from_reader(zip.by_name("ankiforge-identity.json").unwrap()).unwrap();
    let mut bytes = Vec::new();
    zip.by_name("collection.anki21b").unwrap().read_to_end(&mut bytes).unwrap();
    fs::write(database, zstd::decode_all(bytes.as_slice()).unwrap()).unwrap();
    (evidence["identity"].clone(), rusqlite::Connection::open(database).unwrap())
}
fn main() -> Result<(), Box<dyn Error>> {
    let first = project(false)?.build(BuildOptions::to("v1.apkg"))?;
    let second = project(true)?.build(BuildOptions::to("v2.apkg").update_from(first.artifact().path()))?;
    let (before, _) = read(first.artifact().path(), "before.sqlite");
    let (after, db) = read(second.artifact().path(), "after.sqlite");
    assert_eq!(before["models"]["vocab"]["id"], after["models"]["vocab"]["id"]);
    assert_eq!(before["models"]["vocab"]["fields"], after["models"]["vocab"]["fields"]);
    assert_eq!(before["models"]["vocab"]["templates"], after["models"]["vocab"]["templates"]);
    for key in ["term", "diagram"] {
        assert_eq!(before["notes"][key]["guid"], after["notes"][key]["guid"]);
        assert!(after["notes"][key]["mtime_secs"].as_i64().unwrap() > before["notes"][key]["mtime_secs"].as_i64().unwrap());
    }
    assert!(after["models"]["vocab"]["mtime_secs"].as_i64().unwrap() > before["models"]["vocab"]["mtime_secs"].as_i64().unwrap());
    let guid = after["notes"]["term"]["guid"].as_str().unwrap();
    let fields: String = db.query_row("select flds from notes where guid = ?1", [guid], |r| r.get(0))?;
    assert_eq!(fields, "updated\u{1f}answer", "declaration reorder must preserve baseline field storage order");
    let guid = after["notes"]["diagram"]["guid"].as_str().unwrap();
    let fields: String = db.query_row("select flds from notes where guid = ?1", [guid], |r| r.get(0))?;
    assert!(fields.contains("{{c1::image-occlusion:rect:left=0.2:"));
    assert!(fields.contains("{{c2::image-occlusion:rect:left=0.6:"));
    assert_eq!(before["notes"]["diagram"]["masks"], after["notes"]["diagram"]["masks"]);
    let repeat = project(true)?.build(BuildOptions::temporary().update_from("v2.apkg"))?;
    let (same, _) = read(repeat.artifact().path(), "same.sqlite");
    assert_eq!(after, same, "an unchanged update must retain all identity and revision evidence");
    Ok(())
}
