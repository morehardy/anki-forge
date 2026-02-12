#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ValidationConfig {
    #[default]
    Strict,
    Permissive,
}
