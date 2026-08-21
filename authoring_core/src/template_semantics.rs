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
    let declared_fields = declared_fields
        .into_iter()
        .map(|field| field.as_ref().to_string())
        .collect::<BTreeSet<_>>();
    let mut sections = Vec::<SectionCondition>::new();
    let mut terms = Vec::<BTreeSet<String>>::new();
    let mut has_unrepresentable_term = false;
    let mut cursor = 0;

    while let Some(relative_open) = source[cursor..].find("{{") {
        let open = cursor + relative_open;
        if static_fragment_is_visible(&source[cursor..open]) {
            record_term(&sections, None, &mut terms, &mut has_unrepresentable_term);
        }
        let Some(relative_close) = source[open + 2..].find("}}") else {
            return TemplateGenerationRequirement::Unrepresentable;
        };
        let close = open + 2 + relative_close;
        let expression = source[open + 2..close].trim();
        if expression.starts_with('!') {
            cursor = close + 2;
            continue;
        }
        if let Some(field) = expression.strip_prefix('#') {
            let Some(condition) = section_condition(field.trim(), true, &declared_fields) else {
                return TemplateGenerationRequirement::Unrepresentable;
            };
            sections.push(condition);
            cursor = close + 2;
            continue;
        }
        if let Some(field) = expression.strip_prefix('^') {
            let Some(condition) = section_condition(field.trim(), false, &declared_fields) else {
                return TemplateGenerationRequirement::Unrepresentable;
            };
            sections.push(condition);
            cursor = close + 2;
            continue;
        }
        if expression.starts_with('/') {
            if sections.pop().is_none() {
                return TemplateGenerationRequirement::Unrepresentable;
            }
            cursor = close + 2;
            continue;
        }

        let field = expression.rsplit(':').next().unwrap_or_default().trim();
        if declared_fields.contains(field) {
            record_term(
                &sections,
                Some(field),
                &mut terms,
                &mut has_unrepresentable_term,
            );
        } else if matches!(field, "Card" | "Deck" | "Subdeck" | "Type") {
            record_term(&sections, None, &mut terms, &mut has_unrepresentable_term);
        } else {
            return TemplateGenerationRequirement::Unrepresentable;
        }
        cursor = close + 2;
    }

    if static_fragment_is_visible(&source[cursor..]) {
        record_term(&sections, None, &mut terms, &mut has_unrepresentable_term);
    }
    if !sections.is_empty() {
        return TemplateGenerationRequirement::Unrepresentable;
    }

    terms.sort();
    terms.dedup();
    if terms.iter().any(BTreeSet::is_empty) {
        return TemplateGenerationRequirement::Always;
    }
    if has_unrepresentable_term {
        return TemplateGenerationRequirement::Unrepresentable;
    }
    let all_terms = terms.clone();
    terms.retain(|candidate| {
        !all_terms
            .iter()
            .any(|other| other.len() < candidate.len() && other.is_subset(candidate))
    });
    match terms.as_slice() {
        [] => TemplateGenerationRequirement::Unrepresentable,
        [only] => TemplateGenerationRequirement::All(only.iter().cloned().collect()),
        many if many.iter().all(|term| term.len() == 1) => TemplateGenerationRequirement::Any(
            many.iter()
                .flat_map(|term| term.iter().cloned())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
        ),
        _ => TemplateGenerationRequirement::Unrepresentable,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SectionCondition {
    Field { name: String, positive: bool },
    Always(bool),
}

fn section_condition(
    field: &str,
    positive: bool,
    declared_fields: &BTreeSet<String>,
) -> Option<SectionCondition> {
    if declared_fields.contains(field) {
        Some(SectionCondition::Field {
            name: field.to_string(),
            positive,
        })
    } else if matches!(field, "Card" | "Deck" | "Subdeck" | "Type") {
        Some(SectionCondition::Always(positive))
    } else {
        None
    }
}

fn record_term(
    sections: &[SectionCondition],
    rendered_field: Option<&str>,
    terms: &mut Vec<BTreeSet<String>>,
    has_unrepresentable_term: &mut bool,
) {
    let mut fields = BTreeSet::new();
    let mut inverted_fields = BTreeSet::new();
    for condition in sections {
        match condition {
            SectionCondition::Always(true) => {}
            SectionCondition::Always(false) => return,
            SectionCondition::Field {
                name,
                positive: true,
            } => {
                fields.insert(name.clone());
            }
            SectionCondition::Field {
                name,
                positive: false,
            } => {
                inverted_fields.insert(name.clone());
            }
        }
    }
    // A branch guarded by both {{#Field}} and {{^Field}} is unreachable and
    // therefore contributes no generation condition.
    if !fields.is_disjoint(&inverted_fields) {
        return;
    }
    if !inverted_fields.is_empty() {
        *has_unrepresentable_term = true;
        return;
    }
    if let Some(field) = rendered_field {
        fields.insert(field.to_string());
    }
    terms.push(fields);
}

fn static_fragment_is_visible(fragment: &str) -> bool {
    let lower = fragment.to_ascii_lowercase();
    if lower.contains("<img") || lower.contains("<audio") || lower.contains("<video") {
        return true;
    }

    let mut in_tag = false;
    fragment.chars().any(|ch| match ch {
        '<' => {
            in_tag = true;
            false
        }
        '>' => {
            in_tag = false;
            false
        }
        _ => !in_tag && !ch.is_whitespace(),
    })
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
    fn unreachable_inverted_branch_does_not_poison_other_terms() {
        assert_eq!(
            infer_generation_requirement(
                "{{#Prompt}}{{^Prompt}}never{{/Prompt}}{{/Prompt}}{{Context}}",
                ["Prompt", "Context"],
            ),
            TemplateGenerationRequirement::All(vec!["Context".into()])
        );
    }
}
