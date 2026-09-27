use std::{collections::BTreeMap, sync::OnceLock};

use super::{Field, NoteType, Template};

pub(crate) fn basic() -> &'static NoteType {
    static MODEL: OnceLock<NoteType> = OnceLock::new();
    MODEL.get_or_init(|| stock("basic", &["front", "back"], None))
}

pub(crate) fn cloze() -> &'static NoteType {
    static MODEL: OnceLock<NoteType> = OnceLock::new();
    MODEL.get_or_init(|| stock("cloze", &["text", "back_extra"], Some("text")))
}

pub(crate) fn image_occlusion() -> &'static NoteType {
    static MODEL: OnceLock<NoteType> = OnceLock::new();
    MODEL.get_or_init(|| {
        stock(
            "image_occlusion",
            &["occlusion", "image", "header", "back_extra", "comments"],
            Some("occlusion"),
        )
    })
}

fn stock(kind: &'static str, keys: &[&str], cloze: Option<&str>) -> NoteType {
    let defaults = crate::authoring_core::stock::stock_lowering_defaults(kind)
        .expect("built-in note type has canonical defaults");
    let bindings = defaults
        .fields
        .iter()
        .zip(keys)
        .map(|(field, key)| (field.name.as_str(), *key))
        .collect::<BTreeMap<_, _>>();
    let bind = |source: &str| {
        let parsed = crate::authoring_core::parse_template(source);
        crate::authoring_core::template_parser::rewrite_fields(source, &parsed, &bindings)
    };
    let mut builder = NoteType::builder(kind)
        .name(defaults.name.clone())
        .css(defaults.css.clone());
    builder.stock_kind = Some(kind);
    for (index, (field, key)) in defaults.fields.iter().zip(keys).enumerate() {
        let declaration = Field::new(*key).name(field.name.clone());
        builder = builder.field(if index == 0 {
            declaration.sort()
        } else {
            declaration
        });
    }
    for template in &defaults.templates {
        builder = builder.template(
            Template::new("card")
                .name(template.name.clone())
                .front(bind(&template.question_format))
                .back(bind(&template.answer_format)),
        );
    }
    if let Some(field) = cloze {
        builder = builder.cloze_field(field);
    }
    builder
        .build()
        .expect("built-in note type satisfies schema validation")
}
