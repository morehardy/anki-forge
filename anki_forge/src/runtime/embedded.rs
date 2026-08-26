use std::{io::Cursor, sync::OnceLock};

use anyhow::{ensure, Context};
use flate2::read::GzDecoder;
use tempfile::TempDir;

use super::{load_bundle_from_manifest, RuntimeBundle, RuntimeMode};

const EMBEDDED_BUNDLE_VERSION: &str = "0.3.0";
const EMBEDDED_BUNDLE: &[u8] =
    include_bytes!("../../assets/contracts/anki-forge-contract-bundle-0.3.0.tar.gz");

struct EmbeddedRuntime {
    _extraction_dir: TempDir,
    bundle: RuntimeBundle,
}

static EMBEDDED_RUNTIME: OnceLock<Result<EmbeddedRuntime, String>> = OnceLock::new();

/// Returns the compatibility version of the contract bundle shipped in this crate.
pub const fn embedded_bundle_version() -> &'static str {
    EMBEDDED_BUNDLE_VERSION
}

/// Loads the self-contained contract bundle shipped in this crate.
pub fn load_embedded_bundle() -> anyhow::Result<RuntimeBundle> {
    let runtime =
        EMBEDDED_RUNTIME.get_or_init(|| materialize_embedded_runtime().map_err(|e| e.to_string()));
    match runtime {
        Ok(runtime) => Ok(runtime.bundle.clone()),
        Err(message) => anyhow::bail!("failed to load embedded contract bundle: {message}"),
    }
}

fn materialize_embedded_runtime() -> anyhow::Result<EmbeddedRuntime> {
    let extraction_dir =
        tempfile::tempdir().context("create embedded contract extraction directory")?;
    let decoder = GzDecoder::new(Cursor::new(EMBEDDED_BUNDLE));
    let mut archive = tar::Archive::new(decoder);
    archive
        .unpack(extraction_dir.path())
        .context("extract embedded contract bundle")?;

    let manifest_path = extraction_dir.path().join("contracts/manifest.yaml");
    let mut bundle =
        load_bundle_from_manifest(&manifest_path).context("validate embedded contract bundle")?;
    ensure!(
        bundle.runtime.bundle_version == EMBEDDED_BUNDLE_VERSION,
        "embedded contract bundle version mismatch: expected {}, got {}",
        EMBEDDED_BUNDLE_VERSION,
        bundle.runtime.bundle_version
    );
    bundle.runtime.mode = RuntimeMode::Installed;

    Ok(EmbeddedRuntime {
        _extraction_dir: extraction_dir,
        bundle,
    })
}
