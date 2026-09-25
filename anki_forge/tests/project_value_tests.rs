mod common;
use ankiforge::note::{Mask, OcclusionMode};
use ankiforge::{BuildOptions, Content, Field, Media, Note, NoteType, Project, Template};

fn stock_project() -> Project {
    let mut project = Project::new("spanish").unwrap().default_deck("Spanish");
    project.add("hola", Note::basic("hola", "hello")).unwrap();
    project
        .add("capital", Note::cloze("La capital es {{c1::Madrid}}"))
        .unwrap();
    project
}

#[test]
fn borrowed_builds_and_clones_preserve_authoring_values() {
    let project = stock_project();
    let first = project.build(BuildOptions::temporary()).unwrap();
    let repeated = project
        .build(BuildOptions::temporary().update_from(first.artifact().path()))
        .unwrap();
    let cloned = project
        .clone()
        .build(BuildOptions::temporary().update_from(first.artifact().path()))
        .unwrap();
    assert_eq!(first.report().counts(), repeated.report().counts());
    assert_eq!(
        common::entries(first.artifact().path()),
        common::entries(cloned.artifact().path())
    );
    assert_eq!(
        common::fields(first.artifact().path()),
        common::fields(repeated.artifact().path())
    );
}

#[test]
fn structured_occlusion_uses_the_same_project_as_basic_and_cloze() {
    let mut project = stock_project();
    let image = Media::file(common::fixture("occlusion.png")).unwrap();
    let note = Note::image_occlusion(image)
        .mode(OcclusionMode::HideAllGuessOne)
        .mask(Mask::rect("heart", 10, 20, 40, 40))
        .build()
        .unwrap();
    project.add("heart", note).unwrap();
    let output = project.build(BuildOptions::temporary()).unwrap();
    assert_eq!(output.report().counts().notes, 3);
    assert_eq!(output.report().counts().cards, 3);
    assert_eq!(output.report().counts().media, 1);
}

#[test]
fn appending_notes_after_a_build_affects_only_future_outputs() {
    let mut project = stock_project();
    let first = project.build(BuildOptions::temporary()).unwrap();
    project
        .add("adios", Note::basic("adios", "goodbye"))
        .unwrap();
    let next = project.build(BuildOptions::temporary()).unwrap();
    assert_eq!(first.report().counts().notes, 2);
    assert_eq!(common::fields(first.artifact().path()).len(), 2);
    assert_eq!(next.report().counts().notes, 3);
}

#[test]
fn clones_reserve_existing_keys_and_mutate_independently() {
    let project = stock_project();
    let mut cloned = project.clone();
    cloned
        .add("adios", Note::basic("adios", "goodbye"))
        .unwrap();
    let error = cloned
        .add("hola", Note::basic("duplicate", "answer"))
        .unwrap_err();
    assert_eq!(error.code(), "NOTE.KEY_DUPLICATE");
    assert_eq!(
        cloned
            .build(BuildOptions::temporary())
            .unwrap()
            .report()
            .counts()
            .notes,
        3
    );
    assert_eq!(
        project
            .build(BuildOptions::temporary())
            .unwrap()
            .report()
            .counts()
            .notes,
        2
    );
}

#[test]
fn explicit_assets_and_typed_media_share_atomic_conflict_checks() {
    let mut project = Project::new("media").unwrap();
    let original = Media::file(common::fixture("pixel.png"))
        .unwrap()
        .with_export_name("original.png")
        .unwrap();
    project.add_asset(original.clone()).unwrap();
    project
        .add(
            "original",
            Note::basic(Content::html("<b>front</b>"), original.image()),
        )
        .unwrap();
    let model = NoteType::builder("custom")
        .field(Field::new("question"))
        .template(Template::new("card").front("{{question}}"))
        .build()
        .unwrap();
    let extra = original.with_export_name("extra.png").unwrap();
    project
        .add("extra", model.note().field("question", extra.image()))
        .unwrap();
    let output = project.build(BuildOptions::temporary()).unwrap();
    assert_eq!(output.report().counts().notes, 2);
    assert_eq!(output.report().counts().media, 2);
    assert!(common::fields(output.artifact().path())
        .iter()
        .any(|f| f.starts_with("<b>front</b>")));
    let conflicting = Media::file(common::fixture("occlusion.png"))
        .unwrap()
        .with_export_name("original.png")
        .unwrap();
    assert_eq!(
        project.add_asset(conflicting).unwrap_err().code(),
        "MEDIA.DUPLICATE_FILENAME_CONFLICT"
    );
    assert_eq!(
        project
            .build(BuildOptions::temporary())
            .unwrap()
            .report()
            .counts()
            .media,
        2
    );
}

