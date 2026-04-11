use anki_forge::IoRect;
use anki_forge::product::{
    render_image_occlusion_cloze, ProductDocument, ProductNote, ProductNoteType,
    STOCK_BASIC_ID, STOCK_CLOZE_ID, STOCK_IMAGE_OCCLUSION_ID,
};

#[test]
fn product_document_registers_a_basic_notetype() {
    let document = ProductDocument::new("demo-doc").with_basic("basic-main");

    assert_eq!(document.document_id(), "demo-doc");
    assert_eq!(document.note_types().len(), 1);
    assert!(matches!(
        &document.note_types()[0],
        ProductNoteType::Basic(notetype) if notetype.id == "basic-main"
    ));
}

#[test]
fn stock_constants_match_default_notetype_ids() {
    assert_eq!(STOCK_BASIC_ID, "basic");
    assert_eq!(STOCK_CLOZE_ID, "cloze");
    assert_eq!(STOCK_IMAGE_OCCLUSION_ID, "image_occlusion");
}

#[test]
fn stock_builders_capture_tags_on_notes() {
    let document = ProductDocument::new("demo-doc")
        .with_basic(STOCK_BASIC_ID)
        .with_cloze(STOCK_CLOZE_ID)
        .with_image_occlusion(STOCK_IMAGE_OCCLUSION_ID)
        .add_basic_note_with_tags(
            STOCK_BASIC_ID,
            "basic-1",
            "Default",
            "front",
            "back",
            ["basic-tag"],
        )
        .add_cloze_note_with_tags(
            STOCK_CLOZE_ID,
            "cloze-1",
            "Default",
            "A {{c1::cloze}} note",
            "extra",
            ["cloze-tag"],
        )
        .add_image_occlusion_note_with_tags(
            STOCK_IMAGE_OCCLUSION_ID,
            "io-1",
            "Default",
            "occlusion",
            "<img src=\"heart.png\">",
            "Header",
            "back_extra",
            "comments",
            ["io-tag"],
        );

    assert!(matches!(
        &document.notes()[0],
        ProductNote::Basic(note) if note.tags == vec!["basic-tag"]
    ));
    assert!(matches!(
        &document.notes()[1],
        ProductNote::Cloze(note) if note.tags == vec!["cloze-tag"]
    ));
    assert!(matches!(
        &document.notes()[2],
        ProductNote::ImageOcclusion(note) if note.tags == vec!["io-tag"]
    ));
}

#[test]
fn legacy_stock_builders_remain_usable_without_tags() {
    let document = ProductDocument::new("demo-doc")
        .with_basic(STOCK_BASIC_ID)
        .with_cloze(STOCK_CLOZE_ID)
        .with_image_occlusion(STOCK_IMAGE_OCCLUSION_ID)
        .add_basic_note(STOCK_BASIC_ID, "basic-1", "Default", "front", "back")
        .add_cloze_note(
            STOCK_CLOZE_ID,
            "cloze-1",
            "Default",
            "A {{c1::cloze}} note",
            "extra",
        )
        .add_image_occlusion_note(
            STOCK_IMAGE_OCCLUSION_ID,
            "io-1",
            "Default",
            "occlusion",
            "<img src=\"heart.png\">",
            "Header",
            "back_extra",
            "comments",
        );

    assert!(matches!(
        &document.notes()[0],
        ProductNote::Basic(note) if note.tags.is_empty()
    ));
    assert!(matches!(
        &document.notes()[1],
        ProductNote::Cloze(note) if note.tags.is_empty()
    ));
    assert!(matches!(
        &document.notes()[2],
        ProductNote::ImageOcclusion(note) if note.tags.is_empty()
    ));
}

#[test]
fn stock_image_occlusion_helper_renders_rect_markup() {
    let rendered = render_image_occlusion_cloze(
        anki_forge::IoMode::HideAllGuessOne,
        &[IoRect {
            x: 10,
            y: 20,
            width: 80,
            height: 40,
        }],
    )
    .expect("render io cloze");

    assert_eq!(
        rendered,
        "{{c1::image-occlusion:rect:left=10:top=20:width=80:height=40}}<br>"
    );
}

#[test]
fn stock_image_occlusion_helper_renders_hide_one_guess_one_markup() {
    let rendered = render_image_occlusion_cloze(
        anki_forge::IoMode::HideOneGuessOne,
        &[IoRect {
            x: 1,
            y: 2,
            width: 3,
            height: 4,
        }],
    )
    .expect("render io cloze");

    assert_eq!(
        rendered,
        "{{c1,2::image-occlusion:rect:left=1:top=2:width=3:height=4}}<br>"
    );
}

#[test]
fn stock_image_occlusion_helper_rejects_empty_rects() {
    let err = render_image_occlusion_cloze(anki_forge::IoMode::HideAllGuessOne, &[])
        .expect_err("empty rects should fail");

    assert_eq!(err.to_string(), "image occlusion note requires at least one rect");
}
