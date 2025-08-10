use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod commands;
mod scanner;
mod cache;
mod utils;

#[derive(Parser)]
#[command(
    name = "nmrs",
    about = "A fast and interactive CLI tool for managing node_modules directories",
    version = "0.1.0"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "List all node_modules directories")]
    Ls {
        #[arg(help = "Directory path to search")]
        path: PathBuf,
    },
    #[command(about = "Remove selected node_modules directories")]
    Rm {
        #[arg(help = "Directory path to search")]
        path: PathBuf,
    },
    #[command(about = "Clear cached scan results")]
    ClearCache,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Ls { path } => {
            commands::ls::execute(path).await?;
        }
        Commands::Rm { path } => {
            commands::rm::execute(path).await?;
        }
        Commands::ClearCache => {
            commands::cache::clear().await?;
            println!("Cache cleared successfully.");
        }
    }

    Ok(())
}
