use std::collections::{BTreeMap, BTreeSet};

use crate::identity::{resolve_identity, DefaultNonceSource};
use crate::media::{
    ingest_authoring_media, sort_media_references, DiagnosticBehavior, MediaReference,
    MediaReferenceResolution, NormalizeOptions,
};
use crate::media_refs::extract_media_reference_candidates;
use crate::model::{
    DiagnosticItem, NormalizationDiagnostics, NormalizationRequest, NormalizationResult,
    NormalizedIr, NormalizedNote, PolicyRefs,
};
use crate::risk::{assess_risk, unavailable_report};
use crate::selector::{
    parse_selector, resolve_selector, SelectorError, SelectorResolveError, SelectorTarget,
};

pub fn selector_resolve_error_code(error: &SelectorResolveError) -> &'static str {
    match error {
        SelectorResolveError::Unmatched => "PHASE2.SELECTOR_UNMATCHED",
        SelectorResolveError::Ambiguous => "PHASE2.SELECTOR_AMBIGUOUS",
    }
}

pub fn normalize(request: NormalizationRequest) -> NormalizationResult {
    if request.input.media.is_empty() {
        return normalize_with_options(
            request,
            NormalizeOptions {
                base_dir: std::env::current_dir().unwrap_or_else(|_| ".".into()),
                media_store_dir: std::env::temp_dir().join("anki-forge-unused-media-store"),
                media_policy: crate::media::MediaPolicy::default_strict(),
            },
        );
    }

    let policy_refs = PolicyRefs {
        identity_policy_ref: "identity-policy.default@1.0.0".into(),
        risk_policy_ref: request
            .comparison_context
            .as_ref()
            .map(|context| context.risk_policy_ref.clone()),
    };

    invalid_result(
        policy_refs,
        request.comparison_context,
        vec![DiagnosticItem {
            level: "error".into(),
            code: "MEDIA.NORMALIZE_OPTIONS_REQUIRED".into(),
            summary: "media normalization requires NormalizeOptions".into(),
        }],
        "det:unavailable".into(),
        "media normalization requires explicit options".into(),
    )
}

