use anki_forge::product::{ProductDocument, ProductNoteType};

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
