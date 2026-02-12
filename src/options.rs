#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BuildMode {
    #[default]
    Standard,
    Deterministic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ValidationMode {
    #[default]
    Strict,
    Permissive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BuildOptions {
    pub mode: BuildMode,
    pub validation_mode: ValidationMode,
}

impl BuildOptions {
    #[must_use]
    pub const fn new(mode: BuildMode, validation_mode: ValidationMode) -> Self {
        Self {
            mode,
            validation_mode,
        }
    }
}
