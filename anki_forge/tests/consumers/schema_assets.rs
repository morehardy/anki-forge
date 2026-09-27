use ankiforge::{Field, Media, NoteType, Template};
use ankiforge::schema::{NoteTypeBuilder, SchemaErrorKind};

fn model() -> NoteTypeBuilder {
    NoteType::builder("styled")
        .field(Field::new("front"))
        .template(Template::new("card").front("{{front}}"))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let css = Media::bytes(b".card { color: blue; }".to_vec(), "text/css")?
        .with_export_name("cards.css")?;
    let styled = model().asset(css.clone()).asset(css.clone()).build()?;
    assert_eq!(styled.assets().len(), 1);
    assert_eq!(styled.assets()[0].filename(), "cards.css");
    // Explicit assets survive even if a static scanner could not find a reference.
    assert_eq!(styled.assets()[0], css);

    for (left, right) in [
        ("cards.css", "CARDS.css"),
        ("é.css", "e\u{301}.css"),
        ("Straße.css", "STRASSE.css"),
        ("σ.css", "ς.css"),
    ] {
        let first = css.clone().with_export_name(left)?;
        let second = css.clone().with_export_name(right)?;
        let error = model().asset(first).asset(second).build().unwrap_err();
        assert_eq!(error.kind(), SchemaErrorKind::AssetConflict);
        assert_eq!(error.code(), "MEDIA.EXPORT_NAME_COLLISION");
    }
    let other_css = Media::bytes(b".card { color: red; }".to_vec(), "text/css")?
        .with_export_name("cards.css")?;
    let error = model().asset(css.clone()).asset(other_css).build().unwrap_err();
    assert_eq!(error.code(), "MEDIA.DUPLICATE_FILENAME_CONFLICT");
    let preserved = styled.clone();
    drop(styled);
    assert_eq!(preserved.assets()[0], css);
    Ok(())
}
