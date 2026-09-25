use ankiforge::{tools::load_project, BuildOptions};
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};
use tempfile::tempdir;

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_contract_tools"))
        .args(args)
        .output()
        .unwrap()
}
fn input(root: &Path, notes: Value) -> PathBuf {
    let path = root.join("project.json");
    fs::write(
        &path,
        serde_json::to_vec(&json!({
            "format_version":"ankiforge-project-v1", "namespace":"cli-project", "notes":notes
        }))
        .unwrap(),
    )
    .unwrap();
    path
}
fn basic(key: &str, front: &str) -> Value {
    json!({"key":key,"content":{"kind":"basic","front":front,"back":{"kind":"html","value":"<b>answer</b>"}}})
}
fn build(project: &Path, out: &Path, extra: &[&str]) -> Output {
    let mut args = vec![
        "product-build",
        "--project",
        project.to_str().unwrap(),
        "--apkg-out",
        out.to_str().unwrap(),
    ];
    args.extend_from_slice(extra);
    run(&args)
}
fn result(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "{error}; stderr={}; stdout={}",
            String::from_utf8_lossy(&output.stderr),
            String::from_utf8_lossy(&output.stdout)
        )
    })
}

#[test]
fn native_build_emits_an_artifact_and_independent_snapshot_file() {
    let root = tempdir().unwrap();
    let project = input(root.path(), json!([basic("one", "<text>")]));
    let out = root.path().join("one.apkg");
    let report = root.path().join("report.json");
    fs::write(&report, b"previous snapshot").unwrap();
    let output = build(&project, &out, &["--report-json", report.to_str().unwrap()]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let snapshot = result(&output);
    assert_eq!(snapshot["schema_version"], "ankiforge-build-v1");
    assert_eq!(snapshot["result"]["status"], "success");
    assert_eq!(snapshot["report"]["counts"]["cards"], 1);
    assert_eq!(
        snapshot,
        serde_json::from_slice::<Value>(&fs::read(report).unwrap()).unwrap()
    );
    let mut archive = zip::ZipArchive::new(fs::File::open(&out).unwrap()).unwrap();
    assert!(archive.by_name("ankiforge-identity.json").is_ok());
    let db = root.path().join("collection.sqlite");
    fs::write(
        &db,
        zstd::decode_all(archive.by_name("collection.anki21b").unwrap()).unwrap(),
    )
    .unwrap();
    let db = rusqlite::Connection::open(db).unwrap();
    let fields: String = db
        .query_row("SELECT flds FROM notes", [], |row| row.get(0))
        .unwrap();
    assert_eq!(fields, "&lt;text&gt;\u{1f}<b>answer</b>");
}

#[test]
fn comparison_and_build_share_policy_and_preserve_blocked_destination() {
    let root = tempdir().unwrap();
    let project = input(
        root.path(),
        json!([basic("one", "one"), basic("two", "two")]),
    );
    let baseline = root.path().join("previous.apkg");
    assert!(build(&project, &baseline, &[]).status.success());
    input(root.path(), json!([basic("one", "changed")]));
    let compared = run(&[
        "product-compare",
        "--project",
        project.to_str().unwrap(),
        "--baseline",
        baseline.to_str().unwrap(),
    ]);
    assert!(compared.status.success());
    let compared = result(&compared);
    assert_eq!(compared["schema_version"], "ankiforge-comparison-v1");
    assert_eq!(compared["policy"]["allows_publication"], false);
    let out = root.path().join("next.apkg");
    fs::write(&out, "previous destination").unwrap();
    let denied = build(
        &project,
        &out,
        &["--update-from", baseline.to_str().unwrap()],
    );
    assert_eq!(denied.status.code(), Some(2));
    let denied = result(&denied);
    assert_eq!(denied["result"]["code"], "UPDATE.POLICY_BLOCKED");
    assert_eq!(denied["report"]["comparison"]["policy"], compared["policy"]);
    assert_eq!(fs::read_to_string(&out).unwrap(), "previous destination");
    let accepted = build(
        &project,
        &out,
        &[
            "--update-from",
            baseline.to_str().unwrap(),
            "--allow",
            "RISK.NOTE_REMOVED",
        ],
    );
    assert!(accepted.status.success());
    assert_eq!(
        result(&accepted)["report"]["comparison"]["highest_risk"],
        "high"
    );
    assert_eq!(
        result(&accepted)["report"]["comparison"]["policy"]["allows_publication"],
        true
    );
}

#[test]
fn malformed_baseline_and_policy_are_hard_failures() {
    let root = tempdir().unwrap();
    let project = input(root.path(), json!([basic("one", "one")]));
    let out = root.path().join("out.apkg");
    let missing = root.path().join("missing.apkg");
    let failed = build(
        &project,
        &out,
        &[
            "--update-from",
            missing.to_str().unwrap(),
            "--fail-on",
            "critical",
        ],
    );
    assert_eq!(failed.status.code(), Some(4));
    assert_eq!(result(&failed)["result"]["kind"], "io");
    assert!(!out.exists());
    let invalid = build(&project, &out, &["--allow", "UPDATE.EVIDENCE_MISSING"]);
    assert!(!invalid.status.success());
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("UPDATE.RISK_CODE_INVALID"));
    let no_baseline = build(&project, &out, &["--fail-on", "high"]);
    assert_eq!(
        result(&no_baseline)["result"]["code"],
        "BUILD.UPDATE_POLICY_WITHOUT_BASELINE"
    );
}

