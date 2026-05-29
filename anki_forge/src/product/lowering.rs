use std::collections::{BTreeMap, BTreeSet};

use authoring_core::stock::{stock_lowering_defaults, StockLoweringDefaults};

use crate::{
    AuthoringDocument, AuthoringField, AuthoringNote, AuthoringNotetype, AuthoringTemplate,
};

use super::{
    assets::AssetSource,
    diagnostics::{LoweringDiagnostic, ProductDiagnostic, ProductLoweringError},
    helpers::{apply_helpers, HelperDeclaration},
    metadata::FieldMetadataDeclaration,
    model::{
        CustomNoteType, ProductCustomNoteTypeV2, ProductFieldContentV2, ProductGenerationRuleV2,
        ProductIdentityV2, ProductMediaSourceV2, ProductNote, ProductNoteType, ProductNoteTypeV2,
        ProductNoteV2, ProductStockNoteV2,
    },
    ProductDocument,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweringMapping {
    pub kind: &'static str,
    pub source_kind: &'static str,
    pub product_id: String,
    pub authoring_id: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProductSourceMap {
    by_authoring_path: BTreeMap<String, String>,
}

impl ProductSourceMap {
    pub(crate) fn insert(
        &mut self,
        authoring_path: impl Into<String>,
        product_source: impl Into<String>,
    ) {
        self.by_authoring_path
            .insert(authoring_path.into(), product_source.into());
    }

    pub fn source_for_authoring_path(&self, authoring_path: &str) -> Option<&str> {
        self.by_authoring_path
            .get(authoring_path)
            .map(String::as_str)
    }

    pub fn source_for_diagnostic_path(&self, path: &str) -> Option<&str> {
        self.source_for_authoring_path(path)
            .or_else(|| self.source_for_authoring_path(&authoring_media_path(path)))
            .or_else(|| self.source_for_authoring_path(&authoring_media_export_path(path)))
    }
}

#[derive(Debug, Clone)]
pub struct LoweringPlan {
    pub authoring_document: AuthoringDocument,
    pub mappings: Vec<LoweringMapping>,
    pub source_map: ProductSourceMap,
    pub product_diagnostics: Vec<ProductDiagnostic>,
    pub lowering_diagnostics: Vec<LoweringDiagnostic>,
}

pub fn lower_document(document: &ProductDocument) -> Result<LoweringPlan, ProductLoweringError> {
    if let Some(v2) = document.product_v2() {
        return Ok(lower_product_v2_document(document, v2));
    }

    lower_legacy_product_document(document)
}

fn lower_legacy_product_document(
    document: &ProductDocument,
) -> Result<LoweringPlan, ProductLoweringError> {
    let mut notetypes: Vec<AuthoringNotetype> = Vec::new();
    let mut notes: Vec<AuthoringNote> = Vec::new();
    let mut media: Vec<crate::AuthoringMedia> = Vec::new();
    let mut media_by_identity: BTreeMap<String, String> = BTreeMap::new();
    let mut mappings: Vec<LoweringMapping> = Vec::new();
    let mut source_map = ProductSourceMap::default();
    let mut product_diagnostics: Vec<ProductDiagnostic> = Vec::new();
    let mut lowering_diagnostics: Vec<LoweringDiagnostic> = Vec::new();
    let mut notetype_id_counts = BTreeMap::<String, usize>::new();
    for notetype in document.note_types() {
        *notetype_id_counts
            .entry(product_notetype_id(notetype).to_string())
            .or_default() += 1;
    }

    for (notetype_index, notetype) in document.note_types().iter().enumerate() {
        match notetype {
            ProductNoteType::Basic(basic) => {
                let helpers = document.helpers_for(&basic.id);
                match lower_stock_notetype(
                    document,
                    &basic.id,
                    basic.name.clone(),
                    "basic",
                    stock_lowering_defaults("basic")
                        .expect("source-grounded basic lowering defaults"),
                    &helpers,
                ) {
                    Ok(notetype) => {
                        record_notetype_source_paths(
                            &mut source_map,
                            &notetype,
                            notetype_index,
                            notetype_id_counts
                                .get(&basic.id)
                                .copied()
                                .unwrap_or_default()
                                > 1,
                        );
                        notetypes.push(notetype);
                    }
                    Err(diagnostic) => {
                        product_diagnostics.push(diagnostic);
                        continue;
                    }
                }
                mappings.push(LoweringMapping {
                    kind: "notetype",
                    source_kind: "notetype",
                    product_id: basic.id.clone(),
                    authoring_id: basic.id.clone(),
                });
            }
            ProductNoteType::Cloze(cloze) => {
                let helpers = document.helpers_for(&cloze.id);
                match lower_stock_notetype(
                    document,
                    &cloze.id,
                    cloze.name.clone(),
                    "cloze",
                    stock_lowering_defaults("cloze")
                        .expect("source-grounded cloze lowering defaults"),
                    &helpers,
                ) {
                    Ok(notetype) => {
                        record_notetype_source_paths(
                            &mut source_map,
                            &notetype,
                            notetype_index,
                            notetype_id_counts
                                .get(&cloze.id)
                                .copied()
                                .unwrap_or_default()
                                > 1,
                        );
                        notetypes.push(notetype);
                    }
                    Err(diagnostic) => {
                        product_diagnostics.push(diagnostic);
                        continue;
                    }
                }
                mappings.push(LoweringMapping {
                    kind: "notetype",
                    source_kind: "notetype",
                    product_id: cloze.id.clone(),
                    authoring_id: cloze.id.clone(),
                });
            }
            ProductNoteType::ImageOcclusion(io) => {
                let helpers = document.helpers_for(&io.id);
                match lower_stock_notetype(
                    document,
                    &io.id,
                    io.name.clone(),
                    "image_occlusion",
                    stock_lowering_defaults("image_occlusion")
                        .expect("source-grounded image occlusion lowering defaults"),
                    &helpers,
                ) {
                    Ok(notetype) => {
                        record_notetype_source_paths(
                            &mut source_map,
                            &notetype,
                            notetype_index,
                            notetype_id_counts.get(&io.id).copied().unwrap_or_default() > 1,
                        );
                        notetypes.push(notetype);
                    }
                    Err(diagnostic) => {
                        product_diagnostics.push(diagnostic);
                        continue;
                    }
                }
                mappings.push(LoweringMapping {
                    kind: "notetype",
                    source_kind: "notetype",
                    product_id: io.id.clone(),
                    authoring_id: io.id.clone(),
                });
            }
            ProductNoteType::Custom(custom) => {
                let duplicate_key_diagnostics = duplicate_custom_key_diagnostics(custom);
                if !duplicate_key_diagnostics.is_empty() {
                    product_diagnostics.extend(duplicate_key_diagnostics);
                    continue;
                }

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

                let field_name_by_key = custom
                    .fields
                    .iter()
                    .map(|field| {
                        let key = field.key.clone().unwrap_or_else(|| field.name.clone());
                        (key, field.name.clone())
                    })
                    .collect::<BTreeMap<_, _>>();
                let fields = custom
                    .fields
                    .iter()
                    .enumerate()
                    .map(|(ord, field)| {
                        let key = field.key.clone().unwrap_or_else(|| field.name.clone());
                        AuthoringField {
                            name: field.name.clone(),
                            ord: Some(ord as u32),
                            config_id: Some(crate::product::stable_config_id(
                                "field", &custom.id, &key,
                            )),
                            tag: None,
                            prevent_deletion: false,
                        }
                    })
                    .collect();
                let templates = match custom
                    .templates
                    .iter()
                    .enumerate()
                    .map(|(ord, template)| {
                        let key = template
                            .key
                            .clone()
                            .unwrap_or_else(|| template.name.clone());
                        let question_format =
                            lower_generation_rule_front(&custom.id, template, &field_name_by_key)?;
                        Ok(AuthoringTemplate {
                            name: template.name.clone(),
                            ord: Some(ord as u32),
                            config_id: Some(crate::product::stable_config_id(
                                "template", &custom.id, &key,
                            )),
                            question_format,
                            answer_format: template.answer_format.clone(),
                            browser_question_format: document
                                .browser_appearance_for(&custom.id, &template.name)
                                .and_then(|declaration| declaration.question_format),
                            browser_answer_format: document
                                .browser_appearance_for(&custom.id, &template.name)
                                .and_then(|declaration| declaration.answer_format),
                            target_deck_name: document
                                .template_target_deck_for(&custom.id, &template.name)
                                .map(|declaration| declaration.deck_name),
                            browser_font_name: document
                                .browser_appearance_for(&custom.id, &template.name)
                                .and_then(|declaration| declaration.font_name),
                            browser_font_size: document
                                .browser_appearance_for(&custom.id, &template.name)
                                .and_then(|declaration| declaration.font_size),
                        })
                    })
                    .collect::<Result<Vec<_>, ProductDiagnostic>>()
                {
                    Ok(templates) => templates,
                    Err(diagnostic) => {
                        product_diagnostics.push(diagnostic);
                        continue;
                    }
                };

                let notetype = AuthoringNotetype {
                    id: custom.id.clone(),
                    kind: "normal".into(),
                    name: custom.name.clone(),
                    original_stock_kind: None,
                    original_id: None,
                    fields: Some(fields),
                    templates: Some(templates),
                    css: Some(custom.css.clone().unwrap_or_default()),
                    field_metadata: document
                        .field_metadata_for(&custom.id)
                        .into_iter()
                        .map(authoring_field_metadata)
                        .collect(),
                };
                record_notetype_source_paths(
                    &mut source_map,
                    &notetype,
                    notetype_index,
                    notetype_id_counts
                        .get(&custom.id)
                        .copied()
                        .unwrap_or_default()
                        > 1,
                );
                notetypes.push(notetype);
                mappings.push(LoweringMapping {
                    kind: "notetype",
                    source_kind: "notetype",
                    product_id: custom.id.clone(),
                    authoring_id: custom.id.clone(),
                });
            }
        }
    }

    for note in document.notes() {
        let deck_name = match note {
            ProductNote::Basic(basic) => basic.deck_name.clone(),
            ProductNote::Cloze(cloze) => cloze.deck_name.clone(),
            ProductNote::ImageOcclusion(io) => io.deck_name.clone(),
            ProductNote::Custom(custom) => custom.deck_name.clone(),
        };
        match note {
            ProductNote::Basic(basic) => {
                let mut fields: BTreeMap<String, String> = BTreeMap::new();
                fields.insert("Front".into(), basic.front.clone());
                fields.insert("Back".into(), basic.back.clone());

                record_note_field_source_paths(&mut source_map, &basic.id, fields.keys());
                notes.push(AuthoringNote {
                    id: basic.id.clone(),
                    notetype_id: basic.note_type_id.clone(),
                    deck_name: deck_name.clone(),
                    fields,
                    tags: basic.tags.clone(),
                });

                mappings.push(LoweringMapping {
                    kind: "note",
                    source_kind: "note",
                    product_id: basic.id.clone(),
                    authoring_id: basic.id.clone(),
                });
            }
            ProductNote::Cloze(cloze) => {
                let mut fields: BTreeMap<String, String> = BTreeMap::new();
                fields.insert("Text".into(), cloze.text.clone());
                fields.insert("Back Extra".into(), cloze.back_extra.clone());

                record_note_field_source_paths(&mut source_map, &cloze.id, fields.keys());
                notes.push(AuthoringNote {
                    id: cloze.id.clone(),
                    notetype_id: cloze.note_type_id.clone(),
                    deck_name: deck_name.clone(),
                    fields,
                    tags: cloze.tags.clone(),
                });

                mappings.push(LoweringMapping {
                    kind: "note",
                    source_kind: "note",
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

                record_note_field_source_paths(&mut source_map, &io.id, fields.keys());
                notes.push(AuthoringNote {
                    id: io.id.clone(),
                    notetype_id: io.note_type_id.clone(),
                    deck_name: deck_name.clone(),
                    fields,
                    tags: io.tags.clone(),
                });

                mappings.push(LoweringMapping {
                    kind: "note",
                    source_kind: "note",
                    product_id: io.id.clone(),
                    authoring_id: io.id.clone(),
                });
            }
            ProductNote::Custom(note) => {
                record_note_field_source_paths(&mut source_map, &note.id, note.fields.keys());
                notes.push(AuthoringNote {
                    id: note.id.clone(),
                    notetype_id: note.note_type_id.clone(),
                    deck_name,
                    fields: note.fields.clone(),
                    tags: note.tags.clone(),
                });

                mappings.push(LoweringMapping {
                    kind: "note",
                    source_kind: "note",
                    product_id: note.id.clone(),
                    authoring_id: note.id.clone(),
                });
            }
        }
    }

    for asset in document.assets() {
        match asset {
            AssetSource::InlineTemplateStatic { .. } => {
                let lowered_filename = asset.lowered_filename();
                let authoring_media_id = format!("media:{lowered_filename}");
                media_by_identity.insert(asset.identity(), lowered_filename.clone());
                media.push(crate::AuthoringMedia {
                    id: authoring_media_id.clone(),
                    desired_filename: lowered_filename.clone(),
                    source: crate::AuthoringMediaSource::InlineBytes {
                        data_base64: asset.data_base64().into(),
                    },
                    declared_mime: Some(asset.mime().into()),
                });
                mappings.push(LoweringMapping {
                    kind: "media",
                    source_kind: "asset",
                    product_id: asset.product_id(),
                    authoring_id: lowered_filename.clone(),
                });
                record_media_source_path(&mut source_map, &asset.product_id(), &lowered_filename);
                record_media_source_path(&mut source_map, &authoring_media_id, &lowered_filename);
            }
        }
    }

    let mut notetypes_by_id: BTreeMap<String, usize> = BTreeMap::new();
    for (index, notetype) in notetypes.iter().enumerate() {
        notetypes_by_id.insert(notetype.id.clone(), index);
    }

    for binding in document.font_bindings() {
        let Some(index) = notetypes_by_id.get(&binding.note_type_id).copied() else {
            lowering_diagnostics.push(LoweringDiagnostic {
                code: "PHASE5A.FONT_BINDING_UNKNOWN_NOTETYPE",
                message: format!(
                    "font binding for note type '{}' could not resolve a lowered notetype",
                    binding.note_type_id
                ),
            });
            continue;
        };
        let asset_identity = format!("{}/{}", binding.note_type_id, binding.filename);
        let Some(media_filename) = media_by_identity.get(&asset_identity) else {
            lowering_diagnostics.push(LoweringDiagnostic {
                code: "PRODUCT.MEDIA_HELPER_REFERENCE_UNREGISTERED",
                message: format!(
                    "font binding for note type '{}' would reference unregistered bundled asset '{}'",
                    binding.note_type_id, binding.filename
                ),
            });
            continue;
        };
        let notetype = &mut notetypes[index];
        let mut css = notetype.css.take().unwrap_or_default();
        let font_face = format!(
            "@font-face {{ font-family: '{}'; src: url('{}'); }}",
            escape_css_string_literal(&binding.family),
            escape_css_string_literal(media_filename),
        );
        if !css.is_empty() {
            css.push('\n');
        }
        css.push_str(&font_face);
        notetype.css = Some(css);
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
            media,
        },
        mappings,
        source_map,
        product_diagnostics: Vec::new(),
        lowering_diagnostics,
    })
}

fn lower_product_v2_document(
    document: &ProductDocument,
    v2: &crate::product::model::ProductDocumentV2Payload,
) -> LoweringPlan {
    let mut plan = LoweringPlan {
        authoring_document: AuthoringDocument {
            kind: "authoring-ir".into(),
            schema_version: "0.1.0".into(),
            metadata_document_id: document.document_id().to_string(),
            notetypes: Vec::new(),
            notes: Vec::new(),
            media: Vec::new(),
        },
        mappings: Vec::new(),
        source_map: ProductSourceMap::default(),
        product_diagnostics: v2.transport_diagnostics.clone(),
        lowering_diagnostics: Vec::new(),
    };

    let mut declared_stock = BTreeSet::<String>::new();
    let mut custom_declarations = BTreeMap::<String, ProductCustomNoteTypeV2>::new();
    for (index, notetype) in v2.note_types.iter().enumerate() {
        match notetype {
            ProductNoteTypeV2::Stock(stock) => {
                if stock.id != "basic" && stock.id != "cloze" {
                    push_product_diagnostic(
                        &mut plan,
                        "PRODUCT.STOCK_NOTE_TYPE_INVALID",
                        format!(
                            "stock note type '{}' is not supported in product-v2",
                            stock.id
                        ),
                    );
                    continue;
                }
                declared_stock.insert(stock.id.clone());
                validate_v2_stock_generation_rules(&mut plan, stock);
                let defaults = stock_lowering_defaults(&stock.id)
                    .expect("phase 5 stock note type id should have defaults");
                let mut lowered = lower_product_v2_stock_notetype(stock, defaults);
                if let Some(css) = stock.css.clone() {
                    lowered.css = Some(css);
                }
                record_v2_notetype_source_paths(
                    &mut plan.source_map,
                    &lowered,
                    stock.source_path.as_deref(),
                    index,
                );
                plan.mappings.push(LoweringMapping {
                    kind: "notetype",
                    source_kind: "product_v2.notetype",
                    product_id: stock.id.clone(),
                    authoring_id: lowered.id.clone(),
                });
                plan.authoring_document.notetypes.push(lowered);
            }
            ProductNoteTypeV2::Custom(custom) => {
                if custom.id == "basic" || custom.id == "cloze" {
                    push_product_diagnostic(
                        &mut plan,
                        "PRODUCT.RESERVED_ID_KIND_MISMATCH",
                        format!("custom note type '{}' uses a reserved stock id", custom.id),
                    );
                    continue;
                }
                if matches!(custom.identity, Some(ProductIdentityV2::Unknown(_))) {
                    push_product_diagnostic(
                        &mut plan,
                        "PRODUCT.IDENTITY_KIND_UNSUPPORTED",
                        format!(
                            "custom note type '{}' uses an unsupported identity kind",
                            custom.id
                        ),
                    );
                }
                let lowered = lower_product_v2_custom_notetype(&mut plan, custom);
                record_v2_notetype_source_paths(
                    &mut plan.source_map,
                    &lowered,
                    custom.source_path.as_deref(),
                    index,
                );
                custom_declarations.insert(custom.id.clone(), custom.clone());
                plan.mappings.push(LoweringMapping {
                    kind: "notetype",
                    source_kind: "product_v2.notetype",
                    product_id: custom.id.clone(),
                    authoring_id: lowered.id.clone(),
                });
                plan.authoring_document.notetypes.push(lowered);
            }
            ProductNoteTypeV2::Unknown(unknown) => {
                push_product_diagnostic(
                    &mut plan,
                    "PRODUCT.UNSUPPORTED_KIND",
                    format!("unsupported product-v2 note type kind '{}'", unknown.kind),
                );
            }
        }
    }

    let media_export_by_id = v2
        .media
        .iter()
        .map(|media| (media.id.clone(), media.export_as.clone()))
        .collect::<BTreeMap<_, _>>();

    for media in &v2.media {
        match &media.source {
            ProductMediaSourceV2::File { path } => {
                plan.authoring_document.media.push(crate::AuthoringMedia {
                    id: media.id.clone(),
                    desired_filename: media.export_as.clone(),
                    source: crate::AuthoringMediaSource::Path { path: path.clone() },
                    declared_mime: None,
                });
                record_v2_media_source_path(
                    &mut plan.source_map,
                    &media.id,
                    &media.export_as,
                    media.source_path.as_deref(),
                );
            }
            ProductMediaSourceV2::InlineBase64 {
                source_label: _,
                data_base64,
            } => {
                plan.authoring_document.media.push(crate::AuthoringMedia {
                    id: media.id.clone(),
                    desired_filename: media.export_as.clone(),
                    source: crate::AuthoringMediaSource::InlineBytes {
                        data_base64: data_base64.clone(),
                    },
                    declared_mime: None,
                });
                record_v2_media_source_path(
                    &mut plan.source_map,
                    &media.id,
                    &media.export_as,
                    media.source_path.as_deref(),
                );
            }
            ProductMediaSourceV2::Unknown(unknown) => {
                push_product_diagnostic(
                    &mut plan,
                    "PRODUCT.MEDIA_SOURCE_KIND_UNSUPPORTED",
                    format!(
                        "media '{}' uses unsupported source kind '{}'",
                        media.id, unknown.kind
                    ),
                );
            }
        }
    }

    for (serialized_index, note) in v2.notes.iter().enumerate() {
        match note {
            ProductNoteV2::Stock(stock) => {
                if !declared_stock.contains(&stock.note_type_id) {
                    push_product_diagnostic(
                        &mut plan,
                        "PRODUCT.STOCK_NOTE_TYPE_MISSING",
                        format!(
                            "stock note references undeclared note type '{}'",
                            stock.note_type_id
                        ),
                    );
                    continue;
                }
                lower_product_v2_stock_note(
                    &mut plan,
                    stock,
                    serialized_index,
                    &media_export_by_id,
                );
            }
            ProductNoteV2::Custom(custom) => {
                let Some(notetype) = custom_declarations.get(&custom.note_type_id).cloned() else {
                    push_product_diagnostic(
                        &mut plan,
                        "PRODUCT.CUSTOM_NOTE_TYPE_MISSING",
                        format!(
                            "custom note references undeclared note type '{}'",
                            custom.note_type_id
                        ),
                    );
                    continue;
                };
                lower_product_v2_custom_note(
                    &mut plan,
                    custom,
                    &notetype,
                    serialized_index,
                    &media_export_by_id,
                );
            }
            ProductNoteV2::Unknown(unknown) => {
                push_product_diagnostic(
                    &mut plan,
                    "PRODUCT.UNSUPPORTED_KIND",
                    format!("unsupported product-v2 note kind '{}'", unknown.kind),
                );
            }
        }
    }

    plan
}

fn lower_product_v2_stock_notetype(
    stock: &crate::product::model::ProductStockNoteTypeV2,
    defaults: StockLoweringDefaults,
) -> AuthoringNotetype {
    AuthoringNotetype {
        id: stock.id.clone(),
        kind: defaults.kind,
        name: Some(stock.name.clone().unwrap_or(defaults.name)),
        original_stock_kind: Some(defaults.original_stock_kind),
        original_id: None,
        fields: Some(defaults.fields),
        templates: Some(defaults.templates),
        css: Some(defaults.css),
        field_metadata: defaults.field_metadata,
    }
}

fn lower_product_v2_custom_notetype(
    plan: &mut LoweringPlan,
    custom: &ProductCustomNoteTypeV2,
) -> AuthoringNotetype {
    let field_name_by_key = custom
        .fields
        .iter()
        .map(|field| (field.key.clone(), field.name.clone()))
        .collect::<BTreeMap<_, _>>();
    let fields = custom
        .fields
        .iter()
        .enumerate()
        .map(|(ord, field)| AuthoringField {
            name: field.name.clone(),
            ord: Some(ord as u32),
            config_id: Some(crate::product::stable_config_id(
                "field", &custom.id, &field.key,
            )),
            tag: None,
            prevent_deletion: false,
        })
        .collect();
    let templates = custom
        .templates
        .iter()
        .enumerate()
        .map(|(ord, template)| AuthoringTemplate {
            name: template.name.clone(),
            ord: Some(ord as u32),
            config_id: Some(crate::product::stable_config_id(
                "template",
                &custom.id,
                &template.key,
            )),
            question_format: lower_product_v2_generation_rule_front(
                plan,
                &custom.id,
                &template.name,
                &template.front,
                template.generation_rule.as_ref(),
                &field_name_by_key,
            ),
            answer_format: template.back.clone(),
            browser_question_format: None,
            browser_answer_format: None,
            target_deck_name: None,
            browser_font_name: None,
            browser_font_size: None,
        })
        .collect();

    AuthoringNotetype {
        id: custom.id.clone(),
        kind: "normal".into(),
        name: custom.name.clone(),
        original_stock_kind: None,
        original_id: None,
        fields: Some(fields),
        templates: Some(templates),
        css: Some(custom.css.clone().unwrap_or_default()),
        field_metadata: Vec::new(),
    }
}

fn validate_v2_stock_generation_rules(
    plan: &mut LoweringPlan,
    stock: &crate::product::model::ProductStockNoteTypeV2,
) {
    let field_keys = stock
        .fields
        .iter()
        .map(|field| field.key.as_str())
        .collect::<BTreeSet<_>>();
    let field_names = stock
        .fields
        .iter()
        .map(|field| (field.key.clone(), field.name.clone()))
        .collect::<BTreeMap<_, _>>();
    for template in &stock.templates {
        if let Some(rule) = template.generation_rule.as_ref() {
            validate_product_v2_generation_rule(
                plan,
                &stock.id,
                &template.name,
                rule,
                &field_keys,
                &field_names,
            );
        }
    }
}

fn lower_product_v2_generation_rule_front(
    plan: &mut LoweringPlan,
    note_type_id: &str,
    template_name: &str,
    front: &str,
    rule: Option<&ProductGenerationRuleV2>,
    field_name_by_key: &BTreeMap<String, String>,
) -> String {
    let field_keys = field_name_by_key
        .keys()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let Some(rule) = rule else {
        return front.to_string();
    };

    if !validate_product_v2_generation_rule(
        plan,
        note_type_id,
        template_name,
        rule,
        &field_keys,
        field_name_by_key,
    ) {
        return front.to_string();
    }

    match rule {
        ProductGenerationRuleV2::AnkiDefault => front.to_string(),
        ProductGenerationRuleV2::All { fields } => {
            let field_names = fields
                .iter()
                .filter_map(|field| field_name_by_key.get(field).cloned())
                .collect::<Vec<_>>();
            wrap_front_with_all_conditions(front, &field_names)
        }
        ProductGenerationRuleV2::Any { fields } => {
            let field_names = fields
                .iter()
                .filter_map(|field| field_name_by_key.get(field).cloned())
                .collect::<Vec<_>>();
            wrap_front_with_any_conditions(front, &field_names)
        }
        ProductGenerationRuleV2::Cloze { .. } | ProductGenerationRuleV2::Unknown(_) => {
            front.to_string()
        }
    }
}

fn validate_product_v2_generation_rule(
    plan: &mut LoweringPlan,
    note_type_id: &str,
    template_name: &str,
    rule: &ProductGenerationRuleV2,
    field_keys: &BTreeSet<&str>,
    field_name_by_key: &BTreeMap<String, String>,
) -> bool {
    match rule {
        ProductGenerationRuleV2::AnkiDefault => true,
        ProductGenerationRuleV2::All { fields } | ProductGenerationRuleV2::Any { fields } => {
            if fields.is_empty() {
                push_product_diagnostic(
                    plan,
                    "PRODUCT.GENERATION_RULE_INVALID",
                    format!(
                        "template '{template_name}' in note type '{note_type_id}' has an empty generation field list"
                    ),
                );
                return false;
            }
            let mut valid = true;
            for field in fields {
                if !field_keys.contains(field.as_str()) {
                    push_product_diagnostic(
                        plan,
                        "PRODUCT.GENERATION_RULE_INVALID",
                        format!(
                            "template '{template_name}' in note type '{note_type_id}' references unknown generation field key '{field}'"
                        ),
                    );
                    valid = false;
                }
            }
            valid
        }
        ProductGenerationRuleV2::Cloze { field } => {
            if field.trim().is_empty() || !field_name_by_key.contains_key(field) {
                push_product_diagnostic(
                    plan,
                    "PRODUCT.GENERATION_RULE_INVALID",
                    format!(
                        "template '{template_name}' in note type '{note_type_id}' has invalid cloze generation field '{field}'"
                    ),
                );
                return false;
            }
            true
        }
        ProductGenerationRuleV2::Unknown(unknown) => {
            push_product_diagnostic(
                plan,
                "PRODUCT.GENERATION_RULE_INVALID",
                format!(
                    "template '{template_name}' in note type '{note_type_id}' uses unsupported generation rule kind '{}'",
                    unknown.kind
                ),
            );
            false
        }
    }
}

fn lower_product_v2_stock_note(
    plan: &mut LoweringPlan,
    stock: &ProductStockNoteV2,
    serialized_index: usize,
    media_export_by_id: &BTreeMap<String, String>,
) {
    let field_map = match stock.note_type_id.as_str() {
        "basic" => BTreeMap::from([("front", "Front"), ("back", "Back")]),
        "cloze" => BTreeMap::from([("text", "Text"), ("back_extra", "Back Extra")]),
        _ => BTreeMap::new(),
    };
    let mut fields = BTreeMap::new();
    for (source_key, authoring_name) in field_map {
        if let Some(content) = stock.fields.get(source_key) {
            fields.insert(
                authoring_name.to_string(),
                render_v2_content(plan, content, media_export_by_id),
            );
        }
    }

    let note_id = if let Some(stable_id) = stock.stable_id.as_deref() {
        stable_id.to_string()
    } else if stock.note_type_id == "basic" {
        match crate::deck::identity::derive_basic_stock_stable_id_from_front(
            fields.get("Front").map(String::as_str).unwrap_or_default(),
        ) {
            Ok(stable_id) => stable_id,
            Err(error) => {
                push_product_diagnostic(
                    plan,
                    "PRODUCT.IDENTITY_MISSING",
                    format!("could not derive basic stock identity: {error}"),
                );
                return;
            }
        }
    } else {
        match crate::deck::identity::derive_cloze_stock_stable_id_from_text(
            fields.get("Text").map(String::as_str).unwrap_or_default(),
        ) {
            Ok(stable_id) => stable_id,
            Err(error) => {
                push_product_diagnostic(
                    plan,
                    "PRODUCT.IDENTITY_MISSING",
                    format!("could not derive cloze stock identity: {error}"),
                );
                return;
            }
        }
    };

    let authoring_index = plan.authoring_document.notes.len();
    record_v2_note_source_paths(
        &mut plan.source_map,
        &note_id,
        authoring_index,
        serialized_index,
        stock.source_path.as_deref(),
        fields.keys(),
    );
    plan.authoring_document.notes.push(AuthoringNote {
        id: note_id.clone(),
        notetype_id: stock.note_type_id.clone(),
        deck_name: stock.deck_name.clone(),
        fields,
        tags: stock.tags.clone(),
    });
    plan.mappings.push(LoweringMapping {
        kind: "note",
        source_kind: "product_v2.note",
        product_id: stock
            .stable_id
            .clone()
            .unwrap_or_else(|| format!("product_v2.notes[{serialized_index}]")),
        authoring_id: note_id,
    });
}

fn lower_product_v2_custom_note(
    plan: &mut LoweringPlan,
    note: &crate::product::model::ProductCustomNoteV2,
    notetype: &ProductCustomNoteTypeV2,
    serialized_index: usize,
    media_export_by_id: &BTreeMap<String, String>,
) {
    let field_by_key = notetype
        .fields
        .iter()
        .map(|field| (field.key.clone(), field))
        .collect::<BTreeMap<_, _>>();
    let mut invalid = false;
    for key in note.fields.keys() {
        if !field_by_key.contains_key(key) {
            push_product_diagnostic(
                plan,
                "PRODUCT.FIELD_UNKNOWN",
                format!(
                    "custom note for note type '{}' contains unknown field key '{}'",
                    note.note_type_id, key
                ),
            );
            invalid = true;
        }
    }

    let mut fields = BTreeMap::new();
    for declaration in &notetype.fields {
        if let Some(content) = note.fields.get(&declaration.key) {
            fields.insert(
                declaration.name.clone(),
                render_v2_content(plan, content, media_export_by_id),
            );
        }
    }

    for declaration in &notetype.fields {
        if declaration.required
            && fields
                .get(&declaration.name)
                .map(|value| value.is_empty())
                .unwrap_or(true)
        {
            push_product_diagnostic(
                plan,
                "PRODUCT.REQUIRED_FIELD_MISSING",
                format!(
                    "custom note for note type '{}' is missing required field '{}'",
                    note.note_type_id, declaration.key
                ),
            );
            invalid = true;
        }
    }

    if invalid {
        return;
    }

    let note_id = if let Some(stable_id) = note.stable_id.as_deref() {
        stable_id.to_string()
    } else {
        let identity_fields = custom_identity_field_keys(notetype);
        if identity_fields.is_empty() {
            push_product_diagnostic(
                plan,
                "PRODUCT.IDENTITY_MISSING",
                format!(
                    "custom note type '{}' has no identity fields for derived note identity",
                    notetype.id
                ),
            );
            return;
        }
        let selected_fields = identity_fields
            .into_iter()
            .map(|key| {
                let field = field_by_key
                    .get(&key)
                    .expect("identity field should resolve");
                let value = fields
                    .get(&field.name)
                    .map(|value| crate::deck::identity::normalize_field_text_for_identity(value))
                    .unwrap_or_default();
                crate::product::identity::CustomIdentityFieldComponent {
                    key,
                    name: field.name.clone(),
                    value,
                }
            })
            .collect();
        crate::product::identity::derive_custom_notetype_identity(&notetype.id, selected_fields)
            .stable_id
    };

    let authoring_index = plan.authoring_document.notes.len();
    record_v2_note_source_paths(
        &mut plan.source_map,
        &note_id,
        authoring_index,
        serialized_index,
        note.source_path.as_deref(),
        fields.keys(),
    );
    plan.authoring_document.notes.push(AuthoringNote {
        id: note_id.clone(),
        notetype_id: note.note_type_id.clone(),
        deck_name: note.deck_name.clone(),
        fields,
        tags: note.tags.clone(),
    });
    plan.mappings.push(LoweringMapping {
        kind: "note",
        source_kind: "product_v2.note",
        product_id: note
            .stable_id
            .clone()
            .unwrap_or_else(|| format!("product_v2.notes[{serialized_index}]")),
        authoring_id: note_id,
    });
}

fn custom_identity_field_keys(notetype: &ProductCustomNoteTypeV2) -> Vec<String> {
    match notetype.identity.as_ref() {
        Some(ProductIdentityV2::Fields { fields }) => fields.clone(),
        Some(ProductIdentityV2::Unknown(_)) => Vec::new(),
        None => notetype
            .fields
            .iter()
            .filter(|field| field.identity)
            .map(|field| field.key.clone())
            .collect(),
    }
}

fn render_v2_content(
    plan: &mut LoweringPlan,
    content: &ProductFieldContentV2,
    media_export_by_id: &BTreeMap<String, String>,
) -> String {
    match content {
        ProductFieldContentV2::Text { value } => crate::product::content::escape_html(value),
        ProductFieldContentV2::Html { value } => value.clone(),
        ProductFieldContentV2::Sound { media_id } => {
            let Some(export_as) = media_export_by_id.get(media_id) else {
                push_product_diagnostic(
                    plan,
                    "PRODUCT.MEDIA_MISSING",
                    format!("field content references missing media id '{media_id}'"),
                );
                return String::new();
            };
            format!("[sound:{export_as}]")
        }
        ProductFieldContentV2::Image { media_id } => {
            let Some(export_as) = media_export_by_id.get(media_id) else {
                push_product_diagnostic(
                    plan,
                    "PRODUCT.MEDIA_MISSING",
                    format!("field content references missing media id '{media_id}'"),
                );
                return String::new();
            };
            format!(
                "<img src=\"{}\" alt=\"\">",
                crate::product::content::escape_html(export_as)
            )
        }
        ProductFieldContentV2::Unknown(unknown) => {
            push_product_diagnostic(
                plan,
                "PRODUCT.FIELD_CONTENT_KIND_UNSUPPORTED",
                format!(
                    "unsupported product-v2 field content kind '{}'",
                    unknown.kind
                ),
            );
            String::new()
        }
    }
}

fn record_v2_notetype_source_paths(
    source_map: &mut ProductSourceMap,
    notetype: &AuthoringNotetype,
    source_path: Option<&str>,
    index: usize,
) {
    let source = source_path
        .map(str::to_owned)
        .unwrap_or_else(|| format!("product_v2.note_types[{index}]"));
    source_map.insert(
        format!("authoring.note_types[{:?}]", notetype.id),
        source.clone(),
    );
    source_map.insert(format!("product_v2.note_types[{index}]"), source.clone());
    if let Some(templates) = notetype.templates.as_ref() {
        for template in templates {
            source_map.insert(
                format!(
                    "authoring.note_types[{:?}].templates[{:?}].front",
                    notetype.id, template.name
                ),
                format!("{source}.templates[{:?}].front", template.name),
            );
            source_map.insert(
                format!(
                    "authoring.note_types[{:?}].templates[{:?}].back",
                    notetype.id, template.name
                ),
                format!("{source}.templates[{:?}].back", template.name),
            );
        }
    }
    if notetype.css.is_some() {
        source_map.insert(
            format!("authoring.note_types[{:?}].css", notetype.id),
            format!("{source}.css"),
        );
    }
}

fn record_v2_note_source_paths<'a>(
    source_map: &mut ProductSourceMap,
    note_id: &str,
    authoring_index: usize,
    serialized_index: usize,
    source_path: Option<&str>,
    fields: impl IntoIterator<Item = &'a String>,
) {
    let source = source_path
        .map(str::to_owned)
        .unwrap_or_else(|| format!("product_v2.notes[{serialized_index}]"));
    source_map.insert(
        format!("authoring.notes[{authoring_index}]"),
        source.clone(),
    );
    source_map.insert(format!("authoring.notes[{note_id:?}]"), source.clone());
    source_map.insert(
        format!("product_v2.notes[{serialized_index}]"),
        source.clone(),
    );
    for field in fields {
        source_map.insert(
            authoring_note_field_path(note_id, field),
            product_note_field_source(&source, field),
        );
    }
}

fn record_v2_media_source_path(
    source_map: &mut ProductSourceMap,
    media_id: &str,
    export_filename: &str,
    source_path: Option<&str>,
) {
    let source = source_path
        .map(str::to_owned)
        .unwrap_or_else(|| product_media_source(export_filename));
    source_map.insert(authoring_media_path(media_id), source.clone());
    source_map.insert(authoring_media_export_path(export_filename), source.clone());
    source_map.insert(media_id, source.clone());
    source_map.insert(export_filename, source);
}

fn push_product_diagnostic(
    plan: &mut LoweringPlan,
    code: &'static str,
    message: impl Into<String>,
) {
    plan.product_diagnostics.push(ProductDiagnostic {
        code,
        message: message.into(),
    });
}

fn duplicate_custom_key_diagnostics(custom: &CustomNoteType) -> Vec<ProductDiagnostic> {
    let mut diagnostics = Vec::new();

    let mut field_keys: BTreeMap<&str, &str> = BTreeMap::new();
    for field in &custom.fields {
        let key = field.key.as_deref().unwrap_or(field.name.as_str());
        if let Some(first_field) = field_keys.insert(key, field.name.as_str()) {
            diagnostics.push(ProductDiagnostic::duplicate_field_key(
                &custom.id,
                key,
                first_field,
                &field.name,
            ));
        }
    }

    let mut template_keys: BTreeMap<&str, &str> = BTreeMap::new();
    for template in &custom.templates {
        let key = template.key.as_deref().unwrap_or(template.name.as_str());
        if let Some(first_template) = template_keys.insert(key, template.name.as_str()) {
            diagnostics.push(ProductDiagnostic::duplicate_template_key(
                &custom.id,
                key,
                first_template,
                &template.name,
            ));
        }
    }

    diagnostics
}

fn product_notetype_id(notetype: &ProductNoteType) -> &str {
    match notetype {
        ProductNoteType::Basic(basic) => &basic.id,
        ProductNoteType::Cloze(cloze) => &cloze.id,
        ProductNoteType::ImageOcclusion(io) => &io.id,
        ProductNoteType::Custom(custom) => &custom.id,
    }
}

pub(crate) fn authoring_note_field_path(note_id: &str, field_name: &str) -> String {
    format!("authoring.notes[{note_id:?}].fields[{field_name:?}]")
}

pub(crate) fn product_note_field_source(note_source: &str, field_name: &str) -> String {
    format!("{note_source}.fields[{field_name:?}]")
}

pub(crate) fn authoring_media_path(media_id: &str) -> String {
    format!("authoring.media[{media_id:?}]")
}

pub(crate) fn authoring_media_export_path(filename: &str) -> String {
    format!("authoring.media_exports[{filename:?}]")
}

pub(crate) fn product_media_source(filename: &str) -> String {
    format!("project.media[{filename:?}]")
}

pub(crate) fn record_media_source_path(
    source_map: &mut ProductSourceMap,
    media_id: &str,
    export_filename: &str,
) {
    let source = product_media_source(export_filename);
    source_map.insert(authoring_media_path(media_id), source.clone());
    source_map.insert(authoring_media_export_path(export_filename), source.clone());
    source_map.insert(media_id, source.clone());
    source_map.insert(export_filename, source);
}

fn record_note_field_source_paths<'a>(
    source_map: &mut ProductSourceMap,
    note_id: &str,
    fields: impl IntoIterator<Item = &'a String>,
) {
    let note_source = format!("project.notes[{note_id:?}]");
    for field in fields {
        source_map.insert(
            authoring_note_field_path(note_id, field),
            product_note_field_source(&note_source, field),
        );
    }
}

