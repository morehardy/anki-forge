#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub path: String,
    pub reason: String,
    pub severity: Severity,
}

impl Diagnostic {
    #[must_use]
    pub fn new(path: impl Into<String>, reason: impl Into<String>, severity: Severity) -> Self {
        Self {
            path: path.into(),
            reason: reason.into(),
            severity,
        }
    }
}
