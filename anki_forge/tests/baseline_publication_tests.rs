#![cfg(feature = "internal-tools")]

use std::{fs, path::Path};

use anki_forge::build::{BuildFailureCause, BuildStatus, ComparisonStatus, RiskLevel};
use anki_forge::prelude::*;
use tempfile::tempdir;

fn project(back: &str) -> Project {
    let mut project = Project::new("Baseline publication").stable_id("baseline-publication");
    project
        .add_note(Note::basic("front", back).stable_id("note-1"))
        .unwrap();
    project
}

fn baseline(path: &Path) -> Vec<u8> {
    project("previous").write_apkg(path).unwrap();
    fs::read(path).unwrap()
}

fn filesystem_is_case_insensitive(root: &Path) -> bool {
    let probe = root.join("CaseProbe");
    fs::write(&probe, b"probe").unwrap();
    let case_insensitive = root.join("caseprobe").exists();
    fs::remove_file(probe).unwrap();
    case_insensitive
}

fn assert_collision(options: BuildOptions, baseline_path: &Path, original: &[u8]) {
    let result = project("changed").build(options);
    assert!(
        fs::read(baseline_path).unwrap() == original,
        "baseline changed"
    );
    let error = result.expect_err("overlapping paths must fail before writing");
    assert_eq!(error.report.status, BuildStatus::Invalid);
    assert!(error.report.artifact.is_none());
    assert!(error
        .report
        .diagnostic_codes()
        .contains(&"PROJECT.PATH_COLLISION".into()));
}

#[test]
fn output_cannot_replace_comparison_baseline_in_any_update_mode() {
    for mode in [
        UpdateSafetyMode::Strict,
        UpdateSafetyMode::ReportOnly,
        UpdateSafetyMode::Disabled,
    ] {
        let root = tempdir().unwrap();
        let previous = root.path().join("previous.apkg");
        let original = baseline(&previous);
        assert_collision(
            BuildOptions::new()
                .output(&previous)
                .compare_to(&previous)
                .update_safety(mode),
            &previous,
            &original,
        );
    }
}

#[test]
fn implicit_package_cannot_replace_baseline_even_with_separate_output() {
    for explicit_output in [false, true] {
        let root = tempdir().unwrap();
        let previous = root.path().join("package.apkg");
        let output = root.path().join("output.apkg");
        let original = baseline(&previous);
        let mut options = BuildOptions::new()
            .artifacts_dir(root.path())
            .compare_to(&previous);
        if explicit_output {
            options = options.output(&output);
        }
        assert_collision(options, &previous, &original);
        assert!(!output.exists());
    }
}

#[test]
fn relative_and_absolute_baseline_aliases_are_rejected() {
    let cwd = std::env::current_dir().unwrap();
    let root = tempfile::Builder::new()
        .prefix("baseline-alias-")
        .tempdir_in(&cwd)
        .unwrap();
    let previous = root.path().join("previous.apkg");
    let original = baseline(&previous);
    let relative = previous.strip_prefix(&cwd).unwrap();
    assert_collision(
        BuildOptions::new().output(relative).compare_to(&previous),
        &previous,
        &original,
    );
}

#[test]
fn parent_directory_baseline_alias_is_rejected() {
    let root = tempdir().unwrap();
    fs::create_dir(root.path().join("child")).unwrap();
    let previous = root.path().join("previous.apkg");
    let original = baseline(&previous);
    assert_collision(
        BuildOptions::new()
            .output(root.path().join("child/../previous.apkg"))
            .compare_to(&previous),
        &previous,
        &original,
    );
}

#[test]
fn baseline_inside_writable_staging_tree_is_rejected() {
    let root = tempdir().unwrap();
    let staging = root.path().join("staging");
    fs::create_dir(&staging).unwrap();
    let previous = staging.join("previous.apkg");
    let original = baseline(&previous);
    assert_collision(
        BuildOptions::new()
            .artifacts_dir(root.path())
            .compare_to(&previous),
        &previous,
        &original,
    );
    assert!(!root.path().join("package.apkg").exists());
}