fn record_notetype_source_paths(
    source_map: &mut ProductSourceMap,
    notetype: &AuthoringNotetype,
    product_notetype_index: usize,
    duplicate_product_notetype_id: bool,
) {
    let authoring_notetype_source = if duplicate_product_notetype_id {
        format!("authoring.note_types[{product_notetype_index}]")
    } else {
        format!("authoring.note_types[{:?}]", notetype.id)
    };
    let product_notetype_source = if duplicate_product_notetype_id {
        format!("project.note_types[{product_notetype_index}]")
    } else {
        format!("project.note_types[{:?}]", notetype.id)
    };
    if let Some(templates) = notetype.templates.as_ref() {
        for template in templates {
            let authoring_template =
                format!("{authoring_notetype_source}.templates[{:?}]", template.name);
            let product_template =
                format!("{product_notetype_source}.templates[{:?}]", template.name);
            source_map.insert(
                format!("{authoring_template}.front"),
                format!("{product_template}.front"),
            );
            source_map.insert(
                format!("{authoring_template}.back"),
                format!("{product_template}.back"),
            );
            if template.browser_question_format.is_some() {
                source_map.insert(
                    format!("{authoring_template}.browser_front"),
                    format!("{product_template}.browser_front"),
                );
            }
            if template.browser_answer_format.is_some() {
                source_map.insert(
                    format!("{authoring_template}.browser_back"),
                    format!("{product_template}.browser_back"),
                );
            }
        }
    }

    if notetype.css.is_some() {
        source_map.insert(
            format!("{authoring_notetype_source}.css"),
            format!("{product_notetype_source}.css"),
        );
    }
}

