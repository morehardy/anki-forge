mod artifacts;
mod authoring;
mod options;
mod owned;
mod state;

use ankiforge::{Content, Media, Note, NoteType, Project};
use artifacts::NativeArtifact;
use owned::ProcessOwned;
use pyo3::exceptions::{PyException, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use serde_json::{json, Value};
use state::ObjectState;
use std::{error::Error, path::PathBuf};

pyo3::create_exception!(_native, OperationError, PyException);

pub fn domain_error(kind: &str, code: &str, message: &str, details: Value) -> PyErr {
    OperationError::new_err(
        json!({"kind":kind,"code":code,"message":message,"details":details}).to_string(),
    )
}
pub fn error_details(kind: impl std::fmt::Debug, error: &(dyn Error + 'static)) -> Value {
    let mut details = json!({"error_kind":format!("{kind:?}"),"causes":error_causes(error),"source_details": source_details(error)});
    if let Some(error) = error.downcast_ref::<ankiforge::media::MediaError>() {
        details["path"] = json!(error.path());
        details["limit_exceeded"] = error.limit_exceeded().map_or(
            Value::Null,
            |l| json!({"resource":l.resource,"limit":l.limit,"observed":l.observed}),
        );
    }
    if let Some(error) = error.downcast_ref::<ankiforge::schema::SchemaError>() {
        details["location"] = error.location().map_or(Value::Null, |l|json!({"template":l.template.as_str(),"side":format!("{:?}",l.side),"byte_range":{"start":l.byte_range.start,"end":l.byte_range.end}}));
    }
    if let Some(error) = error.downcast_ref::<ankiforge::update::CompareError>() {
        details["report"] = json!(error.report().snapshot());
        details["limit_exceeded"] = inspect_limit(error.limit_exceeded());
    }
    if let Some(error) = error.downcast_ref::<ankiforge::build::BuildError>() {
        details["snapshot"] = json!(error.snapshot());
        details["limit_exceeded"] = inspect_limit(error.limit_exceeded());
    }
    if let Some(error) = error.downcast_ref::<ankiforge::build::PersistError>() {
        details["publication"] = json!(error.publication());
    }
    if let Some(error) = error.downcast_ref::<ankiforge::schema::TemplateBundleError>() {
        details["path"] = json!(error.path());
        details["byte_offset"] = json!(error.byte_offset());
    }
    details
}
fn source_details(error: &(dyn Error + 'static)) -> Vec<Value> {
    let mut details = Vec::new();
    let mut source = error.source();
    while let Some(cause) = source {
        let value = if let Some(e) = cause.downcast_ref::<ankiforge::media::MediaError>() {
            json!({"type":"media", "kind":format!("{:?}",e.kind()), "code":e.code(), "path":e.path(), "limit_exceeded":e.limit_exceeded().map(|l|json!({"resource":l.resource,"limit":l.limit,"observed":l.observed}))})
        } else if let Some(e) =
            cause.downcast_ref::<ankiforge::schema::TemplateBundleLimitExceeded>()
        {
            json!({"type":"bundle_limit", "limit":e.limit, "observed":e.observed})
        } else if let Some(e) = cause.downcast_ref::<ankiforge::schema::SchemaError>() {
            json!({"type":"schema", "kind":format!("{:?}",e.kind()), "code":e.code(), "location":e.location().map(|l|json!({"template":l.template.as_str(),"side":format!("{:?}",l.side),"byte_range":{"start":l.byte_range.start,"end":l.byte_range.end}}))})
        } else if let Some(e) = cause.downcast_ref::<ankiforge::build::InspectLimitExceeded>() {
            json!({"type":"inspect_limit", "resource":e.resource,"entry":e.entry,"limit":e.limit,"observed":e.observed})
        } else if let Some(e) = cause.downcast_ref::<std::io::Error>() {
            json!({"type":"io", "kind":format!("{:?}",e.kind()), "os_code":e.raw_os_error()})
        } else {
            json!({"type":"source", "message":cause.to_string()})
        };
        details.push(value);
        source = cause.source();
    }
    details
}
fn inspect_limit(limit: Option<&ankiforge::build::InspectLimitExceeded>) -> Value {
    limit.map_or(
        Value::Null,
        |l| json!({"resource":l.resource,"entry":l.entry,"limit":l.limit,"observed":l.observed}),
    )
}
pub fn core_error(
    domain: &str,
    kind: impl std::fmt::Debug,
    code: &str,
    error: &(dyn Error + 'static),
) -> PyErr {
    let result = domain_error(domain, code, &error.to_string(), error_details(kind, error));
    let mut source = error.source();
    while let Some(cause) = source {
        if let Some(io) = cause.downcast_ref::<std::io::Error>() {
            let cause = io.raw_os_error().map_or_else(
                || pyo3::exceptions::PyOSError::new_err(io.to_string()),
                |code| PyErr::from(std::io::Error::from_raw_os_error(code)),
            );
            Python::attach(|py| result.set_cause(py, Some(cause)));
            break;
        }
        source = cause.source();
    }
    result
}

macro_rules! mapped {
    ($domain:literal) => {
        |e| core_error($domain, e.kind(), e.code(), &e)
    };
}
pub(crate) use mapped;

#[pyclass(frozen, skip_from_py_object, module = "anki_forge._native")]
struct NativeContent {
    inner: ProcessOwned<Content>,
}
#[pymethods]
impl NativeContent {
    #[staticmethod]
    fn text(value: String) -> Self {
        Self {
            inner: Content::text(value).into(),
        }
    }
    #[staticmethod]
    fn html(value: String) -> Self {
        Self {
            inner: Content::html(value).into(),
        }
    }
    #[staticmethod]
    fn sequence(values: Vec<Py<NativeContent>>) -> PyResult<Self> {
        let contents = values
            .iter()
            .map(|v| Ok(v.get().inner.get()?.clone()))
            .collect::<PyResult<Vec<_>>>()?;
        Ok(Self {
            inner: Content::sequence(contents).into(),
        })
    }
}
#[pyclass(frozen, skip_from_py_object, module = "anki_forge._native")]
struct NativeMedia {
    inner: ProcessOwned<Media>,
}
#[pymethods]
impl NativeMedia {
    #[staticmethod]
    fn file(py: Python<'_>, path: PathBuf, max_bytes: u64) -> PyResult<Self> {
        py.detach(|| {
            Media::file_with_limits(path, ankiforge::media::MediaLimits { max_bytes })
                .map(|inner| Self {
                    inner: inner.into(),
                })
                .map_err(mapped!("media"))
        })
    }
    #[staticmethod]
    fn bytes(
        py: Python<'_>,
        data: &Bound<'_, PyBytes>,
        media_type: String,
        max_bytes: u64,
    ) -> PyResult<Self> {
        let data = data.as_bytes();
        let observed = data.len() as u64;
        if observed > max_bytes {
            // Borrow Python's immutable storage so rejected input is never cloned.
            let code = "MEDIA.RESOURCE_LIMIT_EXCEEDED";
            return Err(domain_error(
                "media",
                code,
                &format!("{code}: media contains at least {observed} bytes; limit is {max_bytes}"),
                json!({
                    "error_kind": "ResourceLimit",
                    "causes": [],
                    "source_details": [],
                    "path": null,
                    "limit_exceeded": {
                        "resource": "media_bytes", "limit": max_bytes, "observed": observed
                    }
                }),
            ));
        }
        // PyBytes stays alive and immutable while copying and snapshotting without the GIL.
        py.detach(|| {
            Media::bytes_with_limits(
                data.to_vec(),
                media_type,
                ankiforge::media::MediaLimits { max_bytes },
            )
            .map(|inner| Self {
                inner: inner.into(),
            })
            .map_err(mapped!("media"))
        })
    }
    fn with_export_name(&self, name: String) -> PyResult<Self> {
        self.inner
            .get()?
            .clone()
            .with_export_name(name)
            .map(|inner| Self {
                inner: inner.into(),
            })
            .map_err(mapped!("media"))
    }
    fn filename(&self) -> PyResult<&str> {
        Ok(self.inner.get()?.filename())
    }
    fn media_type(&self) -> PyResult<&str> {
        Ok(self.inner.get()?.media_type())
    }
    fn size(&self) -> PyResult<u64> {
        Ok(self.inner.get()?.len())
    }
    fn image(&self) -> PyResult<NativeContent> {
        Ok(NativeContent {
            inner: self.inner.get()?.image().into(),
        })
    }
    fn sound(&self) -> PyResult<NativeContent> {
        Ok(NativeContent {
            inner: self.inner.get()?.sound().into(),
        })
    }
}
#[pyclass(frozen, skip_from_py_object, module = "anki_forge._native")]
struct NativeNoteType {
    inner: ProcessOwned<NoteType>,
}
#[pymethods]
impl NativeNoteType {
    #[staticmethod]
    fn build(input: &str, assets: Vec<Py<NativeMedia>>) -> PyResult<Self> {
        let input: authoring::ModelInput = parse(input)?;
        input.build(assets).map(|inner| Self {
            inner: inner.into(),
        })
    }
    #[staticmethod]
    fn from_bundle(py: Python<'_>, path: PathBuf, max_bytes: u64) -> PyResult<Self> {
        py.detach(|| {
            NoteType::from_bundle_with_limits(path, ankiforge::media::MediaLimits { max_bytes })
                .map(|inner| Self {
                    inner: inner.into(),
                })
                .map_err(mapped!("bundle"))
        })
    }
    fn key(&self) -> PyResult<&str> {
        Ok(self.inner.get()?.key())
    }
    fn display_name(&self) -> PyResult<&str> {
        Ok(self.inner.get()?.display_name())
    }
    fn note(&self) -> PyResult<NativeNote> {
        Ok(NativeNote {
            inner: self.inner.get()?.note().into(),
        })
    }
}
fn error_causes(error: &dyn Error) -> Vec<String> {
    let mut out = vec![];
    let mut next = error.source();
    while let Some(e) = next {
        out.push(e.to_string());
        next = e.source();
    }
    out
}
#[pyclass(frozen, skip_from_py_object, module = "anki_forge._native")]
struct NativeNote {
    inner: ProcessOwned<Note>,
}
#[pymethods]
impl NativeNote {
    #[staticmethod]
    fn basic(front: &NativeContent, back: &NativeContent) -> PyResult<Self> {
        Ok(Self {
            inner: Note::basic(front.inner.get()?.clone(), back.inner.get()?.clone()).into(),
        })
    }
    #[staticmethod]
    fn cloze(text: &NativeContent) -> PyResult<Self> {
        Ok(Self {
            inner: Note::cloze(text.inner.get()?.clone()).into(),
        })
    }
    #[staticmethod]
    fn image_occlusion(
        py: Python<'_>,
        image: &NativeMedia,
        masks: Vec<(String, f64, f64, f64, f64)>,
        mode: &str,
    ) -> PyResult<Self> {
        let mode = match mode {
            "hide_all_guess_one" => ankiforge::note::OcclusionMode::HideAllGuessOne,
            "hide_one_guess_one" => ankiforge::note::OcclusionMode::HideOneGuessOne,
            _ => return Err(PyValueError::new_err("unknown occlusion mode")),
        };
        let image = image.inner.get()?.clone();
        py.detach(|| {
            let mut builder = Note::image_occlusion(image).mode(mode);
            for (key, x, y, width, height) in masks {
                builder = builder.mask(ankiforge::note::Mask::rect(key, x, y, width, height));
            }
            builder
                .build()
                .map(|inner| Self {
                    inner: inner.into(),
                })
                .map_err(mapped!("note"))
        })
    }
    fn field(&self, key: String, value: &NativeContent) -> PyResult<Self> {
        Ok(Self {
            inner: self
                .inner
                .get()?
                .clone()
                .field(key, value.inner.get()?.clone())
                .into(),
        })
    }
    fn deck(&self, name: String) -> PyResult<Self> {
        Ok(Self {
            inner: self.inner.get()?.clone().deck(name).into(),
        })
    }
    fn tags(&self, tags: Vec<String>) -> PyResult<Self> {
        Ok(Self {
            inner: self.inner.get()?.clone().tags(tags).into(),
        })
    }
    fn note_type(&self) -> PyResult<NativeNoteType> {
        Ok(NativeNoteType {
            inner: self.inner.get()?.note_type().clone().into(),
        })
    }
}
#[pyclass(frozen, module = "anki_forge._native")]
struct NativeProject {
    state: ObjectState<Project>,
}
#[pymethods]
impl NativeProject {
    #[new]
    #[pyo3(signature=(namespace, name=None, default_deck=None))]
    fn new(
        namespace: String,
        name: Option<String>,
        default_deck: Option<String>,
    ) -> PyResult<Self> {
        let mut project = Project::new(namespace).map_err(mapped!("schema"))?;
        if let Some(name) = name {
            project = project.name(name);
        }
        if let Some(deck) = default_deck {
            project = project.default_deck(deck);
        }
        Ok(Self {
            state: ObjectState::new(project, "PROJECT"),
        })
    }
    fn add(&self, py: Python<'_>, key: String, note: &NativeNote) -> PyResult<()> {
        let note = note.inner.get()?.clone();
        self.state
            .run(py, |p| p.add(key, note).map_err(mapped!("add")))
    }
    fn add_asset(&self, py: Python<'_>, media: &NativeMedia) -> PyResult<()> {
        let media = media.inner.get()?.clone();
        self.state
            .run(py, |p| p.add_asset(media).map_err(mapped!("add")))
    }
    fn len(&self, py: Python<'_>) -> PyResult<usize> {
        self.state.run(py, |p| Ok(p.len()))
    }
    fn build(&self, py: Python<'_>, input: &str) -> PyResult<(String, NativeArtifact)> {
        let request = parse::<options::BuildInput>(input)?.options()?;
        self.state.run(py, |p| match p.build(request) {
            Ok(output) => Ok((
                json!(output.snapshot()).to_string(),
                NativeArtifact::from(output.artifact().clone()),
            )),
            Err(e) => Err(core_error("build", e.kind(), e.code(), &e)),
        })
    }
    fn compare(&self, py: Python<'_>, input: &str) -> PyResult<String> {
        let request = parse::<options::CompareInput>(input)?.options()?;
        self.state.run(py, |p| {
            p.compare(request)
                .map(|r| json!(r.snapshot()).to_string())
                .map_err(mapped!("compare"))
        })
    }
}
fn parse<T: serde::de::DeserializeOwned>(input: &str) -> PyResult<T> {
    serde_json::from_str(input).map_err(|e| PyValueError::new_err(e.to_string()))
}
#[pyfunction]
fn binding_metadata() -> String {
    json!({"binding_version":env!("CARGO_PKG_VERSION"),"core_version":ankiforge::facade_api_version(),"contract_version":ankiforge::embedded_contract_version()}).to_string()
}
#[pyfunction]
fn validate_risk_code(code: &str) -> PyResult<()> {
    code.parse::<ankiforge::update::RiskCode>()
        .map(|_| ())
        .map_err(mapped!("policy"))
}
#[pymodule(gil_used = true)]
fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<NativeProject>()?;
    module.add_class::<NativeMedia>()?;
    module.add_class::<NativeContent>()?;
    module.add_class::<NativeNote>()?;
    module.add_class::<NativeNoteType>()?;
    module.add_class::<NativeArtifact>()?;
    module.add("OperationError", module.py().get_type::<OperationError>())?;
    module.add_function(wrap_pyfunction!(binding_metadata, module)?)?;
    module.add_function(wrap_pyfunction!(validate_risk_code, module)?)?;
    Ok(())
}
