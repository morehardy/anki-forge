#![cfg(feature = "internal-tools")]

use ankiforge::tools;
use ankiforge::tools::{normalize, AuthoringDocument, NormalizationRequest};

#[test]
fn repository_tools_can_check_both_contract_protocols() {
    let result = normalize(NormalizationRequest::new(AuthoringDocument {
        kind: "authoring-ir".into(),
        schema_version: "0.1.0".into(),
        metadata_document_id: "demo-doc".into(),
        notetypes: vec![],
        notes: vec![],
        media: vec![],
    }));

    assert_eq!(result.tool_contract_version, "phase2-v1");
    assert_eq!(tools::writer_contract_version(), "phase3-v1");
}