fn lower_stock_notetype(
    document: &ProductDocument,
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
            let browser_appearance = document.browser_appearance_for(id, &template.name);
            let target_deck = document.template_target_deck_for(id, &template.name);

            Ok(AuthoringTemplate {
                name: template.name,
                ord: template.ord,
                config_id: template.config_id,
                question_format,
                answer_format,
                browser_question_format: browser_appearance
                    .as_ref()
                    .and_then(|declaration| declaration.question_format.clone())
                    .or(template.browser_question_format),
                browser_answer_format: browser_appearance
                    .as_ref()
                    .and_then(|declaration| declaration.answer_format.clone())
                    .or(template.browser_answer_format),
                target_deck_name: target_deck
                    .as_ref()
                    .map(|declaration| declaration.deck_name.clone())
                    .or(template.target_deck_name),
                browser_font_name: browser_appearance
                    .as_ref()
                    .and_then(|declaration| declaration.font_name.clone())
                    .or(template.browser_font_name),
                browser_font_size: browser_appearance
                    .as_ref()
                    .and_then(|declaration| declaration.font_size)
                    .or(template.browser_font_size),
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
        field_metadata: document
            .field_metadata_for(id)
            .into_iter()
            .map(authoring_field_metadata)
            .chain(defaults.field_metadata)
            .collect(),
    })
}

