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

fn named_model(key: &str, name: &str) -> NoteType {
    NoteType::builder(key)
        .name(name)
        .field(Field::new("question"))
        .template(Template::new("card").front("{{question}}"))
        .build()
        .unwrap()
}

#[test]
fn model_name_conflicts_fail_atomically_before_building() {
    for stock_first in [false, true] {
        for name in ["Basic", " Basic ", "\u{a0}Basic\u{2003}"] {
            let mut project = Project::new("model-conflicts").unwrap();
            let custom = named_model("custom", name);
            let asset = Media::file(common::fixture("pixel.png")).unwrap();
            let (first, conflicting, repaired) = if stock_first {
                (
                    Note::basic("first", "answer"),
                    custom.note().field("question", asset.image()),
                    named_model("custom", "Distinct")
                        .note()
                        .field("question", "repaired"),
                )
            } else {
                (
                    custom.note().field("question", "first"),
                    Note::basic("conflicting", asset.image()),
                    custom.note().field("question", "repaired"),
                )
            };
            project.add("first", first).unwrap();
            let before = project.build(BuildOptions::temporary()).unwrap();
            let error = project.add("second", conflicting).unwrap_err();
            assert_eq!(error.kind(), ankiforge::note::AddErrorKind::ModelConflict);
            assert_eq!(error.code(), "NOTE.MODEL_CONFLICT");
            assert_eq!(project.len(), 1);
            let unchanged = project.build(BuildOptions::temporary()).unwrap();
            assert_eq!(unchanged.report().counts(), before.report().counts());
            assert_eq!(
                common::fields(unchanged.artifact().path()),
                common::fields(before.artifact().path())
            );
            project.add("second", repaired).unwrap();
            let after = project.build(BuildOptions::temporary()).unwrap();
            assert_eq!(after.report().counts().notes, 2);
            assert_eq!(after.report().counts().media, 0);
        }
    }
}

