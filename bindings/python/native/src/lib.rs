mod artifacts;
mod authoring;
mod deck;
mod media;
mod observations;
mod options;
mod reports;
mod state;

use anki_forge::build::json_report::DiagnosticJson;
use anki_forge::prelude::{Field, Project, Template};
use artifacts::NativeArtifact;
use media::NativeMediaRef;
use pyo3::exceptions::{PyException, PyValueError};
use pyo3::prelude::*;
use serde_json::{json, Value};
use state::ObjectState;
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
    state: ObjectState<Project>,
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
            state: ObjectState::new(project, "PROJECT"),
        }
    }

    fn add_notetype(&self, py: Python<'_>, note_type: &str) -> PyResult<()> {
        let input: authoring::NoteTypeInput = serde_json::from_str(note_type)
            .map_err(|error| PyValueError::new_err(error.to_string()))?;
        self.state.run(py, |project| {
            project
                .add_notetype(input.into_notetype())
                .map_err(authoring::add_error)?;
            Ok(())
        })
    }

    fn validate(&self, py: Python<'_>) -> PyResult<String> {
        self.state.run(py, |project| {
            let report = project.validate();
            let diagnostics: Vec<_> = report
                .diagnostics
                .iter()
                .map(DiagnosticJson::from)
                .collect();
            Ok(json!(diagnostics).to_string())
        })
    }

    fn import_template_bundle(&self, py: Python<'_>, path: PathBuf) -> PyResult<()> {
        self.state.run(py, |project| {
            project.import_template_bundle(path).map_err(|error| {
                domain_error(
                    "bundle",
                    error.code(),
                    &error.to_string(),
                    json!({"path": error.path(), "byte_offset": error.byte_offset()}),
                )
            })?;
            Ok(())
        })
    }

    fn notes(&self, py: Python<'_>) -> PyResult<String> {
        self.state.run(py, |project| {
            Ok(json!(project
                .authoring_view()
                .notes
                .iter()
                .map(observations::note)
                .collect::<Vec<_>>())
            .to_string())
        })
    }

    fn notetypes(&self, py: Python<'_>) -> PyResult<String> {
        self.state.run(py, |project| {
            Ok(json!(project
                .authoring_view()
                .note_types
                .iter()
                .map(observations::notetype)
                .collect::<Vec<_>>())
            .to_string())
        })
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
                .add_file(&path)
                .and_then(|pending| pending.export_as(export_as))
                .map_err(|error| media::file_error(error, &path))?;
            Ok(NativeMediaRef { inner })
        })
    }

    fn add_media_bytes(
        &self,
        py: Python<'_>,
        source_label: String,
        data: Vec<u8>,
        export_as: String,
    ) -> PyResult<NativeMediaRef> {
        self.state.run(py, |project| {
            let inner = project
                .media_mut()
                .add_bytes(source_label, data)
                .and_then(|pending| pending.export_as(export_as))
                .map_err(media::media_error)?;
            Ok(NativeMediaRef { inner })
        })
    }

    fn build(&self, py: Python<'_>, options: &str) -> PyResult<(String, Option<NativeArtifact>)> {
        let options = serde_json::from_str::<options::BuildInput>(options)
            .map_err(|error| PyValueError::new_err(error.to_string()))?
            .options();
        self.state
            .run(py, |project| reports::build(project.build(options)))
    }

    fn diff_against_apkg(&self, py: Python<'_>, path: PathBuf, limits: &str) -> PyResult<String> {
        let limits = serde_json::from_str::<options::InspectInput>(limits)
            .map_err(|error| PyValueError::new_err(error.to_string()))?
            .limits();
        self.state.run(py, |project| {
            reports::diff(project.diff_against_apkg_with_limits(path, limits))
        })
    }
}

#[pyfunction]
fn default_inspect_limits() -> String {
    options::default_limits().to_string()
}

#[pyfunction]
fn inline_media_limit_bytes() -> usize {
    anki_forge::product::MediaRegistry::inline_limit_bytes()
}

#[pyfunction]
fn build_image_occlusion(input: &str, image: Py<NativeMediaRef>) -> PyResult<String> {
    let input: authoring::ImageOcclusionInput =
        serde_json::from_str(input).map_err(|error| PyValueError::new_err(error.to_string()))?;
    let note = input.into_note(image.get().inner.clone())?;
    Ok(observations::note(&note).to_string())
}

#[pyfunction]
fn render_content(content: &str, references: Vec<Py<NativeMediaRef>>) -> PyResult<String> {
    let input: authoring::ContentInput =
        serde_json::from_str(content).map_err(|error| PyValueError::new_err(error.to_string()))?;
    let media = references
        .into_iter()
        .map(|reference| {
            let inner = &reference.get().inner;
            (inner.filename().to_string(), inner.clone())
        })
        .collect();
    Ok(input.into_content(&media)?.render())
}

#[pyfunction]
fn default_field_key(name: String) -> String {
    Field::new(name).key_ref().as_str().to_string()
}

#[pyfunction]
fn default_template_key(name: String) -> String {
    Template::new(name).key_ref().as_str().to_string()
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
    module.add_class::<deck::NativeDeck>()?;
    module.add_class::<deck::NativeDeckMediaRef>()?;
    module.add_class::<NativeArtifact>()?;
    module.add_class::<NativeMediaRef>()?;
    module.add("OperationError", module.py().get_type::<OperationError>())?;
    module.add_function(wrap_pyfunction!(binding_metadata, module)?)?;
    module.add_function(wrap_pyfunction!(default_field_key, module)?)?;
    module.add_function(wrap_pyfunction!(default_template_key, module)?)?;
    module.add_function(wrap_pyfunction!(render_content, module)?)?;
    module.add_function(wrap_pyfunction!(build_image_occlusion, module)?)?;
    module.add_function(wrap_pyfunction!(inline_media_limit_bytes, module)?)?;
    module.add_function(wrap_pyfunction!(default_inspect_limits, module)?)?;
    Ok(())
}
