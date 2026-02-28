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

    #[must_use]
    pub fn error(path: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::new(path, reason, Severity::Error)
    }

    #[must_use]
    pub fn warning(path: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::new(path, reason, Severity::Warning)
    }

    #[must_use]
    pub fn info(path: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::new(path, reason, Severity::Info)
    }

    #[must_use]
    pub const fn is_error(&self) -> bool {
        matches!(self.severity, Severity::Error)
    }
}
