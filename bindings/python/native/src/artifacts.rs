use ankiforge::build::{ApkgArtifact, PersistError};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use std::path::PathBuf;
use std::sync::Mutex;

#[pyclass(frozen, module = "anki_forge._native")]
pub struct NativeArtifact {
    inner: Mutex<Option<ApkgArtifact>>,
    pid: u32,
}

impl From<ApkgArtifact> for NativeArtifact {
    fn from(artifact: ApkgArtifact) -> Self {
        Self {
            inner: Mutex::new(Some(artifact)),
            pid: std::process::id(),
        }
    }
}

impl NativeArtifact {
    fn check_process(&self) -> PyResult<()> {
        if self.pid != std::process::id() {
            return Err(PyRuntimeError::new_err("BINDING.FORKED_OBJECT"));
        }
        Ok(())
    }

    fn snapshot(&self) -> PyResult<ApkgArtifact> {
        self.check_process()?;
        self.inner
            .lock()
            .map_err(|_| PyRuntimeError::new_err("BINDING.ARTIFACT_FAILED"))?
            .as_ref()
            .cloned()
            .ok_or_else(|| PyRuntimeError::new_err("BINDING.ARTIFACT_CLOSED"))
    }
}

impl Drop for NativeArtifact {
    fn drop(&mut self) {
        if self.pid != std::process::id() {
            // Fork duplicates Arc counts without sharing ownership accounting.
            // Do not run the copied TempDir destructor in the child. get_mut
            // needs no lock, including if a parent thread held it during fork.
            if let Some(artifact) = self
                .inner
                .get_mut()
                .unwrap_or_else(|error| error.into_inner())
                .take()
            {
                std::mem::forget(artifact);
            }
        }
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
                .map_err(persist_error)
        })
    }

    fn path(&self) -> PyResult<PathBuf> {
        self.check_process()?;
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
        self.check_process()?;
        self.inner
            .lock()
            .map_err(|_| PyRuntimeError::new_err("BINDING.ARTIFACT_FAILED"))?
            .take();
        Ok(())
    }
}

fn persist_error(error: PersistError) -> PyErr {
    crate::core_error("persist", error.kind(), error.code(), &error)
}
