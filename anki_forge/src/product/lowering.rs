use std::collections::BTreeMap;

use crate::{AuthoringDocument, AuthoringNote, AuthoringNotetype};

use super::{
    diagnostics::{LoweringDiagnostic, ProductDiagnostic, ProductLoweringError},
    model::{ProductNote, ProductNoteType},
    ProductDocument,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweringMapping {
    pub kind: &'static str,
    pub product_id: String,
    pub authoring_id: String,
}

#[derive(Debug, Clone)]
pub struct LoweringPlan {
    pub authoring_document: AuthoringDocument,
    pub mappings: Vec<LoweringMapping>,
    pub product_diagnostics: Vec<ProductDiagnostic>,
    pub lowering_diagnostics: Vec<LoweringDiagnostic>,
}

pub fn lower_document(document: &ProductDocument) -> Result<LoweringPlan, ProductLoweringError> {
    let mut notetypes: Vec<AuthoringNotetype> = Vec::new();
    let mut notes: Vec<AuthoringNote> = Vec::new();
    let mut mappings: Vec<LoweringMapping> = Vec::new();

    for notetype in document.note_types() {
        match notetype {
            ProductNoteType::Basic(basic) => {
                notetypes.push(AuthoringNotetype {
                    id: basic.id.clone(),
                    kind: "normal".into(),
                    name: basic.name.clone(),
                });
                mappings.push(LoweringMapping {
                    kind: "notetype",
                    product_id: basic.id.clone(),
                    authoring_id: basic.id.clone(),
                });
            }
        }
    }

    for note in document.notes() {
        match note {
            ProductNote::Basic(basic) => {
                let mut fields: BTreeMap<String, String> = BTreeMap::new();
                fields.insert("Front".into(), basic.front.clone());
                fields.insert("Back".into(), basic.back.clone());

                notes.push(AuthoringNote {
                    id: basic.id.clone(),
                    notetype_id: basic.note_type_id.clone(),
                    deck_name: basic.deck_name.clone(),
                    fields,
                    tags: Vec::new(),
                });

                mappings.push(LoweringMapping {
                    kind: "note",
                    product_id: basic.id.clone(),
                    authoring_id: basic.id.clone(),
                });
            }
        }
    }

    Ok(LoweringPlan {
        authoring_document: AuthoringDocument {
            kind: "authoring-ir".into(),
            schema_version: "0.1.0".into(),
            metadata_document_id: document.document_id().to_string(),
            notetypes,
            notes,
            media: Vec::new(),
        },
        mappings,
        product_diagnostics: Vec::new(),
        lowering_diagnostics: Vec::new(),
    })
}