#[test]
fn distinct_custom_keys_cannot_share_a_normalized_model_name() {
    let mut project = Project::new("custom-model-conflict").unwrap();
    project
        .add(
            "first",
            named_model("one", " Shared ")
                .note()
                .field("question", "one"),
        )
        .unwrap();
    let error = project
        .add(
            "second",
            named_model("two", "Shared").note().field("question", "two"),
        )
        .unwrap_err();
    assert_eq!(error.code(), "NOTE.MODEL_CONFLICT");
    project
        .add(
            "second",
            named_model("two", "Other").note().field("question", "two"),
        )
        .unwrap();
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
fn model_name_comparison_preserves_case_unicode_and_internal_spaces() {
    let names = ["Basic", "basic", "É", "E\u{301}", "A B", "A  B"];
    let mut project = Project::new("distinct-model-names").unwrap();
    for (index, name) in names.iter().enumerate() {
        let model = named_model(&format!("model-{index}"), name);
        for copy in 0..2 {
            project
                .add(
                    format!("note-{index}-{copy}"),
                    model.note().field("question", "value"),
                )
                .unwrap();
        }
    }
    let output = project.build(BuildOptions::temporary()).unwrap();
    assert_eq!(output.report().counts().notes, 12);
    let (_root, db) = common::collection(output.artifact().path());
    let mut query = db
        .prepare("SELECT name FROM notetypes ORDER BY name")
        .unwrap();
    let actual = query
        .query_map([], |row| row.get::<_, String>(0))
        .unwrap()
        .map(Result::unwrap)
        .collect::<Vec<_>>();
    let mut expected = names.to_vec();
    expected.sort();
    assert_eq!(actual, expected);
}

#[test]
fn raw_field_delimiters_are_rejected_in_every_authored_content_form() {
    let image = Media::file(common::fixture("pixel.png")).unwrap();
    let custom = named_model("custom-delimiter", "Custom delimiter");
    let cases = [
        Note::basic("a\u{1f}b", "c"),
        Note::basic("a", "b\u{1f}c"),
        Note::basic(Content::html("<b>a\u{1f}b</b>"), "c"),
        Note::basic(
            Content::sequence([
                Content::text("before"),
                Content::sequence([Content::html("<i>valid</i>"), Content::text("a\u{1f}b")]),
            ]),
            "back",
        ),
        Note::basic(
            Content::sequence([
                image.image(),
                Content::sequence([Content::text("valid"), Content::html("a\u{1f}b")]),
            ]),
            "back",
        ),
        Note::cloze("{{c1::answer\u{1f}extra}}"),
        Note::cloze("{{c1::answer::hint\u{1f}extra}}"),
        Note::cloze(Content::html("<b>{{c1::answer::hint\u{1f}extra}}</b>")),
        Note::cloze("{{c1::answer}}").field("back_extra", "a\u{1f}b"),
        custom.note().field("question", "a\u{1f}b"),
        Note::image_occlusion(image)
            .mask(Mask::rect("pixel", 0, 0, 1, 1))
            .build()
            .unwrap()
            .field("header", Content::html("a\u{1f}b")),
    ];
    for (index, note) in cases.into_iter().enumerate() {
        let mut project = Project::new("field-delimiter").unwrap();
        let original = note.clone();
        let error = project
            .add("one", note.clone())
            .expect_err("raw field separator must not enter the project");
        assert_eq!(error.code(), "NOTE.FIELD_CONTENT_INVALID", "case {index}");
        assert_eq!(error.kind(), ankiforge::note::AddErrorKind::InvalidContent);
        assert!(error.to_string().contains("U+001F"));
        assert_eq!(
            note, original,
            "validation must not rewrite the authored value"
        );
        assert!(project.is_empty());
        project
            .add("one", Note::basic("repaired", "answer"))
            .unwrap();
        let output = project.build(BuildOptions::temporary()).unwrap();
        assert_eq!(output.report().counts().notes, 1);
        assert_eq!(output.report().counts().media, 0);
        assert_eq!(
            common::fields(output.artifact().path()),
            ["repaired\u{1f}answer"]
        );
    }
}

#[test]
fn invalid_field_content_does_not_reserve_keys_models_or_media() {
    let mut project = Project::new("atomic-field-content").unwrap();
    project
        .add("existing", Note::basic("unchanged", "answer"))
        .unwrap();
    let before = project.build(BuildOptions::temporary()).unwrap();
    let image = Media::file(common::fixture("pixel.png")).unwrap();
    let model = named_model("custom", "Rejected model");
    let note = model.note().field(
        "question",
        Content::sequence([image.image(), Content::text("bad\u{1f}field")]),
    );
    let error = project.add("retry", note).unwrap_err();
    assert_eq!(error.code(), "NOTE.FIELD_CONTENT_INVALID");
    assert_eq!(project.len(), 1);
    let after = project
        .build(BuildOptions::temporary().update_from(before.artifact().path()))
        .unwrap();
    assert_eq!(
        common::entries(after.artifact().path()),
        common::entries(before.artifact().path())
    );
    let replacement = named_model("custom", "Accepted model");
    project
        .add("retry", replacement.note().field("question", "accepted"))
        .unwrap();
    let output = project.build(BuildOptions::temporary()).unwrap();
    assert_eq!(output.report().counts().notes, 2);
    assert_eq!(output.report().counts().media, 0);
}

#[test]
fn valid_control_characters_and_html_entities_keep_exact_field_boundaries() {
    let controls = "line\nnext\tcolumn\rreturn\u{1e}record\u{7f}end";
    let mut project = Project::new("valid-field-controls").unwrap();
    project.add("text", Note::basic(controls, "back")).unwrap();
    project
        .add(
            "html",
            Note::basic(
                Content::html(format!("<p>{controls} &#31; &#x1f;</p>")),
                "back",
            ),
        )
        .unwrap();
    project
        .add(
            "sequence",
            Note::basic(
                Content::sequence([
                    Content::text("before\n"),
                    Content::html("<b>middle\t</b>"),
                    Content::text("after\r"),
                ]),
                "back",
            ),
        )
        .unwrap();
    project
        .add(
            "cloze",
            Note::cloze(format!("{{{{c1::{controls}::hint\ttext}}}}")),
        )
        .unwrap();
    let output = project.build(BuildOptions::temporary()).unwrap();
    let fields = common::fields(output.artifact().path());
    assert_eq!(fields.len(), 4);
    assert!(fields
        .iter()
        .all(|value| value.split('\u{1f}').count() == 2));
    assert!(fields.contains(&format!("{controls}\u{1f}back")));
    assert!(fields.contains(&format!("<p>{controls} &#31; &#x1f;</p>\u{1f}back")));
    assert!(fields.contains(&"before\n<b>middle\t</b>after\r\u{1f}back".to_string()));
    assert!(fields.contains(&format!("{{{{c1::{controls}::hint\ttext}}}}\u{1f}")));
}
