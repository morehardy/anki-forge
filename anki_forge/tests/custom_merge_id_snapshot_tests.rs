use anki_forge::prelude::*;
use anki_forge::product::stable_config_id;

#[test]
fn stable_config_id_snapshot_values_do_not_drift() {
    assert_eq!(
        stable_config_id("field", "jp-vocab", "expr"),
        2_921_591_957_654_962_622
    );
    assert_eq!(
        stable_config_id("field", "jp-vocab", "meaning"),
        8_939_348_238_921_914_692
    );
    assert_eq!(
        stable_config_id("template", "jp-vocab", "recognition"),
        3_934_332_856_449_685_517
    );
}

#[test]
fn custom_notetype_lowers_keys_to_config_ids() {
    let vocab = NoteType::custom("jp-vocab")
        .name("Japanese Vocabulary")
        .field(Field::new("Expression").key("expr").identity().sort())
        .field(Field::new("Meaning").key("meaning").required())
        .template(
            Template::new("Recognition")
                .key("recognition")
                .front("{{Expression}}")
                .back("{{FrontSide}}<hr id=\"answer\">{{Meaning}}")
                .generate_when(GenerationRule::all(["expr"])),
        )
        .identity(IdentityRecipe::fields(["expr"]));

    let mut project = Project::new("Japanese Core")
        .stable_id("jp-core")
        .default_deck("Japanese::Core");
    project.add_notetype(vocab).expect("add notetype");
    project
        .add_note(
            Note::new("jp-vocab")
                .stable_id("jp-vocab:taberu")
                .text("expr", "食べる")
                .text("meaning", "to eat"),
        )
        .expect("add note");

    let normalized = project.normalize().expect("normalize custom");
    let notetype = normalized
        .notetypes
        .iter()
        .find(|notetype| notetype.id == "jp-vocab")
        .expect("jp vocab notetype");

    assert_eq!(notetype.kind, "normal");
    assert_eq!(notetype.fields[0].name, "Expression");
    assert_eq!(
        notetype.fields[0].config_id,
        Some(2_921_591_957_654_962_622)
    );
    assert_eq!(notetype.templates[0].name, "Recognition");
    assert_eq!(
        notetype.templates[0].config_id,
        Some(3_934_332_856_449_685_517)
    );
}

#[test]
fn custom_note_field_names_take_precedence_over_field_keys() {
    let notetype = NoteType::custom("ambiguous-fields")
        .field(Field::new("Foo").key("bar"))
        .field(Field::new("bar").key("baz"))
        .template(
            Template::new("Card 1")
                .key("card-1")
                .front("{{Foo}}{{bar}}")
                .back("{{FrontSide}}"),
        );

    let mut project = Project::new("Ambiguous")
        .stable_id("ambiguous")
        .default_deck("Ambiguous");
    project.add_notetype(notetype).expect("add notetype");
    project
        .add_note(
            Note::new("ambiguous-fields")
                .stable_id("ambiguous:1")
                .text("Foo", "foo value")
                .text("bar", "visible field wins"),
        )
        .expect("add note");

    let normalized = project.normalize().expect("normalize custom");
    let note = normalized.notes.first().expect("normalized note");

    assert_eq!(
        note.fields.get("bar").map(String::as_str),
        Some("visible field wins")
    );
    assert_eq!(
        note.fields.get("Foo").map(String::as_str),
        Some("foo value")
    );
}

#[test]
fn custom_note_visible_field_value_wins_over_duplicate_stable_key() {
    let notetype = NoteType::custom("jp-vocab")
        .field(Field::new("Expression").key("expr"))
        .template(
            Template::new("Card 1")
                .key("card-1")
                .front("{{Expression}}")
                .back("{{FrontSide}}"),
        );

    let mut project = Project::new("Japanese Core")
        .stable_id("jp-core")
        .default_deck("Japanese::Core");
    project.add_notetype(notetype).expect("add notetype");
    project
        .add_note(
            Note::new("jp-vocab")
                .stable_id("jp-vocab:taberu")
                .text("Expression", "visible value")
                .text("expr", "key value"),
        )
        .expect("add note");

    let normalized = project.normalize().expect("normalize custom");
    let note = normalized.notes.first().expect("normalized note");

    assert_eq!(
        note.fields.get("Expression").map(String::as_str),
        Some("visible value")
    );
}

