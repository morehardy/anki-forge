use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Mutex;

enum State<T> {
    Ready(Box<T>),
    Busy,
    Failed,
}

pub struct ObjectState<T> {
    state: Mutex<State<T>>,
    kind: &'static str,
    pid: u32,
}

impl<T: Send> ObjectState<T> {
    pub fn new(project: T, kind: &'static str) -> Self {
        Self {
            state: Mutex::new(State::Ready(Box::new(project))),
            pid: std::process::id(),
            kind,
        }
    }

    pub fn run<R: Send>(
        &self,
        py: Python<'_>,
        operation: impl FnOnce(&mut T) -> PyResult<R> + Send,
    ) -> PyResult<R> {
        // Never acquire a potentially inherited mutex in a forked child.
        if self.pid != std::process::id() {
            return Err(PyRuntimeError::new_err("BINDING.FORKED_OBJECT"));
        }
        let mut state = self
            .state
            .try_lock()
            .map_err(|_| PyRuntimeError::new_err(format!("BINDING.{}_BUSY", self.kind)))?;
        match &*state {
            State::Busy => {
                return Err(PyRuntimeError::new_err(format!(
                    "BINDING.{}_BUSY",
                    self.kind
                )))
            }
            State::Failed => {
                return Err(PyRuntimeError::new_err(format!(
                    "BINDING.{}_FAILED",
                    self.kind
                )))
            }
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
                .map_err(|_| PyRuntimeError::new_err(format!("BINDING.{}_FAILED", self.kind)))?;
            match result {
                Ok(value) => {
                    *state = State::Ready(project);
                    value
                }
                Err(_) => {
                    *state = State::Failed;
                    Err(PyRuntimeError::new_err(format!(
                        "BINDING.{}_FAILED",
                        self.kind
                    )))
                }
            }
        })
    }
}
