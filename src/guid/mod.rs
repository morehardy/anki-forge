pub mod collision;
pub mod custom;
pub mod deterministic;
pub mod random;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct GuidInput {
    pub note_index: usize,
    pub template_ord: u16,
}

pub trait GuidStrategy {
    fn next_guid(&mut self, input: &GuidInput) -> String;
}
