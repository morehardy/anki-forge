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
    stored_revision_semantics()?;
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


fn semantic_project(deck: &str, target: Option<&str>, extra: bool, zero: bool, back: &str, tag: &str) -> Result<Project, Box<dyn Error>> {
    let mut template = Template::new("card").front("{{front}}").back("{{back}}")
        .browser_front("{{text:front}}").browser_back("{{back}}")
        .generate_when(ankiforge::schema::GenerationRule::all(["front"]));
    if let Some(target) = target { template = template.target_deck(target); }
    let model = NoteType::builder("semantic").name("Stored semantics")
        .field(Field::new("front").sort()).field(Field::new("back"))
        .template(template).css(".card { color: red }").build()?;
    let mut project = Project::new("stored-revisions")?.default_deck(deck);
    project.add("one", model.note().field("front", if zero { "" } else { "question" }).field("back", back).tag(tag))?;
    // This sorts before Z and changes transient deck IDs for the existing cards.
    if extra { project.add("extra", Note::basic("extra", "answer").deck("A"))?; }
    Ok(project)
}

fn stored_revision_semantics() -> Result<(), Box<dyn Error>> {
    use ankiforge::update::RiskCode;
    for (target, zero) in [(None, false), (Some("Z::Target"), false), (None, true)] {
        let original = semantic_project("Z", target, false, zero, "answer", "tag")?
            .build(BuildOptions::temporary())?;
        let (before, _) = read(original.artifact().path(), "semantic-before.sqlite");
        if zero { assert_eq!(original.report().counts().cards, 0); }
        for (label, deck, next_target, extra, back, tag, note_changed, model_changed) in [
            ("same", "Z", target, false, "answer", "tag", false, false),
            ("deck", "Y", target, false, "answer", "tag", target.is_none() && !zero, false),
            ("extra", "Z", target, true, "answer", "tag", false, false),
            ("field", "Z", target, false, "changed answer", "tag", true, false),
            ("tag", "Z", target, false, "answer", "changed_tag", true, false),
            ("target", "Z", Some("Y::Target"), false, "answer", "tag", !zero, true),
        ] {
            let candidate = semantic_project(deck, next_target, extra, zero, back, tag)?;
            let updated = candidate.build(BuildOptions::temporary().update_from(original.artifact().path()))?;
            let (after, _) = read(updated.artifact().path(), "semantic-after.sqlite");
            for (domain, key, changed) in [("notes", "one", note_changed), ("models", "semantic", model_changed)] {
                assert_eq!(before[domain][key]["id"], after[domain][key]["id"], "{label}");
                assert_eq!(before[domain][key]["guid"], after[domain][key]["guid"], "{label}");
                assert_eq!(before[domain][key]["content_hash"] != after[domain][key]["content_hash"], changed, "{label}: {domain}");
                if changed { assert!(after[domain][key]["mtime_secs"].as_i64().unwrap() > before[domain][key]["mtime_secs"].as_i64().unwrap(), "{label}: {domain}"); }
                else { assert_eq!(before[domain][key]["mtime_secs"], after[domain][key]["mtime_secs"], "{label}: {domain}"); }
            }
            let findings = updated.report().comparison().unwrap().findings();
            assert_eq!(findings.iter().any(|finding| finding.code() == RiskCode::NoteChanged), note_changed, "{label}");
            assert_eq!(findings.iter().any(|finding| finding.code() == RiskCode::ModelChanged), model_changed, "{label}");
        }
    }
    let mut project = Project::new("cloze-revisions")?.default_deck("Cloze::Target");
    project.add("cloze", Note::cloze("{{c1::one}} {{c3::three}}"))?;
    let cloze = NoteType::builder("custom-cloze").field(Field::new("text"))
        .template(Template::new("cloze-card").front("{{cloze:text}}").back("{{cloze:text}}").target_deck("Actual::Cloze"))
        .cloze_field("text").build()?;
    project.add("custom-cloze", cloze.note().field("text", "{{c1::one}} {{c3::three}}"))?;
    let first = project.build(BuildOptions::temporary())?;
    let again = project.build(BuildOptions::temporary().update_from(first.artifact().path()))?;
    assert!(again.report().comparison().unwrap().findings().is_empty());
    assert_eq!(again.report().counts().cards, 4);
    Ok(())
}
