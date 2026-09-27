#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedTemplate {
    pub tokens: Vec<TemplateToken>,
    pub issues: Vec<TemplateParseIssue>,
    /// Field references in source order, including both ends of sections.
    pub references: Vec<TemplateFieldReference>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateFieldReference {
    /// The field only, excluding filters, delimiters and surrounding whitespace.
    pub byte_range: std::ops::Range<usize>,
    pub expression_range: std::ops::Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateToken {
    Text(String),
    SectionStart {
        field: String,
        inverted: bool,
        byte_offset: usize,
    },
    SectionEnd {
        field: String,
        byte_offset: usize,
    },
    Render {
        field: String,
        filters: Vec<String>,
        byte_offset: usize,
    },
    Comment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateParseIssueKind {
    Syntax,
    SectionMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateParseIssue {
    pub kind: TemplateParseIssueKind,
    pub message: String,
    pub byte_offset: usize,
}

pub fn parse_template(source: &str) -> ParsedTemplate {
    let mut tokens = Vec::new();
    let mut issues = Vec::new();
    let mut references = Vec::new();
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
            if cursor < source.len() {
                tokens.push(TemplateToken::Text(source[cursor..].to_string()));
            }
            break;
        };
        if open > cursor {
            tokens.push(TemplateToken::Text(source[cursor..open].to_string()));
        }
        let Some(relative_close) = source[open + 2..].find("}}") else {
            issues.push(syntax_issue("unclosed template expression", open));
            break;
        };
        let close = open + 2 + relative_close;
        let expression_range = trim_range(source, open + 2..close);
        let expression = &source[expression_range.clone()];
        if expression.is_empty() {
            issues.push(syntax_issue("template expression cannot be empty", open));
            cursor = close + 2;
            continue;
        }

        if !expression.starts_with('!') {
            let field_start = if expression.starts_with(['#', '^', '/']) {
                expression_range.start + 1
            } else {
                expression
                    .rfind(':')
                    .map_or(expression_range.start, |colon| {
                        expression_range.start + colon + 1
                    })
            };
            references.push(TemplateFieldReference {
                byte_range: trim_range(source, field_start..expression_range.end),
                expression_range: open..close + 2,
            });
        }

        if let Some(field) = expression.strip_prefix('#') {
            let field = field.trim().to_string();
            sections.push((field.clone(), open));
            tokens.push(TemplateToken::SectionStart {
                field,
                inverted: false,
                byte_offset: open,
            });
        } else if let Some(field) = expression.strip_prefix('^') {
            let field = field.trim().to_string();
            sections.push((field.clone(), open));
            tokens.push(TemplateToken::SectionStart {
                field,
                inverted: true,
                byte_offset: open,
            });
        } else if let Some(field) = expression.strip_prefix('/') {
            let field = field.trim().to_string();
            match sections.pop() {
                Some((expected, _)) if expected == field => {}
                Some((expected, _)) => issues.push(TemplateParseIssue {
                    kind: TemplateParseIssueKind::SectionMismatch,
                    message: format!(
                        "template section closes '{field}' but the open section is '{expected}'"
                    ),
                    byte_offset: open,
                }),
                None => issues.push(TemplateParseIssue {
                    kind: TemplateParseIssueKind::SectionMismatch,
                    message: format!("template closes unopened section '{field}'"),
                    byte_offset: open,
                }),
            }
            tokens.push(TemplateToken::SectionEnd {
                field,
                byte_offset: open,
            });
        } else if expression.starts_with('!') {
            tokens.push(TemplateToken::Comment);
        } else {
            let segments = expression.split(':').map(str::trim).collect::<Vec<_>>();
            if segments.iter().any(|segment| segment.is_empty()) {
                issues.push(syntax_issue(
                    "template filter and field segments must be nonempty",
                    open,
                ));
            }
            let field = segments.last().copied().unwrap_or_default().to_string();
            let filters = segments[..segments.len().saturating_sub(1)]
                .iter()
                .map(|filter| (*filter).to_string())
                .collect();
            tokens.push(TemplateToken::Render {
                field,
                filters,
                byte_offset: open,
            });
        }

        cursor = close + 2;
    }

    for (field, offset) in sections {
        issues.push(TemplateParseIssue {
            kind: TemplateParseIssueKind::SectionMismatch,
            message: format!("template section '{field}' is not closed"),
            byte_offset: offset,
        });
    }

    ParsedTemplate {
        tokens,
        issues,
        references,
    }
}

fn trim_range(source: &str, range: std::ops::Range<usize>) -> std::ops::Range<usize> {
    let raw = &source[range.clone()];
    let trimmed_start = raw.trim_start();
    let start = range.start + raw.len() - trimmed_start.len();
    start..start + trimmed_start.trim_end().len()
}

pub fn is_special_template_field(field: &str) -> bool {
    matches!(
        field,
        "Card" | "CardFlag" | "Deck" | "FrontSide" | "Subdeck" | "Tags" | "Type"
    )
}

/// Rewrites only parsed field-reference byte ranges. Unknown bindings (including
/// Anki system expressions) retain their exact original text.
pub(crate) fn rewrite_fields(
    source: &str,
    parsed: &ParsedTemplate,
    bindings: &std::collections::BTreeMap<&str, &str>,
) -> String {
    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    for reference in &parsed.references {
        let field = &source[reference.byte_range.clone()];
        let Some(name) = bindings.get(field) else {
            continue;
        };
        output.push_str(&source[cursor..reference.byte_range.start]);
        output.push_str(name);
        cursor = reference.byte_range.end;
    }
    output.push_str(&source[cursor..]);
    output
}

fn syntax_issue(message: &str, byte_offset: usize) -> TemplateParseIssue {
    TemplateParseIssue {
        kind: TemplateParseIssueKind::Syntax,
        message: message.to_string(),
        byte_offset,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_filters_and_balanced_sections() {
        let parsed = parse_template("{{#Front}}{{text:Front}}{{/Front}}");
        assert!(parsed.issues.is_empty());
        assert!(matches!(
            &parsed.tokens[1],
            TemplateToken::Render { field, filters, .. }
                if field == "Front" && filters == &["text"]
        ));
    }

    #[test]
    fn reports_mismatched_section_names() {
        let parsed = parse_template("{{#Front}}{{/Back}}");
        assert_eq!(parsed.issues.len(), 1);
        assert_eq!(
            parsed.issues[0].kind,
            TemplateParseIssueKind::SectionMismatch
        );
    }
}
