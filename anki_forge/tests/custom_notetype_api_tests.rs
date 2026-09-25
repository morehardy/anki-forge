use ankiforge::{
    diagnostics::Severity,
    schema::{GenerationRule, TemplateSide},
};
use ankiforge::{BuildOptions, Field, NoteType, Project, Template};

#[test]
fn template_target_decks_reject_invalid_names_at_model_completion() {
    for deck in [
        "",
        " ",
        "::Child",
        "Parent::",
        "Parent::::Child",
        " Parent",
        "Parent ::Child",
        "Parent:: Child",
        "Parent\nChild",
        "Parent\u{1f}Child",
    ] {
        let error = NoteType::builder("vocab")
            .field(Field::new("front"))
            .template(
                Template::new("recognition")
                    .front("{{front}}")
                    .target_deck(deck),
            )
            .build()
            .expect_err(&format!("invalid target deck {deck:?} must not complete"));
        assert_eq!(
            error.kind(),
            ankiforge::schema::SchemaErrorKind::InvalidName
        );
        assert_eq!(error.code(), "SCHEMA.NAME_INVALID");
    }
}

#[test]
fn bundle_target_decks_use_the_same_model_validation() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("front.html"), "{{front}}").unwrap();
    std::fs::write(root.path().join("back.html"), "{{front}}").unwrap();
    for deck in ["", "Parent::", "Parent:: Child"] {
        std::fs::write(root.path().join("anki-template.yaml"), format!(
            "format_version: template-bundle-v2\nnote_type:\n  key: vocab\n  fields:\n    - key: front\n  templates:\n    - key: recognition\n      front_file: front.html\n      back_file: back.html\n      target_deck: {deck:?}\n"
        )).unwrap();
        let error = NoteType::from_bundle(root.path()).unwrap_err();
        let cause = std::error::Error::source(&error)
            .unwrap()
            .downcast_ref::<ankiforge::schema::SchemaError>()
            .unwrap();
        assert_eq!(cause.code(), "SCHEMA.NAME_INVALID");
    }
}

#[test]
fn validated_model_retains_keys_labels_and_template_options() {
    let vocab = NoteType::builder("jp-vocab")
        .name("Japanese Vocabulary")
        .field(Field::new("expr").name("Expression").sort())
        .field(Field::new("meaning").name("Meaning").required())
        .field(Field::new("audio").name("Audio"))
        .template(
            Template::new("recognition")
                .name("Recognition")
                .front("{{expr}}")
                .back("{{FrontSide}}<hr>{{meaning}}")
                .browser_front("{{expr}}")
                .browser_back("{{meaning}}")
                .target_deck("Japanese::Recognition")
                .generate_when(GenerationRule::all(["expr"])),
        )
        .build()
        .unwrap();
    assert_eq!(vocab.key(), "jp-vocab");
    assert_eq!(vocab.display_name(), "Japanese Vocabulary");
    assert_eq!(vocab.fields()[0].key().as_str(), "expr");
    assert!(vocab.fields()[0].is_sort());
    assert!(vocab.fields()[1].is_required());
    assert!(!vocab.fields()[2].is_required());
    assert_eq!(vocab.templates()[0].key().as_str(), "recognition");
    assert_eq!(
        vocab.templates()[0].browser_front_source(),
        Some("{{expr}}")
    );
    assert_eq!(
        vocab.templates()[0].target_deck_name(),
        Some("Japanese::Recognition")
    );
}

#[test]
fn completing_a_model_rejects_unknown_template_keys_at_the_original_location() {
    let error = NoteType::builder("vocab")
        .field(Field::new("expr").name("Expression"))
        .template(Template::new("recognition").front("{{TypoField}}"))
        .build()
        .unwrap_err();
    assert_eq!(error.code(), "TEMPLATE.RENDER_FIELD_UNKNOWN");
    let location = error.location().unwrap();
    assert_eq!(location.template.as_str(), "recognition");
    assert_eq!(location.side, TemplateSide::Front);
    assert_eq!(location.byte_range, 2..11);
}

fn warning_project() -> Project {
    let model = NoteType::builder("portable")
        .field(Field::new("front"))
        .template(
            Template::new("card")
                .front("{{addon_filter:front}}")
                .back("{{front}}"),
        )
        .build()
        .unwrap();
    let mut project = Project::new("portable").unwrap();
    project
        .add("hello", model.note().field("front", "hello"))
        .unwrap();
    project
}

#[test]
fn unknown_addon_filters_are_nonfatal_portability_warnings() {
    let output = warning_project().build(BuildOptions::temporary()).unwrap();
    let diagnostic = output
        .report()
        .diagnostics()
        .iter()
        .find(|d| d.code == "TEMPLATE.FILTER_UNKNOWN")
        .unwrap();
    assert_eq!(diagnostic.severity, Severity::Warning);
    assert!(output.artifact().path().is_file());
    assert!(matches!(
        output.snapshot().result,
        ankiforge::build::json::BuildResultSnapshot::Success { .. }
    ));
}

#[test]
fn build_reports_each_template_warning_once() {
    let output = warning_project().build(BuildOptions::temporary()).unwrap();
    assert_eq!(
        output
            .report()
            .diagnostics()
            .iter()
            .filter(|d| d.code == "TEMPLATE.FILTER_UNKNOWN")
            .count(),
        1
    );
}

#[test]
fn model_and_template_keys_do_not_inherit_field_expression_restrictions() {
    let model = NoteType::builder("course:vocab")
        .field(Field::new("front"))
        .template(Template::new("card:recognition").front("{{front}}"))
        .build()
        .unwrap();
    assert_eq!(model.key(), "course:vocab");
    assert_eq!(model.templates()[0].key().as_str(), "card:recognition");
    let mut project = Project::new("course").unwrap();
    project
        .add("word", model.note().field("front", "hello"))
        .unwrap();
    assert_eq!(
        project
            .build(BuildOptions::temporary())
            .unwrap()
            .report()
            .counts()
            .cards,
        1
    );
}

#[test]
fn empty_filter_segments_are_syntax_errors_at_the_authored_expression() {
    for source in [
        "前缀 {{text::front}}",
        "前缀 {{:front}}",
        "前缀 {{text: :front}}",
    ] {
        let error = NoteType::builder("filters")
            .field(Field::new("front"))
            .template(Template::new("card").front(source))
            .build()
            .unwrap_err();
        assert_eq!(error.code(), "TEMPLATE.SYNTAX_INVALID");
        assert_eq!(error.location().unwrap().byte_range, 7..source.len());
    }
}

#[test]
fn generation_rules_reject_duplicate_keys_when_completing_the_model() {
    for rule in [
        GenerationRule::all(["front", "front"]),
        GenerationRule::any(["front", "front"]),
    ] {
        let error = NoteType::builder("duplicates")
            .field(Field::new("front"))
            .template(Template::new("card").front("{{front}}").generate_when(rule))
            .build()
            .unwrap_err();
        assert_eq!(error.code(), "SCHEMA.GENERATION_INVALID");
    }
}
