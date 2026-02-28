#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ValidationConfig {
    #[default]
    Strict,
    Permissive,
}

impl ValidationConfig {
    #[must_use]
    pub const fn is_strict(self) -> bool {
        matches!(self, Self::Strict)
    }

    #[must_use]
    pub const fn is_permissive(self) -> bool {
        matches!(self, Self::Permissive)
    }
}

impl From<crate::options::ValidationMode> for ValidationConfig {
    fn from(mode: crate::options::ValidationMode) -> Self {
        match mode {
            crate::options::ValidationMode::Strict => Self::Strict,
            crate::options::ValidationMode::Permissive => Self::Permissive,
        }
    }
}