#[test]
fn content_and_display_names_do_not_change_explicit_note_identity() {
    let first = stock_project().build(BuildOptions::temporary()).unwrap();
    let before = common::evidence(first.artifact().path());
    let mut next = Project::new("spanish")
        .unwrap()
        .name("Renamed course")
        .default_deck("Renamed deck");
    next.add("hola", Note::basic("<b>hola</b>", "new answer"))
        .unwrap();
    next.add("capital", Note::cloze("Capital: {{c1::Madrid}}"))
        .unwrap();
    let updated = next
        .build(BuildOptions::temporary().update_from(first.artifact().path()))
        .unwrap();
    let after = common::evidence(updated.artifact().path());
    assert_eq!(
        before["identity"]["notes"]["hola"]["guid"],
        after["identity"]["notes"]["hola"]["guid"]
    );
}

#[test]
fn deck_components_with_edge_colons_fail_before_collection_or_build() {
    for deck in [
        ":",
        ":Parent",
        "Parent:",
        "Parent:::Child",
        "Parent::Child:",
    ] {
        let mut default = Project::new("invalid-default").unwrap().default_deck(deck);
        assert_eq!(
            default
                .add("one", Note::basic("front", "back"))
                .unwrap_err()
                .code(),
            "NOTE.DECK_INVALID"
        );
        assert_eq!(
            default.build(BuildOptions::temporary()).unwrap_err().code(),
            "NOTE.DECK_INVALID"
        );
        let mut overridden = Project::new("invalid-override").unwrap();
        assert_eq!(
            overridden
                .add("one", Note::basic("front", "back").deck(deck))
                .unwrap_err()
                .code(),
            "NOTE.DECK_INVALID"
        );
        overridden.add("one", Note::basic("front", "back")).unwrap();
    }
}

#[test]
fn project_default_deck_and_per_note_override_are_both_exported() {
    let mut project = Project::new("destinations").unwrap().default_deck("Course");
    project
        .add("default", Note::basic("one", "answer"))
        .unwrap();
    project
        .add(
            "override",
            Note::basic("two", "answer").deck("Course::Advanced: Part 1"),
        )
        .unwrap();
    let output = project.build(BuildOptions::temporary()).unwrap();
    let (_root, db) = common::collection(output.artifact().path());
    let mut query = db
        .prepare("select d.name from cards c join decks d on d.id=c.did order by d.name")
        .unwrap();
    let names: Vec<String> = query
        .query_map([], |row| row.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert_eq!(names, ["Course", "Course\u{1f}Advanced: Part 1"]);
}

#[test]
fn long_explicit_html_and_unicode_survive_output_and_identity_evidence() {
    let mut project = Project::new("long-html").unwrap();
    let long = "<p>中文 &amp; <em>answer</em></p>".repeat(4096);
    project
        .add(
            "long",
            Note::basic(
                Content::html("<b>问题 &amp; 答案</b>"),
                Content::html(&long),
            )
            .tag("中文"),
        )
        .unwrap();
    let output = project.build(BuildOptions::temporary()).unwrap();
    assert_eq!(
        common::fields(output.artifact().path()),
        [format!("<b>问题 &amp; 答案</b>\u{1f}{long}")]
    );
    assert!(
        common::evidence(output.artifact().path())["identity"]["notes"]["long"]["guid"].is_string()
    );
}

#[test]
fn missing_explicit_html_media_can_be_repaired_after_a_failed_build() {
    let mut project = Project::new("repair").unwrap();
    project
        .add(
            "note",
            Note::basic("front", Content::html("<img src=\"repair.png\">")),
        )
        .unwrap();
    let error = project.build(BuildOptions::temporary()).unwrap_err();
    assert!(error
        .report()
        .diagnostics()
        .iter()
        .any(|d| d.code == "MEDIA.MISSING_REFERENCE"));
    project
        .add_asset(
            Media::file(common::fixture("pixel.png"))
                .unwrap()
                .with_export_name("repair.png")
                .unwrap(),
        )
        .unwrap();
    assert_eq!(
        project
            .build(BuildOptions::temporary())
            .unwrap()
            .report()
            .counts()
            .media,
        1
    );
}
