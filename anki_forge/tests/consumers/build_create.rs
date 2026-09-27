use std::{fs, io::Read};
use ankiforge::{BuildOptions, Content, Field, Media, Note, NoteType, Project, Template};
use ankiforge::build::json::{BuildSnapshot, BuildResultSnapshot, ReportSnapshot};
use prost::Message;

// Decode the published Anki interchange format, independently of crate internals.
#[derive(Clone, PartialEq, Message)]
struct MediaIndex { #[prost(message, repeated, tag="1")] entries: Vec<MediaEntry> }
#[derive(Clone, PartialEq, Message)]
struct MediaEntry { #[prost(string, tag="1")] name: String }
#[derive(Clone, PartialEq, Message)]
struct TemplateConfig {
    #[prost(string, tag="1")] front: String,
    #[prost(string, tag="2")] back: String,
    #[prost(string, tag="3")] browser_front: String,
    #[prost(string, tag="4")] browser_back: String,
}
fn entry(archive: &mut zip::ZipArchive<fs::File>, name: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    archive.by_name(name).unwrap().read_to_end(&mut bytes).unwrap();
    zstd::decode_all(bytes.as_slice()).unwrap()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut project = Project::new("biology")?.name("生物").default_deck("生物::细胞");
    project.add("definition", Note::basic("<b>cell</b> &", Content::html("<b>细胞</b>")))?;
    project.add("cloze", Note::cloze("<b>{{c1::细胞}}</b> {{c2::细胞膜}}"))?;
    let original_image = fs::read("assets/pixel.png")?;
    let image = Media::file("assets/pixel.png")?;
    let image_name = image.filename().to_owned();
    let audio = Media::file("assets/silence.wav")?;
    let audio_name = audio.filename().to_owned();
    let source = "前缀<!-- front -->{{! front }}\n{{# front }}<span>{{ text:front }}</span>{{/ front }}";
    let model = NoteType::builder("vocab").name("词汇")
        .field(Field::new("front").name("正面"))
        .field(Field::new("back").name("背面"))
        .template(Template::new("recognition").name("识别")
            .front(source).back("{{FrontSide}}<hr>{{back}}")
            .browser_front("{{hint:front}}").browser_back("{{^back}}empty{{/back}}{{back}}"))
        .asset(Media::bytes(b".card{color:blue}".to_vec(), "text/css")?.with_export_name("unused.css")?)
        .build()?;
    project.add("media", model.note().field("front", Content::sequence([image.image(), audio.sound()]))
        .field("back", "植物细胞"))?;
    drop(image);
    drop(audio);
    fs::write("assets/pixel.png", b"changed after snapshot")?;
    fs::remove_file("assets/silence.wav")?;
    let output = project.build(BuildOptions::to("biology.apkg"))?;
    assert_eq!(output.report().counts().notes, 3);
    assert_eq!(output.report().counts().cards, 4);
    assert_eq!(output.report().counts().media, 3);
    let snapshot: BuildSnapshot = output.snapshot();
    assert!(matches!(snapshot.result, BuildResultSnapshot::Success { .. }));
    let report: ReportSnapshot = output.report().snapshot();
    let encoded = serde_json::to_value(&report)?;
    assert!(encoded.get("result").is_none() && encoded.get("artifact").is_none());
    let _json = serde_json::to_string(&snapshot)?;
    let mut archive = zip::ZipArchive::new(fs::File::open(output.artifact().path())?)?;
    let collection = entry(&mut archive, "collection.anki21b");
    fs::write("collection.sqlite", collection)?;
    let db = rusqlite::Connection::open("collection.sqlite")?;
    assert_eq!(db.query_row("select count(*) from cards", [], |row| row.get::<_, i64>(0))?, 4);
    let fields = db.prepare("select flds from notes")?.query_map([], |row| row.get::<_, String>(0))?.collect::<Result<Vec<_>, _>>()?;
    assert!(fields.contains(&"&lt;b&gt;cell&lt;/b&gt; &amp;\u{1f}<b>细胞</b>".to_owned()));
    assert!(fields.contains(&"&lt;b&gt;{{c1::细胞}}&lt;/b&gt; {{c2::细胞膜}}\u{1f}".to_owned()));
    assert!(fields.iter().any(|field| field.contains(&image_name) && field.contains(&format!("[sound:{audio_name}]"))));
    let config: Vec<u8> = db.query_row("select config from templates where name='识别'", [], |row| row.get(0))?;
    let template = TemplateConfig::decode(config.as_slice())?;
    assert_eq!(template.front, "前缀<!-- front -->{{! front }}\n{{# 正面 }}<span>{{ text:正面 }}</span>{{/ 正面 }}");
    assert_eq!(template.back, "{{FrontSide}}<hr>{{背面}}");
    assert_eq!(template.browser_front, "{{hint:正面}}");
    assert_eq!(template.browser_back, "{{^背面}}empty{{/背面}}{{背面}}");
    let index = MediaIndex::decode(entry(&mut archive, "media").as_slice())?;
    assert_eq!(index.entries.len(), 3);
    for (position, media) in index.entries.iter().enumerate() {
        let content = entry(&mut archive, &position.to_string());
        if media.name == image_name { assert_eq!(content, original_image); }
        else if media.name == "unused.css" { assert_eq!(content, b".card{color:blue}"); }
        else { assert_eq!(media.name, audio_name); assert!(content.starts_with(b"RIFF")); }
    }
    let temporary = project.build(BuildOptions::temporary())?;
    let path = temporary.artifact().path().to_owned();
    let observation = temporary.snapshot();
    let handle = temporary.artifact().clone();
    drop(temporary);
    assert!(path.exists());
    let saved = handle.persist_to("saved.apkg")?;
    drop(handle);
    assert!(!path.exists(), "a JSON snapshot must not own the file");
    assert!(saved.path().exists());
    drop(observation);
    Ok(())
}
