pub mod baseline;
pub mod current;
pub mod diagnostics;
pub mod lockfile;
pub mod merge_safety;
pub mod model;
pub(crate) mod note_revisions;
pub(crate) mod notetype_ids;
pub mod reconcile;
pub mod report;

pub use diagnostics::{
    classify_project_stable_id_missing, EvidenceCondition, UpdateDiagnosticClass,
};
pub use model::{effective_mode, validate_writer_policy_ref};
pub use model::{EffectiveMode, ModeSelectionError};
