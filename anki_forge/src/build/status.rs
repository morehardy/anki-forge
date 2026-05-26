use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuildStatus {
    #[default]
    Success,
    Blocked,
    Invalid,
    Error,
}

impl BuildStatus {
    pub fn highest<I>(statuses: I) -> Self
    where
        I: IntoIterator<Item = BuildStatus>,
    {
        statuses.into_iter().max().unwrap_or(BuildStatus::Success)
    }

    pub fn is_success(self) -> bool {
        matches!(self, BuildStatus::Success)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonStatus {
    #[default]
    NotRequested,
    Complete,
    Partial,
    Unavailable,
}
