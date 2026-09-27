//! A native value belongs to the process that created its ownership accounting.
use pyo3::{exceptions::PyRuntimeError, PyResult};

pub struct ProcessOwned<T> {
    value: Option<T>,
    pid: u32,
}
impl<T> From<T> for ProcessOwned<T> {
    fn from(value: T) -> Self {
        Self {
            value: Some(value),
            pid: std::process::id(),
        }
    }
}
impl<T> ProcessOwned<T> {
    pub fn get(&self) -> PyResult<&T> {
        if self.pid != std::process::id() {
            return Err(PyRuntimeError::new_err("BINDING.FORKED_OBJECT"));
        }
        Ok(self.value.as_ref().expect("owned value exists until drop"))
    }
}
impl<T> Drop for ProcessOwned<T> {
    fn drop(&mut self) {
        if self.pid != std::process::id() {
            // Forget only the forked copy: it does not own the parent's
            // temporary files or inherited lock/Arc accounting.
            if let Some(value) = self.value.take() {
                std::mem::forget(value);
            }
        }
    }
}
