mod common;
use ankiforge::schema::GenerationRule;
use ankiforge::{BuildOptions, Content, Field, Note, NoteType, Project, Template};

fn single(note: Note) -> Project {
    let mut p = Project::new("smoke").unwrap();
    p.add("one", note).unwrap();
    p
}
fn normal(front: &str, rule: GenerationRule) -> NoteType {
    NoteType::builder("normal")
        .field(Field::new("prompt").name("Prompt"))
        .field(Field::new("extra").name("Extra"))
        .field(Field::new("context").name("Context"))
        .template(
            Template::new("card")
                .front(front)
                .back("{{extra}}")
                .generate_when(rule),
        )
        .build()
        .unwrap()
}
fn card_ordinals(output: &ankiforge::BuildOutput) -> Vec<u32> {
    let (_root, db) = common::collection(output.artifact().path());
    let mut query = db.prepare("select ord from cards order by ord").unwrap();
    query
        .query_map([], |row| row.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

#[test]
fn stock_basic_and_cloze_share_one_project_with_distinct_card_counts() {
    let mut p = single(Note::basic("front", "back"));
    p.add(
        "two",
        Note::cloze("{{c1::Madrid}} is in {{c2::Spain}} and {{c1::Europe}}"),
    )
    .unwrap();
    let output = p.build(BuildOptions::temporary()).unwrap();
    assert_eq!(output.report().counts().notes, 2);
    assert_eq!(output.report().counts().cards, 3);
    assert_eq!(card_ordinals(&output), [0, 0, 1]);
}

#[test]
fn many_templates_create_unique_card_ids_across_notes() {
    let mut builder = NoteType::builder("many").field(Field::new("front"));
    for index in 0..12 {
        builder = builder.template(Template::new(format!("card-{index}")).front("{{front}}"));
    }
    let model = builder.build().unwrap();
    let mut project = single(model.note().field("front", "first"));
    project
        .add("two", model.note().field("front", "second"))
        .unwrap();
    let output = project.build(BuildOptions::temporary()).unwrap();
    let (_root, db) = common::collection(output.artifact().path());
    let count: usize = db
        .query_row("select count(distinct id) from cards", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 24);
    assert_eq!(output.report().counts().cards, 24);
}

#[test]
fn custom_cloze_uses_distinct_supported_ordinals() {
    let model = NoteType::builder("language-cloze")
        .field(Field::new("text").name("Sentence").sort().required())
        .field(Field::new("extra").name("Extra"))
        .cloze_field("text")
        .template(
            Template::new("cloze")
                .front("{{cloze:text}}")
                .back("{{cloze:text}}<br>{{extra}}"),
        )
        .build()
        .unwrap();
    let output = single(
        model
            .note()
            .field("text", "{{c1::Madrid}} is in {{c3::Spain}}"),
    )
    .build(BuildOptions::temporary())
    .unwrap();
    assert_eq!(card_ordinals(&output), [0, 2]);
}

#[test]
fn optional_fields_can_be_omitted_without_changing_order() {
    let model = normal("{{prompt}}", GenerationRule::AnkiDefault);
    let output = single(model.note().field("prompt", "hello"))
        .build(BuildOptions::temporary())
        .unwrap();
    assert_eq!(
        common::fields(output.artifact().path()),
        ["hello\u{1f}\u{1f}"]
    );
}

#[test]
fn missing_required_field_fails_during_atomic_add() {
    let model = NoteType::builder("required")
        .field(Field::new("prompt").required())
        .field(Field::new("extra"))
        .template(Template::new("card").front("{{prompt}}"))
        .build()
        .unwrap();
    let mut project = Project::new("required").unwrap();
    let error = project
        .add("one", model.note().field("extra", "context"))
        .unwrap_err();
    assert_eq!(error.code(), "NOTE.FIELD_REQUIRED");
    project
        .add("one", model.note().field("prompt", "fixed"))
        .unwrap();
    assert_eq!(
        project
            .build(BuildOptions::temporary())
            .unwrap()
            .report()
            .counts()
            .notes,
        1
    );
}

#[test]
fn a_non_first_sort_field_is_written_to_anki() {
    let model = NoteType::builder("sorted")
        .field(Field::new("prompt"))
        .field(Field::new("sort_key").sort())
        .template(
            Template::new("card")
                .front("{{prompt}}")
                .back("{{sort_key}}"),
        )
        .build()
        .unwrap();
    let output = single(
        model
            .note()
            .field("prompt", "visible prompt")
            .field("sort_key", "001"),
    )
    .build(BuildOptions::temporary())
    .unwrap();
    let (_root, db) = common::collection(output.artifact().path());
    let actual: String = db
        .query_row("select cast(sfld as text) from notes", [], |row| row.get(0))
        .unwrap();
    assert_eq!(actual, "1"); // Anki stores numeric sort fields using SQLite numeric affinity.
}

#[test]
fn static_front_and_unconditional_inverted_front_generate_without_fields() {
    for front in ["Always visible", "Always{{^prompt}}fallback{{/prompt}}"] {
        let model = normal(front, GenerationRule::AnkiDefault);
        let output = single(model.note())
            .build(BuildOptions::temporary())
            .unwrap();
        assert_eq!(output.report().counts().cards, 1);
    }
}

#[test]
fn empty_media_attribute_field_does_not_generate_a_card() {
    let model = normal("<img src=\"{{prompt}}\">", GenerationRule::AnkiDefault);
    let output = single(model.note())
        .build(BuildOptions::temporary())
        .unwrap();
    assert_eq!(output.report().counts().cards, 0);
}

#[test]
fn positive_section_requires_both_guard_and_rendered_field() {
    let model = normal(
        "{{#prompt}}{{extra}}{{/prompt}}",
        GenerationRule::AnkiDefault,
    );
    let output = single(model.note().field("extra", "not enough by itself"))
        .build(BuildOptions::temporary())
        .unwrap();
    assert_eq!(output.report().counts().cards, 0);
    let output = single(
        model
            .note()
            .field("prompt", "guard")
            .field("extra", "visible"),
    )
    .build(BuildOptions::temporary())
    .unwrap();
    assert_eq!(output.report().counts().cards, 1);
}

#[test]
fn unrepresentable_default_front_requires_an_explicit_generation_rule() {
    let front = "{{#prompt}}{{extra}}{{/prompt}}{{context}}";
    let model = normal(front, GenerationRule::AnkiDefault);
    let error = single(model.note().field("context", "visible"))
        .build(BuildOptions::temporary())
        .unwrap_err();
    assert_eq!(error.code(), "TEMPLATE.GENERATION_RULE_REQUIRED");
    let model = normal(front, GenerationRule::any(["context"]));
    let output = single(model.note().field("context", "visible"))
        .build(BuildOptions::temporary())
        .unwrap();
    assert_eq!(output.report().counts().cards, 1);
}

#[test]
fn malformed_cloze_is_rejected_before_publication() {
    for text in ["{{c1::unclosed", "{{c0::zero}}", "{{c1::}}"] {
        let error = single(Note::cloze(text))
            .build(BuildOptions::temporary())
            .unwrap_err();
        assert!(
            error
                .report()
                .diagnostics()
                .iter()
                .any(|d| d.code.contains("CLOZE")),
            "{error:?}"
        );
        assert!(error.publications().is_empty());
    }
}

#[test]
fn content_semantics_do_not_depend_on_note_kind() {
    let mut project = single(Note::basic("<b>text</b>", Content::html("<b>html</b>")));
    project
        .add("two", Note::cloze("{{c1::<b>text</b>}}"))
        .unwrap();
    let output = project.build(BuildOptions::temporary()).unwrap();
    let fields = common::fields(output.artifact().path());
    assert!(fields.contains(&"&lt;b&gt;text&lt;/b&gt;\u{1f}<b>html</b>".into()));
    assert!(fields.contains(&"{{c1::&lt;b&gt;text&lt;/b&gt;}}\u{1f}".into()));
}

#[test]
fn artifact_bytes_are_a_complete_package_while_the_handle_is_alive() {
    let output = single(Note::basic("front", "back"))
        .build(BuildOptions::temporary())
        .unwrap();
    let bytes = std::fs::read(output.artifact().path()).unwrap();
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    assert!(archive.by_name("collection.anki21b").is_ok());
    assert!(archive.by_name("ankiforge-identity.json").is_ok());
}
