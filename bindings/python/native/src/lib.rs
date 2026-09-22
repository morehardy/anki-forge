mod artifacts;
mod authoring;
mod media;
mod state;

use anki_forge::build::BuildReportJson;
use anki_forge::prelude::{BuildOptions, Project};
use artifacts::NativeArtifact;
use media::NativeMediaRef;
use pyo3::exceptions::{PyException, PyValueError};
use pyo3::prelude::*;
use serde_json::{json, Value};
use state::ProjectState;
use std::path::PathBuf;

pyo3::create_exception!(_native, OperationError, PyException);

pub fn domain_error(kind: &str, code: &str, message: &str, details: Value) -> PyErr {
    OperationError::new_err(
        json!({
            "kind": kind, "code": code, "message": message, "details": details
        })
        .to_string(),
    )
}

#[pyclass(frozen, module = "anki_forge._native")]
struct NativeProject {
    state: ProjectState,
}

#[pymethods]
impl NativeProject {
    #[new]
    #[pyo3(signature = (name, stable_id=None, default_deck=None))]
    fn new(name: String, stable_id: Option<String>, default_deck: Option<String>) -> Self {
        let mut project = Project::new(name);
        if let Some(id) = stable_id {
            project = project.stable_id(id);
        }
        if let Some(deck) = default_deck {
            project = project.default_deck(deck);
        }
        Self {
            state: ProjectState::new(project),
        }
    }

    fn add_note(
        &self,
        py: Python<'_>,
        note: &str,
        references: Vec<Py<NativeMediaRef>>,
    ) -> PyResult<()> {
        let input: authoring::NoteInput =
            serde_json::from_str(note).map_err(|error| PyValueError::new_err(error.to_string()))?;
        let media = references
            .into_iter()
            .map(|reference| {
                let inner = &reference.get().inner;
                (inner.filename().to_string(), inner.clone())
            })
            .collect();
        self.state.run(py, |project| {
            project
                .add_note(input.into_note(&media)?)
                .map_err(authoring::add_error)?;
            Ok(())
        })
    }

    fn add_media_file(
        &self,
        py: Python<'_>,
        path: PathBuf,
        export_as: String,
    ) -> PyResult<NativeMediaRef> {
        self.state.run(py, |project| {
            let inner = project
                .media_mut()
                .add_file(path)
                .and_then(|pending| pending.export_as(export_as))
                .map_err(media::media_error)?;
            Ok(NativeMediaRef { inner })
        })
    }

    #[pyo3(signature = (output=None))]
    fn build(
        &self,
        py: Python<'_>,
        output: Option<PathBuf>,
    ) -> PyResult<(String, Option<NativeArtifact>)> {
        self.state.run(py, |project| {
            let mut options = BuildOptions::new();
            if let Some(path) = output {
                options = options.output(path);
            }
            let (report, cause) = match project.build(options) {
                Ok(report) => (report, None),
                Err(error) => (*error.report, Some(format!("{:?}", error.cause))),
            };
            let mut json = serde_json::to_value(BuildReportJson::from_report(&report))
                .map_err(|error| PyValueError::new_err(error.to_string()))?;
            json["failure_cause"] = json!(cause);
            Ok((json.to_string(), report.artifact.map(NativeArtifact::from)))
        })
    }
}

#[pyfunction]
fn binding_metadata() -> String {
    json!({
        "binding_version": env!("CARGO_PKG_VERSION"),
        "core_version": anki_forge::facade_api_version(),
        "contract_version": anki_forge::embedded_contract_version(),
    })
    .to_string()
}

#[pymodule(gil_used = true)]
fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<NativeProject>()?;
    module.add_class::<NativeArtifact>()?;
    module.add_class::<NativeMediaRef>()?;
    module.add("OperationError", module.py().get_type::<OperationError>())?;
    module.add_function(wrap_pyfunction!(binding_metadata, module)?)?;
    Ok(())
}
