use serde::{Deserialize, Serialize};

use super::diagnostics::ProductDiagnostic;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HelperDeclaration {
    AnswerDivider { title: String },
    BackExtraPanel { title: Option<String> },
}

pub fn apply_helpers(
    note_kind: &str,
    question_format: &str,
    answer_format: &str,
    helpers: &[HelperDeclaration],
) -> Result<(String, String), ProductDiagnostic> {
    let next_question = question_format.to_string();
    let mut next_answer = answer_format.to_string();

    for helper in helpers {
        match helper {
            HelperDeclaration::AnswerDivider { title } => {
                if note_kind != "basic" {
                    return Err(ProductDiagnostic {
                        code: "PHASE5A.HELPER_SCOPE_INVALID",
                        message: format!(
                            "AnswerDivider is only valid for basic note types, got {note_kind}"
                        ),
                    });
                }

                next_answer = inject_answer_divider(&next_answer, title);
            }
            HelperDeclaration::BackExtraPanel { title } => {
                if !matches!(note_kind, "cloze" | "image_occlusion") {
                    return Err(ProductDiagnostic {
                        code: "PHASE5A.HELPER_SCOPE_INVALID",
                        message: format!(
                            "BackExtraPanel is only valid for cloze and image_occlusion note types, got {note_kind}"
                        ),
                    });
                }

                let panel_title = title.clone().unwrap_or_else(|| "More".into());
                let panel_markup = format!(
                    "{{{{#Back Extra}}}}<div class=\"af-back-extra-panel\"><h3>{panel_title}</h3>{{{{Back Extra}}}}</div>{{{{/Back Extra}}}}"
                );

                if let Some(updated) = replace_once(
                    &next_answer,
                    "{{#Back Extra}}<div>{{Back Extra}}</div>{{/Back Extra}}",
                    &panel_markup,
                ) {
                    next_answer = updated;
                } else if let Some(updated) =
                    replace_once(&next_answer, "{{Back Extra}}", &panel_markup)
                {
                    next_answer = updated;
                } else {
                    next_answer.push('\n');
                    next_answer.push_str(&panel_markup);
                }
            }
        }
    }

    Ok((next_question, next_answer))
}

fn inject_answer_divider(answer: &str, title: &str) -> String {
    replace_once(
        answer,
        "<hr id=answer>",
        &format!("<hr id=answer><div class=\"af-answer-divider\">{title}</div>"),
    )
    .unwrap_or_else(|| answer.to_string())
}

fn replace_once(input: &str, target: &str, replacement: &str) -> Option<String> {
    let index = input.find(target)?;
    let mut updated = String::with_capacity(input.len() - target.len() + replacement.len());
    updated.push_str(&input[..index]);
    updated.push_str(replacement);
    updated.push_str(&input[index + target.len()..]);
    Some(updated)
}
