use anyhow::{ensure, Context};
use serde::Deserialize;
use std::{
    collections::BTreeSet,
    fs::File,
    io::Read,
    path::{Component, Path, PathBuf},
};

const MANIFEST: &str = "anki-template.yaml";
const MANIFEST_LIMIT: u64 = 256 << 10;

// Deliberately decodes only path declarations. Historical manifests must remain
// hashable for contract governance, without making their authoring APIs usable.
#[derive(Deserialize)]
struct PathDeclarations {
    css_file: Option<String>,
    note_type: NoteTypePaths,
    #[serde(default)]
    assets: Vec<AssetPath>,
}

#[derive(Deserialize)]
struct NoteTypePaths {
    templates: Vec<TemplatePaths>,
}

#[derive(Deserialize)]
struct TemplatePaths {
    front_file: String,
    back_file: String,
    browser_front_file: Option<String>,
    browser_back_file: Option<String>,
}

#[derive(Deserialize)]
struct AssetPath {
    path: String,
}

/// Enumerates declared, root-contained input paths for contract hashing and
/// packaging. This reads current or historical declarations; it does not validate
/// a model or enable authoring with an older template format.
pub fn contract_template_bundle_paths(root: impl AsRef<Path>) -> anyhow::Result<Vec<PathBuf>> {
    let root = root
        .as_ref()
        .canonicalize()
        .context("resolve template bundle root")?;
    let manifest = resolve(&root, MANIFEST)?;
    let mut bytes = Vec::new();
    File::open(&manifest)?
        .take(MANIFEST_LIMIT + 1)
        .read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() as u64 <= MANIFEST_LIMIT,
        "template bundle manifest exceeds {MANIFEST_LIMIT} bytes"
    );
    let declarations: PathDeclarations =
        serde_yaml::from_slice(&bytes).context("decode template bundle path declarations")?;
    let mut paths = BTreeSet::from([PathBuf::from(MANIFEST)]);
    let mut insert = |path: String| -> anyhow::Result<()> {
        resolve(&root, &path)?;
        paths.insert(PathBuf::from(path));
        Ok(())
    };
    if let Some(path) = declarations.css_file {
        insert(path)?;
    }
    for template in declarations.note_type.templates {
        insert(template.front_file)?;
        insert(template.back_file)?;
        if let Some(path) = template.browser_front_file {
            insert(path)?;
        }
        if let Some(path) = template.browser_back_file {
            insert(path)?;
        }
    }
    for asset in declarations.assets {
        insert(asset.path)?;
    }
    Ok(paths.into_iter().collect())
}

fn resolve(root: &Path, relative: &str) -> anyhow::Result<PathBuf> {
    let path = Path::new(relative);
    ensure!(
        !relative.is_empty()
            && !relative.contains(['\\', ':'])
            && !path.is_absolute()
            && !path.components().any(|part| matches!(part, Component::ParentDir | Component::RootDir | Component::Prefix(_))),
        "TEMPLATE.BUNDLE_PATH_UNSAFE: input must be a portable relative path inside the bundle: {relative}"
    );
    let canonical = root.join(path).canonicalize().with_context(|| {
        format!("TEMPLATE.BUNDLE_FILE_INVALID: resolve declared template bundle input: {relative}")
    })?;
    ensure!(
        canonical.starts_with(root),
        "TEMPLATE.BUNDLE_PATH_UNSAFE: input resolves outside the bundle: {relative}"
    );
    ensure!(
        canonical.is_file(),
        "template bundle input is not a regular file: {relative}"
    );
    Ok(canonical)
}