#[cfg(unix)]
#[test]
fn staging_manifest_symlink_cannot_overwrite_external_baseline() {
    let root = tempdir().unwrap();
    let previous = root.path().join("previous.apkg");
    let original = baseline(&previous);
    let artifacts = root.path().join("artifacts");
    let staging = artifacts.join("staging");
    fs::create_dir_all(&staging).unwrap();
    let manifest = staging.join("manifest.json");
    std::os::unix::fs::symlink(&previous, &manifest).unwrap();

    assert_collision(
        BuildOptions::new()
            .artifacts_dir(&artifacts)
            .compare_to(&previous),
        &previous,
        &original,
    );
    assert!(!artifacts.join("package.apkg").exists());
    assert!(fs::symlink_metadata(manifest)
        .unwrap()
        .file_type()
        .is_symlink());
}

#[cfg(unix)]
#[test]
fn baseline_cannot_escape_writable_staging_through_a_symlink() {
    let root = tempdir().unwrap();
    let previous = root.path().join("previous.apkg");
    let original = baseline(&previous);
    let artifacts = root.path().join("artifacts");
    let media = artifacts.join("staging/media");
    fs::create_dir_all(&media).unwrap();
    let alias = media.join("previous.apkg");
    std::os::unix::fs::symlink(&previous, &alias).unwrap();
    let artifacts_alias = root.path().join("artifacts-alias");
    std::os::unix::fs::symlink(&artifacts, &artifacts_alias).unwrap();
    for artifacts in [&artifacts, &artifacts_alias] {
        assert_collision(
            BuildOptions::new()
                .artifacts_dir(artifacts)
                .compare_to(&alias),
            &previous,
            &original,
        );
    }
}

#[test]
fn parent_components_can_select_a_baseline_outside_staging() {
    let root = tempdir().unwrap();
    let staging = root.path().join("staging");
    fs::create_dir(&staging).unwrap();
    let previous = root.path().join("previous.apkg");
    let original = baseline(&previous);
    let report = project("changed")
        .build(
            BuildOptions::new()
                .artifacts_dir(root.path())
                .compare_to(staging.join("../previous.apkg")),
        )
        .unwrap();
    assert_eq!(report.comparison, ComparisonStatus::Complete);
    assert!(fs::read(previous).unwrap() == original);
}

#[test]
fn staging_manifest_hard_link_cannot_overwrite_external_baseline() {
    let root = tempdir().unwrap();
    let previous = root.path().join("previous.apkg");
    let original = baseline(&previous);
    let artifacts = root.path().join("artifacts");
    fs::create_dir_all(artifacts.join("staging")).unwrap();
    fs::hard_link(&previous, artifacts.join("staging/manifest.json")).unwrap();

    assert_collision(
        BuildOptions::new()
            .artifacts_dir(&artifacts)
            .compare_to(&previous),
        &previous,
        &original,
    );
    assert!(!artifacts.join("package.apkg").exists());
}

#[test]
fn new_case_variant_lockfile_never_replaces_the_published_apkg() {
    let root = tempdir().unwrap();
    let case_insensitive = filesystem_is_case_insensitive(root.path());

    for explicit_output in [false, true] {
        let artifacts = root
            .path()
            .join(if explicit_output { "output" } else { "package" });
        let output = artifacts.join("package.apkg");
        let lockfile = artifacts.join("PACKAGE.APKG");
        let mut options = BuildOptions::new().first_update_safe_build(&lockfile);
        if explicit_output {
            options = options.output(&output);
        } else {
            options = options.artifacts_dir(&artifacts);
        }
        let result = project("previous").build(options);

        assert!(
            anki_forge::writer::inspect_apkg(&output).is_ok(),
            "the published path must remain an APKG, not lockfile JSON"
        );
        if case_insensitive {
            let error = result.expect_err("case-folding aliases must not succeed");
            assert_eq!(error.report.status, BuildStatus::Invalid);
            assert!(error
                .report
                .diagnostic_codes()
                .contains(&"PROJECT.PATH_COLLISION".into()));
            assert!(
                !error
                    .report
                    .update_safety
                    .as_ref()
                    .unwrap()
                    .lockfile_written
            );
            assert_eq!(error.report.artifact.as_ref().unwrap().path, output);
        } else {
            let report = result.expect("case-sensitive filesystems permit these distinct paths");
            assert!(report.update_safety.as_ref().unwrap().lockfile_written);
            assert!(
                serde_json::from_slice::<serde_json::Value>(&fs::read(lockfile).unwrap()).is_ok()
            );
        }
    }
}

