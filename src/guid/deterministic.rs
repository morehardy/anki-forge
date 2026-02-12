use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::guid::{GuidInput, GuidStrategy};

#[derive(Debug, Default)]
pub struct DeterministicGuidStrategy;

impl GuidStrategy for DeterministicGuidStrategy {
    fn next_guid(&mut self, input: &GuidInput) -> String {
        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        format!("d{:x}", hasher.finish())
    }
}
