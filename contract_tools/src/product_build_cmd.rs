use std::path::PathBuf;

use anki_forge::build::{BuildOptions, BuildReportJson, BuildStatus, RiskLevel};
use anki_forge::product::ProductDocument;

pub enum ProductBuildOutcome {
    Success(String),
    ReportFailure { json: String, exit_code: i32 },
}

pub fn run(
    manifest: &str,
    product_input: &str,
    apkg_out: &str,
    compare_to: Option<&str>,
    fail_on: Option<&str>,
    report_json: Option<&str>,
    output: &str,
) -> anyhow::Result<ProductBuildOutcome> {
    let manifest = crate::manifest::load_manifest(manifest)?;
    crate::manifest::resolve_asset_path(&manifest, "build_report_schema")?;
    let runtime_bundle = anki_forge::runtime::load_bundle_from_manifest(&manifest.path)?;
    let writer_policy = anki_forge::runtime::load_writer_policy(&runtime_bundle, "default")?;
    let build_context = anki_forge::runtime::load_build_context(&runtime_bundle, "default")?;
    let raw = std::fs::read_to_string(product_input)?;
    let document: ProductDocument = serde_json::from_str(&raw)?;

    let mut options = BuildOptions::new().output(PathBuf::from(apkg_out));
    if let Some(compare_to) = compare_to {
        options = options.compare_to(compare_to);
    }
    if let Some(fail_on) = fail_on {
        options = options.fail_on(parse_risk_level(fail_on)?);
    }
    if let Some(report_json) = report_json {
        options = options.report_json(report_json);
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
            Ok(ProductBuildOutcome::Success(body))
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

fn exit_code_for_status(status: BuildStatus) -> i32 {
    match status {
        BuildStatus::Success => 0,
        BuildStatus::Blocked => 2,
        BuildStatus::Invalid => 3,
        BuildStatus::Error => 4,
    }
}
