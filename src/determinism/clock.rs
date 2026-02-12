use std::time::{Duration, SystemTime};

pub trait Clock {
    fn now(&self) -> SystemTime;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> SystemTime {
        SystemTime::now()
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct FixedClock {
    offset_secs: u64,
}

impl FixedClock {
    #[must_use]
    pub const fn new(offset_secs: u64) -> Self {
        Self { offset_secs }
    }
}

impl Clock for FixedClock {
    fn now(&self) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(self.offset_secs)
    }
}
