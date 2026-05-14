mod cli;
mod commands;
mod diagnostic;
mod lang;
mod output;
mod project;
mod utils;

use anyhow::Result;
use clap::Parser;

use cli::{Cli, Commands};

use crate::lang::Lang;

fn main() {
    let exit_code = match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("{error:#}");
            2
        }
    };

    std::process::exit(exit_code);
}

fn run() -> Result<i32> {
    let cli = Cli::parse();

    let lang = Lang::parse(&cli.lang);

    let exit_code = match cli.command {
        Commands::Scan(args) => commands::scan::run(args, lang)?,
        Commands::Lint(args) => commands::lint::run(args, lang)?,
        Commands::Check(args) => commands::check::run(args, lang)?,
        Commands::Doctor(args) => commands::doctor::run(args, lang)?,
        Commands::FormatI18n(args) => commands::format_i18n::run(args, lang)?,
    };

    Ok(exit_code)
}
