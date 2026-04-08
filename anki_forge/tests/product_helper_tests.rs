use anki_forge::product::{HelperDeclaration, ProductDocument};

#[test]
fn answer_divider_helper_injects_a_named_divider_into_basic_answer_template() {
    let lowering = ProductDocument::new("demo-doc")
        .with_basic("basic-main")
        .with_helper(
            "basic-main",
            HelperDeclaration::AnswerDivider {
                title: "Answer".into(),
            },
        )
        .add_basic_note("basic-main", "note-1", "Default", "front", "back")
        .lower()
        .expect("lower helper-enhanced document");

    let template = &lowering.authoring_document.notetypes[0].templates.as_ref().unwrap()[0];
    assert!(template.answer_format.contains("Answer"));
}

#[test]
fn back_extra_panel_helper_rejects_basic_note_types() {
    let error = ProductDocument::new("demo-doc")
        .with_basic("basic-main")
        .with_helper(
            "basic-main",
            HelperDeclaration::BackExtraPanel {
                title: Some("More".into()),
            },
        )
        .lower()
        .expect_err("expected invalid helper scope");

    assert!(error
        .product_diagnostics
        .iter()
        .any(|item| item.code == "PHASE5A.HELPER_SCOPE_INVALID"));
}
