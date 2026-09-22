use anki_forge::prelude::MediaRef;
use pyo3::prelude::*;

#[pyclass(frozen, module = "anki_forge._native")]
pub struct NativeMediaRef {
    pub inner: MediaRef,
}

#[pymethods]
impl NativeMediaRef {
    #[getter]
    fn filename(&self) -> &str {
        self.inner.filename()
    }
}

pub fn media_error(error: anyhow::Error) -> PyErr {
    let code = error
        .downcast_ref::<anki_forge::deck::MediaError>()
        .map(|error| error.code().to_string())
        .unwrap_or_else(|| "BINDING.MEDIA_FAILED".to_string());
    crate::domain_error("media", &code, &error.to_string(), serde_json::Value::Null)
}