pub fn normalize_with_options(
    request: NormalizationRequest,
    options: NormalizeOptions,
) -> NormalizationResult {
    let risk_policy_ref = request
        .comparison_context
        .as_ref()
        .map(|context| context.risk_policy_ref.clone());

    let policy_refs = PolicyRefs {
        identity_policy_ref: "identity-policy.default@1.0.0".into(),
        risk_policy_ref,
    };

    let metadata_document_id = request.input.metadata_document_id.trim().to_string();

    if metadata_document_id.is_empty() {
        return invalid_result(
            policy_refs,
            request.comparison_context,
            vec![DiagnosticItem {
                level: "error".into(),
                code: "PHASE2.MISSING_DOCUMENT_ID".into(),
                summary: "metadata_document_id cannot be blank".into(),
            }],
            "det:unavailable".into(),
            "missing document id".into(),
        );
    }

    if let Some(raw_selector) = request.target_selector.as_deref() {
        let selector = match parse_selector(raw_selector) {
            Ok(selector) => selector,
            Err(error) => {
                return invalid_result(
                    policy_refs,
                    request.comparison_context,
                    vec![DiagnosticItem {
                        level: "error".into(),
                        code: "PHASE2.SELECTOR_INVALID".into(),
                        summary: selector_invalid_summary(&error).into(),
                    }],
                    format!("det:{metadata_document_id}"),
                    "target selector did not match any resolvable object".into(),
                );
            }
        };

        let targets = vec![SelectorTarget::new(
            request.input.kind.clone(),
            [("id", metadata_document_id.clone())],
        )];

        if let Err(error) = resolve_selector(&selector, &targets) {
            return invalid_result(
                policy_refs,
                request.comparison_context,
                vec![DiagnosticItem {
                    level: "error".into(),
                    code: selector_resolve_error_code(&error).into(),
                    summary: "target_selector resolution failed".into(),
                }],
                format!("det:{metadata_document_id}"),
                "target selector did not match any resolvable object".into(),
            );
        }
    }

    let mut diagnostics = Vec::new();
    let mut nonce_source = DefaultNonceSource;
    let resolved_identity = match resolve_identity(&request, &mut diagnostics, &mut nonce_source) {
        Ok(identity) => identity,
        Err(error) => {
            let (code, summary) = identity_error_diagnostic(&error);
            diagnostics.push(DiagnosticItem {
                level: "error".into(),
                code: code.into(),
                summary,
            });

            return invalid_result(
                policy_refs,
                request.comparison_context,
                diagnostics,
                format!("det:{metadata_document_id}"),
                "identity resolution failed".into(),
            );
        }
    };

    let mut seen_notetype_ids = BTreeSet::new();
    for notetype in &request.input.notetypes {
        if !seen_notetype_ids.insert(notetype.id.as_str()) {
            return invalid_result(
                policy_refs,
                request.comparison_context,
                vec![DiagnosticItem {
                    level: "error".into(),
                    code: "PHASE3.DUPLICATE_NOTETYPE_ID".into(),
                    summary: format!("duplicate notetype id: {}", notetype.id),
                }],
                format!("det:{metadata_document_id}"),
                "writer-ready normalization requires unique notetype ids".into(),
            );
        }
    }

    let normalized_notetypes = match request
        .input
        .notetypes
        .iter()
        .map(crate::stock::resolve_stock_notetype)
        .collect::<anyhow::Result<Vec<_>>>()
    {
        Ok(normalized_notetypes) => normalized_notetypes,
        Err(error) => {
            diagnostics.push(DiagnosticItem {
                level: "error".into(),
                code: "PHASE3.UNSUPPORTED_STOCK_KIND".into(),
                summary: error.to_string(),
            });
            return invalid_result(
                policy_refs,
                request.comparison_context,
                diagnostics,
                format!("det:{metadata_document_id}"),
                "stock notetype resolution failed".into(),
            );
        }
    };

    let normalized_notetype_by_id: BTreeMap<&str, &crate::model::NormalizedNotetype> =
        normalized_notetypes
            .iter()
            .map(|notetype| (notetype.id.as_str(), notetype))
            .collect();

    let mut normalized_notes = Vec::with_capacity(request.input.notes.len());
    for note in &request.input.notes {
        let Some(notetype) = normalized_notetype_by_id.get(note.notetype_id.as_str()) else {
            return invalid_result(
                policy_refs,
                request.comparison_context,
                vec![DiagnosticItem {
                    level: "error".into(),
                    code: "PHASE3.UNKNOWN_NOTETYPE_ID".into(),
                    summary: format!(
                        "note {} references unknown notetype_id {}",
                        note.id, note.notetype_id
                    ),
                }],
                format!("det:{metadata_document_id}"),
                "writer-ready note references an unknown notetype".into(),
            );
        };

        let expected_fields: BTreeSet<&str> = notetype
            .fields
            .iter()
            .map(|field| field.name.as_str())
            .collect();
        let actual_fields: BTreeSet<&str> = note.fields.keys().map(String::as_str).collect();
        if actual_fields != expected_fields {
            return invalid_result(
                policy_refs,
                request.comparison_context,
                vec![DiagnosticItem {
                    level: "error".into(),
                    code: "PHASE3.NOTE_FIELD_MISMATCH".into(),
                    summary: format!(
                        "note {} fields must exactly match resolved stock fields for notetype_id {}; expected {:?}, got {:?}",
                        note.id,
                        note.notetype_id,
                        expected_fields,
                        actual_fields,
                    ),
                }],
                format!("det:{metadata_document_id}"),
                "writer-ready note fields do not match the resolved stock lane".into(),
            );
        }

        normalized_notes.push(NormalizedNote {
            id: note.id.clone(),
            notetype_id: note.notetype_id.clone(),
            deck_name: note.deck_name.clone(),
            fields: note.fields.clone(),
            tags: note.tags.clone(),
            mtime_secs: None,
        });
    }

    let ingest = match ingest_authoring_media(&request.input.media, &options) {
        Ok(ingest) => ingest,
        Err(error) => {
            let items = error
                .diagnostics
                .into_iter()
                .map(media_ingest_diagnostic_to_item)
                .collect::<Vec<_>>();
            return invalid_result(
                policy_refs,
                request.comparison_context,
                items,
                format!("det:{metadata_document_id}"),
                "media ingestion failed".into(),
            );
        }
    };

    diagnostics.extend(
        ingest
            .diagnostics
            .iter()
            .cloned()
            .map(media_ingest_diagnostic_to_item),
    );

    let (media_references, media_reference_diagnostics) =
        resolve_media_references(&normalized_notes, &ingest.bindings);
    let mut media_diagnostics = media_reference_diagnostics;
    media_diagnostics.extend(unused_binding_diagnostics(
        &ingest.bindings,
        &media_references,
        options.media_policy.unused_binding_behavior,
    ));
    if media_diagnostics.iter().any(|item| item.level == "error") {
        diagnostics.extend(media_diagnostics);
        return invalid_result(
            policy_refs,
            request.comparison_context,
            diagnostics,
            format!("det:{metadata_document_id}"),
            "media reference resolution failed".into(),
        );
    }
    diagnostics.extend(media_diagnostics);

    let normalized_ir = NormalizedIr {
        kind: "normalized-ir".into(),
        schema_version: request.input.schema_version,
        document_id: metadata_document_id,
        resolved_identity: resolved_identity.clone(),
        notetypes: normalized_notetypes,
        notes: normalized_notes,
        media_objects: ingest.objects,
        media_bindings: ingest.bindings,
        media_references,
    };

    let merge_risk_report = assess_risk(&normalized_ir, request.comparison_context.as_ref());

    NormalizationResult {
        kind: "normalization-result".into(),
        result_status: "success".into(),
        tool_contract_version: crate::tool_contract_version().into(),
        policy_refs,
        comparison_context: request.comparison_context.clone(),
        diagnostics: NormalizationDiagnostics {
            kind: "normalization-diagnostics".into(),
            status: "valid".into(),
            items: diagnostics,
        },
        normalized_ir: Some(normalized_ir),
        merge_risk_report,
    }
}

