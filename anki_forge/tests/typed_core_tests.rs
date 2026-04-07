use anki_forge::{
    normalize, writer_tool_contract_version, AuthoringDocument, NormalizationRequest,
};

#[test]
fn typed_facade_reexports_phase2_and_phase3_core_surfaces() {
    let result = normalize(NormalizationRequest::new(AuthoringDocument {
        kind: "authoring-ir".into(),
        schema_version: "0.1.0".into(),
        metadata_document_id: "demo-doc".into(),
        notetypes: vec![],
        notes: vec![],
        media: vec![],
    }));

    assert_eq!(result.tool_contract_version, "phase2-v1");
    assert_eq!(writer_tool_contract_version(), "phase3-v1");
}
