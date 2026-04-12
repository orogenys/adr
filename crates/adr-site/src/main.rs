mod components;
mod error;
mod site;

use std::{
    path::{Path, PathBuf},
    process::ExitCode,
};

use adr_core::load_config;
use clap::{Args, Parser, Subcommand};
use error::SiteError;
use site::build_site;

#[derive(Debug, Parser)]
#[command(name = "adr-site")]
#[command(about = "Build a static website for ADRs")]
struct Cli {
    #[arg(long, global = true, value_name = "FILE")]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Build(BuildArgs),
}

#[derive(Debug, Args)]
struct BuildArgs {
    #[arg(long, value_name = "DIR", default_value = "dist")]
    output: PathBuf,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), SiteError> {
    let cli = Cli::parse();
    let config = load_config(config_start_path(cli.config.as_deref()))?;

    match cli.command {
        Commands::Build(args) => build_site(&config, &args.output),
    }
}

fn config_start_path(path: Option<&Path>) -> &Path {
    path.unwrap_or_else(|| Path::new("."))
}