fn selector_invalid_summary(error: &SelectorError) -> &'static str {
    match error {
        SelectorError::Empty => "target_selector cannot be blank",
        SelectorError::ArrayIndexNotAllowed => "target_selector cannot use array index segments",
        SelectorError::InvalidPredicate => "target_selector does not match selector grammar",
    }
}

fn media_ingest_diagnostic_to_item(
    diagnostic: crate::media::MediaIngestDiagnostic,
) -> DiagnosticItem {
    DiagnosticItem {
        level: diagnostic.level,
        code: diagnostic.code,
        summary: diagnostic.summary,
    }
}

fn resolve_media_references(
    notes: &[NormalizedNote],
    bindings: &[crate::media::MediaBinding],
) -> (Vec<MediaReference>, Vec<DiagnosticItem>) {
    let binding_by_filename = bindings
        .iter()
        .map(|binding| (binding.export_filename.as_str(), binding.id.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut references = Vec::new();
    let mut diagnostics = Vec::new();

    for note in notes {
        for (field_name, field_value) in &note.fields {
            for candidate in extract_media_reference_candidates(
                "note",
                &note.id,
                "field",
                field_name,
                field_value,
            ) {
                let resolution = if let Some(reason) = candidate.unsafe_reason {
                    diagnostics.push(DiagnosticItem {
                        level: "error".into(),
                        code: "MEDIA.UNSAFE_REFERENCE".into(),
                        summary: format!(
                            "unsafe media reference {} in note {} field {}: {}",
                            candidate.raw_ref, candidate.owner_id, candidate.location_name, reason
                        ),
                    });
                    MediaReferenceResolution::Skipped {
                        skip_reason: reason,
                    }
                } else if let Some(reason) = candidate.skip_reason {
                    MediaReferenceResolution::Skipped {
                        skip_reason: reason,
                    }
                } else if let Some(local_ref) = candidate.normalized_local_ref {
                    if let Some(media_id) = binding_by_filename.get(local_ref.as_str()) {
                        MediaReferenceResolution::Resolved {
                            media_id: (*media_id).to_string(),
                        }
                    } else {
                        diagnostics.push(DiagnosticItem {
                            level: "error".into(),
                            code: "MEDIA.MISSING_REFERENCE".into(),
                            summary: format!(
                                "missing media reference {} in note {} field {}",
                                candidate.raw_ref, candidate.owner_id, candidate.location_name
                            ),
                        });
                        MediaReferenceResolution::Missing
                    }
                } else {
                    MediaReferenceResolution::Skipped {
                        skip_reason: "unresolved-candidate".into(),
                    }
                };

                references.push(MediaReference {
                    owner_kind: candidate.owner_kind,
                    owner_id: candidate.owner_id,
                    location_kind: candidate.location_kind,
                    location_name: candidate.location_name,
                    raw_ref: candidate.raw_ref,
                    ref_kind: candidate.ref_kind,
                    resolution,
                });
            }
        }
    }

    sort_media_references(&mut references);
    (references, diagnostics)
}

fn unused_binding_diagnostics(
    bindings: &[crate::media::MediaBinding],
    references: &[MediaReference],
    behavior: DiagnosticBehavior,
) -> Vec<DiagnosticItem> {
    let level = match behavior {
        DiagnosticBehavior::Ignore => return Vec::new(),
        DiagnosticBehavior::Info => "info",
        DiagnosticBehavior::Warning => "warning",
        DiagnosticBehavior::Error => "error",
    };
    let referenced_media_ids = references
        .iter()
        .filter_map(|reference| match &reference.resolution {
            MediaReferenceResolution::Resolved { media_id } => Some(media_id.as_str()),
            MediaReferenceResolution::Missing | MediaReferenceResolution::Skipped { .. } => None,
        })
        .collect::<BTreeSet<_>>();

    bindings
        .iter()
        .filter(|binding| !referenced_media_ids.contains(binding.id.as_str()))
        .map(|binding| DiagnosticItem {
            level: level.into(),
            code: "MEDIA.UNUSED_BINDING".into(),
            summary: format!(
                "unused media binding {} for export filename {}",
                binding.id, binding.export_filename
            ),
        })
        .collect()
}

fn identity_error_diagnostic(error: &anyhow::Error) -> (&'static str, String) {
    if let Some(identity_error) = error.downcast_ref::<crate::identity::IdentityError>() {
        match identity_error {
            crate::identity::IdentityError::ReasonCodeRequired => (
                "PHASE2.REASON_CODE_REQUIRED",
                "override modes require reason_code".into(),
            ),
            crate::identity::IdentityError::ExternalIdRequired => (
                "PHASE2.EXTERNAL_ID_REQUIRED",
                "external override requires external_id".into(),
            ),
            crate::identity::IdentityError::UnsupportedOverride(mode) => (
                "PHASE2.IDENTITY_OVERRIDE_UNSUPPORTED",
                format!("unsupported identity override mode: {mode}"),
            ),
        }
    } else {
        ("PHASE2.IDENTITY_OVERRIDE_UNSUPPORTED", error.to_string())
    }
}

fn invalid_result(
    policy_refs: PolicyRefs,
    comparison_context: Option<crate::model::ComparisonContext>,
    diagnostics: Vec<DiagnosticItem>,
    current_artifact_fingerprint: String,
    comparison_reason: String,
) -> NormalizationResult {
    let merge_risk_report = unavailable_report(
        comparison_context.as_ref(),
        current_artifact_fingerprint,
        comparison_reason,
    );

    NormalizationResult {
        kind: "normalization-result".into(),
        result_status: "invalid".into(),
        tool_contract_version: crate::tool_contract_version().into(),
        policy_refs,
        comparison_context,
        diagnostics: NormalizationDiagnostics {
            kind: "normalization-diagnostics".into(),
            status: "invalid".into(),
            items: diagnostics,
        },
        normalized_ir: None,
        merge_risk_report,
    }
}