#[test]
fn package_remains_successful_when_the_separate_report_write_fails() {
    let root = tempdir().unwrap();
    let project = input(root.path(), json!([basic("one", "one")]));
    let out = root.path().join("out.apkg");
    let bad_report = root.path().join("absent/report.json");
    let output = build(
        &project,
        &out,
        &["--report-json", bad_report.to_str().unwrap()],
    );
    assert_eq!(output.status.code(), Some(4));
    assert_eq!(result(&output)["result"]["status"], "success");
    assert!(out.exists());
    assert!(String::from_utf8_lossy(&output.stderr).contains("report write failed"));
}

#[test]
fn legacy_transport_and_removed_update_switches_are_rejected() {
    let root = tempdir().unwrap();
    let project = root.path().join("old.json");
    fs::write(
        &project,
        r#"{"product_document_version":"product-v2","document_id":"old","notes":[]}"#,
    )
    .unwrap();
    let out = root.path().join("out.apkg");
    assert!(!build(&project, &out, &[]).status.success());
    let project = input(root.path(), json!([basic("one", "one")]));
    for flag in [
        "--identity-lockfile",
        "--write-identity-lockfile",
        "--update-safety",
        "--compare-to",
        "--manifest",
    ] {
        assert!(!build(&project, &out, &[flag]).status.success(), "{flag}");
    }
    assert!(!out.exists());
}

#[test]
fn report_paths_cannot_replace_inputs_baselines_or_artifacts() {
    let root = tempdir().unwrap();
    let project = input(root.path(), json!([basic("one", "one")]));
    let out = root.path().join("out.apkg");
    for protected in [&project, &out] {
        assert!(!build(
            &project,
            &out,
            &["--report-json", protected.to_str().unwrap()]
        )
        .status
        .success());
    }
    assert!(!out.exists());
    assert!(serde_json::from_slice::<Value>(&fs::read(project).unwrap()).is_ok());
}

#[test]
fn report_hardlink_cannot_replace_the_project_input() {
    let root = tempdir().unwrap();
    let project = input(root.path(), json!([basic("one", "one")]));
    let original = fs::read(&project).unwrap();
    let alias = root.path().join("report.json");
    fs::hard_link(&project, &alias).unwrap();
    let out = root.path().join("out.apkg");
    let output = build(&project, &out, &["--report-json", alias.to_str().unwrap()]);
    assert!(!output.status.success());
    assert!(!out.exists());
    assert_eq!(fs::read(&project).unwrap(), original);
}

