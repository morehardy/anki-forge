use ankiforge::{BuildOptions, Note, Project};
use ankiforge::build::{BuildError, BuildErrorKind, InspectLimits};
use std::{error::Error, fs, io::{Read, Write}, path::Path};

fn copy_with(input: &Path, output: &str, mut edit: impl FnMut(&str, Vec<u8>) -> Option<Vec<u8>>) {
    let mut source = zip::ZipArchive::new(fs::File::open(input).unwrap()).unwrap();
    let mut target = zip::ZipWriter::new(fs::File::create(output).unwrap());
    for i in 0..source.len() {
        let mut entry = source.by_index(i).unwrap();
        let mut data = Vec::new(); entry.read_to_end(&mut data).unwrap();
        if let Some(data) = edit(entry.name(), data) {
            target.start_file(entry.name(), zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored)).unwrap();
            target.write_all(&data).unwrap();
        }
    }
    target.finish().unwrap();
}
fn has_source<T: Error + 'static>(error: &dyn Error) -> bool {
    let mut source = error.source();
    while let Some(cause) = source { if cause.is::<T>() { return true; } source = cause.source(); }
    false
}
fn failure(project: &Project, baseline: &str, code: &str) -> BuildError {
    fs::write("protected.apkg", "keep previous destination").unwrap();
    let error = project.build(BuildOptions::to("protected.apkg").update_from(baseline)).unwrap_err();
    assert_eq!(error.code(), code, "{error}");
    assert!(error.publications().is_empty());
    assert_eq!(fs::read_to_string("protected.apkg").unwrap(), "keep previous destination");
    error
}
fn main() -> Result<(), Box<dyn Error>> {
    let mut project = Project::new("proof")?;
    project.add("one", Note::basic("question", "answer"))?;
    let baseline = project.build(BuildOptions::to("baseline.apkg"))?;
    copy_with(baseline.artifact().path(), "missing.apkg", |name, data| (name != "ankiforge-identity.json").then_some(data));
    assert_eq!(failure(&project, "missing.apkg", "UPDATE.EVIDENCE_MISSING").kind(), BuildErrorKind::Validation);
    copy_with(baseline.artifact().path(), "json.apkg", |name, data| Some(if name == "ankiforge-identity.json" { b"{invalid json".to_vec() } else { data }));
    assert!(has_source::<serde_json::Error>(&failure(&project, "json.apkg", "UPDATE.EVIDENCE_INVALID")));
    copy_with(baseline.artifact().path(), "mapping.apkg", |name, data| {
        if name != "ankiforge-identity.json" { return Some(data); }
        let mut value: serde_json::Value = serde_json::from_slice(&data).unwrap();
        value["identity"]["notes"]["one"]["guid"] = "another-note".into();
        Some(serde_json::to_vec(&value).unwrap())
    });
    failure(&project, "mapping.apkg", "UPDATE.EVIDENCE_INVALID");
    // A correct checksum alone does not establish semantic consistency with
    // the SQLite collection. Invalid mappings must still be rejected.
    for what in ["card_key", "model_kind", "sort_field"] {
        let path = format!("inconsistent-{what}.apkg");
        copy_with(baseline.artifact().path(), &path, |name, data| {
            if name != "ankiforge-identity.json" { return Some(data); }
            let mut value: serde_json::Value = serde_json::from_slice(&data).unwrap();
            match what {
                "card_key" => { value["identity"]["notes"]["one"]["cards"] = serde_json::json!({"template:unknown-card": 0}); }
                "model_kind" => { value["identity"]["models"].as_object_mut().unwrap().values_mut().next().unwrap()["kind"] = "cloze".into(); }
                "sort_field" => { value["identity"]["models"].as_object_mut().unwrap().values_mut().next().unwrap()["sort_field"] = "back".into(); }
                _ => unreachable!(),
            }
            value["identity"].sort_all_objects();
            value["identity_blake3"] = blake3::hash(&serde_json::to_vec(&value["identity"]).unwrap()).to_hex().to_string().into();
            Some(serde_json::to_vec(&value).unwrap())
        });
        failure(&project, &path, "UPDATE.EVIDENCE_INVALID");
    }
    copy_with(baseline.artifact().path(), "collection.apkg", |name, data| {
        if name != "collection.anki21b" { return Some(data); }
        let bytes = zstd::decode_all(data.as_slice()).unwrap(); fs::write("tampered.sqlite", bytes).unwrap();
        let db = rusqlite::Connection::open("tampered.sqlite").unwrap();
        db.execute("UPDATE notes SET flds = 'edited outside publisher'", []).unwrap(); drop(db);
        Some(zstd::encode_all(fs::read("tampered.sqlite").unwrap().as_slice(), 0).unwrap())
    });
    failure(&project, "collection.apkg", "UPDATE.EVIDENCE_INVALID");
    let mut wrong = Project::new("other-project")?; wrong.add("one", Note::basic("question", "answer"))?;
    failure(&wrong, "baseline.apkg", "UPDATE.NAMESPACE_MISMATCH");
    let error = failure(&project, "does-not-exist.apkg", "BUILD.INSPECT_FAILED");
    assert_eq!(error.kind(), BuildErrorKind::Io); assert!(has_source::<std::io::Error>(&error));
    let wrapped = anyhow::Error::new(error).context("updating publisher output");
    assert_eq!(wrapped.downcast_ref::<BuildError>().unwrap().kind(), BuildErrorKind::Io);

    // Duplicate central-directory names are ambiguous even when both entries
    // contain individually valid evidence. ZipWriter itself disallows them,
    // so patch an equal-length alternate name after creating the fixture.
    fs::copy(baseline.artifact().path(), "duplicate.apkg")?;
    let data = {
        let mut zip = zip::ZipArchive::new(fs::File::open(baseline.artifact().path())?)?;
        let mut bytes = Vec::new(); zip.by_name("ankiforge-identity.json")?.read_to_end(&mut bytes)?; bytes
    };
    let file = fs::OpenOptions::new().read(true).write(true).open("duplicate.apkg")?;
    let mut zip = zip::ZipWriter::new_append(file)?;
    zip.start_file("ankiforge-identitz.json", zip::write::SimpleFileOptions::default())?;
    zip.write_all(&data)?; zip.finish()?;
    let mut bytes = fs::read("duplicate.apkg")?;
    let old = b"ankiforge-identitz.json"; let new = b"ankiforge-identity.json";
    for index in (0..=bytes.len()-old.len()).rev() { if &bytes[index..index+old.len()] == old { bytes[index..index+old.len()].copy_from_slice(new); } }
    fs::write("duplicate.apkg", bytes)?;
    failure(&project, "duplicate.apkg", "BUILD.INSPECT_FAILED");

    let mut limits = InspectLimits::default(); limits.max_identity_bytes = 1;
    let error = project.build(BuildOptions::temporary().update_from("baseline.apkg").inspect_limits(limits.clone())).unwrap_err();
    assert_eq!(error.kind(), BuildErrorKind::ResourceLimit);
    assert_eq!(error.limit_exceeded().unwrap().resource, "identity_bytes");
    assert_eq!(error.limit_exceeded().unwrap().limit, 1);
    // Without a baseline, the independently counted candidate has the same finite budget.
    let candidate_error = project.build(BuildOptions::temporary().inspect_limits(limits)).unwrap_err();
    assert_eq!(candidate_error.limit_exceeded().unwrap().resource, "identity_bytes");
    assert_eq!(candidate_error.report().counts().notes, 1);
    let _output = project.build(BuildOptions::temporary().update_from("baseline.apkg").inspect_limits(InspectLimits::default()))?;
    Ok(())
}
