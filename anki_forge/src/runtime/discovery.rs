use std::path::{Path, PathBuf};

use anyhow::{bail, Context};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeMode {
    Workspace,
    Installed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedRuntime {
    pub mode: RuntimeMode,
    pub manifest_path: PathBuf,
    pub bundle_root: PathBuf,
    pub bundle_version: String,
}

pub fn discover_workspace_runtime(start: impl AsRef<Path>) -> anyhow::Result<ResolvedRuntime> {
    let start = start.as_ref();
    let start = start
        .canonicalize()
        .with_context(|| format!("resolve workspace start path: {}", start.display()))?;

    let mut current = if start.is_dir() {
        start
    } else {
        start
            .parent()
            .unwrap_or_else(|| Path::new("/"))
            .to_path_buf()
    };

    loop {
        let manifest_path = current.join("contracts/manifest.yaml");
        if manifest_path.is_file() {
            return super::assets::load_bundle_from_manifest(manifest_path)
                .map(|bundle| bundle.runtime);
        }

        if !current.pop() {
            break;
        }
    }

    bail!("failed to discover contracts/manifest.yaml from workspace path")
}
