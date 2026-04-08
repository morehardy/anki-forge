use std::collections::BTreeMap;

use authoring_core::stock::{stock_lowering_defaults, StockLoweringDefaults};

use crate::{
    AuthoringDocument, AuthoringField, AuthoringNote, AuthoringNotetype, AuthoringTemplate,
};

use super::{
    diagnostics::{LoweringDiagnostic, ProductDiagnostic, ProductLoweringError},
    helpers::{apply_helpers, HelperDeclaration},
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
                let helpers = document.helpers_for(&basic.id);
                match lower_stock_notetype(
                    &basic.id,
                    basic.name.clone(),
                    "basic",
                    stock_lowering_defaults("basic")
                        .expect("source-grounded basic lowering defaults"),
                    &helpers,
                ) {
                    Ok(notetype) => notetypes.push(notetype),
                    Err(diagnostic) => {
                        product_diagnostics.push(diagnostic);
                        continue;
                    }
                }
                mappings.push(LoweringMapping {
                    kind: "notetype",
                    product_id: basic.id.clone(),
                    authoring_id: basic.id.clone(),
                });
            }
            ProductNoteType::Cloze(cloze) => {
                let helpers = document.helpers_for(&cloze.id);
                match lower_stock_notetype(
                    &cloze.id,
                    cloze.name.clone(),
                    "cloze",
                    stock_lowering_defaults("cloze")
                        .expect("source-grounded cloze lowering defaults"),
                    &helpers,
                ) {
                    Ok(notetype) => notetypes.push(notetype),
                    Err(diagnostic) => {
                        product_diagnostics.push(diagnostic);
                        continue;
                    }
                }
                mappings.push(LoweringMapping {
                    kind: "notetype",
                    product_id: cloze.id.clone(),
                    authoring_id: cloze.id.clone(),
                });
            }
            ProductNoteType::ImageOcclusion(io) => {
                let helpers = document.helpers_for(&io.id);
                match lower_stock_notetype(
                    &io.id,
                    io.name.clone(),
                    "image_occlusion",
                    stock_lowering_defaults("image_occlusion")
                        .expect("source-grounded image occlusion lowering defaults"),
                    &helpers,
                ) {
                    Ok(notetype) => notetypes.push(notetype),
                    Err(diagnostic) => {
                        product_diagnostics.push(diagnostic);
                        continue;
                    }
                }
                mappings.push(LoweringMapping {
                    kind: "notetype",
                    product_id: io.id.clone(),
                    authoring_id: io.id.clone(),
                });
            }
            ProductNoteType::Custom(custom) => {
                let helpers = document.helpers_for(&custom.id);
                if !helpers.is_empty() {
                    match apply_helpers("custom", "", "", &helpers) {
                        Ok(_) => {}
                        Err(diagnostic) => {
                            product_diagnostics.push(diagnostic);
                            continue;
                        }
                    }
                }

                notetypes.push(AuthoringNotetype {
                    id: custom.id.clone(),
                    kind: "normal".into(),
                    name: custom.name.clone(),
                    original_stock_kind: None,
                    original_id: None,
                    fields: Some(
                        custom
                            .fields
                            .iter()
                            .enumerate()
                            .map(|(ord, field)| AuthoringField {
                                name: field.name.clone(),
                                ord: Some(ord as u32),
                                config_id: None,
                                tag: None,
                                prevent_deletion: false,
                            })
                            .collect(),
                    ),
                    templates: Some(
                        custom
                            .templates
                            .iter()
                            .enumerate()
                            .map(|(ord, template)| AuthoringTemplate {
                                name: template.name.clone(),
                                ord: Some(ord as u32),
                                config_id: None,
                                question_format: template.question_format.clone(),
                                answer_format: template.answer_format.clone(),
                                browser_question_format: None,
                                browser_answer_format: None,
                                target_deck_name: None,
                                browser_font_name: None,
                                browser_font_size: None,
                            })
                            .collect(),
                    ),
                    css: Some(custom.css.clone().unwrap_or_default()),
                    field_metadata: vec![],
                });
                mappings.push(LoweringMapping {
                    kind: "notetype",
                    product_id: custom.id.clone(),
                    authoring_id: custom.id.clone(),
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
            ProductNote::Custom(note) => {
                notes.push(AuthoringNote {
                    id: note.id.clone(),
                    notetype_id: note.note_type_id.clone(),
                    deck_name: note.deck_name.clone(),
                    fields: note.fields.clone(),
                    tags: note.tags.clone(),
                });

                mappings.push(LoweringMapping {
                    kind: "note",
                    product_id: note.id.clone(),
                    authoring_id: note.id.clone(),
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

fn lower_stock_notetype(
    id: &str,
    name_override: Option<String>,
    note_kind: &str,
    defaults: StockLoweringDefaults,
    helpers: &[HelperDeclaration],
) -> Result<AuthoringNotetype, ProductDiagnostic> {
    let templates = defaults
        .templates
        .into_iter()
        .map(|template| {
            let (question_format, answer_format) = apply_helpers(
                note_kind,
                &template.question_format,
                &template.answer_format,
                helpers,
            )?;

            Ok(AuthoringTemplate {
                name: template.name,
                ord: template.ord,
                config_id: template.config_id,
                question_format,
                answer_format,
                browser_question_format: template.browser_question_format,
                browser_answer_format: template.browser_answer_format,
                target_deck_name: template.target_deck_name,
                browser_font_name: template.browser_font_name,
                browser_font_size: template.browser_font_size,
            })
        })
        .collect::<Result<Vec<_>, ProductDiagnostic>>()?;

    Ok(AuthoringNotetype {
        id: id.into(),
        kind: defaults.kind,
        name: Some(name_override.unwrap_or(defaults.name)),
        original_stock_kind: Some(defaults.original_stock_kind),
        original_id: None,
        fields: Some(defaults.fields),
        templates: Some(templates),
        css: Some(defaults.css),
        field_metadata: defaults.field_metadata,
    })
}
