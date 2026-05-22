pub mod baseline;
pub mod current;
pub mod diagnostics;
pub mod lockfile;
pub mod merge_safety;
pub mod model;
pub mod reconcile;
pub mod report;

pub use diagnostics::{
    classify_project_stable_id_missing, EvidenceCondition, UpdateDiagnosticClass,
};
pub use model::{EffectiveMode, ModeSelectionError};
pub use model::{effective_mode, validate_writer_policy_ref};
