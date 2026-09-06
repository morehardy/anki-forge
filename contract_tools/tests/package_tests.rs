use anki_forge::{prelude::*, writer::inspect_apkg};
use flate2::read::GzDecoder;
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};
use tar::Archive;
use tempfile::{tempdir, TempDir};

fn extract_artifact(artifact_path: &Path) -> TempDir {
    let extract_dir = tempdir().expect("temp dir");
    let file = File::open(artifact_path).expect("artifact should exist");
    let mut archive = Archive::new(GzDecoder::new(file));
    archive
        .unpack(extract_dir.path())
        .expect("artifact should unpack");
    extract_dir
}

fn artifact_entries(artifact_path: &Path) -> Vec<PathBuf> {
    let file = File::open(artifact_path).expect("artifact should exist");
    let mut archive = Archive::new(GzDecoder::new(file));
    archive
        .entries()
        .expect("artifact entries should be readable")
        .map(|entry| {
            entry
                .expect("artifact entry should be readable")
                .path()
                .expect("artifact entry path should be readable")
                .into_owned()
        })
        .collect()
}

#[test]
fn package_command_emits_a_bundle_artifact_with_manifest_and_contract_assets() {
    let manifest_path = contract_tools::contract_manifest_path();
    let out_dir = tempdir().expect("temp dir");

    let artifact_path = contract_tools::package::build_artifact(&manifest_path, out_dir.path())
        .expect("package artifact should be created");

    assert_eq!(
        artifact_path.file_name().and_then(|name| name.to_str()),
        Some("anki-forge-contract-bundle-0.6.2.tar.gz")
    );

    let extracted_root = extract_artifact(&artifact_path);
    let extracted_manifest = extracted_root.path().join("contracts/manifest.yaml");

    assert!(
        extracted_manifest.exists(),
        "artifact should unpack a manifest"
    );
    contract_tools::gates::run_all(&extracted_manifest).expect("extracted artifact should verify");
}

#[test]
fn packaged_template_bundle_fixtures_build_and_inspect() {
    let out_dir = tempdir().expect("package output");
    let artifact = contract_tools::package::build_artifact(
        contract_tools::contract_manifest_path(),
        out_dir.path(),
    )
    .expect("package contracts");
    let extracted = extract_artifact(&artifact);

    for (bundle, note, note_type_id, cards, media, css, front, back, browser_front, target_deck) in [
        (
            "custom-normal",
            Note::new("language-card")
                .stable_id("normal:1")
                .text("prompt", "hello"),
            "language-card",
            1,
            1,
            ".card { background-image: url(icon.svg); }\n",
            // The explicit `all: [prompt]` rule gates the review front.
            "{{#Prompt}}{{Prompt}}\n{{/Prompt}}",
            "{{Prompt}}<br>{{Extra}}<img src=\"icon.svg\">\n",
            "{{Prompt}}\n",
            "Languages::Custom",
        ),
        (
            "custom-cloze",
            Note::new("language-cloze")
                .stable_id("cloze:1")
                .text("text", "{{c1::Madrid}} is in {{c2::Spain}}"),
            "language-cloze",
            2,
            0,
            ".cloze { color: #c00; }\n",
            "{{cloze:Sentence}}\n",
            "{{cloze:Sentence}}<br>{{Extra}}\n",
            "{{text:Sentence}}\n",
            "Languages::Cloze",
        ),
    ] {
        let root = extracted
            .path()
            .join("contracts/fixtures/template-bundle")
            .join(bundle);
        let mut project = Project::new(bundle)
            .stable_id(bundle)
            .default_deck("Templates");
        project
            .import_template_bundle(&root)
            .expect("import extracted fixture");
        project.add_note(note).expect("add fixture note");
        let apkg = out_dir.path().join(format!("{bundle}.apkg"));
        let report = project.write_apkg(&apkg).expect("build extracted fixture");
        assert_eq!(report.counts.notes, 1);
        assert_eq!(report.counts.cards, cards);
        assert_eq!(report.counts.media, media);

        let inspected = inspect_apkg(&apkg).expect("inspect fixture APKG");
        assert_eq!(inspected.observation_status, "complete");
        assert!(inspected
            .observations
            .notetypes
            .iter()
            .any(|value| { value["id"] == note_type_id && value["css"] == css }));
        assert!(
            inspected.observations.templates.iter().any(|value| {
                value["notetype_id"] == note_type_id
                    && value["question_format"] == front
                    && value["answer_format"] == back
            }),
            "{bundle}: {:?}",
            inspected.observations.templates
        );
        assert!(inspected
            .observations
            .browser_templates
            .iter()
            .any(|value| {
                value["notetype_id"] == note_type_id
                    && value["browser_question_format"] == browser_front
            }));
        assert!(inspected
            .observations
            .template_target_decks
            .iter()
            .any(|value| {
                value["notetype_id"] == note_type_id && value["target_deck_name"] == target_deck
            }));
        if bundle == "custom-normal" {
            assert!(inspected
                .observations
                .browser_templates
                .iter()
                .any(|value| {
                    value["notetype_id"] == note_type_id
                        && value["browser_answer_format"] == "{{Extra}}\n"
                }));
            assert!(inspected
                .observations
                .media
                .iter()
                .any(|value| value["filename"] == "icon.svg"));
        }
    }
}

