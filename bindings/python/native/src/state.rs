use anki_forge::prelude::Project;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Mutex;

enum State {
    Ready(Box<Project>),
    Busy,
    Failed,
}

pub struct ProjectState {
    state: Mutex<State>,
    pid: u32,
}

impl ProjectState {
    pub fn new(project: Project) -> Self {
        Self {
            state: Mutex::new(State::Ready(Box::new(project))),
            pid: std::process::id(),
        }
    }

    pub fn run<T: Send>(
        &self,
        py: Python<'_>,
        operation: impl FnOnce(&mut Project) -> PyResult<T> + Send,
    ) -> PyResult<T> {
        // Never acquire a potentially inherited mutex in a forked child.
        if self.pid != std::process::id() {
            return Err(PyRuntimeError::new_err("BINDING.FORKED_OBJECT"));
        }
        let mut state = self
            .state
            .try_lock()
            .map_err(|_| PyRuntimeError::new_err("BINDING.PROJECT_BUSY"))?;
        match &*state {
            State::Busy => return Err(PyRuntimeError::new_err("BINDING.PROJECT_BUSY")),
            State::Failed => return Err(PyRuntimeError::new_err("BINDING.PROJECT_FAILED")),
            State::Ready(_) => {}
        }
        let State::Ready(mut project) = std::mem::replace(&mut *state, State::Busy) else {
            unreachable!()
        };
        drop(state);
        py.detach(|| {
            let result = catch_unwind(AssertUnwindSafe(|| operation(&mut project)));
            let mut state = self
                .state
                .lock()
                .map_err(|_| PyRuntimeError::new_err("BINDING.PROJECT_FAILED"))?;
            match result {
                Ok(value) => {
                    *state = State::Ready(project);
                    value
                }
                Err(_) => {
                    *state = State::Failed;
                    Err(PyRuntimeError::new_err("BINDING.PROJECT_FAILED"))
                }
            }
        })
    }
}