fn lower_generation_rule_front(
    note_type_id: &str,
    template: &crate::product::model::CustomTemplate,
    field_name_by_key: &BTreeMap<String, String>,
) -> Result<String, ProductDiagnostic> {
    let Some(rule) = &template.generation_rule else {
        return Ok(template.question_format.clone());
    };

    match rule {
        crate::product::model::CustomGenerationRule::AnkiDefault => {
            Ok(template.question_format.clone())
        }
        crate::product::model::CustomGenerationRule::All { fields } => {
            let field_names =
                generation_field_names(note_type_id, template, fields, field_name_by_key)?;
            Ok(wrap_front_with_all_conditions(
                &template.question_format,
                &field_names,
            ))
        }
        crate::product::model::CustomGenerationRule::Any { fields } => {
            let field_names =
                generation_field_names(note_type_id, template, fields, field_name_by_key)?;
            Ok(wrap_front_with_any_conditions(
                &template.question_format,
                &field_names,
            ))
        }
        crate::product::model::CustomGenerationRule::Cloze { .. } => Err(ProductDiagnostic {
            code: "TEMPLATE.CLOZE_RULE_REQUIRES_STOCK_CLOZE",
            message: format!(
                "custom normal note type '{}' template '{}' cannot use cloze generation",
                note_type_id, template.name
            ),
        }),
    }
}

