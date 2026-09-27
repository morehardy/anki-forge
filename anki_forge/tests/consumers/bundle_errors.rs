use std::{error::Error, fs};
use ankiforge::{NoteType, schema::{SchemaError, TemplateBundleError, TemplateBundleErrorKind as Kind, TemplateBundleLimitExceeded}};

const VALID: &str = "format_version: template-bundle-v2\nnote_type:\n  key: cloze\n  cloze_field: text\n  fields: [{key: text}]\n  templates: [{key: card, front_file: front.html, back_file: back.html}]\n";
fn load(source: &str) -> TemplateBundleError {
    fs::write("bundle/anki-template.yaml", source).unwrap();
    NoteType::from_bundle("bundle").unwrap_err()
}
fn main() -> Result<(), Box<dyn Error>> {
    fs::create_dir("bundle")?;
    fs::write("bundle/front.html", "{{cloze:text}}")?;
    fs::write("bundle/back.html", "{{cloze:text}}")?;
    fs::write("bundle/anki-template.yaml", VALID)?;
    let model = NoteType::from_bundle("bundle")?;
    assert_eq!(model.cloze_field().unwrap().as_str(), "text");
    assert_eq!(model.fields()[0].display_name(), "text");
    assert_eq!(model.templates()[0].display_name(), "card");

    let version = load(&VALID.replace("template-bundle-v2", "template-bundle-v1"));
    assert_eq!(version.kind(), Kind::UnsupportedVersion);
    assert_eq!(version.code(), "TEMPLATE.BUNDLE_VERSION_UNSUPPORTED");
    let unknown = load(&VALID.replace("key: text}", "key: text, identity: true}"));
    assert_eq!(unknown.kind(), Kind::InvalidManifest);
    assert_eq!(unknown.code(), "TEMPLATE.BUNDLE_MANIFEST_INVALID");
    assert!(unknown.source().is_some());
    let missing = load(&VALID.replace("front.html", "missing.html"));
    assert_eq!(missing.kind(), Kind::Io);
    assert_eq!(missing.source().unwrap().downcast_ref::<std::io::Error>().unwrap().kind(), std::io::ErrorKind::NotFound);

    for path in ["../outside.html", "/outside.html", "C:\\outside.html"] {
        let error = load(&VALID.replace("front.html", path));
        assert_eq!(error.kind(), Kind::UnsafePath, "{path}");
        assert_eq!(error.code(), "TEMPLATE.BUNDLE_PATH_UNSAFE");
    }
    fs::write("outside.html", "{{cloze:text}}")?;
    #[cfg(unix)] {
        std::os::unix::fs::symlink(std::env::current_dir()?.join("outside.html"), "bundle/escape.html")?;
        assert_eq!(load(&VALID.replace("front.html", "escape.html")).kind(), Kind::UnsafePath);
    }
    fs::write("bundle/front.html", [0xff, 0xfe])?;
    let encoding = load(VALID);
    assert_eq!(encoding.kind(), Kind::InvalidFile);
    assert!(encoding.source().unwrap().downcast_ref::<std::string::FromUtf8Error>().is_some());
    fs::write("bundle/front.html", vec![b'x'; (2 << 20) + 1])?;
    let limit = load(VALID);
    assert_eq!(limit.kind(), Kind::ResourceLimit);
    assert_eq!(limit.code(), "TEMPLATE.BUNDLE_RESOURCE_LIMIT_EXCEEDED");
    let facts = limit.source().unwrap().downcast_ref::<TemplateBundleLimitExceeded>().unwrap();
    assert_eq!(facts.limit, 2 << 20);
    assert_eq!(facts.observed, (2 << 20) + 1);

    fs::write("bundle/front.html", "{{cloze:text}}")?;
    let duplicate_assets = format!("{VALID}assets:\n  - {{path: front.html, export_as: A.css}}\n  - {{path: back.html, export_as: a.css}}\n");
    let error = load(&duplicate_assets);
    assert_eq!(error.kind(), Kind::InvalidSchema);
    assert_eq!(error.code(), "MEDIA.EXPORT_NAME_COLLISION");
    assert!(error.source().unwrap().downcast_ref::<SchemaError>().is_some());
    fs::write("bundle/anki-template.yaml", VALID)?;
    let _retry = NoteType::from_bundle("bundle")?;
    Ok(())
}