#[test]
fn new_case_variant_report_never_overwrites_a_published_file() {
    for alias_lockfile in [false, true] {
        let root = tempdir().unwrap();
        let case_insensitive = filesystem_is_case_insensitive(root.path());
        let output = root.path().join("deck.apkg");
        let lockfile = root.path().join("identity.json");
        let report_path = root.path().join(if alias_lockfile {
            "IDENTITY.JSON"
        } else {
            "DECK.APKG"
        });
        let result = project("previous").build(
            BuildOptions::new()
                .output(&output)
                .first_update_safe_build(&lockfile)
                .report_json(&report_path),
        );

        assert!(anki_forge::writer::inspect_apkg(&output).is_ok());
        if case_insensitive {
            let error = result.expect_err("a conflicting report must not be silently skipped");
            assert_eq!(error.report.status, BuildStatus::Invalid);
            assert!(error
                .report
                .diagnostic_codes()
                .contains(&"PROJECT.PATH_COLLISION".into()));
            assert_eq!(
                error
                    .report
                    .update_safety
                    .as_ref()
                    .unwrap()
                    .lockfile_written,
                alias_lockfile
            );
            assert_eq!(lockfile.exists(), alias_lockfile);
        } else {
            result.expect("case-sensitive filesystems permit distinct report paths");
            let json: serde_json::Value =
                serde_json::from_slice(&fs::read(report_path).unwrap()).unwrap();
            assert_eq!(json["kind"], "anki-forge-build-report");
        }
        if lockfile.exists() {
            let json: serde_json::Value =
                serde_json::from_slice(&fs::read(lockfile).unwrap()).unwrap();
            assert_eq!(json["schema_version"], "identity-lockfile-v1");
        }
    }
}

#[cfg(unix)]
#[test]
fn symlinked_parent_directory_alias_is_rejected() {
    let root = tempdir().unwrap();
    let real = root.path().join("real");
    let alias = root.path().join("alias");
    fs::create_dir(&real).unwrap();
    std::os::unix::fs::symlink(&real, &alias).unwrap();
    let previous = real.join("previous.apkg");
    let original = baseline(&previous);
    assert_collision(
        BuildOptions::new()
            .output(alias.join("previous.apkg"))
            .compare_to(&previous),
        &previous,
        &original,
    );
}

#[cfg(unix)]
#[test]
fn unverifiable_dangling_symlink_fails_before_writes() {
    let root = tempdir().unwrap();
    let previous = root.path().join("previous.apkg");
    let original = baseline(&previous);
    let output = root.path().join("output.apkg");
    let artifacts = root.path().join("artifacts");
    std::os::unix::fs::symlink(root.path().join("missing.apkg"), &output).unwrap();
    let error = project("changed")
        .build(
            BuildOptions::new()
                .output(&output)
                .artifacts_dir(&artifacts)
                .compare_to(&previous),
        )
        .unwrap_err();
    assert_eq!(error.cause, BuildFailureCause::Io);
    assert_eq!(error.report.status, BuildStatus::Error);
    assert!(error
        .report
        .diagnostic_codes()
        .contains(&"PROJECT.BUILD_IO".into()));
    assert!(!artifacts.exists());
    assert!(fs::symlink_metadata(output)
        .unwrap()
        .file_type()
        .is_symlink());
    assert!(fs::read(previous).unwrap() == original);
}

