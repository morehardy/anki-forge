use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "contract_tools")]
#[command(about = "Internal contract verification tooling")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Verify {
        #[arg(long)]
        manifest: String,
    },
    Summary {
        #[arg(long)]
        manifest: String,
    },
    Package {
        #[arg(long)]
        manifest: String,
        #[arg(long)]
        out_dir: String,
    },
    PackageRuntimeAssets {
        #[arg(long)]
        manifest: String,
    },
    Normalize {
        #[arg(long)]
        manifest: String,
        #[arg(long)]
        input: String,
        #[arg(long, default_value = "contract-json")]
        output: String,
    },
    Build {
        #[arg(long)]
        manifest: String,
        #[arg(long)]
        input: String,
        #[arg(long, default_value = "default")]
        writer_policy: String,
        #[arg(long, default_value = "default")]
        build_context: String,
        #[arg(long)]
        artifacts_dir: String,
        #[arg(long, default_value = "contract-json")]
        output: String,
    },
    ProductBuild {
        #[arg(long)]
        manifest: String,
        #[arg(long)]
        product_input: String,
        #[arg(long)]
        apkg_out: String,
        #[arg(long)]
        compare_to: Option<String>,
        #[arg(long)]
        fail_on: Option<String>,
        #[arg(long)]
        report_json: Option<String>,
        #[arg(long)]
        identity_lockfile: Option<String>,
        #[arg(long)]
        write_identity_lockfile: bool,
        #[arg(long)]
        update_safety: Option<String>,
        #[arg(long, default_value = "contract-json")]
        output: String,
    },
    Inspect {
        #[arg(long)]
        staging: Option<String>,
        #[arg(long)]
        apkg: Option<String>,
        #[arg(long, default_value = "contract-json")]
        output: String,
    },
    Diff {
        #[arg(long)]
        left: String,
        #[arg(long)]
        right: String,
        #[arg(long, default_value = "contract-json")]
        output: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Verify { manifest } => {
            contract_tools::gates::run_all(&manifest)?;
            println!("verification passed");
        }
        Command::Summary { manifest } => {
            println!("{}", contract_tools::summary::render(&manifest)?);
        }
        Command::Package { manifest, out_dir } => {
            let artifact_path = contract_tools::package::build_artifact(&manifest, &out_dir)?;
            println!("{}", artifact_path.display());
        }
        Command::PackageRuntimeAssets { manifest } => {
            let manifest = contract_tools::manifest::load_manifest(&manifest)?;
            let paths = contract_tools::package::runtime_asset_relative_paths(&manifest)?;
            println!("{}", serde_json::to_string(&paths)?);
        }
        Command::Normalize {
            manifest,
            input,
            output,
        } => {
            print!(
                "{}",
                contract_tools::normalize_cmd::run(&manifest, &input, &output)?
            );
        }
        Command::Build {
            manifest,
            input,
            writer_policy,
            build_context,
            artifacts_dir,
            output,
        } => {
            print!(
                "{}",
                contract_tools::build_cmd::run(
                    &manifest,
                    &input,
                    &writer_policy,
                    &build_context,
                    &artifacts_dir,
                    &output,
                )?
            );
        }
        Command::ProductBuild {
            manifest,
            product_input,
            apkg_out,
            compare_to,
            fail_on,
            report_json,
            identity_lockfile,
            write_identity_lockfile,
            update_safety,
            output,
        } => {
            match contract_tools::product_build_cmd::run(
                &manifest,
                &product_input,
                &apkg_out,
                compare_to.as_deref(),
                fail_on.as_deref(),
                report_json.as_deref(),
                identity_lockfile.as_deref(),
                write_identity_lockfile,
                update_safety.as_deref(),
                &output,
            )? {
                contract_tools::product_build_cmd::ProductBuildOutcome::Success(body) => {
                    print!("{body}");
                }
                contract_tools::product_build_cmd::ProductBuildOutcome::ReportFailure {
                    json,
                    exit_code,
                } => {
                    print!("{json}");
                    std::process::exit(exit_code);
                }
            }
        }
        Command::Inspect {
            staging,
            apkg,
            output,
        } => {
            print!(
                "{}",
                contract_tools::inspect_cmd::run(staging.as_deref(), apkg.as_deref(), &output)?
            );
        }
        Command::Diff {
            left,
            right,
            output,
        } => {
            print!("{}", contract_tools::diff_cmd::run(&left, &right, &output)?);
        }
    }

    Ok(())
}
