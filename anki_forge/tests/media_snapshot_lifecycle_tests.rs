//! Filesystem-observable resource ownership through an isolated default consumer.
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    sync::OnceLock,
    time::{Duration, Instant},
};

struct Consumer {
    executable: PathBuf,
}

fn consumer() -> &'static Consumer {
    static CONSUMER: OnceLock<Consumer> = OnceLock::new();
    CONSUMER.get_or_init(|| {
        let consumer_name = format!("ankiforge_media_snapshot_consumer_{}", std::process::id());
        let root = tempfile::Builder::new()
            .prefix("ankiforge-media-consumer-")
            .tempdir()
            .unwrap();
        fs::create_dir(root.path().join("src")).unwrap();
        fs::write(
            root.path().join("src/main.rs"),
            include_str!("consumers/media_snapshot_lifecycle.rs"),
        )
        .unwrap();
        let crate_path = serde_json::to_string(env!("CARGO_MANIFEST_DIR")).unwrap();
        fs::write(
            root.path().join("Cargo.toml"),
            format!(
                r#"[package]
name = "{consumer_name}"
version = "0.0.0"
edition = "2021"
[workspace]
[dependencies]
ankiforge = {{ path = {crate_path}, default-features = false }}
anyhow = "1"
serde_json = "1"
blake3 = "1"
zip = {{ version = "2", default-features = false, features = ["deflate"] }}
zstd = "0.13"
"#
            ),
        )
        .unwrap();
        let target = Path::new(env!("CARGO_MANIFEST_DIR")).join("../target/public-consumer");
        let output = Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()))
            .args(["build", "--offline", "--quiet"])
            .current_dir(root.path())
            .env("CARGO_TARGET_DIR", &target)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "default consumer build failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let executable = target
            .join("debug")
            .join(format!("{consumer_name}{}", std::env::consts::EXE_SUFFIX));
        Consumer { executable }
    })
}

fn wave(path: &Path, bytes: u64) {
    let mut header = [0_u8; 44];
    header[..4].copy_from_slice(b"RIFF");
    header[4..8].copy_from_slice(&((bytes - 8) as u32).to_le_bytes());
    header[8..16].copy_from_slice(b"WAVEfmt ");
    header[16..20].copy_from_slice(&16_u32.to_le_bytes());
    header[20..22].copy_from_slice(&1_u16.to_le_bytes());
    header[22..24].copy_from_slice(&1_u16.to_le_bytes());
    header[24..28].copy_from_slice(&8000_u32.to_le_bytes());
    header[28..32].copy_from_slice(&8000_u32.to_le_bytes());
    header[32..34].copy_from_slice(&1_u16.to_le_bytes());
    header[34..36].copy_from_slice(&8_u16.to_le_bytes());
    header[36..40].copy_from_slice(b"data");
    header[40..44].copy_from_slice(&((bytes - 44) as u32).to_le_bytes());
    let mut file = fs::File::create(path).unwrap();
    file.write_all(&header).unwrap();
    let buffer = [128_u8; 64 * 1024];
    let mut remaining = bytes - 44;
    while remaining > 0 {
        let count = remaining.min(buffer.len() as u64) as usize;
        file.write_all(&buffer[..count]).unwrap();
        remaining -= count as u64;
    }
}

fn output_with_timeout(mut command: Command) -> Output {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let start = Instant::now();
    while child.try_wait().unwrap().is_none() {
        if start.elapsed() > Duration::from_secs(60) {
            child.kill().unwrap();
            let output = child.wait_with_output().unwrap();
            panic!(
                "snapshot consumer timed out:\n{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    child.wait_with_output().unwrap()
}

fn run(mode: &str, cap_file_size: bool) {
    let executable = &consumer().executable;
    let workspace = tempfile::Builder::new()
        .prefix("ankiforge-media-case-")
        .tempdir()
        .unwrap();
    let temp = workspace.path().join("snapshots");
    fs::create_dir(&temp).unwrap();
    wave(&workspace.path().join("source.wav"), 8 << 20);
    wave(&workspace.path().join("sentinel.wav"), 1536 << 10);
    let mut command = if cap_file_size {
        // POSIX shell limits affect only the exec'd consumer. Ignoring SIGXFSZ
        // makes write(2) return EFBIG after a real partial spool write.
        let mut command = Command::new("sh");
        command.args([
            "-c",
            "trap '' XFSZ; ulimit -f 4096; exec \"$1\" \"$2\" \"$3\"",
            "snapshot-limit",
        ]);
        command.arg(executable).arg(mode).arg(workspace.path());
        command
    } else {
        let mut command = Command::new(executable);
        command.arg(mode).arg(workspace.path());
        command
    };
    let requested_temp = if mode.starts_with("relative-temp-") {
        command.current_dir(workspace.path());
        Path::new("snapshots")
    } else {
        &temp
    };
    command
        .env("TMPDIR", requested_temp)
        .env("TMP", requested_temp)
        .env("TEMP", requested_temp);
    let output = output_with_timeout(command);
    assert!(
        output.status.success(),
        "{mode} failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let observations: Vec<serde_json::Value> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert!(!observations.is_empty());
    assert_eq!(observations.last().unwrap()["storage"]["files"], 0);
    assert_eq!(
        fs::read_dir(&temp).unwrap().count(),
        0,
        "child exit must leave no owned snapshot behind"
    );
}

#[test]
fn source_deletion_and_last_content_owner_control_snapshot_lifetime() {
    run("lifecycle", false);
}

#[test]
fn same_large_content_from_file_and_bytes_retains_one_spool() {
    run("duplicate", false);
}

#[test]
fn rejected_values_clean_their_snapshot_without_removing_existing_owners() {
    run("validation-failure", false);
}

#[test]
fn relative_temp_directory_keeps_file_snapshots_owned_across_chdir() {
    run("relative-temp-file", false);
}

#[test]
fn relative_temp_directory_keeps_byte_snapshots_owned_across_chdir() {
    run("relative-temp-bytes", false);
}

#[cfg(unix)]
#[test]
fn failed_partial_spool_write_is_cleaned_and_other_owners_survive() {
    run("write-failure", true);
}
