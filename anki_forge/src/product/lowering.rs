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
    let mut product_diagnostics: Vec<ProductDiagnostic> = Vec::new();

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
            ProductNoteType::Cloze(cloze) => {
                notetypes.push(AuthoringNotetype {
                    id: cloze.id.clone(),
                    kind: "cloze".into(),
                    name: cloze.name.clone(),
                });
                mappings.push(LoweringMapping {
                    kind: "notetype",
                    product_id: cloze.id.clone(),
                    authoring_id: cloze.id.clone(),
                });
            }
            ProductNoteType::ImageOcclusion(io) => {
                notetypes.push(AuthoringNotetype {
                    id: io.id.clone(),
                    kind: "cloze".into(),
                    name: io.name.clone(),
                });
                mappings.push(LoweringMapping {
                    kind: "notetype",
                    product_id: io.id.clone(),
                    authoring_id: io.id.clone(),
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
            ProductNote::Cloze(cloze) => {
                let mut fields: BTreeMap<String, String> = BTreeMap::new();
                fields.insert("Text".into(), cloze.text.clone());
                fields.insert("Back Extra".into(), cloze.back_extra.clone());

                notes.push(AuthoringNote {
                    id: cloze.id.clone(),
                    notetype_id: cloze.note_type_id.clone(),
                    deck_name: cloze.deck_name.clone(),
                    fields,
                    tags: Vec::new(),
                });

                mappings.push(LoweringMapping {
                    kind: "note",
                    product_id: cloze.id.clone(),
                    authoring_id: cloze.id.clone(),
                });
            }
            ProductNote::ImageOcclusion(io) => {
                if io.image.trim().is_empty() {
                    product_diagnostics.push(ProductDiagnostic::io_image_required(&io.id));
                    continue;
                }

                let mut fields: BTreeMap<String, String> = BTreeMap::new();
                fields.insert("Occlusion".into(), io.occlusion.clone());
                fields.insert("Image".into(), io.image.clone());
                fields.insert("Header".into(), io.header.clone());
                fields.insert("Back Extra".into(), io.back_extra.clone());
                fields.insert("Comments".into(), io.comments.clone());

                notes.push(AuthoringNote {
                    id: io.id.clone(),
                    notetype_id: io.note_type_id.clone(),
                    deck_name: io.deck_name.clone(),
                    fields,
                    tags: Vec::new(),
                });

                mappings.push(LoweringMapping {
                    kind: "note",
                    product_id: io.id.clone(),
                    authoring_id: io.id.clone(),
                });
            }
        }
    }

    if !product_diagnostics.is_empty() {
        return Err(ProductLoweringError {
            product_diagnostics,
            lowering_diagnostics: Vec::new(),
        });
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
