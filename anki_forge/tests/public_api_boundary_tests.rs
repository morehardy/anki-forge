#![cfg(feature = "internal-tools")]

use std::{fs, process::Command};

use anki_forge::prelude::*;

#[test]
fn product_source_map_is_not_a_public_mutation_surface() {
    let product_mod =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/product/mod.rs"))
            .expect("read product mod");
    let lowering = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/product/lowering.rs"
    ))
    .expect("read product lowering");

    assert!(
        !product_mod.contains("ProductSourceMap"),
        "ProductSourceMap should not be re-exported from anki_forge::product"
    );
    assert!(
        !lowering.contains("pub fn insert(&mut self"),
        "ProductSourceMap::insert should not be public API"
    );
}

#[test]
fn prelude_exports_product_happy_path_types() {
    let mut project = Project::new("Spanish")
        .stable_id("spanish")
        .default_deck("Spanish");
    project
        .add_note(Note::basic("hola", "hello").stable_id("es:hola"))
        .expect("add note");

    let _options = BuildOptions::new().inspect(true);
}

#[test]
fn advanced_authoring_reexports_are_namespaced() {
    let document = anki_forge::authoring::AuthoringDocument {
        kind: "authoring-ir".into(),
        schema_version: "0.1.0".into(),
        metadata_document_id: "doc".into(),
        notetypes: vec![],
        notes: vec![],
        media: vec![],
    };

    assert_eq!(document.kind, "authoring-ir");
}

#[test]
fn advanced_writer_reexports_are_namespaced() {
    let _build = anki_forge::writer::build;
    let _policy: Option<anki_forge::writer::WriterPolicy> = None;
    let _target: Option<anki_forge::writer::BuildArtifactTarget> = None;
}

#[test]
fn crate_root_preserves_intended_short_exports() {
    let mut deck = anki_forge::Deck::new("Spanish");
    deck.basic()
        .note("hola", "hello")
        .stable_id("es:hola")
        .add()
        .expect("add root deck note");

    let _severity = anki_forge::Severity::Warning;
}

#[test]
fn crate_root_does_not_reexport_core_surfaces() {
    let lib = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lib.rs"))
        .expect("read lib.rs");

    // This catches the migration-era direct flattening that used to live in lib.rs.
    for forbidden in [
        "Backward-compatible root re-exports",
        "pub use authoring_core",
        "pub use writer_core",
        "pub const build",
    ] {
        assert!(
            !lib.contains(forbidden),
            "crate root should not contain migration-era core re-export: {forbidden}"
        );
    }
}

#[test]
fn external_crates_cannot_import_advanced_symbols_from_root() {
    let probe = tempfile::Builder::new()
        .prefix("anki-forge-root-boundary-")
        .tempdir()
        .expect("create boundary probe crate");
    let manifest_dir = serde_json::to_string(env!("CARGO_MANIFEST_DIR"))
        .expect("encode manifest dir as TOML string");

    fs::create_dir(probe.path().join("src")).expect("create probe src dir");
    fs::write(
        probe.path().join("Cargo.toml"),
        format!(
            r#"[package]
name = "anki_forge_root_boundary_probe"
version = "0.0.0"
edition = "2021"

[dependencies]
anki_forge = {{ path = {manifest_dir} }}
"#
        ),
    )
    .expect("write probe Cargo.toml");

    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    for (symbol, snippet) in [
        (
            "AuthoringDocument",
            "use anki_forge::AuthoringDocument;\nfn main() { let _ = std::any::type_name::<AuthoringDocument>(); }\n",
        ),
        (
            "AuthoringMediaSource",
            "use anki_forge::AuthoringMediaSource;\nfn main() { let _ = std::any::type_name::<AuthoringMediaSource>(); }\n",
        ),
        (
            "NormalizationRequest",
            "use anki_forge::NormalizationRequest;\nfn main() { let _ = std::any::type_name::<NormalizationRequest>(); }\n",
        ),
        (
            "NormalizedIr",
            "use anki_forge::NormalizedIr;\nfn main() { let _ = std::any::type_name::<NormalizedIr>(); }\n",
        ),
        (
            "WriterPolicy",
            "use anki_forge::WriterPolicy;\nfn main() { let _ = std::any::type_name::<WriterPolicy>(); }\n",
        ),
        (
            "BuildArtifactTarget",
            "use anki_forge::BuildArtifactTarget;\nfn main() { let _ = std::any::type_name::<BuildArtifactTarget>(); }\n",
        ),
        (
            "InspectReport",
            "use anki_forge::InspectReport;\nfn main() { let _ = std::any::type_name::<InspectReport>(); }\n",
        ),
        (
            "writer_build",
            "use anki_forge::writer_build;\nfn main() { let _ = writer_build; }\n",
        ),
        ("build value", "fn main() { let _ = anki_forge::build; }\n"),
        (
            "to_authoring_canonical_json",
            "use anki_forge::to_authoring_canonical_json;\nfn main() { let _ = to_authoring_canonical_json; }\n",
        ),
        (
            "to_writer_canonical_json",
            "use anki_forge::to_writer_canonical_json;\nfn main() { let _ = to_writer_canonical_json; }\n",
        ),
    ] {
        fs::write(probe.path().join("src/main.rs"), snippet)
            .unwrap_or_else(|err| panic!("write probe for {symbol}: {err}"));

        let output = Command::new(&cargo)
            .arg("check")
            .arg("--quiet")
            .arg("--offline")
            .current_dir(probe.path())
            .env("CARGO_TARGET_DIR", probe.path().join("target"))
            .output()
            .unwrap_or_else(|err| panic!("run cargo check for {symbol}: {err}"));

        assert!(
            !output.status.success(),
            "advanced root symbol {symbol} unexpectedly compiled for an external crate"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("unresolved import")
                || stderr.contains("no `")
                || stderr.contains("expected value"),
            "unexpected compiler output while checking {symbol}:\n{stderr}"
        );
    }
}