fn generation_field_names(
    note_type_id: &str,
    template: &crate::product::model::CustomTemplate,
    fields: &[String],
    field_name_by_key: &BTreeMap<String, String>,
) -> Result<Vec<String>, ProductDiagnostic> {
    let mut field_names = Vec::with_capacity(fields.len());
    for field in fields {
        let Some(field_name) = field_name_by_key.get(field) else {
            return Err(ProductDiagnostic {
                code: "TEMPLATE.REQUIRED_FIELD_MISSING",
                message: format!(
                    "template '{}' in note type '{}' references unknown field key '{}'",
                    template.name, note_type_id, field
                ),
            });
        };
        field_names.push(field_name.clone());
    }
    Ok(field_names)
}

fn wrap_front_with_all_conditions(front: &str, field_keys: &[String]) -> String {
    field_keys
        .iter()
        .rev()
        .fold(front.to_string(), |inner, field| {
            format!("{{{{#{field}}}}}{inner}{{{{/{field}}}}}")
        })
}

fn wrap_front_with_any_conditions(front: &str, field_keys: &[String]) -> String {
    let Some((field, rest)) = field_keys.split_first() else {
        return String::new();
    };
    let guarded_front = format!("{{{{#{field}}}}}{front}{{{{/{field}}}}}");
    if rest.is_empty() {
        return guarded_front;
    }
    format!(
        "{guarded_front}{{{{^{field}}}}}{}{{{{/{field}}}}}",
        wrap_front_with_any_conditions(front, rest)
    )
}

fn escape_css_string_literal(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\'', "\\'")
}

fn authoring_field_metadata(
    field: FieldMetadataDeclaration,
) -> authoring_core::AuthoringFieldMetadata {
    authoring_core::AuthoringFieldMetadata {
        field_name: field.field_name,
        label: field.label,
        role_hint: field.role_hint,
    }
}
