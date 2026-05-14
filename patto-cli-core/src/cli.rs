use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "patto-core")]
#[command(version)]
#[command(about = "Native analysis core for Patto CLI")]
pub struct Cli {
    #[arg(long, global = true, default_value = "auto")]
    pub lang: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Scan(CommonArgs),
    Lint(CommonArgs),
    Check(CommonArgs),
    Doctor(CommonArgs),
    #[command(name = "format-i18n")]
    FormatI18n(CommonArgs),
}

#[derive(Debug, Clone, clap::Args)]
pub struct CommonArgs {
    #[arg(long)]
    pub json: bool,

    #[arg(long, default_value = ".")]
    pub root: PathBuf,
}
