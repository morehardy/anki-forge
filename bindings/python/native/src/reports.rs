use crate::artifacts::NativeArtifact;
use anki_forge::build::{
    json_report::DiagnosticJson, BuildError, BuildFailureCause, BuildReport, BuildReportJson,
};
use anki_forge::diff::{ProjectDiffError, ProjectDiffReport};
use pyo3::{exceptions::PyValueError, PyResult};
use serde_json::{json, Value};

pub fn failure_cause(cause: BuildFailureCause) -> &'static str {
    match cause {
        BuildFailureCause::MissingArtifact => "missing_artifact",
        BuildFailureCause::Diagnostics => "diagnostics",
        BuildFailureCause::PolicyBlocked => "policy_blocked",
        BuildFailureCause::Invalid => "invalid",
        BuildFailureCause::Io => "io",
        BuildFailureCause::Internal => "internal",
    }
}

pub fn build(
    result: Result<BuildReport, BuildError>,
) -> PyResult<(String, Option<NativeArtifact>)> {
    let (report, cause, code) = match result {
        Ok(report) => (report, None, None),
        Err(error) => {
            let code = error.code().as_str().to_string();
            (*error.report, Some(failure_cause(error.cause)), Some(code))
        }
    };
    let mut value = serde_json::to_value(BuildReportJson::from_report(&report))
        .map_err(|error| PyValueError::new_err(error.to_string()))?;
    value["failure_cause"] = json!(cause);
    value["failure_code"] = json!(code);
    Ok((value.to_string(), report.artifact.map(NativeArtifact::from)))
}

pub fn diff_value(report: &ProjectDiffReport) -> PyResult<Value> {
    let mut value =
        serde_json::to_value(report).map_err(|error| PyValueError::new_err(error.to_string()))?;
    value["diagnostics"] = json!(report
        .diagnostics
        .iter()
        .map(DiagnosticJson::from)
        .collect::<Vec<_>>());
    Ok(value)
}

pub fn diff(result: Result<ProjectDiffReport, ProjectDiffError>) -> PyResult<String> {
    let (report, cause) = match result {
        Ok(report) => (report, None),
        Err(error) => (*error.report, Some(failure_cause(error.cause))),
    };
    let mut value = diff_value(&report)?;
    value["failure_cause"] = json!(cause);
    Ok(value.to_string())
}
