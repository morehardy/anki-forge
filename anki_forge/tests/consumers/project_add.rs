use ankiforge::{Field, Media, Note, NoteType, Project, Template};
use ankiforge::note::{AddError, AddErrorKind};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    fn standard_error<T: std::error::Error + Send + Sync + 'static>() {}
    standard_error::<AddError>();
    assert_eq!(Project::new("  ").unwrap_err().code(), "SCHEMA.NAMESPACE_INVALID");
    let mut project = Project::new("biology")?.name("生物").default_deck("生物::细胞");
    assert!(project.is_empty());
    project.add("cell", Note::basic("cell", "细胞"))?;
    let error = project.add("cell", Note::basic("changed", "修改")).unwrap_err();
    assert_eq!(error.kind(), AddErrorKind::DuplicateKey);
    assert_eq!(error.code(), "NOTE.KEY_DUPLICATE");
    assert_eq!(project.len(), 1);

    let asset = Media::bytes(b".card { color: blue; }".to_vec(), "text/css")?
        .with_export_name("cards.css")?;
    project.add_asset(asset.clone())?;
    let extra = Media::bytes(b"new".to_vec(), "text/plain")?.with_export_name("new.txt")?;
    let collision = asset.clone().with_export_name("CARDS.css")?;
    let invalid_add = NoteType::builder("vocab")
        .field(Field::new("front"))
        .template(Template::new("card").front("{{front}}"))
        .asset(extra).asset(collision).build()?;
    let error = project.add("vocab", invalid_add.note().field("front", "cell")).unwrap_err();
    assert_eq!(error.kind(), AddErrorKind::MediaConflict);
    assert_eq!(error.code(), "MEDIA.EXPORT_NAME_COLLISION");
    assert_eq!(project.len(), 1);
    // Neither the model nor the asset preceding the collision was registered.
    let replacement = Media::bytes(b"different".to_vec(), "text/plain")?
        .with_export_name("new.txt")?;
    project.add_asset(replacement)?;
    let model = NoteType::builder("vocab").name("词汇")
        .field(Field::new("front").name("正面").required())
        .template(Template::new("card").front("{{front}}"))
        .build()?;
    project.add("vocab", model.note().field("front", "cell"))?;
    project.add("another", model.note().field("front", "membrane"))?;
    assert_eq!(project.len(), 3);
    let error = project.add("wrong-key", model.note().field("正面", "display name is not a key")).unwrap_err();
    assert_eq!(error.kind(), AddErrorKind::UnknownField);
    let error = project.add("empty", model.note()).unwrap_err();
    assert_eq!(error.kind(), AddErrorKind::RequiredField);
    let conflicting = NoteType::builder("vocab").name("different definition")
        .field(Field::new("front"))
        .template(Template::new("card").front("{{front}}"))
        .build()?;
    assert_eq!(project.add("conflict", conflicting.note()).unwrap_err().kind(), AddErrorKind::ModelConflict);
    let mut other = Project::new("different-project")?;
    let image = Media::file("assets/pixel.png")?;
    other.add("same-values", model.note().field("front", image.image()))?;
    assert_eq!(other.len(), 1);
    Ok(())
}
