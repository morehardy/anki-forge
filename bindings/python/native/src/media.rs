use anki_forge::prelude::MediaRef;
use pyo3::prelude::*;
use serde_json::json;
use std::path::Path;

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
    media_error_with_path(error, None)
}

pub fn file_error(error: anyhow::Error, path: &Path) -> PyErr {
    media_error_with_path(error, Some(path))
}

fn media_error_with_path(error: anyhow::Error, path: Option<&Path>) -> PyErr {
    use anki_forge::deck::MediaError;
    let code = error
        .downcast_ref::<MediaError>()
        .map(|error| error.code().to_string())
        .unwrap_or_else(|| "BINDING.MEDIA_FAILED".to_string());
    let mut details = match error.downcast_ref::<MediaError>() {
        Some(
            MediaError::SourceMissing { path, .. }
            | MediaError::SourceNotRegularFile { path }
            | MediaError::SourceReadFailed { path, .. },
        ) => json!({"path": path}),
        Some(MediaError::SourceEmpty { label } | MediaError::InvalidSourceLabel { label, .. }) => {
            json!({"source_label": label})
        }
        Some(MediaError::InlineTooLarge {
            label,
            size_bytes,
            limit_bytes,
        }) => json!({"source_label": label, "size_bytes": size_bytes, "limit_bytes": limit_bytes}),
        Some(MediaError::UnsafeFilename { name, .. } | MediaError::ConflictingPayload { name }) => {
            json!({"filename": name})
        }
        _ => json!({}),
    };
    if let Some(path) = path {
        details["path"] = json!(path);
    }
    crate::domain_error("media", &code, &error.to_string(), details)
}
