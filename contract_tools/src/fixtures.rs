use anyhow::{ensure, Context};
use jsonschema::JSONSchema;
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

#[derive(Debug, Deserialize)]
struct Phase2AuthoringInput {
    kind: String,
    schema_version: String,
    metadata: Phase2AuthoringMetadata,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Phase2AuthoringMetadata {
    document_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Phase2RequestParams {
    authoring_input: String,
    #[serde(default)]
    comparison_context: Option<Value>,
    #[serde(default)]
    identity_override_mode: Option<String>,
    #[serde(default)]
    target_selector: Option<String>,
    #[serde(default)]
    external_id: Option<String>,
    #[serde(default)]
    reason_code: Option<String>,
    #[serde(default)]
    reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Phase2NormalizationCase {
    kind: String,
    #[serde(flatten)]
    request: Phase2RequestParams,
    #[serde(default)]
    expected_result: Option<String>,
    #[serde(default)]
    expected_result_status: Option<String>,
    #[serde(default)]
    expected_diagnostic_codes: Vec<String>,
    #[serde(default)]
    resolved_identity_prefix: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Phase2RiskCase {
    kind: String,
    #[serde(flatten)]
    request: Phase2RequestParams,
    #[serde(default)]
    expected_result: Option<String>,
    #[serde(default)]
    expected_result_status: Option<String>,
    #[serde(default)]
    expected_comparison_status: Option<String>,
    #[serde(default)]
    expected_overall_level: Option<String>,
    #[serde(default)]
    expected_comparison_reasons: Vec<String>,
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
        "phase2-normalization-minimal-success",
        "phase2-normalization-identity-random-warning",
        "phase2-risk-complete-low",
        "phase2-risk-partial-high",
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

        if let Some(compatibility_class) = &case.compatibility_class {
            ensure!(
                compatibility_class_ids.contains(compatibility_class.as_str()),
                "unknown compatibility class in fixture catalog: {}",
                compatibility_class
            );
        }
        for rule_id in &case.upgrade_rules {
            ensure!(
                upgrade_rule_ids.contains(rule_id.as_str()),
                "unknown upgrade rule in fixture catalog: {}",
                rule_id
            );
        }

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
            "phase2-normalization" => {
                run_phase2_normalization_case(
                    &manifest,
                    &authoring_ir_schema,
                    &input_path,
                    case.id.as_str(),
                )?;
            }
            "phase2-risk" => {
                run_phase2_risk_case(&manifest, &authoring_ir_schema, &input_path, case.id.as_str())?;
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

fn run_phase2_normalization_case(
    manifest: &crate::manifest::LoadedManifest,
    authoring_ir_schema: &JSONSchema,
    case_path: &Path,
    case_id: &str,
) -> anyhow::Result<()> {
    let case: Phase2NormalizationCase = load_yaml_model(case_path)?;
    ensure!(
        case.kind == "phase2-normalization-case",
        "phase2 normalization fixture must declare kind=phase2-normalization-case: {}",
        case_id
    );
    ensure!(
        case.expected_result.is_some()
            || case.expected_result_status.is_some()
            || !case.expected_diagnostic_codes.is_empty()
            || case.resolved_identity_prefix.is_some(),
        "phase2 normalization fixture must declare executable expectations: {}",
        case_id
    );

    let request = build_phase2_request(
        manifest,
        authoring_ir_schema,
        &case.request,
    )?;
    let actual = authoring_core::normalize(request);

    if let Some(expected_result) = &case.expected_result {
        compare_canonical_json(
            manifest,
            &actual,
            expected_result,
            case_id,
            "phase2 normalization output mismatch",
        )?;
    }
    if let Some(expected_status) = &case.expected_result_status {
        ensure!(
            actual.result_status == *expected_status,
            "phase2 normalization result_status mismatch: {}",
            case_id
        );
    }
    for expected_code in &case.expected_diagnostic_codes {
        ensure!(
            actual
                .diagnostics
                .items
                .iter()
                .any(|item| item.code == *expected_code),
            "phase2 normalization fixture missing expected diagnostic code {}: {}",
            expected_code,
            case_id
        );
    }
    if let Some(expected_prefix) = &case.resolved_identity_prefix {
        let normalized_ir = actual.normalized_ir.as_ref().with_context(|| {
            format!(
                "phase2 normalization fixture must emit normalized_ir for case {}",
                case_id
            )
        })?;
        ensure!(
            normalized_ir.resolved_identity.starts_with(expected_prefix),
            "phase2 normalization fixture resolved_identity must start with {}: {}",
            expected_prefix,
            case_id
        );
    }

    Ok(())
}

fn run_phase2_risk_case(
    manifest: &crate::manifest::LoadedManifest,
    authoring_ir_schema: &JSONSchema,
    case_path: &Path,
    case_id: &str,
) -> anyhow::Result<()> {
    let case: Phase2RiskCase = load_yaml_model(case_path)?;
    ensure!(
        case.kind == "phase2-risk-case",
        "phase2 risk fixture must declare kind=phase2-risk-case: {}",
        case_id
    );
    ensure!(
        case.expected_result.is_some()
            || case.expected_result_status.is_some()
            || case.expected_comparison_status.is_some()
            || case.expected_overall_level.is_some()
            || !case.expected_comparison_reasons.is_empty(),
        "phase2 risk fixture must declare executable expectations: {}",
        case_id
    );

    let request = build_phase2_request(
        manifest,
        authoring_ir_schema,
        &case.request,
    )?;
    let actual = authoring_core::normalize(request);
    let report = actual.merge_risk_report.as_ref().with_context(|| {
        format!(
            "phase2 risk fixture must emit merge_risk_report for case {}",
            case_id
        )
    })?;

    if let Some(expected_result) = &case.expected_result {
        compare_canonical_json(
            manifest,
            report,
            expected_result,
            case_id,
            "phase2 risk output mismatch",
        )?;
    }
    if let Some(expected_status) = &case.expected_result_status {
        ensure!(
            actual.result_status == *expected_status,
            "phase2 risk normalization result_status mismatch: {}",
            case_id
        );
    }
    if let Some(expected_status) = &case.expected_comparison_status {
        ensure!(
            report.comparison_status == *expected_status,
            "phase2 risk comparison_status mismatch: {}",
            case_id
        );
    }
    if let Some(expected_level) = &case.expected_overall_level {
        ensure!(
            report.overall_level == *expected_level,
            "phase2 risk overall_level mismatch: {}",
            case_id
        );
    }
    if !case.expected_comparison_reasons.is_empty() {
        ensure!(
            report.comparison_reasons == case.expected_comparison_reasons,
            "phase2 risk comparison_reasons mismatch: {}",
            case_id
        );
    }

    Ok(())
}

fn build_phase2_request(
    manifest: &crate::manifest::LoadedManifest,
    authoring_ir_schema: &JSONSchema,
    params: &Phase2RequestParams,
) -> anyhow::Result<authoring_core::NormalizationRequest> {
    let input =
        load_phase2_authoring_input(manifest, authoring_ir_schema, &params.authoring_input)?;
    let mut request = authoring_core::NormalizationRequest::new(input);
    if let Some(context) = params.comparison_context.clone() {
        request.comparison_context =
            Some(serde_json::from_value(context).context("phase2 comparison_context must match the contract model")?);
    }
    request.identity_override_mode = params.identity_override_mode.clone();
    request.target_selector = params.target_selector.clone();
    request.external_id = params.external_id.clone();
    request.reason_code = params.reason_code.clone();
    request.reason = params.reason.clone();
    Ok(request)
}

fn load_phase2_authoring_input(
    manifest: &crate::manifest::LoadedManifest,
    authoring_ir_schema: &JSONSchema,
    authoring_input: &str,
) -> anyhow::Result<authoring_core::AuthoringDocument> {
    let input_path = resolve_contract_relative_path(&manifest.contracts_root, authoring_input)?;
    let input_value = load_json_value(&input_path)?;
    validate_value(authoring_ir_schema, &input_value).with_context(|| {
        format!(
            "phase2 authoring input must satisfy authoring_ir_schema: {}",
            input_path.display()
        )
    })?;
    let input: Phase2AuthoringInput = serde_json::from_value(input_value).with_context(|| {
        format!(
            "phase2 authoring input must map into the execution model: {}",
            input_path.display()
        )
    })?;

    Ok(authoring_core::AuthoringDocument {
        kind: input.kind,
        schema_version: input.schema_version,
        metadata_document_id: input.metadata.document_id,
    })
}

fn compare_canonical_json(
    manifest: &crate::manifest::LoadedManifest,
    actual: &impl serde::Serialize,
    expected_relative_path: &str,
    case_id: &str,
    mismatch_message: &str,
) -> anyhow::Result<()> {
    let actual_text = authoring_core::to_canonical_json(actual)?;
    let expected_path =
        resolve_contract_relative_path(&manifest.contracts_root, expected_relative_path)
            .with_context(|| {
                format!(
                    "phase2 expected artifact must resolve safely for case {}",
                    case_id
                )
            })?;
    let expected_value = load_json_value(&expected_path)?;
    let expected_text = authoring_core::to_canonical_json(&expected_value)?;

    ensure!(actual_text == expected_text, "{}: {}", mismatch_message, case_id);
    Ok(())
}
