use std::collections::BTreeMap;

use crate::diagnostics::{Diagnostic, DiagnosticCode, Severity, SourcePath};

use super::model::{FieldMergeEntry, IdentityIndex, NotetypeIdentityEntry, TemplateMergeEntry};

pub fn compare_notetype_merge_safety(
    current: &IdentityIndex,
    baseline: &IdentityIndex,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let current_by_id: BTreeMap<_, _> = current
        .notetypes
        .iter()
        .map(|notetype| (notetype.note_type_id.as_str(), notetype))
        .collect();
    let baseline_by_id: BTreeMap<_, _> = baseline
        .notetypes
        .iter()
        .map(|notetype| (notetype.note_type_id.as_str(), notetype))
        .collect();

    for current_notetype in &current.notetypes {
        let Some(baseline_notetype) = baseline_by_id.get(current_notetype.note_type_id.as_str())
        else {
            diagnostics.push(warning(
                "UPDATE.NOTETYPE_SET_CHANGED",
                &current_notetype.note_type_id,
                "change_kind=added; notetype was added",
            ));
            continue;
        };
        compare_notetype(current_notetype, baseline_notetype, &mut diagnostics);
    }
    for baseline_notetype in &baseline.notetypes {
        if !current_by_id.contains_key(baseline_notetype.note_type_id.as_str()) {
            diagnostics.push(warning(
                "UPDATE.NOTETYPE_SET_CHANGED",
                &baseline_notetype.note_type_id,
                "change_kind=removed; notetype was removed",
            ));
        }
    }

    diagnostics
}

fn compare_notetype(
    current: &NotetypeIdentityEntry,
    baseline: &NotetypeIdentityEntry,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if current.name != baseline.name {
        diagnostics.push(warning(
            "UPDATE.NOTETYPE_RENAMED",
            &current.note_type_id,
            "notetype name changed",
        ));
    }
    compare_fields(current, baseline, diagnostics);
    compare_templates(current, baseline, diagnostics);
}

fn compare_fields(
    current: &NotetypeIdentityEntry,
    baseline: &NotetypeIdentityEntry,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let baseline_by_key: BTreeMap<_, _> = baseline
        .fields
        .iter()
        .map(|field| (field.field_key.as_str(), field))
        .collect();
    for field in &current.fields {
        if let Some(old) = baseline_by_key.get(field.field_key.as_str()) {
            compare_field(field, old, &current.note_type_id, diagnostics);
        }
    }
}

fn compare_field(
    current: &FieldMergeEntry,
    baseline: &FieldMergeEntry,
    notetype_id: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if current.config_id != baseline.config_id {
        diagnostics.push(error(
            "UPDATE.FIELD_MERGE_ID_CHANGED",
            notetype_id,
            "field config id changed",
        ));
        return;
    }
    if current.field_name != baseline.field_name {
        diagnostics.push(warning(
            "UPDATE.FIELD_RENAMED",
            notetype_id,
            "field name changed",
        ));
    }
    if current.ord != baseline.ord {
        diagnostics.push(warning(
            "UPDATE.FIELD_ORD_CHANGED",
            notetype_id,
            "field ord changed",
        ));
    }
}

fn compare_templates(
    current: &NotetypeIdentityEntry,
    baseline: &NotetypeIdentityEntry,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let current_by_key: BTreeMap<_, _> = current
        .templates
        .iter()
        .map(|template| (template.template_key.as_str(), template))
        .collect();
    let baseline_by_key: BTreeMap<_, _> = baseline
        .templates
        .iter()
        .map(|template| (template.template_key.as_str(), template))
        .collect();
    for template in &current.templates {
        if let Some(old) = baseline_by_key.get(template.template_key.as_str()) {
            compare_template(template, old, &current.note_type_id, diagnostics);
        } else {
            diagnostics.push(warning(
                "UPDATE.TEMPLATE_SET_CHANGED",
                &current.note_type_id,
                "change_kind=added; template was added",
            ));
        }
    }
    for template in &baseline.templates {
        if !current_by_key.contains_key(template.template_key.as_str()) {
            diagnostics.push(warning(
                "UPDATE.TEMPLATE_SET_CHANGED",
                &current.note_type_id,
                "change_kind=removed; template was removed",
            ));
        }
    }
}

fn compare_template(
    current: &TemplateMergeEntry,
    baseline: &TemplateMergeEntry,
    notetype_id: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if current.config_id != baseline.config_id {
        diagnostics.push(error(
            "UPDATE.TEMPLATE_MERGE_ID_CHANGED",
            notetype_id,
            "template config id changed",
        ));
        return;
    }
    if current.template_name != baseline.template_name {
        diagnostics.push(warning(
            "UPDATE.TEMPLATE_RENAMED",
            notetype_id,
            "template name changed",
        ));
    }
    if current.ord != baseline.ord {
        diagnostics.push(warning(
            "UPDATE.TEMPLATE_ORD_CHANGED",
            &template_source(notetype_id, &current.template_name),
            "template ord changed",
        ));
    }
}

fn template_source(notetype_id: &str, template_name: &str) -> String {
    format!("notetype[id='{notetype_id}']::template[{template_name}]")
}

fn error(code: &str, source: &str, message: &str) -> Diagnostic {
    Diagnostic {
        code: DiagnosticCode::new(code),
        severity: Severity::Error,
        message: message.into(),
        source: Some(SourcePath::new(source)),
        help: None,
    }
}

fn warning(code: &str, source: &str, message: &str) -> Diagnostic {
    Diagnostic {
        code: DiagnosticCode::new(code),
        severity: Severity::Warning,
        message: message.into(),
        source: Some(SourcePath::new(source)),
        help: None,
    }
}
