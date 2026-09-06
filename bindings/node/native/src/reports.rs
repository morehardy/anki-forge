use crate::json_numbers::safe_json_numbers;
use anki_forge::build::{BuildReport, BuildReportJson};
use serde_json::{json, Value};

pub fn domain_failure(kind: &str, error: anyhow::Error) -> String {
    let code = if let Some(error) = error.downcast_ref::<anki_forge::deck::MediaError>() {
        error.code().as_str().to_string()
    } else if let Some(error) = error.downcast_ref::<anki_forge::product::ProductNoteError>() {
        error.code().as_str().to_string()
    } else if let Some(error) = error.downcast_ref::<anki_forge::deck::DeckError>() {
        error.code().as_str().to_string()
    } else {
        "BINDING.OPERATION_FAILED".to_string()
    };
    failure(kind, &code, &error.to_string(), Value::Null)
}

pub fn diff_report(report: &anki_forge::diff::ProjectDiffReport) -> Value {
    let mut value = serde_json::to_value(report).expect("diff report serializes");
    value["diagnostics"] = json!(report
        .diagnostics
        .iter()
        .map(anki_forge::build::json_report::DiagnosticJson::from)
        .collect::<Vec<_>>());
    safe_json_numbers(&mut value);
    value
}

pub fn success(value: Value) -> String {
    json!({"ok": true, "value": value}).to_string()
}

pub fn failure(kind: &str, code: &str, message: &str, details: Value) -> String {
    json!({"ok": false, "error": {"kind": kind, "code": code, "message": message, "details": details}}).to_string()
}

pub fn build_report(report: &BuildReport) -> Value {
    let mut value = serde_json::to_value(BuildReportJson::from_report(report))
        .expect("core build report serialization is infallible");
    safe_json_numbers(&mut value);
    json!({"report": value, "pretty": report.pretty_report()})
}
