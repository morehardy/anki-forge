use std::collections::{BTreeMap, BTreeSet};

use crate::identity::{resolve_identity, DefaultNonceSource};
use crate::model::{
    DiagnosticItem, NormalizationDiagnostics, NormalizationRequest, NormalizationResult,
    NormalizedIr, NormalizedMedia, NormalizedNote, PolicyRefs,
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

        let expected_fields: BTreeSet<&str> =
            notetype.fields.iter().map(|field| field.name.as_str()).collect();
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
        });
    }

    let normalized_media = request
        .input
        .media
        .iter()
        .map(|media| NormalizedMedia {
            filename: media.filename.clone(),
            mime: media.mime.clone(),
            data_base64: media.data_base64.clone(),
        })
        .collect::<Vec<_>>();

    let normalized_ir = NormalizedIr {
        kind: "normalized-ir".into(),
        schema_version: request.input.schema_version,
        document_id: metadata_document_id,
        resolved_identity: resolved_identity.clone(),
        notetypes: normalized_notetypes,
        notes: normalized_notes,
        media: normalized_media,
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
