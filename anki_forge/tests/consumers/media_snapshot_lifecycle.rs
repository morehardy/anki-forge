//! Standalone default-public-API probe. The parent isolates TMPDIR before startup.
use ankiforge::{BuildOptions, Content, Media, Note, Project};
use anyhow::{ensure, Context};
use serde_json::{json, Value};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    time::Instant,
};

fn inventory(directory: &Path) -> anyhow::Result<Value> {
    let mut sizes = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        ensure!(
            metadata.is_file(),
            "unexpected retained directory in isolated snapshot storage"
        );
        sizes.push(metadata.len());
    }
    sizes.sort_unstable();
    Ok(json!({"files":sizes.len(),"logical_bytes":sizes.iter().sum::<u64>(),"sizes":sizes}))
}

fn emit(phase: &str, temp: &Path, extra: Value) -> anyhow::Result<Value> {
    let value = json!({"phase":phase,"storage":inventory(temp)?,"observations":extra});
    println!("{value}");
    std::io::stdout().flush()?;
    Ok(value)
}

fn one_snapshot(temp: &Path, bytes: u64) -> anyhow::Result<()> {
    let observed = inventory(temp)?;
    ensure!(
        observed["files"] == 1 && observed["logical_bytes"] == bytes,
        "expected one retained snapshot of {bytes} bytes: {observed}"
    );
    Ok(())
}

fn digest(mut reader: impl Read) -> anyhow::Result<blake3::Hash> {
    let mut digest = blake3::Hasher::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    Ok(digest.finalize())
}

fn verify_archive(path: &Path, expected: blake3::Hash) -> anyhow::Result<()> {
    let mut archive = zip::ZipArchive::new(File::open(path)?)?;
    let mut payloads = 0;
    for index in 0..archive.len() {
        let entry = archive.by_index(index)?;
        if entry.name().parse::<usize>().is_ok() {
            let reader = zstd::stream::read::Decoder::new(entry)?;
            ensure!(
                digest(reader)? == expected,
                "packaged media changed after source deletion"
            );
            payloads += 1;
        }
    }
    ensure!(
        payloads == 1,
        "one logical asset must have one archive payload"
    );
    Ok(())
}

fn lifecycle(workspace: &Path, temp: &Path) -> anyhow::Result<()> {
    let source = workspace.join("source.wav");
    let bytes = source.metadata()?.len();
    let expected = digest(File::open(&source)?)?;
    let original = Media::file(&source)?.with_export_name("shared.wav")?;
    one_snapshot(temp, bytes)?;
    let clone = original.clone();
    let content = Content::sequence([Content::text("Listen: "), original.sound()]);
    let note = Note::basic("Question", content.clone());
    let mut first = Project::new("first-owner")?;
    first.add("one", note.clone())?;
    let mut second = Project::new("second-owner")?;
    second.add("one", note.clone())?;
    emit(
        "all_owner_types",
        temp,
        json!({"notes":2,"source_bytes":bytes}),
    )?;

    // Mutation and deletion affect only the caller's source, not the owned bytes.
    fs::write(&source, b"source replaced")?;
    fs::remove_file(&source)?;
    drop(original);
    drop(clone);
    drop(note);
    one_snapshot(temp, bytes)?;
    emit(
        "media_and_note_dropped",
        temp,
        json!({"content_and_projects_remain":true}),
    )?;

    let output = second.build(BuildOptions::to(workspace.join("owned.apkg")))?;
    ensure!(output.report().counts().media == 1);
    verify_archive(output.artifact().path(), expected)?;
    drop(output);
    one_snapshot(temp, bytes)?;
    drop(first);
    one_snapshot(temp, bytes)?;
    drop(second);
    one_snapshot(temp, bytes)?;
    emit(
        "only_content_remains",
        temp,
        json!({"package_payload_verified":true}),
    )?;
    drop(content);
    ensure!(
        inventory(temp)?["files"] == 0,
        "last content owner did not clean storage"
    );
    emit("all_owners_dropped", temp, json!({}))?;
    Ok(())
}

fn duplicate(workspace: &Path, temp: &Path) -> anyhow::Result<()> {
    let source = workspace.join("source.wav");
    let bytes = source.metadata()?.len();
    let other_path = workspace.join("same-content.wav");
    fs::copy(&source, &other_path)?;
    let first = Media::file(&source)?;
    let second = Media::file(&other_path)?;
    let third = Media::bytes(fs::read(&source)?, "audio/wav")?;
    ensure!(first == second && second == third);
    one_snapshot(temp, bytes)?;
    emit(
        "file_file_bytes_imports",
        temp,
        json!({"imports":3,"equal_public_values":true}),
    )?;
    fs::remove_file(&source)?;
    fs::remove_file(&other_path)?;
    drop(first);
    drop(second);
    one_snapshot(temp, bytes)?;
    drop(third);
    ensure!(inventory(temp)?["files"] == 0);
    emit("all_duplicates_dropped", temp, json!({}))?;
    Ok(())
}

