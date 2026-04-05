use anyhow::Result;
use sha1::{Digest, Sha1};

use crate::{to_canonical_json, BuildContext};

pub fn policy_ref(id: &str, version: &str) -> String {
    format!("{id}@{version}")
}

pub fn build_context_ref(context: &BuildContext) -> Result<String> {
    let canonical = to_canonical_json(context)?;
    let digest = Sha1::digest(canonical.as_bytes());
    Ok(format!("build-context:{}", hex::encode(digest)))
}
