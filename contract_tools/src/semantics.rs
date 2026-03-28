use anyhow::{ensure, Context};
use serde::Deserialize;
use std::{
    fs,
    path::Path,
};

use crate::manifest::{load_manifest, resolve_asset_path, resolve_contract_relative_path};

#[derive(Debug, Deserialize)]
struct SemanticsFrontmatter {
    asset_refs: Vec<String>,
}

#[derive(Debug)]
pub struct SemanticsDoc {
    pub asset_refs: Vec<String>,
}

pub fn load_semantics_doc(path: impl AsRef<Path>) -> anyhow::Result<SemanticsDoc> {
    let path = path.as_ref();
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read semantics doc: {}", path.display()))?;
    let frontmatter = extract_frontmatter(&raw)
        .with_context(|| format!("failed to parse semantics frontmatter: {}", path.display()))?;
    let frontmatter: SemanticsFrontmatter = serde_yaml::from_str(&frontmatter)
        .with_context(|| format!("semantics frontmatter must be valid YAML: {}", path.display()))?;

    Ok(SemanticsDoc {
        asset_refs: frontmatter.asset_refs,
    })
}

pub fn run_semantics_gates(manifest_path: impl AsRef<Path>) -> anyhow::Result<()> {
    let manifest = load_manifest(manifest_path)?;
    for key in ["validation_semantics", "path_semantics", "compatibility_semantics"] {
        let doc_path = resolve_asset_path(&manifest, key)?;
        let doc = load_semantics_doc(&doc_path)
            .with_context(|| format!("failed semantics gate for {}", doc_path.display()))?;

        ensure!(
            !doc.asset_refs.is_empty(),
            "semantics doc must declare at least one asset_ref: {}",
            doc_path.display()
        );

        for asset_ref in &doc.asset_refs {
            ensure!(
                manifest.data.assets.values().any(|candidate| candidate == asset_ref),
                "semantic asset ref must be declared in manifest: {}",
                asset_ref
            );
            resolve_contract_relative_path(&manifest.contracts_root, &asset_ref)
                .with_context(|| {
                    format!(
                        "failed to resolve semantics asset reference `{}` from {}",
                        asset_ref,
                        doc_path.display()
                    )
                })?;
        }
    }

    Ok(())
}

fn extract_frontmatter(raw: &str) -> anyhow::Result<String> {
    let mut lines = raw.lines();
    ensure!(
        matches!(lines.next(), Some("---")),
        "semantics docs must start with YAML frontmatter"
    );

    let mut frontmatter = Vec::new();
    let mut found_closing = false;
    for line in lines {
        if line == "---" {
            found_closing = true;
            break;
        }
        frontmatter.push(line);
    }

    ensure!(found_closing, "semantics docs must end YAML frontmatter with ---");

    Ok(frontmatter.join("\n"))
}
