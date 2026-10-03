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

fn relative_temp(mode: &str, workspace: &Path, temp: &Path) -> anyhow::Result<()> {
    #[cfg(unix)]
    ensure!(!std::env::temp_dir().is_absolute(), "exercise an actual relative TMPDIR");
    let source = workspace.join("source.wav");
    let length = source.metadata()?.len();
    let expected = digest(File::open(&source)?)?;
    let original = if mode == "relative-temp-file" {
        Media::file(&source)?
    } else {
        Media::bytes(fs::read(&source)?, "audio/wav")?
    }.with_export_name("owned.wav")?;
    one_snapshot(temp, length)?;
    let filename = fs::read_dir(temp)?.next().context("snapshot file")??.file_name();
    let other = workspace.join("other");
    let other_temp = other.join("snapshots");
    fs::create_dir_all(&other_temp)?;
    let decoy = other_temp.join(filename);
    fs::write(&decoy, b"unrelated snapshot")?;
    let retained = original.clone();
    drop(original);
    fs::remove_file(source)?;
    std::env::set_current_dir(&other)?;
    let mut project = Project::new("relative-temp-directory")?;
    project.add("one", Note::basic(retained.sound(), "answer"))?;
    let output = project.build(BuildOptions::to(workspace.join("relative-temp.apkg")))?;
    verify_archive(output.artifact().path(), expected)?;
    drop(output);
    drop(project);
    one_snapshot(temp, length)?;
    drop(retained);
    ensure!(inventory(temp)?["files"] == 0, "last owner cleans the original directory");
    ensure!(fs::read(decoy)? == b"unrelated snapshot", "cleanup must not touch the new cwd");
    emit("relative_temp_owner_dropped", temp, json!({"package_payload_verified":true,"decoy_preserved":true}))?;
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
    let usage_error = project.add("misuse", Note::basic(Media::file(&altered)?.image(), "answer")).unwrap_err();
    ensure!(usage_error.code() == "NOTE.MEDIA_USAGE_INVALID");
    ensure!(inventory(temp)? == before, "retained media usage error owns no rejected snapshot");
    let conflict_error = project.add_asset(Media::file(&altered)?.with_export_name("keep.wav")?).unwrap_err();
    ensure!(inventory(temp)? == before, "retained media conflict error owns no rejected snapshot");
    emit(
        "failures_preserve_existing_owner",
        temp,
        json!({"name_error":"MEDIA.EXPORT_NAME_INVALID","add_error":error.code()}),
    )?;
    drop(project);
    drop(retained);
    ensure!(inventory(temp)?["files"] == 0);
    ensure!(usage_error.context().note_key() == Some("misuse"));
    ensure!(conflict_error.detail().is_some());
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

fn rotation_failure(workspace: &Path, temp: &Path) -> anyhow::Result<()> {
    let mut owners = Vec::new();
    for index in 0..8_u8 {
        owners.push(Media::bytes(vec![index; 1024 * 1024], "application/octet-stream")?);
    }
    let before = inventory(temp)?;
    ensure!(before["files"] == 1 && before["logical_bytes"] == 4 * 1024 * 1024);

    // The active block is full. Temporarily remove the configured directory so
    // creating its replacement fails after the old writer has been closed.
    let hidden = workspace.join("unavailable-snapshots");
    fs::rename(temp, &hidden)?;
    let failed = Media::bytes(vec![8; 1024 * 1024], "application/octet-stream");
    fs::rename(&hidden, temp)?;
    let error = failed.unwrap_err();
    ensure!(error.kind() == ankiforge::media::MediaErrorKind::Io);
    let cause = std::error::Error::source(&error)
        .and_then(|error| error.downcast_ref::<std::io::Error>())
        .context("replacement failure must retain its I/O source")?;
    ensure!(cause.kind() == std::io::ErrorKind::NotFound);
    ensure!(inventory(temp)? == before, "failed rotation changed the old block");

    let retry = Media::bytes(vec![8; 1024 * 1024], "application/octet-stream")?;
    let after = inventory(temp)?;
    ensure!(after["files"] == 2 && after["logical_bytes"] == 5 * 1024 * 1024);
    let mut project = Project::new("retry-failed-spill-rotation")?;
    project.add("one", Note::basic("Question", "Answer"))?;
    for media in owners.iter().chain(std::iter::once(&retry)) {
        project.add_asset(media.clone())?;
    }
    let output = project.build(BuildOptions::to(workspace.join("rotation-retry.apkg")))?;
    ensure!(output.report().counts().media == 9);
    drop((output, project, owners));
    one_snapshot(temp, 1024 * 1024)?;
    drop(retry);
    ensure!(inventory(temp)?["files"] == 0);
    emit("failed_rotation_retried_and_exported", temp, json!({}))?;
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
        "fd-stress" => {
            let mut owners = Vec::new();
            for index in 0..300_u16 {
                let mut bytes = vec![index as u8; 1024 * 1024];
                bytes[..2].copy_from_slice(&index.to_le_bytes());
                owners.push(Media::bytes(bytes, "application/octet-stream")?);
            }
            let storage = inventory(&temp)?;
            ensure!(storage["logical_bytes"] == 296 * 1024 * 1024);
            ensure!(storage["files"] == 74, "exercise more blocks than allowed descriptors");
            let mut project = Project::new("bounded-reader-descriptors")?;
            project.add("one", Note::basic("Question", "Answer"))?;
            for media in &owners { project.add_asset(media.clone())?; }
            let output = project.build(BuildOptions::to(workspace.join("fd-stress.apkg")))?;
            ensure!(output.report().counts().media == 300,
                "all blocks must remain exportable under the descriptor limit");
            drop((output, project));
            let last = owners.pop().context("last segment")?;
            drop(owners);
            ensure!(inventory(&temp)?["files"] == 1, "only the final live block remains");
            drop(last);
            ensure!(inventory(&temp)?["files"] == 0);

            // Keep the resident budget occupied, then repeatedly release the
            // final owner of an active block. A leaked writer descriptor would
            // exhaust the same 64-FD limit even though all its paths vanished.
            let mut resident = Vec::new();
            for index in 0..4_u8 {
                resident.push(Media::bytes(vec![250 + index; 1024 * 1024], "application/octet-stream")?);
            }
            for index in 0..96_u8 {
                let media = Media::bytes(vec![index; 64 * 1024], "application/octet-stream")?;
                one_snapshot(&temp, 64 * 1024)?;
                drop(media);
                ensure!(inventory(&temp)?["files"] == 0, "final owner retained an active block");
            }
            drop(resident);
            emit("many_blocks_keep_only_active_writer", &temp, json!({"released_active_blocks":96}))?;
            Ok(())
        }
        "append-failure" => {
            let mut owners = Vec::new();
            for index in 0..64_u8 {
                owners.push(Media::bytes(vec![index; 64 * 1024], "application/octet-stream")?);
            }
            let retained = Media::bytes(vec![254; 4096], "application/octet-stream")?;
            let before = inventory(&temp)?;
            ensure!(before["files"] == 1 && before["logical_bytes"] == 4096);
            let error = Media::bytes(vec![255; 64 * 1024], "application/octet-stream").unwrap_err();
            let cause = std::error::Error::source(&error)
                .and_then(|error| error.downcast_ref::<std::io::Error>()).context("original I/O cause")?;
            ensure!(cause.kind() == std::io::ErrorKind::FileTooLarge);
            ensure!(inventory(&temp)? == before, "partial append changed existing storage");
            let path = fs::read_dir(&temp)?.next().context("retained block")??.path();
            ensure!(fs::read(path)? == vec![254; 4096], "failed append overwrote a live segment");
            let duplicate = Media::bytes(vec![254; 4096], "application/octet-stream")?;
            ensure!(duplicate == retained && inventory(&temp)? == before);
            drop(duplicate);

            // A failed writer must be sealed even when truncation succeeded.
            // The next distinct small import needs a new block at offset zero.
            let retry = Media::bytes(vec![253; 4096], "application/octet-stream")?;
            let after = inventory(&temp)?;
            ensure!(after["files"] == 2 && after["logical_bytes"] == 8192,
                "retry reused the failed block instead of creating a fresh one");
            let mut payloads = fs::read_dir(&temp)?
                .map(|entry| fs::read(entry?.path()))
                .collect::<std::io::Result<Vec<_>>>()?;
            payloads.sort();
            ensure!(payloads == vec![vec![253; 4096], vec![254; 4096]],
                "retry changed the preserved prefix or started at a stale offset");
            drop(retained);
            one_snapshot(&temp, 4096)?;
            drop(retry);
            drop(owners);
            ensure!(inventory(&temp)?["files"] == 0);
            emit("failed_append_preserved_existing_segment", &temp, json!({}))?;
            Ok(())
        }
        "budget-failure" => {
            let mut owners = Vec::new();
            for index in 0..64_u8 {
                owners.push(Media::bytes(vec![index; 64 * 1024], "application/octet-stream")?);
            }
            ensure!(inventory(&temp)?["files"] == 0);
            let error = Media::bytes(vec![255; 64 * 1024], "application/octet-stream").unwrap_err();
            ensure!(error.kind() == ankiforge::media::MediaErrorKind::Io);
            ensure!(inventory(&temp)?["files"] == 0, "failed spill leaked partial storage");
            drop(owners);
            let retry = Media::bytes(vec![255; 64 * 1024], "application/octet-stream")?;
            ensure!(inventory(&temp)?["files"] == 0);
            drop(retry);
            emit("failed_budget_spill_cleaned", &temp, json!({}))?;
            Ok(())
        }
        "bounded-small" => {
            let mut owners = Vec::new();
            for index in 0..256_u16 {
                owners.push(Media::bytes(vec![index as u8; 64 * 1024], "application/octet-stream")?);
            }
            let storage = inventory(&temp)?;
            ensure!(storage["logical_bytes"] == 12 * 1024 * 1024,
                "small snapshots exceeded the shared 4 MiB memory budget: {storage}");
            ensure!(storage["files"].as_u64().unwrap() <= 3,
                "small snapshots must share bounded spill blocks: {storage}");
            ensure!(storage["sizes"].as_array().unwrap().iter().all(|v| v.as_u64().unwrap() <= 4 * 1024 * 1024),
                "spill blocks must have bounded retained storage");
            let duplicate = Media::bytes(vec![255; 64 * 1024], "application/octet-stream")?;
            let before = inventory(&temp)?;
            drop(duplicate);
            ensure!(inventory(&temp)? == before, "duplicate snapshot must share storage");
            let mut project = Project::new("shared-spill-blocks")?;
            for media in &owners {project.add_asset(media.clone())?;}
            project.add("one", Note::basic("Question", "Answer"))?;
            let output = project.build(BuildOptions::to(workspace.join("segments.apkg")))?;
            ensure!(output.report().counts().media == 256);
            drop(output);
            drop(project);
            drop(owners);
            ensure!(inventory(&temp)?["files"] == 0);
            let small = Media::bytes(vec![255; 64 * 1024], "application/octet-stream")?;
            ensure!(inventory(&temp)?["files"] == 0, "released memory budget was not reusable");
            drop(small);
            emit("bounded_snapshots_released", &temp, json!({}))?;
            Ok(())
        }
        "measure-baseline" => {
            emit("baseline", &temp, json!({"imports":0}))?;
            Ok(())
        }
        "lifecycle" => lifecycle(&workspace, &temp),
        "duplicate" => duplicate(&workspace, &temp),
        "relative-temp-file" | "relative-temp-bytes" => relative_temp(&mode, &workspace, &temp),
        "validation-failure" => validation_failure(&workspace, &temp),
        "rotation-failure" => rotation_failure(&workspace, &temp),
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
