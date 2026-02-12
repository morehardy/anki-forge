#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CollisionPolicy {
    Error,
    KeepFirst,
    Regenerate,
}
