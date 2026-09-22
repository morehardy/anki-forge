use anki_forge::prelude::ApkgArtifact;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use std::path::PathBuf;
use std::sync::Mutex;

#[pyclass(frozen, module = "anki_forge._native")]
pub struct NativeArtifact {
    inner: Mutex<Option<ApkgArtifact>>,
}

impl From<ApkgArtifact> for NativeArtifact {
    fn from(artifact: ApkgArtifact) -> Self {
        Self {
            inner: Mutex::new(Some(artifact)),
        }
    }
}

impl NativeArtifact {
    fn snapshot(&self) -> PyResult<ApkgArtifact> {
        self.inner
            .lock()
            .map_err(|_| PyRuntimeError::new_err("BINDING.ARTIFACT_FAILED"))?
            .as_ref()
            .cloned()
            .ok_or_else(|| PyRuntimeError::new_err("BINDING.ARTIFACT_CLOSED"))
    }
}

#[pymethods]
impl NativeArtifact {
    fn clone_handle(&self) -> PyResult<Self> {
        self.snapshot().map(Self::from)
    }

    fn persist_to(&self, py: Python<'_>, path: PathBuf) -> PyResult<Self> {
        // Retain a distinct core owner before releasing the interpreter, so
        // another thread can close this handle without removing the source.
        let artifact = self.snapshot()?;
        py.detach(|| {
            artifact
                .persist_to(path)
                .map(Self::from)
                .map_err(PyErr::from)
        })
    }

    fn path(&self) -> PyResult<PathBuf> {
        let guard = self
            .inner
            .lock()
            .map_err(|_| PyRuntimeError::new_err("BINDING.ARTIFACT_FAILED"))?;
        guard
            .as_ref()
            .map(|artifact| artifact.path().to_path_buf())
            .ok_or_else(|| PyRuntimeError::new_err("BINDING.ARTIFACT_CLOSED"))
    }

    fn close(&self) -> PyResult<()> {
        self.inner
            .lock()
            .map_err(|_| PyRuntimeError::new_err("BINDING.ARTIFACT_FAILED"))?
            .take();
        Ok(())
    }
}