#[test]
fn project_loader_owns_media_and_builds_custom_cloze_and_structured_io() {
    let root = tempdir().unwrap();
    let fixtures =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../anki_forge/tests/fixtures/public-api");
    fs::copy(
        fixtures.join("occlusion.png"),
        root.path().join("image.png"),
    )
    .unwrap();
    let path = root.path().join("project.json");
    fs::write(&path, serde_json::to_vec(&json!({
        "format_version":"ankiforge-project-v1", "namespace":"assets",
        "assets":[
            {"key":"image","source":{"kind":"file","path":"image.png"}},
            {"key":"audio","source":{"kind":"bytes","data":fs::read(fixtures.join("silence.wav")).unwrap(),"mime":"audio/wav"}},
            {"key":"font","source":{"kind":"bytes","data":fs::read(fixtures.join("labels.woff")).unwrap(),"mime":"font/woff"},"export_as":"labels.woff"}
        ],
        "models":[{"kind":"custom","key":"vocab","fields":[{"key":"front","name":"题目","sort":true}],
            "templates":[{"key":"card","front":"{{front}}","back":"{{FrontSide}}","browser_front":"{{text:front}}","target_deck":"Custom","generation_rule":{"kind":"all","fields":["front"]}}],
            "css":"@font-face{font-family:Labels;src:url(labels.woff)}", "assets":["font"]}],
        "notes":[
            {"key":"custom","content":{"kind":"custom","model":"vocab","fields":{"front":{"kind":"sequence","items":["Caption",{"kind":"image","asset":"image"},{"kind":"sound","asset":"audio"}]}}}},
            {"key":"cloze","content":{"kind":"cloze","text":"{{c1::one}} and {{c2::two}}"}},
            {"key":"io","content":{"kind":"image_occlusion","image":"image","mode":"hide_one_guess_one","masks":[{"key":"nucleus","x":1,"y":1,"width":10,"height":10}],"fields":{"header":"Identify"}}}
        ]
    })).unwrap()).unwrap();
    let project = load_project(&path, None).unwrap();
    fs::remove_file(root.path().join("image.png")).unwrap();
    let output = project.build(BuildOptions::temporary()).unwrap();
    assert_eq!(output.report().counts().notes, 3);
    assert_eq!(output.report().counts().cards, 4);
    assert_eq!(output.report().counts().media, 3);
    let inspected = ankiforge::tools::inspect_apkg(output.artifact().path()).unwrap();
    assert!(inspected
        .observations
        .templates
        .iter()
        .any(|template| template["question_format"] == "{{#题目}}{{题目}}{{/题目}}"));
}

#[test]
fn apkg_output_cannot_replace_project_input_or_its_file_aliases() {
    let cwd = std::env::current_dir().unwrap();
    let root = tempfile::tempdir_in(&cwd).unwrap();
    let project = input(root.path(), json!([basic("one", "one")]));
    let original = fs::read(&project).unwrap();
    let hardlink = root.path().join("hardlink.apkg");
    fs::hard_link(&project, &hardlink).unwrap();
    let mut aliases = vec![
        project.clone(),
        project.strip_prefix(&cwd).unwrap().to_owned(),
        root.path().join(".").join("project.json"),
        hardlink,
        root.path().join("missing").join("..").join("project.json"),
    ];
    #[cfg(unix)]
    {
        let symlink = root.path().join("symlink.apkg");
        std::os::unix::fs::symlink(&project, &symlink).unwrap();
        aliases.push(symlink);
    }
    for alias in aliases {
        let output = build(&project, &alias, &[]);
        assert!(!output.status.success(), "alias: {alias:?}");
        assert!(String::from_utf8_lossy(&output.stderr)
            .contains("APKG output must not replace project input"));
        assert_eq!(fs::read(&project).unwrap(), original);
        if alias.exists() {
            assert_eq!(fs::read(&alias).unwrap(), original);
        }
    }
    assert!(build(&project, &root.path().join("valid.apkg"), &[])
        .status
        .success());
    assert_eq!(fs::read(&project).unwrap(), original);
}

#[test]
fn report_path_rejects_missing_parent_alias_without_side_effects() {
    let root = tempdir().unwrap();
    let project = input(root.path(), json!([basic("one", "one")]));
    let original = fs::read(&project).unwrap();
    let output = root.path().join("output.apkg");
    let report = root.path().join("missing").join("..").join("project.json");
    assert!(!build(
        &project,
        &output,
        &["--report-json", report.to_str().unwrap()]
    )
    .status
    .success());
    assert_eq!(fs::read(&project).unwrap(), original);
    assert!(!root.path().join("missing").exists());
    assert!(!output.exists());
}
