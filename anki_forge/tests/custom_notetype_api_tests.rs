use anki_forge::prelude::*;

#[test]
fn custom_notetype_builder_records_keys_and_identity_recipe() {
    let vocab = NoteType::custom("jp-vocab")
        .name("Japanese Vocabulary")
        .field(Field::new("Expression").key("expr").identity().sort())
        .field(Field::new("Meaning").key("meaning").required())
        .field(Field::new("Audio").key("audio").optional())
        .template(
            Template::new("Recognition")
                .key("recognition")
                .front("{{Expression}}")
                .back("{{FrontSide}}<hr id=\"answer\">{{Meaning}}")
                .browser_front("{{Expression}}")
                .browser_back("{{Meaning}}")
                .target_deck("Japanese::Recognition")
                .generate_when(GenerationRule::all(["expr"])),
        )
        .identity(IdentityRecipe::fields(["expr"]));

    assert_eq!(vocab.id(), "jp-vocab");
    assert_eq!(vocab.name_ref(), Some("Japanese Vocabulary"));
    assert_eq!(vocab.fields()[0].key_ref().as_str(), "expr");
    assert!(vocab.fields()[0].is_identity());
    assert!(vocab.fields()[0].is_sort());
    assert!(vocab.fields()[1].is_required());
    assert!(vocab.fields()[2].is_optional());
    assert_eq!(vocab.templates()[0].key_ref().as_str(), "recognition");
    assert_eq!(
        vocab.templates()[0]
            .browser_front_source()
            .map(|source| source.as_str()),
        Some("{{Expression}}")
    );
    assert_eq!(
        vocab.templates()[0].target_deck_name(),
        Some("Japanese::Recognition")
    );
    assert_eq!(
        vocab.identity_ref().expect("identity").field_keys(),
        vec![FieldKey::new("expr")]
    );
}