#[test]
fn custom_any_generation_rule_renders_front_at_most_once() {
    let notetype = NoteType::custom("jp-vocab")
        .field(Field::new("Expression").key("expr"))
        .field(Field::new("Meaning").key("meaning"))
        .template(
            Template::new("Any")
                .key("any")
                .front("{{Expression}} / {{Meaning}}")
                .back("{{FrontSide}}")
                .generate_when(GenerationRule::any(["expr", "meaning"])),
        );

    let mut project = Project::new("Japanese Core")
        .stable_id("jp-core")
        .default_deck("Japanese::Core");
    project.add_notetype(notetype).expect("add notetype");
    project
        .add_note(
            Note::new("jp-vocab")
                .stable_id("jp-vocab:taberu")
                .text("expr", "食べる")
                .text("meaning", "to eat"),
        )
        .expect("add note");

    let normalized = project.normalize().expect("normalize custom");
    let template = &normalized
        .notetypes
        .iter()
        .find(|notetype| notetype.id == "jp-vocab")
        .expect("jp vocab notetype")
        .templates[0];

    assert_eq!(
        template.question_format,
        "{{#Expression}}{{Expression}} / {{Meaning}}{{/Expression}}{{^Expression}}{{#Meaning}}{{Expression}} / {{Meaning}}{{/Meaning}}{{/Expression}}"
    );
}

#[test]
fn custom_notetype_rejects_duplicate_field_keys() {
    let notetype = NoteType::custom("jp-vocab")
        .field(Field::new("Expression").key("duplicate"))
        .field(Field::new("Reading").key("duplicate"))
        .template(
            Template::new("Card 1")
                .key("card-1")
                .front("{{Expression}}")
                .back("{{FrontSide}}"),
        );

    let mut project = Project::new("Japanese Core")
        .stable_id("jp-core")
        .default_deck("Japanese::Core");
    project.add_notetype(notetype).expect("add notetype");

    let err = project
        .normalize()
        .expect_err("duplicate field keys must not collide");
    assert!(
        err.to_string().contains("NOTETYPE.FIELD_KEY_DUPLICATE"),
        "unexpected error: {err}"
    );
}

#[test]
fn custom_notetype_rejects_duplicate_template_keys() {
    let notetype = NoteType::custom("jp-vocab")
        .field(Field::new("Expression").key("expr"))
        .template(
            Template::new("Recognition")
                .key("duplicate")
                .front("{{Expression}}")
                .back("{{FrontSide}}"),
        )
        .template(
            Template::new("Recall")
                .key("duplicate")
                .front("{{Expression}}")
                .back("{{FrontSide}}"),
        );

    let mut project = Project::new("Japanese Core")
        .stable_id("jp-core")
        .default_deck("Japanese::Core");
    project.add_notetype(notetype).expect("add notetype");

    let err = project
        .normalize()
        .expect_err("duplicate template keys must not collide");
    assert!(
        err.to_string().contains("NOTETYPE.TEMPLATE_KEY_DUPLICATE"),
        "unexpected error: {err}"
    );
}

#[test]
fn custom_notetype_rejects_cloze_generation_rule() {
    let vocab = NoteType::custom("jp-vocab")
        .field(Field::new("Expression").key("expr"))
        .template(
            Template::new("Cloze")
                .key("cloze")
                .front("{{cloze:Expression}}")
                .back("{{cloze:Expression}}")
                .generate_when(GenerationRule::Cloze {
                    field: FieldKey::new("expr"),
                }),
        );

    let mut project = Project::new("Japanese Core")
        .stable_id("jp-core")
        .default_deck("Japanese::Core");
    project.add_notetype(vocab).expect("add notetype");

    let err = project
        .normalize()
        .expect_err("custom cloze is out of scope");
    assert!(
        err.to_string()
            .contains("TEMPLATE.CLOZE_RULE_REQUIRES_STOCK_CLOZE"),
        "unexpected error: {err}"
    );
}
