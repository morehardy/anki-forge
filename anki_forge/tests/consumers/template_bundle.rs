use std::{collections::BTreeMap, error::Error, fs, io::Read};
use ankiforge::{BuildOptions, Content, Field, Media, NoteType, Project, Template};
use ankiforge::schema::{TemplateBundleError, TemplateBundleErrorKind};
use anyhow::Context;
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
fn assert_assets(output: &ankiforge::build::BuildOutput, expected: &BTreeMap<String, Vec<u8>>) {
    assert!(output.report().diagnostics().iter().any(|diagnostic| diagnostic.severity == ankiforge::diagnostics::Severity::Warning));
    assert!(matches!(output.snapshot().result, ankiforge::build::json::BuildResultSnapshot::Success { .. }));
    let mut archive = zip::ZipArchive::new(fs::File::open(output.artifact().path()).unwrap()).unwrap();
    let media = MediaIndex::decode(entry(&mut archive, "media").as_slice()).unwrap();
    let actual = media.entries.iter().enumerate()
        .map(|(i, media)| (media.name.clone(), entry(&mut archive, &i.to_string())))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(&actual, expected);
    let collection = entry(&mut archive, "collection.anki21b");
    fs::write("bundle.sqlite", collection).unwrap();
    let db = rusqlite::Connection::open("bundle.sqlite").unwrap();
    let fields: String = db.query_row("select flds from notes", [], |row| row.get(0)).unwrap();
    assert_eq!(fields, "<img src=\"徽标.png\">A\u{1f}字型");
    let css: Vec<u8> = db.query_row("select config from notetypes", [], |row| row.get(0)).unwrap();
    assert!(css.windows(b"labels.woff".len()).any(|bytes| bytes == b"labels.woff"));
}
fn note(model: &NoteType) -> ankiforge::Note {
    model.note().field("front", Content::html("<img src=\"徽标.png\">A"))
        .field("back", "字型")
}
const MANIFEST: &str = r#"
format_version: template-bundle-v2
note_type:
  key: labeled
  name: 标签
  fields:
    - {key: front, name: 正面, required: true, sort: true}
    - {key: back, name: 背面}
  templates:
    - key: recognition
      name: 识别
      front_file: front.html
      back_file: back.html
      browser_front_file: browser.html
      generation_rule: {kind: all, fields: [front]}
css_file: style.css
assets:
  - {path: assets/labels.woff, export_as: labels.woff}
  - {path: assets/pixel.png, export_as: 徽标.png}
  - {path: assets/unused.css, export_as: unused.css}
"#;
fn main() -> Result<(), Box<dyn Error>> {
    fn error_traits<T: Error + Send + Sync + 'static>() {}
    error_traits::<TemplateBundleError>();
    let expected = BTreeMap::from([
        ("labels.woff".into(), fs::read("assets/labels.woff")?),
        ("徽标.png".into(), fs::read("assets/pixel.png")?),
        ("unused.css".into(), b".not-referenced {color:blue}".to_vec()),
    ]);
    let font = Media::file("assets/labels.woff")?.with_export_name("labels.woff")?;
    assert_eq!(font.media_type(), "font/woff");
    let css = "@font-face {font-family:labels;src:url('labels.woff')} .card{font-family:labels}";
    let model = NoteType::builder("labeled").name("标签")
        .field(Field::new("front").name("正面").required().sort())
        .field(Field::new("back").name("背面"))
        .template(Template::new("recognition").name("识别")
            .front("{{front}}").back("{{FrontSide}}<hr>{{back}}")
            .browser_front("{{text:front}}")
            .generate_when(ankiforge::schema::GenerationRule::all(["front"])))
        .css(css).asset(font).build()?;
    let mut direct = Project::new("bundle-assets")?;
    direct.add_asset(Media::bytes(expected["徽标.png"].clone(), "image/png")?.with_export_name("徽标.png")?)?;
    direct.add_asset(Media::bytes(expected["unused.css"].clone(), "text/css")?.with_export_name("unused.css")?)?;
    direct.add("named-assets", note(&model))?;

    fs::create_dir_all("bundle/assets")?;
    fs::write("bundle/anki-template.yaml", MANIFEST)?;
    fs::write("bundle/front.html", "{{front}}")?;
    fs::write("bundle/back.html", "{{FrontSide}}<hr>{{back}}")?;
    fs::write("bundle/browser.html", "{{text:front}}")?;
    fs::write("bundle/style.css", css)?;
    fs::write("bundle/assets/labels.woff", &expected["labels.woff"])?;
    fs::write("bundle/assets/pixel.png", &expected["徽标.png"])?;
    fs::write("bundle/assets/unused.css", &expected["unused.css"])?;
    let bundle = NoteType::from_bundle("bundle")?;
    assert_eq!(bundle.fields(), model.fields());
    assert_eq!(bundle.templates(), model.templates());
    assert_eq!(bundle.assets().len(), 3);

    let error = NoteType::from_bundle_with_limits("bundle", ankiforge::media::MediaLimits { max_bytes: 1 }).unwrap_err();
    assert_eq!(error.kind(), TemplateBundleErrorKind::ResourceLimit);
    assert_eq!(error.code(), "MEDIA.RESOURCE_LIMIT_EXCEEDED");
    let media = error.source().unwrap().downcast_ref::<ankiforge::media::MediaError>().unwrap();
    assert_eq!(media.limit_exceeded().unwrap().limit, 1);
    fs::write("bundle/front.html", "中文 {{unknown}}")?;
    let error = NoteType::from_bundle("bundle").context("importing custom model").unwrap_err();
    let error = error.downcast_ref::<TemplateBundleError>().unwrap();
    assert_eq!(error.kind(), TemplateBundleErrorKind::InvalidSchema);
    assert_eq!(error.code(), "TEMPLATE.RENDER_FIELD_UNKNOWN");
    assert!(error.path().unwrap().ends_with("front.html"));
    let schema = error.source().unwrap().downcast_ref::<ankiforge::schema::SchemaError>().unwrap();
    assert_eq!(schema.location().unwrap().byte_range, 9..16);
    fs::remove_dir_all("bundle")?;
    fs::remove_dir_all("assets")?;
    let mut loaded = Project::new("bundle-assets")?;
    loaded.add("named-assets", note(&bundle))?;
    assert_assets(&direct.build(BuildOptions::temporary())?, &expected);
    assert_assets(&loaded.build(BuildOptions::temporary())?, &expected);
    // A returned model owns every asset and can be reused after the bundle disappears.
    let mut other = Project::new("second-project")?;
    other.add("reuse", note(&bundle))?;
    assert_assets(&other.build(BuildOptions::temporary())?, &expected);
    Ok(())
}
