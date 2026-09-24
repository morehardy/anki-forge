use std::{error::Error, fs};
use ankiforge::{BuildOptions, Note, Project};
use ankiforge::build::{BuildError, BuildErrorKind, InspectLimits, PersistError, PersistErrorKind};
use ankiforge::build::json::{BuildResultSnapshot, PublicationStage};
use anyhow::Context;

fn main() -> Result<(), Box<dyn Error>> {
    fn standard<T: Error + Send + Sync + 'static>() {}
    standard::<BuildError>(); standard::<PersistError>();
    let mut project = Project::new("errors")?;
    project.add("one", Note::basic("q", "a"))?;
    let mut limits = InspectLimits::default(); limits.max_archive_bytes = 1;
    let error = project.build(BuildOptions::to("limited.apkg").inspect_limits(limits)).unwrap_err();
    assert_eq!(error.kind(), BuildErrorKind::ResourceLimit);
    assert_eq!(error.code(), "INSPECT.RESOURCE_LIMIT_EXCEEDED");
    let limit = error.limit_exceeded().unwrap();
    assert_eq!(limit.resource, "archive_bytes");
    assert_eq!(limit.limit, 1);
    assert!(limit.observed > limit.limit);
    assert!(!std::path::Path::new("limited.apkg").exists());
    let json = serde_json::to_value(error.snapshot())?;
    assert_eq!(json["result"]["status"], "failure");
    let wrapped = Err::<(), _>(error).context("publish lesson").unwrap_err();
    assert_eq!(wrapped.downcast_ref::<BuildError>().unwrap().kind(), BuildErrorKind::ResourceLimit);
    assert_eq!(wrapped.downcast_ref::<BuildError>().unwrap().code(), "INSPECT.RESOURCE_LIMIT_EXCEEDED");
    project.build(BuildOptions::to("retry.apkg"))?;

    fs::create_dir("occupied")?;
    fs::write("occupied/keep", "original")?;
    let error = project.build(BuildOptions::to("occupied")).unwrap_err();
    assert_eq!(error.kind(), BuildErrorKind::Publication);
    assert_eq!(error.code(), "PERSIST.IO_FAILED");
    assert_eq!(error.publications()[0].stage, PublicationStage::NotPublished);
    assert_eq!(fs::read_to_string("occupied/keep")?, "original");
    let persist = error.source().unwrap().downcast_ref::<PersistError>().unwrap();
    assert!(persist.source().unwrap().downcast_ref::<std::io::Error>().is_some());
    assert!(matches!(error.snapshot().result, BuildResultSnapshot::Failure { .. }));

    let output = project.build(BuildOptions::temporary())?;
    let error = output.artifact().persist_to(output.artifact().path()).unwrap_err();
    assert_eq!(error.kind(), PersistErrorKind::InvalidDestination);
    assert!(output.artifact().path().exists());
    let _saved = output.artifact().persist_to("survived.apkg")?;
    let empty = Project::new("empty")?.build(BuildOptions::temporary()).unwrap_err();
    assert_eq!(empty.kind(), BuildErrorKind::Validation);
    assert_eq!(empty.code(), "PROJECT.EMPTY");
    Ok(())
}