fn validation_failure(workspace: &Path, temp: &Path) -> anyhow::Result<()> {
    let source = workspace.join("source.wav");
    let retained = Media::file(&source)?.with_export_name("keep.wav")?;
    let before = inventory(temp)?;
    let error = Media::file(&source)?
        .with_export_name("../invalid.wav")
        .unwrap_err();
    ensure!(error.code() == "MEDIA.EXPORT_NAME_INVALID");
    ensure!(
        inventory(temp)? == before,
        "failed naming changed retained storage"
    );

    let mut project = Project::new("atomic-owner")?;
    project.add("one", Note::basic(retained.sound(), "answer"))?;
    // Force distinct content to acquire separate owned storage before add fails.
    let altered = workspace.join("different.wav");
    fs::copy(&source, &altered)?;
    let mut writer = fs::OpenOptions::new().append(true).open(&altered)?;
    writer.write_all(b"different content")?;
    drop(writer);
    let error = project
        .add(
            "one",
            Note::basic(Media::file(&altered)?.sound(), "not added"),
        )
        .unwrap_err();
    ensure!(error.code() == "NOTE.KEY_DUPLICATE");
    ensure!(
        project.len() == 1 && inventory(temp)? == before,
        "failed add retained the rejected asset"
    );
    emit(
        "failures_preserve_existing_owner",
        temp,
        json!({"name_error":"MEDIA.EXPORT_NAME_INVALID","add_error":error.code()}),
    )?;
    drop(project);
    drop(retained);
    ensure!(inventory(temp)?["files"] == 0);
    emit("existing_owner_dropped", temp, json!({}))?;
    Ok(())
}

fn write_failure(workspace: &Path, temp: &Path) -> anyhow::Result<()> {
    // The parent imposes a process-only RLIMIT_FSIZE after creating both inputs.
    // A small existing snapshot survives while a larger import fails mid-write.
    let retained = Media::file(workspace.join("sentinel.wav"))?;
    let before = inventory(temp)?;
    ensure!(before["files"] == 1);
    let error = Media::file(workspace.join("source.wav")).unwrap_err();
    ensure!(error.kind() == ankiforge::media::MediaErrorKind::Io);
    let source = std::error::Error::source(&error)
        .and_then(|error| error.downcast_ref::<std::io::Error>())
        .context("snapshot write failure must retain its I/O source")?;
    ensure!(
        source.kind() == std::io::ErrorKind::FileTooLarge,
        "expected process file-size cap, got {source:?}"
    );
    ensure!(
        inventory(temp)? == before,
        "failed stream retained a partial snapshot or removed an existing one"
    );
    emit(
        "partial_write_cleaned",
        temp,
        json!({"kind":format!("{:?}",error.kind()),"code":error.code(),"source_kind":format!("{:?}",source.kind())}),
    )?;
    drop(retained);
    ensure!(inventory(temp)?["files"] == 0);
    emit("sentinel_dropped", temp, json!({}))?;
    Ok(())
}

fn measure(mode: &str, workspace: &Path, temp: &Path, repeats: usize) -> anyhow::Result<()> {
    let source = workspace.join("source.wav");
    let bytes = source.metadata()?.len();
    let started = Instant::now();
    let mut owners = Vec::new();
    for index in 0..repeats {
        let import_start = Instant::now();
        let media = match mode {
            "measure-file" => Media::file(&source)?,
            "measure-bytes" => Media::bytes(fs::read(&source)?, "audio/wav")?,
            _ => unreachable!(),
        };
        ensure!(media.len() == bytes);
        owners.push(media);
        one_snapshot(temp, bytes)?;
        emit(
            "import_complete",
            temp,
            json!({"import":index+1,"retained_owners":owners.len(),"input_bytes":bytes,"import_ms":import_start.elapsed().as_secs_f64()*1000.0}),
        )?;
    }
    drop(owners);
    ensure!(inventory(temp)?["files"] == 0);
    emit(
        "all_owners_dropped",
        temp,
        json!({"total_ms":started.elapsed().as_secs_f64()*1000.0,"mode":mode,"imports":repeats}),
    )?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let mut arguments = std::env::args().skip(1);
    let mode = arguments.next().context("mode")?;
    let workspace = PathBuf::from(arguments.next().context("workspace")?);
    let temp = workspace.join("snapshots");
    ensure!(
        fs::canonicalize(std::env::temp_dir())? == fs::canonicalize(&temp)?,
        "snapshot temp root was not isolated before process startup"
    );
    ensure!(inventory(&temp)?["files"] == 0);
    match mode.as_str() {
        "measure-baseline" => {
            emit("baseline", &temp, json!({"imports":0}))?;
            Ok(())
        }
        "lifecycle" => lifecycle(&workspace, &temp),
        "duplicate" => duplicate(&workspace, &temp),
        "validation-failure" => validation_failure(&workspace, &temp),
        "write-failure" => write_failure(&workspace, &temp),
        "measure-file" | "measure-bytes" => measure(
            &mode,
            &workspace,
            &temp,
            arguments.next().unwrap_or_else(|| "1".into()).parse()?,
        ),
        _ => anyhow::bail!("unknown mode"),
    }
}
