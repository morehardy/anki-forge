use anyhow::{ensure, Context};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::Value;
use std::{collections::HashSet, fs, path::Path};

use crate::{
    manifest::{load_manifest, resolve_asset_path, resolve_contract_relative_path},
    registry::load_registry,
    schema::{load_schema, validate_value},
};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixtureCatalog {
    pub cases: Vec<FixtureCase>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixtureCase {
    pub id: String,
    pub category: String,
    pub input: String,
    #[serde(default)]
    pub expected: Option<String>,
    #[serde(default)]
    pub compatibility_class: Option<String>,
    #[serde(default)]
    pub upgrade_rules: Vec<String>,
    #[serde(default)]
    pub target_asset: Option<String>,
    #[serde(default)]
    pub affected_paths: Vec<String>,
    #[serde(default)]
    pub expected_bundle_bump: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EvolutionFixture {
    kind: String,
    compatibility_class: String,
    upgrade_rules: Vec<String>,
    target_asset: String,
    affected_paths: Vec<String>,
    expected_bundle_bump: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CompatibilityClasses {
    classes: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UpgradeRules {
    rules: Vec<UpgradeRule>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UpgradeRule {
    id: String,
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
    let authoring_ir_schema_path = resolve_asset_path(&manifest, "authoring_ir_schema")?;
    let service_envelope_schema_path = resolve_asset_path(&manifest, "service_envelope_schema")?;
    let validation_report_schema_path = resolve_asset_path(&manifest, "validation_report_schema")?;
    let registry_path = resolve_asset_path(&manifest, "error_registry")?;
    let compatibility_classes_path = resolve_asset_path(&manifest, "compatibility_classes")?;
    let upgrade_rules_path = resolve_asset_path(&manifest, "upgrade_rules")?;

    let catalog = load_fixture_catalog(&catalog_path)?;
    let registry = load_registry(&registry_path)?;
    let compatibility_classes: CompatibilityClasses = load_yaml_model(&compatibility_classes_path)?;
    let upgrade_rules: UpgradeRules = load_yaml_model(&upgrade_rules_path)?;

    let authoring_ir_schema = load_schema(&authoring_ir_schema_path)?;
    let service_envelope_schema = load_schema(&service_envelope_schema_path)?;
    let validation_report_schema = load_schema(&validation_report_schema_path)?;

    ensure!(
        !catalog.cases.is_empty(),
        "fixture catalog must not be empty"
    );

    let registry_codes: HashSet<String> =
        registry.codes.iter().map(|code| code.id.clone()).collect();
    let compatibility_class_ids: HashSet<String> =
        compatibility_classes.classes.into_iter().collect();
    let upgrade_rule_ids: HashSet<String> = upgrade_rules
        .rules
        .into_iter()
        .map(|rule| rule.id)
        .collect();

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
        let input_path = resolve_contract_relative_path(&manifest.contracts_root, &case.input)
            .with_context(|| format!("fixture input must resolve safely for case {}", case.id))?;

        match case.category.as_str() {
            "valid" => {
                let input_value = load_json_value(&input_path)?;
                validate_value(&authoring_ir_schema, &input_value).with_context(|| {
                    format!(
                        "valid fixture must satisfy authoring_ir_schema: {}",
                        case.id
                    )
                })?;
            }
            "invalid" => {
                let input_value = load_json_value(&input_path)?;
                ensure!(
                    validate_value(&authoring_ir_schema, &input_value).is_err(),
                    "invalid fixture must fail authoring_ir_schema validation: {}",
                    case.id
                );
            }
            "service-envelope" => {
                let input_value = load_json_value(&input_path)?;
                validate_value(&service_envelope_schema, &input_value).with_context(|| {
                    format!(
                        "service-envelope fixture must satisfy service_envelope_schema: {}",
                        case.id
                    )
                })?;
            }
            "evolution" => {
                let evolution: EvolutionFixture = load_yaml_model(&input_path)?;
                ensure!(
                    evolution.kind == "evolution-case",
                    "evolution fixture must declare kind=evolution-case: {}",
                    case.id
                );
                ensure!(
                    case.compatibility_class.as_deref()
                        == Some(evolution.compatibility_class.as_str()),
                    "evolution fixture metadata must match catalog: compatibility_class {}",
                    case.id
                );
                ensure!(
                    case.upgrade_rules == evolution.upgrade_rules,
                    "evolution fixture metadata must match catalog: upgrade_rules {}",
                    case.id
                );
                ensure!(
                    case.target_asset.as_deref() == Some(evolution.target_asset.as_str()),
                    "evolution fixture metadata must match catalog: target_asset {}",
                    case.id
                );
                ensure!(
                    case.affected_paths == evolution.affected_paths,
                    "evolution fixture metadata must match catalog: affected_paths {}",
                    case.id
                );
                ensure!(
                    case.expected_bundle_bump.as_deref()
                        == Some(evolution.expected_bundle_bump.as_str()),
                    "evolution fixture metadata must match catalog: expected_bundle_bump {}",
                    case.id
                );
                ensure!(
                    matches!(evolution.expected_bundle_bump.as_str(), "minor" | "major"),
                    "evolution fixture expected_bundle_bump must be minor or major: {}",
                    case.id
                );
                ensure!(
                    compatibility_class_ids.contains(evolution.compatibility_class.as_str()),
                    "unknown compatibility class in evolution fixture: {}",
                    evolution.compatibility_class
                );
                for rule_id in &evolution.upgrade_rules {
                    ensure!(
                        upgrade_rule_ids.contains(rule_id.as_str()),
                        "unknown upgrade rule in evolution fixture: {}",
                        rule_id
                    );
                }
                resolve_contract_relative_path(&manifest.contracts_root, &evolution.target_asset)
                    .with_context(|| {
                    format!(
                        "evolution fixture target asset must resolve safely: {}",
                        case.id
                    )
                })?;
                ensure!(
                    !evolution.affected_paths.is_empty(),
                    "evolution fixture affected_paths must not be empty: {}",
                    case.id
                );
                for affected_path in &evolution.affected_paths {
                    resolve_contract_relative_path(&manifest.contracts_root, affected_path)
                        .with_context(|| {
                            format!(
                                "evolution fixture affected path must resolve safely: {}",
                                case.id
                            )
                        })?;
                }
            }
            other => {
                ensure!(
                    false,
                    "unsupported fixture category: {} ({})",
                    case.id,
                    other
                );
            }
        }

        if case.category == "evolution" {
            ensure!(
                case.compatibility_class.is_some(),
                "evolution case must declare compatibility_class: {}",
                case.id
            );
            ensure!(
                !case.upgrade_rules.is_empty(),
                "evolution case must declare upgrade_rules: {}",
                case.id
            );
            ensure!(
                case.target_asset.is_some(),
                "evolution case must declare target_asset: {}",
                case.id
            );
            ensure!(
                !case.affected_paths.is_empty(),
                "evolution case must declare affected_paths: {}",
                case.id
            );
            ensure!(
                case.expected_bundle_bump.is_some(),
                "evolution case must declare expected_bundle_bump: {}",
                case.id
            );
        }

        if let Some(expected) = &case.expected {
            let expected_path = resolve_contract_relative_path(&manifest.contracts_root, expected)
                .with_context(|| {
                    format!(
                        "fixture expected report must resolve safely for case {}",
                        case.id
                    )
                })?;
            let expected_value = load_json_value(&expected_path)?;
            validate_value(&validation_report_schema, &expected_value).with_context(|| {
                format!(
                    "expected report must satisfy validation_report_schema: {}",
                    case.id
                )
            })?;

            let diagnostics = expected_value
                .get("diagnostics")
                .and_then(Value::as_array)
                .context("validation report must contain diagnostics array")?;
            ensure!(
                !diagnostics.is_empty(),
                "validation report diagnostics array must not be empty for case {}",
                case.id
            );
            for diagnostic in diagnostics {
                let code = diagnostic
                    .get("code")
                    .and_then(Value::as_str)
                    .context("diagnostic code must be a string")?;
                ensure!(
                    registry_codes.contains(code),
                    "diagnostic code must exist in registry: {}",
                    code
                );
            }
        } else if case.category == "invalid" {
            ensure!(
                false,
                "invalid fixture case must declare an expected report: {}",
                case.id
            );
        }
    }

    Ok(())
}

fn load_yaml_model<T>(path: impl AsRef<Path>) -> anyhow::Result<T>
where
    T: DeserializeOwned,
{
    let path = path.as_ref();
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read YAML asset: {}", path.display()))?;
    serde_yaml::from_str(&raw)
        .with_context(|| format!("YAML asset must match its model: {}", path.display()))
}

fn load_json_value(path: impl AsRef<Path>) -> anyhow::Result<Value> {
    let path = path.as_ref();
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read JSON asset: {}", path.display()))?;
    serde_json::from_str(&raw)
        .with_context(|| format!("JSON asset must be valid JSON: {}", path.display()))
}
