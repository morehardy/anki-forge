use anyhow::{ensure, Context};
use serde::Deserialize;
use std::{
    collections::HashSet,
    fs,
    path::Path,
};

use crate::{
    fixtures::load_fixture_catalog,
    manifest::{load_manifest, resolve_asset_path, resolve_contract_relative_path},
};

#[derive(Debug, Deserialize)]
struct CompatibilityClasses {
    classes: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct UpgradeRules {
    rules: Vec<UpgradeRule>,
}

#[derive(Debug, Deserialize)]
struct UpgradeRule {
    id: String,
}

pub fn run_versioning_gates(manifest_path: impl AsRef<Path>) -> anyhow::Result<()> {
    let manifest = load_manifest(manifest_path)?;
    let policy_path = resolve_asset_path(&manifest, "version_policy")?;
    let classes_path = resolve_asset_path(&manifest, "compatibility_classes")?;
    let rules_path = resolve_asset_path(&manifest, "upgrade_rules")?;
    let catalog_path = resolve_asset_path(&manifest, "fixture_catalog")?;

    let policy = fs::read_to_string(&policy_path)
        .with_context(|| format!("failed to read version policy: {}", policy_path.display()))?;
    let policy_lower = policy.to_lowercase();
    ensure!(
        policy_lower.contains("bundle version is the only public compatibility axis")
            && policy_lower.contains("component versions are internal governance metadata only"),
        "version policy must describe the bundle-version public axis and internal component versions: {}",
        policy_path.display()
    );

    let classes: CompatibilityClasses = load_yaml_model(&classes_path)?;
    let rules: UpgradeRules = load_yaml_model(&rules_path)?;
    let catalog = load_fixture_catalog(&catalog_path)?;

    ensure!(
        !classes.classes.is_empty(),
        "compatibility class catalog must not be empty"
    );
    ensure!(!rules.rules.is_empty(), "upgrade rule catalog must not be empty");
    ensure!(
        !catalog.cases.is_empty(),
        "fixture catalog must not be empty for versioning checks"
    );

    let class_ids: HashSet<&str> = classes.classes.iter().map(String::as_str).collect();
    let rule_ids: HashSet<&str> = rules.rules.iter().map(|rule| rule.id.as_str()).collect();

    for required_class in [
        "additive_compatible",
        "behavior_tightening_compatible",
        "behavior_changing_incompatible",
    ] {
        ensure!(
            class_ids.contains(required_class),
            "required compatibility class is missing: {}",
            required_class
        );
    }

    for required_rule in [
        "migration_notes_required",
        "fixture_updates_required",
        "executable_checks_required",
    ] {
        ensure!(
            rule_ids.contains(required_rule),
            "required upgrade rule is missing: {}",
            required_rule
        );
    }

    let mut saw_compatible = false;
    let mut saw_incompatible = false;

    for case in catalog.cases.iter().filter(|case| case.category == "evolution") {
        let class = case
            .compatibility_class
            .as_deref()
            .with_context(|| format!("evolution case is missing compatibility_class: {}", case.id))?;
        ensure!(
            class_ids.contains(class),
            "unknown compatibility class in evolution fixture: {} ({})",
            case.id,
            class
        );

        let expected_bump = case.expected_bundle_bump.as_deref().with_context(|| {
            format!("evolution case is missing expected_bundle_bump: {}", case.id)
        })?;
        ensure!(
            matches!(expected_bump, "minor" | "major"),
            "evolution case must declare a minor or major bundle bump: {}",
            case.id
        );

        match class {
            "behavior_changing_incompatible" => {
                saw_incompatible = true;
                ensure!(
                    expected_bump == "major",
                    "incompatible evolution must require a major bump: {}",
                    case.id
                );
            }
            _ => {
                saw_compatible = true;
                ensure!(
                    expected_bump == "minor",
                    "compatible evolution must require a minor bump: {}",
                    case.id
                );
            }
        }

        ensure!(
            !case.upgrade_rules.is_empty(),
            "evolution case must declare at least one upgrade rule: {}",
            case.id
        );

        for rule_id in &case.upgrade_rules {
            ensure!(
                rule_ids.contains(rule_id.as_str()),
                "unknown upgrade rule in evolution fixture: {} ({})",
                case.id,
                rule_id
            );
        }

        let target_asset = case
            .target_asset
            .as_deref()
            .with_context(|| format!("evolution case is missing target_asset: {}", case.id))?;
        resolve_contract_relative_path(&manifest.contracts_root, target_asset).with_context(
            || format!("failed to resolve evolution target asset for case {}", case.id),
        )?;

        ensure!(
            !case.affected_paths.is_empty(),
            "evolution case must declare affected_paths: {}",
            case.id
        );
        for affected_path in &case.affected_paths {
            resolve_contract_relative_path(&manifest.contracts_root, affected_path).with_context(
                || format!("failed to resolve evolution affected path for case {}", case.id),
            )?;
        }
    }

    ensure!(
        saw_compatible && saw_incompatible,
        "fixture catalog must include both compatible and incompatible evolution examples"
    );

    Ok(())
}

fn load_yaml_model<T>(path: impl AsRef<Path>) -> anyhow::Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let path = path.as_ref();
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read YAML asset: {}", path.display()))?;
    serde_yaml::from_str(&raw)
        .with_context(|| format!("YAML asset must match its model: {}", path.display()))
}
