use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateGenerationRequirement {
    Always,
    All(Vec<String>),
    Any(Vec<String>),
    Unrepresentable,
}

pub fn infer_generation_requirement(
    source: &str,
    declared_fields: impl IntoIterator<Item = impl AsRef<str>>,
) -> TemplateGenerationRequirement {
    let fields = declared_fields
        .into_iter()
        .map(|field| field.as_ref().to_string())
        .collect::<BTreeSet<_>>();
    let parsed = crate::parse_template(source);
    if !parsed.issues.is_empty() || !template_is_monotone(&parsed.tokens, &fields) {
        return TemplateGenerationRequirement::Unrepresentable;
    }

    if template_renders_nonempty(&parsed.tokens, &BTreeSet::new()) {
        return TemplateGenerationRequirement::Always;
    }

    let singleton_fields = fields
        .iter()
        .filter(|field| {
            template_renders_nonempty(&parsed.tokens, &BTreeSet::from([(*field).clone()]))
        })
        .cloned()
        .collect::<BTreeSet<_>>();
    if !singleton_fields.is_empty() {
        let outside_singletons = fields
            .difference(&singleton_fields)
            .cloned()
            .collect::<BTreeSet<_>>();
        return if template_renders_nonempty(&parsed.tokens, &outside_singletons) {
            TemplateGenerationRequirement::Unrepresentable
        } else if singleton_fields.len() == 1 {
            TemplateGenerationRequirement::All(singleton_fields.into_iter().collect())
        } else {
            TemplateGenerationRequirement::Any(singleton_fields.into_iter().collect())
        };
    }

    if !template_renders_nonempty(&parsed.tokens, &fields) {
        return TemplateGenerationRequirement::Unrepresentable;
    }
    let required = fields
        .iter()
        .filter(|field| {
            let values = fields
                .iter()
                .filter(|candidate| *candidate != *field)
                .cloned()
                .collect::<BTreeSet<_>>();
            !template_renders_nonempty(&parsed.tokens, &values)
        })
        .cloned()
        .collect::<BTreeSet<_>>();
    if required.is_empty() || !template_renders_nonempty(&parsed.tokens, &required) {
        TemplateGenerationRequirement::Unrepresentable
    } else {
        TemplateGenerationRequirement::All(required.into_iter().collect())
    }
}

fn template_is_monotone(tokens: &[crate::TemplateToken], fields: &BTreeSet<String>) -> bool {
    tokens.iter().all(|token| match token {
        crate::TemplateToken::SectionStart {
            field, inverted, ..
        } => !inverted && (fields.contains(field) || always_nonempty_special_field(field)),
        crate::TemplateToken::Render { field, .. } => {
            fields.contains(field) || always_nonempty_special_field(field)
        }
        _ => true,
    })
}

fn always_nonempty_special_field(field: &str) -> bool {
    always_nonempty_special_field_value(field).is_some()
}

fn always_nonempty_special_field_value(field: &str) -> Option<&'static str> {
    match field {
        "Card" => Some("Card 1"),
        "Deck" => Some("Deck"),
        "Type" => Some("Note Type"),
        _ => None,
    }
}

fn template_renders_nonempty(
    tokens: &[crate::TemplateToken],
    nonempty_fields: &BTreeSet<String>,
) -> bool {
    let mut rendered = String::new();
    let mut active_sections = Vec::new();
    for token in tokens {
        match token {
            crate::TemplateToken::SectionStart { field, .. } => active_sections
                .push(nonempty_fields.contains(field) || always_nonempty_special_field(field)),
            crate::TemplateToken::SectionEnd { .. } => {
                active_sections.pop();
            }
            crate::TemplateToken::Text(text) if active_sections.iter().all(|active| *active) => {
                rendered.push_str(text);
            }
            crate::TemplateToken::Render { field, .. }
                if active_sections.iter().all(|active| *active) =>
            {
                if nonempty_fields.contains(field) {
                    rendered.push_str("field-value");
                } else if let Some(value) = always_nonempty_special_field_value(field) {
                    rendered.push_str(value);
                }
            }
            crate::TemplateToken::Text(_)
            | crate::TemplateToken::Render { .. }
            | crate::TemplateToken::Comment => {}
        }
    }

    !crate::strip_html_preserving_media_filenames(&rendered)
        .trim()
        .is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_front_is_always_generated() {
        assert_eq!(
            infer_generation_requirement("<div>Always visible</div>", ["Context"]),
            TemplateGenerationRequirement::Always
        );
    }

    #[test]
    fn direct_fields_form_an_any_requirement() {
        assert_eq!(
            infer_generation_requirement("<div>{{Front}}</div>{{Back}}", ["Front", "Back"]),
            TemplateGenerationRequirement::Any(vec!["Back".into(), "Front".into()])
        );
    }

    #[test]
    fn positive_section_forms_an_all_requirement() {
        assert_eq!(
            infer_generation_requirement("{{#Prompt}}{{Extra}}{{/Prompt}}", ["Prompt", "Extra"],),
            TemplateGenerationRequirement::All(vec!["Extra".into(), "Prompt".into()])
        );
    }

    #[test]
    fn disjunction_of_conjunctions_is_unrepresentable() {
        assert_eq!(
            infer_generation_requirement(
                "{{#Prompt}}{{Extra}}{{/Prompt}}{{Context}}",
                ["Prompt", "Extra", "Context"],
            ),
            TemplateGenerationRequirement::Unrepresentable
        );
    }

    #[test]
    fn media_attribute_depends_on_its_field() {
        assert_eq!(
            infer_generation_requirement(r#"<img src="{{Image}}">"#, ["Image"]),
            TemplateGenerationRequirement::All(vec!["Image".into()])
        );
    }

    #[test]
    fn script_style_and_whitespace_entities_are_not_visible_static_content() {
        assert_eq!(
            infer_generation_requirement(
                "<script>ignored()</script><style>.x{}</style>&nbsp;{{Front}}",
                ["Front"],
            ),
            TemplateGenerationRequirement::All(vec!["Front".into()])
        );
    }

    #[test]
    fn subdeck_is_not_inferred_as_always_nonempty() {
        assert_eq!(
            infer_generation_requirement("{{Subdeck}}", ["Front"]),
            TemplateGenerationRequirement::Unrepresentable
        );
    }
}
