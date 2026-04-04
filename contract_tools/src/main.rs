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
    Normalize {
        #[arg(long)]
        manifest: String,
        #[arg(long)]
        input: String,
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
    }

    Ok(())
}
