use std::{fs, path::Path, process::Command};

struct FacadeProbe {
    root: tempfile::TempDir,
}

impl FacadeProbe {
    fn new() -> Self {
        let root = tempfile::Builder::new()
            .prefix("anki-forge-stable-facade-")
            .tempdir()
            .expect("create facade probe crate");
        let manifest_dir = serde_json::to_string(env!("CARGO_MANIFEST_DIR"))
            .expect("encode manifest directory as TOML string");

        fs::create_dir(root.path().join("src")).expect("create probe source directory");
        fs::write(
            root.path().join("Cargo.toml"),
            format!(
                r#"[package]
name = "anki_forge_stable_facade_probe"
version = "0.0.0"
edition = "2021"

[dependencies]
ankiforge = {{ path = {manifest_dir}, default-features = false }}
"#
            ),
        )
        .expect("write probe manifest");
        Self { root }
    }

    fn check(&self, source: &str) -> std::process::Output {
        fs::write(self.root.path().join("src/main.rs"), source).expect("write probe source");
        Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
            .args(["check", "--quiet", "--offline"])
            .current_dir(self.root.path())
            .env(
                "CARGO_TARGET_DIR",
                Path::new(env!("CARGO_MANIFEST_DIR")).join("../target/public-consumer"),
            )
            .output()
            .expect("run facade probe")
    }
}

const CONTROL: &str = r#"
use ankiforge::{Project, Note, NoteType, Field, Template, Content, Media, BuildOptions, BuildOutput};
use ankiforge::{note, schema, media, build, update, diagnostics};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut project = Project::new("boundary-probe")?;
    project.add("first", Note::basic("front", "back"))?;
    let _: BuildOptions = BuildOptions::temporary().inspect_limits(build::InspectLimits::default());
    let _: Option<(NoteType, Field, Template, Content, Media, BuildOutput)> = None;
    let _: Option<(note::AddError, schema::SchemaError, media::MediaError,
        update::CompareError, diagnostics::Diagnostic)> = None;
    Ok(())
}
"#;

#[test]
fn default_features_compile_the_documented_domains() {
    let output = FacadeProbe::new().check(CONTROL);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn default_features_hide_implementation_and_retired_entry_points() {
    let probe = FacadeProbe::new();
    let control = probe.check(CONTROL);
    assert!(
        control.status.success(),
        "{}",
        String::from_utf8_lossy(&control.stderr)
    );
    for symbol in [
        "prelude",
        "Deck",
        "deck",
        "product",
        "authoring",
        "authoring_core",
        "writer",
        "writer_core",
        "runtime",
        "update_safety",
        "risk",
        "diff",
        "build_backend",
        "diagnostics_backend",
        "tools",
    ] {
        let output = probe.check(&format!("use ankiforge::{symbol}; fn main() {{}}"));
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!output.status.success(), "{symbol} is reachable");
        assert!(
            stderr.contains("private module") || stderr.contains("unresolved import"),
            "unrelated failure for {symbol}: {stderr}"
        );
    }
}

#[test]
fn default_features_hide_lowering_and_normalization_ir() {
    let probe = FacadeProbe::new();
    let control = probe.check(CONTROL);
    assert!(
        control.status.success(),
        "{}",
        String::from_utf8_lossy(&control.stderr)
    );
    for method in [
        "normalize",
        "lower",
        "to_authoring_document",
        "lowering_plan",
    ] {
        let output = probe.check(&format!(
            "fn main() {{ let project = ankiforge::Project::new(\"probe\").unwrap(); let _ = project.{method}(); }}"));
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!output.status.success(), "{method} exposes IR");
        assert!(
            stderr.contains(&format!("no method named `{method}`")),
            "{stderr}"
        );
    }
}
