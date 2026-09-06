use std::path::PathBuf;

use anki_forge::build::{
    BuildOptions, BuildReportJson, BuildStatus, ProjectNormalizeOptions, RiskLevel,
};
use anki_forge::product::ProductDocument;

pub enum ProductBuildOutcome {
    Success(String),
    ReportFailure { json: String, exit_code: i32 },
}

pub struct ProductBuildRequest<'a> {
    pub manifest: &'a str,
    pub product_input: &'a str,
    pub base_dir: Option<&'a str>,
    pub apkg_out: &'a str,
    pub compare_to: Option<&'a str>,
    pub fail_on: Option<&'a str>,
    pub report_json: Option<&'a str>,
    pub identity_lockfile: Option<&'a str>,
    pub write_identity_lockfile: bool,
    pub update_safety: Option<&'a str>,
    pub output: &'a str,
}

pub fn run(request: ProductBuildRequest<'_>) -> anyhow::Result<ProductBuildOutcome> {
    let ProductBuildRequest {
        manifest,
        product_input,
        base_dir,
        apkg_out,
        compare_to,
        fail_on,
        report_json,
        identity_lockfile,
        write_identity_lockfile,
        update_safety,
        output,
    } = request;

    let manifest = crate::manifest::load_manifest(manifest)?;
    crate::manifest::resolve_asset_path(&manifest, "build_report_schema")?;
    let runtime_bundle = anki_forge::runtime::load_bundle_from_manifest(&manifest.path)?;
    let writer_policy = anki_forge::runtime::load_writer_policy(&runtime_bundle, "default")?;
    let build_context = anki_forge::runtime::load_build_context(&runtime_bundle, "default")?;
    let product_input_path = PathBuf::from(product_input);
    let product_input_base_dir = base_dir
        .map(PathBuf::from)
        .or_else(|| product_input_path.parent().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("."));
    let raw = std::fs::read_to_string(&product_input_path)?;
    let document: ProductDocument = serde_json::from_str(&raw)?;

    let mut options = BuildOptions::new()
        .output(PathBuf::from(apkg_out))
        .normalize_options(ProjectNormalizeOptions::strict().base_dir(product_input_base_dir));
    if let Some(compare_to) = compare_to {
        options = options.compare_to(compare_to);
    }
    if let Some(fail_on) = fail_on {
        options = options.fail_on(parse_risk_level(fail_on)?);
    }
    if let Some(report_json) = report_json {
        options = options.report_json(report_json);
    }
    if let Some(identity_lockfile) = identity_lockfile {
        options = options.identity_lockfile(identity_lockfile);
    }
    if write_identity_lockfile {
        options = options.write_identity_lockfile(true);
    }
    if let Some(update_safety) = update_safety {
        options = options.update_safety(parse_update_safety_mode(update_safety)?);
    }

    let result = anki_forge::runtime::build_product_document_with_writer_stack(
        document,
        options,
        writer_policy,
        build_context,
    );

    match result {
        Ok(report) => {
            let body = render(&report, output)?;
            if report.status == BuildStatus::Success {
                Ok(ProductBuildOutcome::Success(body))
            } else {
                Ok(ProductBuildOutcome::ReportFailure {
                    json: body,
                    exit_code: exit_code_for_status(report.status),
                })
            }
        }
        Err(err) => {
            let body = render(&err.report, output)?;
            let exit_code = exit_code_for_status(err.report.status);
            Ok(ProductBuildOutcome::ReportFailure {
                json: body,
                exit_code,
            })
        }
    }
}

fn render(report: &anki_forge::build::BuildReport, output: &str) -> anyhow::Result<String> {
    match output {
        "contract-json" => Ok(serde_json::to_string_pretty(
            &BuildReportJson::from_report(report),
        )?),
        "human" => Ok(format!(
            "status: {:?}\ncomparison: {:?}\nhighest_risk: {:?}\n",
            report.status,
            report.comparison,
            report.risk.as_ref().and_then(|risk| risk.highest_level)
        )),
        other => anyhow::bail!("unsupported product-build output mode: {other}"),
    }
}

fn parse_risk_level(value: &str) -> anyhow::Result<RiskLevel> {
    match value {
        "info" => Ok(RiskLevel::Info),
        "low" => Ok(RiskLevel::Low),
        "medium" => Ok(RiskLevel::Medium),
        "high" => Ok(RiskLevel::High),
        "critical" => Ok(RiskLevel::Critical),
        other => anyhow::bail!("unsupported fail-on level: {other}"),
    }
}

fn parse_update_safety_mode(value: &str) -> anyhow::Result<anki_forge::build::UpdateSafetyMode> {
    match value {
        "strict" => Ok(anki_forge::build::UpdateSafetyMode::Strict),
        "report-only" | "report_only" => Ok(anki_forge::build::UpdateSafetyMode::ReportOnly),
        "disabled" => Ok(anki_forge::build::UpdateSafetyMode::Disabled),
        other => anyhow::bail!("unsupported update-safety mode: {other}"),
    }
}

fn exit_code_for_status(status: BuildStatus) -> i32 {
    match status {
        BuildStatus::Success => 0,
        BuildStatus::Blocked => 2,
        BuildStatus::Invalid => 3,
        BuildStatus::Error => 4,
    }
}
