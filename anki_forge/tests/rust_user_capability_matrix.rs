//! Public operations, with retained artifacts for manual import when requested.
mod common;
use ankiforge::{
    note::{Mask, OcclusionMode},
    update::{CompareOptions, RiskCode, UpdatePolicy},
};
use ankiforge::{
    BuildOptions, BuildOutput, Content, Field, Media, Note, NoteType, Project, Template,
};
use std::{fs, path::PathBuf};

fn root() -> PathBuf {
    let root = PathBuf::from(
        std::env::var_os("ANKI_FORGE_CAPABILITY_ARTIFACT_DIR")
            .expect("run scripts/run_rust_user_capabilities.sh"),
    );
    fs::create_dir_all(&root).unwrap();
    root
}
fn project() -> Project {
    let mut p = Project::new("capability")
        .unwrap()
        .default_deck("Capability");
    p.add("one", Note::basic("front", "back")).unwrap();
    p
}
fn save(output: BuildOutput) {
    let root = root();
    let package = output
        .artifact()
        .persist_to(root.join("package.apkg"))
        .unwrap();
    let evidence = common::evidence(package.path());
    assert_eq!(evidence["format_version"], "ankiforge-identity-v1");
    fs::write(
        root.join("build.json"),
        serde_json::to_vec_pretty(&output.snapshot()).unwrap(),
    )
    .unwrap();
    fs::write(root.join("apkg.inspect.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "counts":output.report().counts(), "identity":evidence, "fields":common::fields(package.path())})).unwrap()).unwrap();
    if std::env::var("ANKI_FORGE_CAPABILITY_MODE").as_deref() == Ok("manual-desktop") {
        fs::write(root.join("manual-checklist.md"), format!(
            "# Manual Anki import\n\nPackage: {}\n\nSHA-256: ANKI_FORGE_SHA256_PENDING\n\n- Anki version:\n- Import settings:\n- Notes/cards before and after:\n- Rendered content and media:\n- Existing scheduling preserved:\n- Result:\n",package.path().display())).unwrap();
    }
}
fn built(p: Project) {
    save(p.build(BuildOptions::temporary()).unwrap());
}
fn image() -> Media {
    Media::file(common::fixture("occlusion.png")).unwrap()
}
fn custom(reverse: bool, renamed: bool) -> NoteType {
    let mut b = NoteType::builder("custom");
    for key in if reverse {
        ["back", "front"]
    } else {
        ["front", "back"]
    } {
        b = b.field(Field::new(key).name(if renamed {
            format!("Label {key}")
        } else {
            key.into()
        }));
    }
    for key in if reverse {
        ["reverse", "forward"]
    } else {
        ["forward", "reverse"]
    } {
        b = b.template(Template::new(key).front("{{front}}").back("{{back}}"));
    }
    b.build().unwrap()
}
fn custom_project(reverse: bool, renamed: bool) -> Project {
    let model = custom(reverse, renamed);
    let mut p = Project::new("capability").unwrap();
    p.add(
        "one",
        model.note().field("front", "front").field("back", "back"),
    )
    .unwrap();
    p
}

#[test]
#[ignore]
fn duplicate_note_key() {
    let mut p = project();
    assert_eq!(
        p.add("one", Note::basic("duplicate", "back"))
            .unwrap_err()
            .code(),
        "NOTE.KEY_DUPLICATE"
    );
    built(p);
}
#[test]
#[ignore]
fn blank_namespace() {
    assert!(Project::new(" ").is_err());
    built(project());
}
#[test]
#[ignore]
fn blank_note_key() {
    let mut p = project();
    assert_eq!(
        p.add(" ", Note::basic("q", "a")).unwrap_err().code(),
        "NOTE.KEY_INVALID"
    );
    built(p);
}
#[test]
#[ignore]
fn missing_media_source() {
    assert!(Media::file(root().join("missing.png")).is_err());
    built(project());
}
#[test]
#[ignore]
fn missing_media_reference() {
    let mut p = Project::new("capability").unwrap();
    p.add(
        "one",
        Note::basic("front", Content::html("<img src=\"missing.png\">")),
    )
    .unwrap();
    let error = p.build(BuildOptions::temporary()).unwrap_err();
    assert!(error
        .report()
        .diagnostics()
        .iter()
        .any(|d| d.code == "MEDIA.MISSING_REFERENCE"));
    p.add_asset(image().with_export_name("missing.png").unwrap())
        .unwrap();
    built(p);
}
#[test]
#[ignore]
fn explicit_unreferenced_asset_is_retained() {
    let mut p = project();
    p.add_asset(
        Media::bytes(b".card{color:red}".to_vec(), "text/css")
            .unwrap()
            .with_export_name("theme.css")
            .unwrap(),
    )
    .unwrap();
    let out = p.build(BuildOptions::temporary()).unwrap();
    assert_eq!(out.report().counts().media, 1);
    save(out);
}
#[test]
#[ignore]
fn unsafe_media_reference() {
    let mut p = Project::new("capability").unwrap();
    p.add(
        "one",
        Note::basic("front", Content::html("<img src=\"../unsafe.png\">")),
    )
    .unwrap();
    let error = p.build(BuildOptions::temporary()).unwrap_err();
    assert!(error
        .report()
        .diagnostics()
        .iter()
        .any(|d| d.code.contains("UNSAFE")));
    built(project());
}
#[test]
#[ignore]
fn unsafe_media_export_filename() {
    assert!(image().with_export_name("../unsafe.png").is_err());
    built(project());
}
#[test]
#[ignore]
fn mime_mismatch() {
    assert_eq!(
        Media::bytes(fs::read(common::fixture("pixel.png")).unwrap(), "audio/wav")
            .unwrap_err()
            .code(),
        "MEDIA.TYPE_MISMATCH"
    );
    built(project());
}
#[test]
#[ignore]
fn baseline_apkg_unreadable() {
    assert!(project()
        .build(BuildOptions::temporary().update_from(root().join("missing.apkg")))
        .is_err());
    built(project());
}
#[test]
#[ignore]
fn update_preserves_guid() {
    let first = project()
        .build(BuildOptions::to(root().join("v1.apkg")))
        .unwrap();
    let mut p = Project::new("capability").unwrap();
    p.add("one", Note::basic("front", "changed answer"))
        .unwrap();
    let next = p
        .build(BuildOptions::temporary().update_from(first.artifact().path()))
        .unwrap();
    assert_eq!(
        common::evidence(first.artifact().path())["identity"]["notes"]["one"]["guid"],
        common::evidence(next.artifact().path())["identity"]["notes"]["one"]["guid"]
    );
    save(next);
}
#[test]
#[ignore]
fn update_adds_new_note() {
    let first = project()
        .build(BuildOptions::to(root().join("v1.apkg")))
        .unwrap();
    let mut p = project();
    p.add("two", Note::basic("new", "answer")).unwrap();
    let out = p
        .build(BuildOptions::temporary().update_from(first.artifact().path()))
        .unwrap();
    assert_eq!(out.report().counts().notes, 2);
    save(out);
}
#[test]
#[ignore]
fn field_rename_preserves_keys_and_ids() {
    let first = custom_project(false, false)
        .build(BuildOptions::to(root().join("v1.apkg")))
        .unwrap();
    let next = custom_project(false, true)
        .build(BuildOptions::temporary().update_from(first.artifact().path()))
        .unwrap();
    assert_eq!(
        common::evidence(first.artifact().path())["identity"]["models"]["custom"]["fields"],
        common::evidence(next.artifact().path())["identity"]["models"]["custom"]["fields"]
    );
    save(next);
}
#[test]
#[ignore]
fn template_reorder_preserves_card_ordinals() {
    let first = custom_project(false, false)
        .build(BuildOptions::to(root().join("v1.apkg")))
        .unwrap();
    let next = custom_project(true, false)
        .build(BuildOptions::temporary().update_from(first.artifact().path()))
        .unwrap();
    assert_eq!(
        common::evidence(first.artifact().path())["identity"]["notes"]["one"]["cards"],
        common::evidence(next.artifact().path())["identity"]["notes"]["one"]["cards"]
    );
    save(next);
}
#[test]
#[ignore]
fn conflicting_model_is_rejected_atomically() {
    let mut p = custom_project(false, false);
    let other = custom(false, true);
    assert!(p.add("two", other.note().field("front", "front")).is_err());
    assert_eq!(
        p.build(BuildOptions::temporary())
            .unwrap()
            .report()
            .counts()
            .notes,
        1
    );
    built(p);
}
#[test]
#[ignore]
fn unknown_template_field_fails_at_schema_completion() {
    let error = NoteType::builder("invalid")
        .field(Field::new("front"))
        .template(Template::new("card").front("{{unknown}}"))
        .build()
        .unwrap_err();
    assert_eq!(error.code(), "TEMPLATE.RENDER_FIELD_UNKNOWN");
    built(project());
}
#[test]
#[ignore]
fn basic_apkg() {
    built(project());
}
#[test]
#[ignore]
fn cloze_apkg() {
    let mut p = Project::new("capability").unwrap();
    p.add("one", Note::cloze("{{c1::Madrid}} in {{c2::Spain}}"))
        .unwrap();
    let out = p.build(BuildOptions::temporary()).unwrap();
    assert_eq!(out.report().counts().cards, 2);
    save(out);
}
#[test]
#[ignore]
fn image_occlusion_apkg() {
    let mut p = Project::new("capability").unwrap();
    for (key, mode) in [
        ("all", OcclusionMode::HideAllGuessOne),
        ("one", OcclusionMode::HideOneGuessOne),
    ] {
        p.add(
            key,
            Note::image_occlusion(image())
                .mode(mode)
                .mask(Mask::rect("nucleus", 10, 10, 20, 20))
                .mask(Mask::rect("wall", 50, 20, 20, 20))
                .build()
                .unwrap(),
        )
        .unwrap();
    }
    let out = p.build(BuildOptions::temporary()).unwrap();
    assert_eq!(out.report().counts().cards, 4);
    save(out);
}
#[test]
#[ignore]
fn owned_snapshot_survives_source_removal() {
    let path = root().join("snapshot.png");
    fs::copy(common::fixture("pixel.png"), &path).unwrap();
    let media = Media::file(&path).unwrap();
    fs::remove_file(path).unwrap();
    let mut p = Project::new("capability").unwrap();
    p.add("one", Note::basic(media.image(), "image")).unwrap();
    built(p);
}
#[test]
#[ignore]
fn mixed_stock_and_custom_apkg() {
    let mut p = project();
    p.add("cloze", Note::cloze("{{c1::fact}}")).unwrap();
    p.add(
        "custom",
        custom(false, false).note().field("front", "front"),
    )
    .unwrap();
    let out = p.build(BuildOptions::temporary()).unwrap();
    assert_eq!(out.report().counts().notes, 3);
    assert_eq!(out.report().counts().cards, 4);
    save(out);
}
#[test]
#[ignore]
fn custom_bundle_apkg() {
    let model = NoteType::from_bundle(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../contracts/fixtures/template-bundle/custom-cloze"),
    )
    .unwrap();
    let mut p = Project::new("capability").unwrap();
    p.add("one", model.note().field("text", "{{c1::word}}"))
        .unwrap();
    built(p);
}
#[test]
#[ignore]
fn compare_and_policy_acceptance_preserve_original_risks() {
    let mut before = project();
    before
        .add("removed", Note::basic("removed", "answer"))
        .unwrap();
    let first = before
        .build(BuildOptions::to(root().join("v1.apkg")))
        .unwrap();
    let comparison = project()
        .compare(CompareOptions::against(first.artifact().path()))
        .unwrap();
    assert!(!comparison.policy().allows_publication());
    let out = project()
        .build(
            BuildOptions::temporary()
                .update_from(first.artifact().path())
                .update_policy(UpdatePolicy::default().allow(RiskCode::NoteRemoved)),
        )
        .unwrap();
    assert!(out
        .report()
        .comparison()
        .unwrap()
        .findings()
        .iter()
        .any(|f| f.code() == RiskCode::NoteRemoved));
    save(out);
}
