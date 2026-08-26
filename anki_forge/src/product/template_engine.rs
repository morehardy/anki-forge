use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateIssueSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateIssue {
    pub code: &'static str,
    pub severity: TemplateIssueSeverity,
    pub message: String,
    pub byte_offset: usize,
}

pub struct TemplateEngine;

impl TemplateEngine {
    pub fn validate(
        source: &str,
        declared_fields: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Vec<TemplateIssue> {
        let fields = declared_fields
            .into_iter()
            .map(|field| field.as_ref().to_string())
            .collect::<BTreeSet<_>>();
        validate_template_source(source, &fields)
    }

    pub fn cloze_fields(source: &str) -> BTreeSet<String> {
        crate::authoring_core::parse_template(source)
            .tokens
            .into_iter()
            .filter_map(|token| match token {
                crate::authoring_core::TemplateToken::Render { field, filters, .. }
                    if !field.is_empty() && filters.iter().any(|filter| filter == "cloze") =>
                {
                    Some(field)
                }
                _ => None,
            })
            .collect()
    }
}

fn validate_template_source(source: &str, fields: &BTreeSet<String>) -> Vec<TemplateIssue> {
    let parsed = crate::authoring_core::parse_template(source);
    let mut issues = parsed
        .issues
        .into_iter()
        .map(|issue| TemplateIssue {
            code: match issue.kind {
                crate::authoring_core::TemplateParseIssueKind::Syntax => "TEMPLATE.SYNTAX_INVALID",
                crate::authoring_core::TemplateParseIssueKind::SectionMismatch => {
                    "TEMPLATE.SECTION_MISMATCH"
                }
            },
            severity: TemplateIssueSeverity::Error,
            message: issue.message,
            byte_offset: issue.byte_offset,
        })
        .collect::<Vec<_>>();
    for token in parsed.tokens {
        match token {
            crate::authoring_core::TemplateToken::SectionStart {
                field, byte_offset, ..
            } => {
                validate_field(&field, fields, byte_offset, &mut issues);
            }
            crate::authoring_core::TemplateToken::Render {
                field,
                filters,
                byte_offset,
            } => validate_render_expression(&field, &filters, fields, byte_offset, &mut issues),
            crate::authoring_core::TemplateToken::Text(_)
            | crate::authoring_core::TemplateToken::SectionEnd { .. }
            | crate::authoring_core::TemplateToken::Comment => {}
        }
    }
    issues.sort_by_key(|issue| issue.byte_offset);
    issues
}

fn validate_render_expression(
    field: &str,
    filters: &[String],
    fields: &BTreeSet<String>,
    offset: usize,
    issues: &mut Vec<TemplateIssue>,
) {
    for filter in filters {
        if !matches!(filter.as_str(), "cloze" | "hint" | "text" | "type") {
            issues.push(TemplateIssue {
                code: "TEMPLATE.FILTER_UNKNOWN",
                severity: TemplateIssueSeverity::Warning,
                message: format!("template uses unknown filter '{filter}'"),
                byte_offset: offset,
            });
        }
    }
    validate_field(field, fields, offset, issues);
}

fn validate_field(
    field: &str,
    fields: &BTreeSet<String>,
    offset: usize,
    issues: &mut Vec<TemplateIssue>,
) {
    if field.is_empty() {
        issues.push(syntax_issue("template field name cannot be empty", offset));
    } else if !fields.contains(field) && !crate::authoring_core::is_special_template_field(field) {
        issues.push(TemplateIssue {
            code: "TEMPLATE.RENDER_FIELD_UNKNOWN",
            severity: TemplateIssueSeverity::Error,
            message: format!("template references unknown field '{field}'"),
            byte_offset: offset,
        });
    }
}

fn syntax_issue(message: &str, byte_offset: usize) -> TemplateIssue {
    TemplateIssue {
        code: "TEMPLATE.SYNTAX_INVALID",
        severity: TemplateIssueSeverity::Error,
        message: message.to_string(),
        byte_offset,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_multi_word_field_names_as_complete_names() {
        assert!(TemplateEngine::validate("{{Back Extra}}", ["Back Extra"]).is_empty());

        let issues = TemplateEngine::validate("{{Back Extr}}", ["Back Extra"]);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].code, "TEMPLATE.RENDER_FIELD_UNKNOWN");
    }

    #[test]
    fn validates_fields_filters_and_balanced_sections() {
        let issues = TemplateEngine::validate(
            "{{#Expression}}{{text:Expression}}{{/Expression}}{{FrontSide}}",
            ["Expression"],
        );

        assert!(issues.is_empty());
    }

    #[test]
    fn reports_unknown_fields_and_mismatched_sections() {
        let issues =
            TemplateEngine::validate("{{#Expression}}{{Typo}}{{/Meaning}}", ["Expression"]);

        assert_eq!(
            issues.iter().map(|issue| issue.code).collect::<Vec<_>>(),
            vec!["TEMPLATE.RENDER_FIELD_UNKNOWN", "TEMPLATE.SECTION_MISMATCH"]
        );
    }

    #[test]
    fn rejects_empty_render_and_section_fields() {
        for source in ["{{text:}}", "{{#}}{{/}}"] {
            let issues = TemplateEngine::validate(source, ["Front"]);
            assert!(
                issues
                    .iter()
                    .any(|issue| issue.code == "TEMPLATE.SYNTAX_INVALID"),
                "{source}: {issues:?}"
            );
        }
    }
}
