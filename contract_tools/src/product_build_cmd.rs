use std::{
    io::Write,
    path::{Path, PathBuf},
};

use ankiforge::{
    build::BuildErrorKind,
    update::{CompareOptions, RiskLevel, UpdatePolicy},
    BuildOptions,
};
use anyhow::{ensure, Context};

pub enum ProductBuildOutcome {
    Success(String),
    ReportFailure {
        json: String,
        exit_code: i32,
        followup_error: Option<String>,
    },
}

pub struct ProductBuildRequest<'a> {
    pub project: &'a str,
    pub base_dir: Option<&'a str>,
    pub apkg_out: &'a str,
    pub update_from: Option<&'a str>,
    pub fail_on: Option<&'a str>,
    pub allow: &'a [String],
    pub report_json: Option<&'a str>,
    pub output: &'a str,
}

pub struct ProductCompareRequest<'a> {
    pub project: &'a str,
    pub base_dir: Option<&'a str>,
    pub baseline: &'a str,
    pub fail_on: Option<&'a str>,
    pub allow: &'a [String],
    pub report_json: Option<&'a str>,
    pub output: &'a str,
}

pub fn run(request: ProductBuildRequest<'_>) -> anyhow::Result<ProductBuildOutcome> {
    validate_output(request.output)?;
    validate_report_path(
        request.report_json,
        &[request.project, request.apkg_out],
        request.update_from,
    )?;
    let project = ankiforge::tools::load_project(request.project, request.base_dir.map(Path::new))?;
    let mut options = BuildOptions::to(request.apkg_out);
    if let Some(path) = request.update_from {
        options = options.update_from(path);
    }
    if request.fail_on.is_some() || !request.allow.is_empty() {
        options = options.update_policy(policy(request.fail_on, request.allow)?);
    }
    let (snapshot, exit_code) = match project.build(options) {
        Ok(output) => (output.snapshot(), 0),
        Err(error) => {
            let exit = match error.kind() {
                BuildErrorKind::PolicyBlocked => 2,
                BuildErrorKind::Configuration | BuildErrorKind::Validation => 3,
                _ => 4,
            };
            (error.snapshot(), exit)
        }
    };
    finish(
        serde_json::to_value(snapshot)?,
        exit_code,
        request.report_json,
        request.output,
    )
}

pub fn compare(request: ProductCompareRequest<'_>) -> anyhow::Result<ProductBuildOutcome> {
    validate_output(request.output)?;
    validate_report_path(
        request.report_json,
        &[request.project, request.baseline],
        None,
    )?;
    let project = ankiforge::tools::load_project(request.project, request.base_dir.map(Path::new))?;
    let options = CompareOptions::against(request.baseline)
        .update_policy(policy(request.fail_on, request.allow)?);
    match project.compare(options) {
        Ok(report) => finish(
            serde_json::to_value(report.snapshot())?,
            0,
            request.report_json,
            request.output,
        ),
        Err(error) => {
            // An incomplete comparison is not a ComparisonSnapshot.
            let snapshot = serde_json::json!({
                "schema_version": "ankiforge-comparison-error-v1",
                "code": error.code(), "message": error.to_string(),
                "report": error.report().snapshot(),
            });
            finish(snapshot, 3, request.report_json, request.output)
        }
    }
}

fn policy(level: Option<&str>, allow: &[String]) -> anyhow::Result<UpdatePolicy> {
    let mut policy = UpdatePolicy::default();
    if let Some(level) = level {
        policy = policy.fail_on(match level {
            "info" => RiskLevel::Info,
            "low" => RiskLevel::Low,
            "medium" => RiskLevel::Medium,
            "high" => RiskLevel::High,
            "critical" => RiskLevel::Critical,
            other => anyhow::bail!("unsupported fail-on level: {other}"),
        });
    }
    for code in allow {
        policy = policy.allow(code.parse()?);
    }
    Ok(policy)
}

fn validate_output(output: &str) -> anyhow::Result<()> {
    ensure!(
        matches!(output, "contract-json" | "human"),
        "unsupported output mode: {output}"
    );
    Ok(())
}

fn finish(
    snapshot: serde_json::Value,
    exit_code: i32,
    report_json: Option<&str>,
    output: &str,
) -> anyhow::Result<ProductBuildOutcome> {
    let json = serde_json::to_string_pretty(&snapshot)?;
    let body = if output == "contract-json" {
        json.clone()
    } else if let Some(result) = snapshot.get("result") {
        format!("result: {}\n", serde_json::to_string_pretty(result)?)
    } else {
        format!("comparison: {}\n", serde_json::to_string_pretty(&snapshot)?)
    };
    if let Some(path) = report_json {
        if let Err(error) = write_report(Path::new(path), &json) {
            return Ok(ProductBuildOutcome::ReportFailure {
                json: body, exit_code: 4,
                followup_error: Some(format!("report write failed at {path:?}: {error:#}; the operation result above remains valid")),
            });
        }
    }
    Ok(if exit_code == 0 {
        ProductBuildOutcome::Success(body)
    } else {
        ProductBuildOutcome::ReportFailure {
            json: body,
            exit_code,
            followup_error: None,
        }
    })
}

fn write_report(path: &Path, json: &str) -> anyhow::Result<()> {
    let parent = path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut temporary = tempfile::Builder::new()
        .prefix(".ankiforge-report-")
        .tempfile_in(parent)?;
    temporary.write_all(json.as_bytes())?;
    temporary.as_file().sync_all()?;
    temporary.persist(path).map_err(|error| error.error)?;
    #[cfg(unix)]
    std::fs::File::open(parent)?
        .sync_all()
        .context("report was replaced but directory synchronization failed")?;
    Ok(())
}

fn validate_report_path(
    report: Option<&str>,
    protected: &[&str],
    baseline: Option<&str>,
) -> anyhow::Result<()> {
    let Some(report) = report else {
        return Ok(());
    };
    let report = resolved_destination(Path::new(report))?;
    for path in protected.iter().copied().chain(baseline) {
        let path = resolved_destination(Path::new(path))?;
        ensure!(
            report != path
                && !(report.exists() && path.exists() && same_file::is_same_file(&report, &path)?),
            "report path must not replace project input, APKG output or baseline"
        );
    }
    Ok(())
}

fn resolved_destination(path: &Path) -> anyhow::Result<PathBuf> {
    if path.exists() {
        return Ok(path.canonicalize()?);
    }
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut parent = absolute.as_path();
    let mut suffix = Vec::new();
    while !parent.exists() {
        if let Some(name) = parent.file_name() {
            suffix.push(name.to_owned());
        }
        parent = parent
            .parent()
            .ok_or_else(|| anyhow::anyhow!("invalid output path"))?;
    }
    let mut resolved = parent.canonicalize()?;
    for part in suffix.iter().rev() {
        resolved.push(part);
    }
    Ok(resolved)
}