#[test]
fn hard_link_baseline_alias_is_rejected() {
    let root = tempdir().unwrap();
    let previous = root.path().join("previous.apkg");
    let output = root.path().join("alias.apkg");
    let original = baseline(&previous);
    fs::hard_link(&previous, &output).unwrap();
    assert_collision(
        BuildOptions::new().output(&output).compare_to(&previous),
        &previous,
        &original,
    );
    assert_eq!(fs::read(output).unwrap(), original);
}

#[cfg(unix)]
#[test]
fn symlink_baseline_aliases_are_rejected_in_both_directions() {
    let root = tempdir().unwrap();
    let previous = root.path().join("previous.apkg");
    let alias = root.path().join("alias.apkg");
    let original = baseline(&previous);
    std::os::unix::fs::symlink(&previous, &alias).unwrap();
    for (output, compare_to) in [(&alias, &previous), (&previous, &alias)] {
        assert_collision(
            BuildOptions::new().output(output).compare_to(compare_to),
            &previous,
            &original,
        );
        assert!(fs::symlink_metadata(&alias)
            .unwrap()
            .file_type()
            .is_symlink());
    }
}

#[test]
fn report_json_cannot_overwrite_baseline_even_on_early_failure() {
    let root = tempdir().unwrap();
    let previous = root.path().join("previous.apkg");
    let output = root.path().join("output.apkg");
    let original = baseline(&previous);
    let error = Project::new("Empty")
        .build(
            BuildOptions::new()
                .output(&output)
                .compare_to(&previous)
                .report_json(&previous),
        )
        .unwrap_err();
    assert_eq!(fs::read(&previous).unwrap(), original);
    assert!(!output.exists());
    assert!(error
        .report
        .diagnostic_codes()
        .contains(&"PROJECT.PATH_COLLISION".into()));
}

#[test]
fn lockfile_writer_cannot_overwrite_baseline_in_disabled_mode() {
    let root = tempdir().unwrap();
    let previous = root.path().join("previous.apkg");
    let original = baseline(&previous);
    assert_collision(
        BuildOptions::new()
            .output(root.path().join("output.apkg"))
            .compare_to(&previous)
            .identity_lockfile(&previous)
            .write_identity_lockfile(true)
            .update_safety(UpdateSafetyMode::Disabled),
        &previous,
        &original,
    );
}

#[test]
fn path_collision_is_reported_before_normalization_and_safe_report_is_written() {
    let root = tempdir().unwrap();
    let previous = root.path().join("previous.apkg");
    let artifacts = root.path().join("artifacts");
    let report_path = root.path().join("report.json");
    let original = baseline(&previous);
    assert_collision(
        BuildOptions::new()
            .output(&previous)
            .compare_to(&previous)
            .artifacts_dir(&artifacts)
            .report_json(&report_path),
        &previous,
        &original,
    );
    assert!(
        !artifacts.exists(),
        "preflight must not create caller artifacts"
    );
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(report_path).unwrap()).unwrap();
    assert_eq!(report["status"], "invalid");
    assert!(report["artifact"].is_null());
}