#[test]
fn package_template_bundles_include_only_declared_inputs_at_their_relative_paths() {
    let initial_output = tempdir().expect("initial output");
    let initial = contract_tools::package::build_artifact(
        contract_tools::contract_manifest_path(),
        initial_output.path(),
    )
    .expect("initial package");
    let source = extract_artifact(&initial);
    let bundle = source
        .path()
        .join("contracts/fixtures/template-bundle/custom-normal");
    fs::create_dir(bundle.join("nested")).expect("nested templates");
    fs::rename(
        bundle.join("front.html"),
        bundle.join("nested/卡片 front.html"),
    )
    .expect("move front template");
    let manifest_path = bundle.join("anki-template.yaml");
    let manifest = fs::read_to_string(&manifest_path).expect("template manifest");
    fs::write(
        &manifest_path,
        manifest
            .replace(
                "front_file: front.html",
                "front_file: nested/卡片 front.html",
            )
            .replace(
                "browser_front_file: browser-front.html",
                "browser_front_file: nested/卡片 front.html",
            ),
    )
    .expect("shared nested template path");
    fs::write(bundle.join(".env"), "unreferenced local settings").expect("unreferenced file");
    fs::write(bundle.join("assets/unused.svg"), "unreferenced asset").expect("unreferenced asset");

    let output = tempdir().expect("repackaged output");
    let artifact = contract_tools::package::build_artifact(
        source.path().join("contracts/manifest.yaml"),
        output.path(),
    )
    .expect("repackage with nested dependencies");
    let prefix = Path::new("contracts/fixtures/template-bundle/custom-normal");
    let entries = artifact_entries(&artifact)
        .into_iter()
        .filter_map(|path| path.strip_prefix(prefix).ok().map(Path::to_path_buf))
        .collect::<Vec<_>>();
    assert_eq!(
        entries,
        [
            "anki-template.yaml",
            "assets/icon.svg",
            "back.html",
            "browser-back.html",
            "nested/卡片 front.html",
            "style.css",
        ]
        .map(PathBuf::from)
    );

    let extracted = extract_artifact(&artifact);
    Project::new("Nested bundle")
        .import_template_bundle(extracted.path().join(prefix))
        .expect("declared paths remain usable after extraction");
}

#[cfg(unix)]
#[test]
fn package_template_bundles_preserve_internal_symlink_aliases_and_reject_escapes() {
    let initial_output = tempdir().expect("initial output");
    let initial = contract_tools::package::build_artifact(
        contract_tools::contract_manifest_path(),
        initial_output.path(),
    )
    .expect("initial package");
    let source = extract_artifact(&initial);
    let prefix = Path::new("contracts/fixtures/template-bundle/custom-normal");
    let bundle = source.path().join(prefix);
    let alias = bundle.join("linked-front.html");
    std::os::unix::fs::symlink("front.html", &alias).expect("internal alias");
    let manifest_path = bundle.join("anki-template.yaml");
    let manifest = fs::read_to_string(&manifest_path).expect("template manifest");
    fs::write(
        &manifest_path,
        manifest.replace("front_file: front.html", "front_file: linked-front.html"),
    )
    .expect("use alias");
    let output = tempdir().expect("package output");
    let contract_manifest = source.path().join("contracts/manifest.yaml");
    let artifact = contract_tools::package::build_artifact(&contract_manifest, output.path())
        .expect("package internal alias");
    let extracted = extract_artifact(&artifact);
    let exported_alias = extracted.path().join(prefix).join("linked-front.html");
    assert!(fs::symlink_metadata(&exported_alias)
        .expect("alias payload")
        .file_type()
        .is_file());
    Project::new("Alias bundle")
        .import_template_bundle(extracted.path().join(prefix))
        .expect("alias resolves in extracted bundle");

    fs::remove_file(alias).expect("remove internal alias");
    let outside = source.path().join("contracts/outside.html");
    fs::write(&outside, "{{Prompt}}").expect("outside template");
    std::os::unix::fs::symlink(outside, bundle.join("linked-front.html")).expect("escaping alias");
    let error = contract_tools::package::build_artifact(&contract_manifest, output.path())
        .expect_err("escaping dependency must not be packaged");
    assert!(format!("{error:#}").contains("TEMPLATE.BUNDLE_PATH_UNSAFE"));
}

#[test]
fn package_command_excludes_transient_media_store_tmp_files() {
    let manifest_path = contract_tools::contract_manifest_path();
    let contracts_root = manifest_path.parent().expect("manifest should have parent");
    let tmp_file = contracts_root
        .join("fixtures/phase3/inputs/.anki-forge-media/tmp/package-regression/leak.tmp");
    fs::create_dir_all(tmp_file.parent().expect("tmp file should have parent"))
        .expect("tmp dir should be created");
    fs::write(&tmp_file, "transient").expect("tmp file should be written");

    let out_dir = tempdir().expect("temp dir");
    let result = contract_tools::package::build_artifact(&manifest_path, out_dir.path());
    fs::remove_dir_all(contracts_root.join("fixtures/phase3/inputs/.anki-forge-media/tmp"))
        .expect("tmp dir should be removed");

    let artifact_path = result.expect("package artifact should be created");
    let entries = artifact_entries(&artifact_path);
    assert!(
        !entries.iter().any(|entry| {
            entry
            == Path::new(
                "contracts/fixtures/phase3/inputs/.anki-forge-media/tmp/package-regression/leak.tmp"
            )
        }),
        "artifact should not include transient media-store tmp files"
    );
}

#[test]
fn package_command_is_reproducible() {
    let manifest_path = contract_tools::contract_manifest_path();
    let first_out = tempdir().expect("first output dir");
    let second_out = tempdir().expect("second output dir");

    let first = contract_tools::package::build_artifact(&manifest_path, first_out.path())
        .expect("first package artifact");
    let second = contract_tools::package::build_artifact(&manifest_path, second_out.path())
        .expect("second package artifact");

    assert_eq!(
        fs::read(first).expect("read first artifact"),
        fs::read(second).expect("read second artifact"),
        "equal contract inputs must produce byte-identical bundles"
    );
}
