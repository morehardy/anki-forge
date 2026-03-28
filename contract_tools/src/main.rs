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
    }

    Ok(())
}
