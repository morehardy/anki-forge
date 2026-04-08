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

                next_answer = next_answer.replace(
                    "<hr id=answer>",
                    &format!("<hr id=answer><div class=\"af-answer-divider\">{title}</div>"),
                );
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

                if next_answer.contains("{{#Back Extra}}<div>{{Back Extra}}</div>{{/Back Extra}}") {
                    next_answer = next_answer.replace(
                        "{{#Back Extra}}<div>{{Back Extra}}</div>{{/Back Extra}}",
                        &panel_markup,
                    );
                } else if next_answer.contains("{{Back Extra}}") {
                    next_answer = next_answer.replace("{{Back Extra}}", &panel_markup);
                } else {
                    next_answer.push_str(&format!("\n{panel_markup}"));
                }
            }
        }
    }

    Ok((next_question, next_answer))
}
