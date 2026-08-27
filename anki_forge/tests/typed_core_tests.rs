#![cfg(feature = "internal-tools")]

use anki_forge::authoring::{normalize, AuthoringDocument, NormalizationRequest};
use anki_forge::writer_tool_contract_version;

#[test]
fn typed_facade_exposes_core_surfaces_through_namespaced_modules() {
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
