mod error;
mod expr;
mod feature;
mod handler;
mod runtime;
mod runtime_impl;
mod walk;
mod walk_impl;
mod yaml;

use anyhow::Result;
use clap::{Args, Parser, Subcommand, ValueEnum};
use handler::{OutputFormat, run_check};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "dir-lint",
    about = "A linter that validates directory structure against YAML rules"
)]
#[command(args_conflicts_with_subcommands = true)]
struct Cli {
    /// Top-level args used when no subcommand is given (defaults to `check`).
    #[command(flatten)]
    check_args: CheckArgs,
    #[command(subcommand)]
    command: Option<Commands>,
}

/// Arguments for the `check` subcommand.
#[derive(Args)]
struct CheckArgs {
    /// Path to the config file.
    #[arg(short, long, default_value = ".dir-lint.yaml")]
    config: PathBuf,
    /// Output format (human: stderr diagnostics, json: stdout JSON).
    #[arg(long, value_enum, default_value_t = FormatArg::Human)]
    format: FormatArg,
}

/// Output format (CLI argument).
#[derive(Copy, Clone, Debug, ValueEnum)]
enum FormatArg {
    /// Human-readable diagnostics (emits codespan output to stderr).
    Human,
    /// JSON (emits diagnostics and dir trace as pretty JSON to stdout).
    Json,
}

#[derive(Subcommand)]
enum Commands {
    /// Check directory structure against the config.
    Check(CheckArgs),
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let args = match cli.command {
        Some(Commands::Check(args)) => args,
        None => cli.check_args,
    };
    let format = match args.format {
        FormatArg::Human => OutputFormat::Human,
        FormatArg::Json => OutputFormat::Json,
    };
    run_check(&args.config, format)?;
    Ok(())
}
