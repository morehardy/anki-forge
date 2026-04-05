pub mod canonical_json;
pub mod model;
pub mod policy;

pub use canonical_json::to_canonical_json;
pub use model::{
    BuildContext, BuildDiagnosticItem, BuildDiagnostics, DiffChange, DiffReport,
    InspectObservations, InspectReport, PackageBuildResult, VerificationGateRule,
    VerificationPolicy, WriterPolicy,
};
pub use policy::{build_context_ref, policy_ref};

pub fn tool_contract_version() -> &'static str {
    "phase3-v1"
}
