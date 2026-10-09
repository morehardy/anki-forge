//! Default configuration models shared with the Cargo build script.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriterPolicy {
    pub id: String,
    pub version: String,
    pub compatibility_target: String,
    pub stock_notetype_mode: String,
    pub media_entry_mode: String,
    pub apkg_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildContext {
    pub id: String,
    pub version: String,
    pub emit_apkg: bool,
    pub materialize_staging: bool,
    pub media_resolution_mode: String,
    pub unresolved_asset_behavior: String,
    pub fingerprint_mode: String,
}