#[test]
fn policy_block_preserves_output_package_and_identity_lockfile() {
    let root = tempdir().unwrap();
    let previous = root.path().join("previous.apkg");
    let output = root.path().join("output.apkg");
    let lockfile = root.path().join("identity.json");
    let artifacts = root.path().join("artifacts");
    let package = artifacts.join("package.apkg");
    let report_path = root.path().join("report.json");
    project("previous")
        .build(
            BuildOptions::new()
                .output(&previous)
                .first_update_safe_build(&lockfile),
        )
        .unwrap();
    let original = fs::read(&previous).unwrap();
    let original_lockfile = fs::read(&lockfile).unwrap();
    fs::create_dir(&artifacts).unwrap();
    fs::write(&output, b"previous published output").unwrap();
    fs::write(&package, b"previous artifact package").unwrap();

    let error = project("changed")
        .build(
            BuildOptions::new()
                .output(&output)
                .artifacts_dir(&artifacts)
                .compare_to(&previous)
                .fail_on(RiskLevel::Low)
                .identity_lockfile(&lockfile)
                .write_identity_lockfile(true)
                .report_json(&report_path),
        )
        .expect_err("changed content must block at low risk");

    assert_eq!(error.cause, BuildFailureCause::PolicyBlocked);
    assert_eq!(
        error.report.ensure_success().unwrap_err().cause,
        BuildFailureCause::PolicyBlocked,
        "an unpublished blocked report must preserve its failure cause"
    );
    assert_eq!(error.report.status, BuildStatus::Blocked);
    assert_eq!(error.report.comparison, ComparisonStatus::Complete);
    assert!(!error
        .report
        .diff
        .as_ref()
        .unwrap()
        .artifact_diff
        .as_ref()
        .unwrap()
        .changes
        .is_empty());
    assert!(
        error.report.artifact.is_none(),
        "unpublished candidate must not escape"
    );
    assert!(
        !error
            .report
            .update_safety
            .as_ref()
            .unwrap()
            .lockfile_written
    );
    assert_eq!(fs::read(previous).unwrap(), original);
    assert_eq!(fs::read(output).unwrap(), b"previous published output");
    assert_eq!(fs::read(package).unwrap(), b"previous artifact package");
    assert_eq!(fs::read(lockfile).unwrap(), original_lockfile);
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(report_path).unwrap()).unwrap();
    assert_eq!(report["status"], "blocked");
    assert!(report["artifact"].is_null());
    assert_eq!(report["update_safety"]["lockfile_written"], false);
}

#[test]
fn policy_block_does_not_create_an_implicit_package() {
    let root = tempdir().unwrap();
    let previous = root.path().join("previous.apkg");
    let artifacts = root.path().join("artifacts");
    let lockfile = root.path().join("new-identity.json");
    baseline(&previous);
    let error = project("changed")
        .build(
            BuildOptions::new()
                .artifacts_dir(&artifacts)
                .compare_to(&previous)
                .identity_lockfile(&lockfile)
                .write_identity_lockfile(true)
                .fail_on(RiskLevel::Low),
        )
        .unwrap_err();
    assert_eq!(error.report.status, BuildStatus::Blocked);
    assert!(!artifacts.join("package.apkg").exists());
    assert!(!lockfile.exists());
}

#[test]
fn distinct_outputs_publish_after_comparison_and_update_report_paths() {
    let root = tempdir().unwrap();
    let previous = root.path().join("previous.apkg");
    let original = baseline(&previous);
    let artifacts = root.path().join("artifacts");
    let output = root.path().join("output.apkg");
    let lockfile = root.path().join("identity.json");
    let report_path = root.path().join("report.json");
    let report = project("changed")
        .build(
            BuildOptions::new()
                .output(&output)
                .artifacts_dir(&artifacts)
                .compare_to(&previous)
                .fail_on(RiskLevel::Critical)
                .identity_lockfile(&lockfile)
                .write_identity_lockfile(true)
                .report_json(&report_path),
        )
        .unwrap();
    assert_eq!(report.comparison, ComparisonStatus::Complete);
    assert!(!report
        .diff
        .as_ref()
        .unwrap()
        .artifact_diff
        .as_ref()
        .unwrap()
        .changes
        .is_empty());
    assert_eq!(report.artifact.as_ref().unwrap().path, output);
    assert!(report.update_safety.as_ref().unwrap().lockfile_written);
    assert_eq!(fs::read(previous).unwrap(), original);
    assert_eq!(
        fs::read(&output).unwrap(),
        fs::read(artifacts.join("package.apkg")).unwrap()
    );
    let json: serde_json::Value = serde_json::from_slice(&fs::read(report_path).unwrap()).unwrap();
    assert_eq!(json["artifact"]["path"], output.to_str().unwrap());
    assert_eq!(json["update_safety"]["lockfile_written"], true);
}
