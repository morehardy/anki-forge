#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MediaConflictPolicy {
    Error,
    KeepFirst,
    Rename,
}
