use std::collections::BTreeSet;

use super::{GenerationRule, NoteTypeBuilder, SchemaError, SchemaErrorKind as Kind, TemplateSide};

pub(super) fn validate(model: &NoteTypeBuilder) -> Result<(), SchemaError> {
    symbol(&model.key, "model key", false)?;
    name(&model.name, "model name", false)?;
    if model.fields.is_empty() || model.templates.is_empty() {
        return Err(SchemaError::new(
            Kind::InvalidStructure,
            "SCHEMA.DECLARATIONS_MISSING",
            "a model needs at least one field and one template",
        ));
    }
    let mut fields = BTreeSet::new();
    let mut field_names = BTreeSet::new();
    let mut sort_count = 0;
    for field in &model.fields {
        symbol(field.key.as_str(), "field key", true)?;
        name(&field.name, "field name", true)?;
        if !fields.insert(field.key.as_str()) || !field_names.insert(field.name.as_str()) {
            return Err(SchemaError::new(
                Kind::Duplicate,
                "SCHEMA.FIELD_DUPLICATE",
                "field keys and names must each be unique",
            ));
        }
        sort_count += usize::from(field.sort);
    }
    if sort_count > 1 {
        return Err(SchemaError::new(
            Kind::InvalidStructure,
            "SCHEMA.SORT_FIELD_DUPLICATE",
            "select at most one sort field",
        ));
    }
    for field in &model.fields {
        if fields.contains(field.name.as_str()) && field.name != field.key.as_str() {
            return Err(SchemaError::new(
                Kind::Duplicate,
                "SCHEMA.FIELD_SYMBOL_CONFLICT",
                "a field name cannot equal another field's stable key",
            ));
        }
    }
    if let Some(cloze) = &model.cloze_field {
        if !fields.contains(cloze.as_str()) || model.templates.len() != 1 {
            return Err(SchemaError::new(
                Kind::InvalidCloze,
                "SCHEMA.CLOZE_DECLARATION_INVALID",
                "a cloze model needs one template and a declared cloze field",
            ));
        }
    }
    let mut template_keys = BTreeSet::new();
    let mut template_names = BTreeSet::new();
    for template in &model.templates {
        symbol(template.key.as_str(), "template key", false)?;
        name(&template.name, "template name", false)?;
        if !template_keys.insert(template.key.as_str())
            || !template_names.insert(template.name.as_str())
        {
            return Err(SchemaError::new(
                Kind::Duplicate,
                "SCHEMA.TEMPLATE_DUPLICATE",
                "template keys and names must each be unique",
            ));
        }
        match &template.generation {
            GenerationRule::AnkiDefault => {}
            GenerationRule::All(keys) | GenerationRule::Any(keys) => {
                if keys.is_empty()
                    || keys.iter().any(|key| !fields.contains(key.as_str()))
                    || keys.iter().collect::<BTreeSet<_>>().len() != keys.len()
                    || model.cloze_field.is_some()
                {
                    return Err(SchemaError::new(
                        Kind::InvalidGeneration,
                        "SCHEMA.GENERATION_INVALID",
                        "generation rules need distinct declared fields and a normal model",
                    ));
                }
            }
        }
        for (side, source) in [
            (TemplateSide::Front, Some(template.front.as_str())),
            (TemplateSide::Back, Some(template.back.as_str())),
            (
                TemplateSide::BrowserFront,
                template.browser_front.as_deref(),
            ),
            (TemplateSide::BrowserBack, template.browser_back.as_deref()),
        ] {
            let Some(source) = source else { continue };
            let parsed = crate::authoring_core::parse_template(source);
            if let Some(issue) = parsed.issues.first() {
                let code = match issue.kind {
                    crate::authoring_core::TemplateParseIssueKind::Syntax => {
                        "TEMPLATE.SYNTAX_INVALID"
                    }
                    crate::authoring_core::TemplateParseIssueKind::SectionMismatch => {
                        "TEMPLATE.SECTION_MISMATCH"
                    }
                };
                let end = parsed
                    .references
                    .iter()
                    .find(|reference| reference.expression_range.start == issue.byte_offset)
                    .map_or(source.len(), |reference| reference.expression_range.end);
                return Err(
                    SchemaError::new(Kind::InvalidTemplate, code, &issue.message).at(
                        template.key.clone(),
                        side,
                        issue.byte_offset..end,
                    ),
                );
            }
            for reference in &parsed.references {
                let field = &source[reference.byte_range.clone()];
                if field.is_empty()
                    || (!fields.contains(field)
                        && !crate::authoring_core::is_special_template_field(field))
                {
                    let code = if field.is_empty() {
                        "TEMPLATE.SYNTAX_INVALID"
                    } else {
                        "TEMPLATE.RENDER_FIELD_UNKNOWN"
                    };
                    return Err(SchemaError::new(
                        Kind::InvalidTemplate,
                        code,
                        format!("template references unknown field key {field:?}"),
                    )
                    .at(
                        template.key.clone(),
                        side,
                        reference.byte_range.clone(),
                    ));
                }
            }
            let cloze_refs = parsed
                .tokens
                .iter()
                .filter_map(|token| match token {
                    crate::authoring_core::TemplateToken::Render { field, filters, .. }
                        if filters.iter().any(|filter| filter == "cloze") =>
                    {
                        Some(field.as_str())
                    }
                    _ => None,
                })
                .collect::<BTreeSet<_>>();
            match &model.cloze_field {
                Some(field)
                    if (side == TemplateSide::Front && !cloze_refs.contains(field.as_str()))
                        || cloze_refs.iter().any(|key| *key != field.as_str()) =>
                {
                    return Err(SchemaError::new(Kind::InvalidCloze, "SCHEMA.CLOZE_FILTER_MISMATCH", "cloze filters must render the declared cloze field; the front must contain one")
                        .at(template.key.clone(), side, 0..source.len()));
                }
                None if !cloze_refs.is_empty() => {
                    return Err(SchemaError::new(
                        Kind::InvalidCloze,
                        "SCHEMA.CLOZE_FIELD_REQUIRED",
                        "declare a cloze field before using cloze filters",
                    )
                    .at(template.key.clone(), side, 0..source.len()));
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn symbol(value: &str, label: &str, field: bool) -> Result<(), SchemaError> {
    if value.trim().is_empty()
        || value.trim() != value
        || value.chars().any(char::is_control)
        || (field
            && (value.chars().any(|c| "{}:#^/!".contains(c))
                || crate::authoring_core::is_special_template_field(value)))
    {
        Err(SchemaError::new(
            Kind::InvalidKey,
            "SCHEMA.KEY_INVALID",
            format!("invalid {label}: {value:?}"),
        ))
    } else {
        Ok(())
    }
}

fn name(value: &str, label: &str, field: bool) -> Result<(), SchemaError> {
    if value.trim().is_empty()
        || value.chars().any(char::is_control)
        || (field
            && (value.trim() != value
                || value.chars().any(|c| "{}:#^/!".contains(c))
                || crate::authoring_core::is_special_template_field(value)))
    {
        Err(SchemaError::new(
            Kind::InvalidName,
            "SCHEMA.NAME_INVALID",
            format!("invalid {label}: {value:?}"),
        ))
    } else {
        Ok(())
    }
}
