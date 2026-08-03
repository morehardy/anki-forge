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
        let mut fields = BTreeSet::new();
        let mut remaining = source;
        while let Some(open) = remaining.find("{{") {
            let after_open = &remaining[open + 2..];
            let Some(close) = after_open.find("}}") else {
                break;
            };
            let expression = after_open[..close].trim();
            let segments = expression.split(':').map(str::trim).collect::<Vec<_>>();
            if segments.len() >= 2 && segments[..segments.len() - 1].contains(&"cloze") {
                if let Some(field) = segments.last().filter(|field| !field.is_empty()) {
                    fields.insert((*field).to_string());
                }
            }
            remaining = &after_open[close + 2..];
        }
        fields
    }
}

fn validate_template_source(source: &str, fields: &BTreeSet<String>) -> Vec<TemplateIssue> {
    let mut issues = Vec::new();
    let mut sections = Vec::<(String, usize)>::new();
    let mut cursor = 0;

    while cursor < source.len() {
        let next_open = source[cursor..].find("{{").map(|offset| cursor + offset);
        let next_close = source[cursor..].find("}}").map(|offset| cursor + offset);
        if next_close.is_some_and(|close| next_open.is_none_or(|open| close < open)) {
            let offset = next_close.expect("checked as some");
            issues.push(syntax_issue("unexpected closing delimiter", offset));
            cursor = offset + 2;
            continue;
        }
        let Some(open) = next_open else {
            break;
        };
        let Some(relative_close) = source[open + 2..].find("}}") else {
            issues.push(syntax_issue("unclosed template expression", open));
            break;
        };
        let close = open + 2 + relative_close;
        let expression = source[open + 2..close].trim();
        if expression.is_empty() {
            issues.push(syntax_issue("template expression cannot be empty", open));
            cursor = close + 2;
            continue;
        }

        if let Some(field) = expression
            .strip_prefix('#')
            .or_else(|| expression.strip_prefix('^'))
        {
            let field = field.trim();
            validate_field(field, fields, open, &mut issues);
            sections.push((field.to_string(), open));
        } else if let Some(field) = expression.strip_prefix('/') {
            let field = field.trim();
            match sections.pop() {
                Some((expected, _)) if expected == field => {}
                Some((expected, _)) => issues.push(TemplateIssue {
                    code: "TEMPLATE.SECTION_MISMATCH",
                    severity: TemplateIssueSeverity::Error,
                    message: format!(
                        "template section closes '{field}' but the open section is '{expected}'"
                    ),
                    byte_offset: open,
                }),
                None => issues.push(TemplateIssue {
                    code: "TEMPLATE.SECTION_MISMATCH",
                    severity: TemplateIssueSeverity::Error,
                    message: format!("template closes unopened section '{field}'"),
                    byte_offset: open,
                }),
            }
        } else if !expression.starts_with('!') {
            validate_render_expression(expression, fields, open, &mut issues);
        }

        cursor = close + 2;
    }

    for (field, offset) in sections {
        issues.push(TemplateIssue {
            code: "TEMPLATE.SECTION_MISMATCH",
            severity: TemplateIssueSeverity::Error,
            message: format!("template section '{field}' is not closed"),
            byte_offset: offset,
        });
    }
    issues
}

fn validate_render_expression(
    expression: &str,
    fields: &BTreeSet<String>,
    offset: usize,
    issues: &mut Vec<TemplateIssue>,
) {
    let segments = expression.split(':').map(str::trim).collect::<Vec<_>>();
    let Some(field) = segments.last().copied() else {
        return;
    };

    for filter in segments.iter().take(segments.len().saturating_sub(1)) {
        if !matches!(*filter, "cloze" | "hint" | "text" | "type") {
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
    const SPECIAL_FIELDS: &[&str] = &[
        "Card",
        "CardFlag",
        "Deck",
        "FrontSide",
        "Subdeck",
        "Tags",
        "Type",
    ];
    if field.is_empty() {
        issues.push(syntax_issue("template field name cannot be empty", offset));
    } else if !fields.contains(field) && !SPECIAL_FIELDS.contains(&field) {
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
