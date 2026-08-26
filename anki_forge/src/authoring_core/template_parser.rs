#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedTemplate {
    pub tokens: Vec<TemplateToken>,
    pub issues: Vec<TemplateParseIssue>,
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
        let expression = source[open + 2..close].trim();
        if expression.is_empty() {
            issues.push(syntax_issue("template expression cannot be empty", open));
            cursor = close + 2;
            continue;
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

    ParsedTemplate { tokens, issues }
}

pub fn is_special_template_field(field: &str) -> bool {
    matches!(
        field,
        "Card" | "CardFlag" | "Deck" | "FrontSide" | "Subdeck" | "Tags" | "Type"
    )
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
