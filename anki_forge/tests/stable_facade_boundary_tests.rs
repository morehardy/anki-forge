use std::{fs, process::Command};

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
anki_forge = {{ path = {manifest_dir}, default-features = false }}
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
            .env("CARGO_TARGET_DIR", self.root.path().join("target"))
            .output()
            .expect("run facade probe")
    }
}

#[test]
fn default_features_compile_the_documented_facade() {
    let output = FacadeProbe::new().check(
        r#"
use anki_forge::prelude::*;
use anki_forge::{Deck, Project, Severity};

fn main() {
    let _deck = Deck::new("Stable Deck");
    let _project = Project::new("Stable Project");
    let _severity = Severity::Warning;
    let mut limits = InspectLimits::default();
    limits.max_collection_bytes = 128 << 20;
    let _options = BuildOptions::new().inspect_limits(limits);
    assert!(!anki_forge::facade_api_version().is_empty());
    assert!(!anki_forge::embedded_contract_version().is_empty());
}
"#,
    );

    assert!(
        output.status.success(),
        "documented facade failed to compile:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn default_features_hide_repository_internal_modules() {
    let probe = FacadeProbe::new();
    for module in [
        "authoring",
        "build",
        "deck",
        "diagnostics",
        "diff",
        "product",
        "risk",
        "runtime",
        "update_safety",
        "writer",
    ] {
        let output = probe.check(&format!(
            "use anki_forge::{module};\nfn main() {{ let _ = stringify!({module}); }}\n"
        ));
        assert!(
            !output.status.success(),
            "internal module {module} unexpectedly compiled for default consumers"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("private module") || stderr.contains("unresolved import"),
            "unexpected compiler output for {module}:\n{stderr}"
        );
    }
}

#[test]
fn default_features_hide_project_normalization_ir() {
    let output = FacadeProbe::new().check(
        r#"
use anki_forge::Project;

fn main() {
    let project = Project::new("Stable Project");
    let _ = project.normalize();
}
"#,
    );

    assert!(
        !output.status.success(),
        "Project::normalize unexpectedly exposed internal normalization IR"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("no method named `normalize`")
            || stderr.contains("no method named 'normalize'"),
        "unexpected compiler output for Project::normalize:\n{stderr}"
    );
}
