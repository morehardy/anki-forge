use crate::guid::{GuidInput, GuidStrategy};

pub struct CustomGuidStrategy<F> {
    callback: F,
}

impl<F> CustomGuidStrategy<F>
where
    F: FnMut(&GuidInput) -> String,
{
    #[must_use]
    pub fn new(callback: F) -> Self {
        Self { callback }
    }
}

impl<F> GuidStrategy for CustomGuidStrategy<F>
where
    F: FnMut(&GuidInput) -> String,
{
    fn next_guid(&mut self, input: &GuidInput) -> String {
        (self.callback)(input)
    }
}
