use anyhow::{ensure, Context};
use serde::Deserialize;
use std::{fs, path::Path};

use crate::manifest::{load_manifest, resolve_asset_path, resolve_contract_relative_path};

#[derive(Debug, Deserialize)]
pub struct FixtureCatalog {
    pub cases: Vec<FixtureCase>,
}

#[derive(Debug, Deserialize)]
pub struct FixtureCase {
    pub id: String,
    pub category: String,
    pub input: String,
    pub expected: Option<String>,
    #[serde(default)]
    pub compatibility_class: Option<String>,
    #[serde(default)]
    pub upgrade_rules: Option<Vec<String>>,
}

pub fn load_fixture_catalog(path: impl AsRef<Path>) -> anyhow::Result<FixtureCatalog> {
    let path = path.as_ref();
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read fixture catalog: {}", path.display()))?;
    serde_yaml::from_str(&raw).with_context(|| {
        format!(
            "fixture catalog must be valid YAML and match the catalog model: {}",
            path.display()
        )
    })
}

pub fn run_fixture_gates(manifest_path: impl AsRef<Path>) -> anyhow::Result<()> {
    let manifest = load_manifest(manifest_path)?;
    let catalog_path = resolve_asset_path(&manifest, "fixture_catalog")
        .context("fixture catalog asset must be declared in the manifest")?;
    let catalog = load_fixture_catalog(&catalog_path)?;

    ensure!(
        !catalog.cases.is_empty(),
        "fixture catalog must not be empty"
    );

    let required_case_ids = [
        "minimal-authoring-ir",
        "missing-document-id",
        "minimal-service-envelope",
        "additive-compatible",
        "incompatible-path-change",
    ];
    for case_id in required_case_ids {
        ensure!(
            catalog.cases.iter().any(|case| case.id == case_id),
            "fixture catalog must include {case_id}"
        );
    }

    let has_compatible_evolution = catalog.cases.iter().any(|case| {
        case.category == "evolution"
            && case.compatibility_class.as_deref() == Some("additive_compatible")
    });
    let has_incompatible_evolution = catalog.cases.iter().any(|case| {
        case.category == "evolution"
            && case.compatibility_class.as_deref() == Some("behavior_changing_incompatible")
    });
    ensure!(
        has_compatible_evolution && has_incompatible_evolution,
        "fixture catalog must include compatible and incompatible evolution cases"
    );

    for case in &catalog.cases {
        resolve_contract_relative_path(&manifest.contracts_root, &case.input).with_context(|| {
            format!(
                "fixture catalog input must resolve within contracts/: case {}",
                case.id
            )
        })?;

        if case.category == "invalid" {
            ensure!(
                case.expected.is_some(),
                "invalid fixture case {} must declare an expected report",
                case.id
            );
        }

        if let Some(expected) = &case.expected {
            resolve_contract_relative_path(&manifest.contracts_root, expected).with_context(
                || {
                    format!(
                        "fixture catalog expected report must resolve within contracts/: case {}",
                        case.id
                    )
                },
            )?;
        }
    }
    Ok(())
}
