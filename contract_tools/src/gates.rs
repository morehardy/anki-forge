use std::path::Path;

use crate::{
    compat_oracle, fixtures, manifest::load_manifest, policies, registry, schema, semantics,
    versioning,
};

pub fn run_all(manifest_path: impl AsRef<Path>) -> anyhow::Result<()> {
    let manifest_path = manifest_path.as_ref();

    load_manifest(manifest_path)?;
    schema::run_schema_gates(manifest_path)?;
    semantics::run_semantics_gates(manifest_path)?;
    policies::run_policy_gates(manifest_path)?;
    registry::run_registry_gates(manifest_path)?;
    fixtures::run_fixture_gates(manifest_path)?;
    compat_oracle::run_compat_oracle_gates(manifest_path)?;
    versioning::run_versioning_gates(manifest_path)?;

    Ok(())
}
