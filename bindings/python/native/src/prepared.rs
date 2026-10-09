use ankiforge::build::PreparedPublication;
use pyo3::{exceptions::PyRuntimeError, prelude::*};
use std::sync::Mutex;

struct State {
    owner: Option<PreparedPublication>,
    reason: &'static str,
}

#[pyclass(frozen, module = "ankiforge._native")]
pub struct NativePreparedPublication {
    state: Mutex<State>,
    report: String,
    pid: u32,
}

impl From<PreparedPublication> for NativePreparedPublication {
    fn from(owner: PreparedPublication) -> Self {
        Self {
            report: serde_json::json!(owner.report().snapshot()).to_string(),
            state: Mutex::new(State {
                owner: Some(owner),
                reason: "closed",
            }),
            pid: std::process::id(),
        }
    }
}

impl NativePreparedPublication {
    fn check_process(&self) -> PyResult<()> {
        if self.pid != std::process::id() {
            return Err(PyRuntimeError::new_err("BINDING.FORKED_OBJECT"));
        }
        Ok(())
    }
}

#[pymethods]
impl NativePreparedPublication {
    fn report(&self) -> &str {
        &self.report
    }

    fn publish(&self, py: Python<'_>) -> PyResult<(String, crate::artifacts::NativeArtifact)> {
        self.check_process()?;
        let owner = {
            let mut state = self
                .state
                .lock()
                .map_err(|_| PyRuntimeError::new_err("BINDING.PREPARED_FAILED"))?;
            let owner = state.owner.take().ok_or_else(|| {
                crate::domain_error(
                    "prepared",
                    "BUILD.PREPARED_UNAVAILABLE",
                    "prepared publication is unavailable",
                    serde_json::json!({"error_kind":"Unavailable", "reason":state.reason}),
                )
            })?;
            state.reason = "consumed";
            owner
        };
        py.detach(|| {
            owner
                .publish()
                .map(|output| {
                    (
                        serde_json::json!(output.snapshot()).to_string(),
                        crate::artifacts::NativeArtifact::from(output.artifact().clone()),
                    )
                })
                .map_err(|error| crate::core_error("build", error.kind(), error.code(), &error))
        })
    }

    fn close(&self, py: Python<'_>) -> PyResult<()> {
        self.check_process()?;
        let owner = self
            .state
            .lock()
            .map_err(|_| PyRuntimeError::new_err("BINDING.PREPARED_FAILED"))?
            .owner
            .take();
        py.detach(|| drop(owner));
        Ok(())
    }
}

impl Drop for NativePreparedPublication {
    fn drop(&mut self) {
        if self.pid != std::process::id() {
            // Never wait on an inherited lock or release a parent's candidate.
            if let Some(owner) = self
                .state
                .get_mut()
                .unwrap_or_else(|e| e.into_inner())
                .owner
                .take()
            {
                std::mem::forget(owner);
            }
        }
    }
}
