use crate::guid::{GuidInput, GuidStrategy};

#[derive(Debug, Default)]
pub struct RandomGuidStrategy {
    counter: u64,
}

impl GuidStrategy for RandomGuidStrategy {
    fn next_guid(&mut self, input: &GuidInput) -> String {
        self.counter = self.counter.wrapping_add(1);
        format!("r{}-{}", input.note_index, self.counter)
    }
}