#[test]
fn build_options_expose_update_safety_builder_methods() {
    use anki_forge::build::{BuildOptions, UpdateSafetyMode};

    let options = BuildOptions::new()
        .compare_to("previous.apkg")
        .identity_lockfile("anki-forge.lock.json")
        .write_identity_lockfile(true)
        .update_safety(UpdateSafetyMode::ReportOnly);

    assert_eq!(
        options.compare_to.as_deref(),
        Some(std::path::Path::new("previous.apkg"))
    );
    assert_eq!(
        options.identity_lockfile.as_deref(),
        Some(std::path::Path::new("anki-forge.lock.json"))
    );
    assert!(options.write_identity_lockfile);
    assert_eq!(options.update_safety, Some(UpdateSafetyMode::ReportOnly));
}

#[test]
fn build_options_expose_update_safe_workflow_sugar() {
    use anki_forge::build::{BuildOptions, UpdateSafetyMode};

    let first = BuildOptions::new().first_update_safe_build("anki-forge.lock.json");
    assert_eq!(
        first.identity_lockfile.as_deref(),
        Some(std::path::Path::new("anki-forge.lock.json"))
    );
    assert!(first.write_identity_lockfile);
    assert_eq!(first.update_safety, Some(UpdateSafetyMode::Strict));

    let next = BuildOptions::new().update_safe("anki-forge.lock.json");
    assert_eq!(
        next.identity_lockfile.as_deref(),
        Some(std::path::Path::new("anki-forge.lock.json"))
    );
    assert!(!next.write_identity_lockfile);
    assert_eq!(next.update_safety, Some(UpdateSafetyMode::Strict));
}

#[test]
fn build_api_exports_phase4_report_types() {
    use anki_forge::build::{
        BuildPolicyResult, BuildPolicyStatus, BuildStatus, ComparisonStatus, RiskLevel,
    };

    let _status = BuildStatus::Success;
    let _comparison = ComparisonStatus::NotRequested;
    let _level = RiskLevel::High;
    let _policy = BuildPolicyResult {
        status: BuildPolicyStatus::NotEvaluated,
        threshold: None,
        highest_risk: None,
        blocking_findings: Vec::new(),
    };
}

#[test]
fn build_options_expose_phase4_builder_methods() {
    use anki_forge::build::{BuildOptions, RiskLevel};

    let options = BuildOptions::new()
        .fail_on(RiskLevel::High)
        .report_json("build-report.json");

    assert_eq!(options.fail_on, Some(RiskLevel::High));
    assert_eq!(
        options.report_json.as_deref(),
        Some(std::path::Path::new("build-report.json"))
    );
}
