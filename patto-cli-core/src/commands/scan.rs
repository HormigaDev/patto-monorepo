use anyhow::Result;
use std::time::Instant;

use crate::cli::CommonArgs;
use crate::diagnostic::{Diagnostic, DiagnosticLevel};
use crate::lang::{self, Lang};
use crate::output::scan_output::ScanCommandOutput;
use crate::output::{OutputStats, OutputStatus, print_json};
use crate::project::{
    INDEX_RELATIVE_PATH, empty_project_index, index_write_failed_diagnostic, scan_project,
    write_project_index,
};
use crate::utils::root_not_exists;

pub fn run(args: CommonArgs, lang: Lang) -> Result<i32> {
    let started_at = Instant::now();
    let mut diagnostics = Vec::new();

    if root_not_exists(&mut diagnostics, &args.root, lang) {
        let output = empty_output(started_at, diagnostics);
        if args.json {
            print_json(&output)?;
        } else {
            println!("{}", lang::text(lang, "cli.scan.root-invalid"));
        }
        return Ok(1);
    }

    let project_scan = scan_project(&args.root, lang)?;
    diagnostics.extend(project_scan.diagnostics);

    let mut index_path = None;
    if let Err(error) = write_project_index(&project_scan.root, &project_scan.index) {
        diagnostics.push(index_write_failed_diagnostic(lang, &error));
    } else {
        index_path = Some(INDEX_RELATIVE_PATH.to_string());
    }

    let has_errors = diagnostics
        .iter()
        .any(|diagnostic| matches!(diagnostic.level, DiagnosticLevel::Error));

    let output = ScanCommandOutput {
        status: if has_errors {
            OutputStatus::Failed
        } else {
            OutputStatus::Ok
        },
        command: "scan".to_string(),
        diagnostics,
        stats: OutputStats {
            files_scanned: project_scan.files_scanned,
            directories_scanned: project_scan.directories_scanned,
            duration_ms: started_at.elapsed().as_millis(),
        },
        index_path,
        index: project_scan.index,
    };

    if args.json {
        print_json(&output)?;
    } else {
        let files = output.stats.files_scanned.to_string();
        let commands = (output.index.summary.commands
            + output.index.summary.subcommands
            + output.index.summary.subcommand_groups)
            .to_string();
        let index = output
            .index_path
            .as_deref()
            .map(str::to_string)
            .unwrap_or_else(|| lang::text(lang, "common.not-written"));
        println!(
            "{}",
            lang::message(
                lang,
                "cli.scan.completed",
                &[
                    ("files", files.as_str()),
                    ("commands", commands.as_str()),
                    ("index", index.as_str()),
                ],
            )
        );
    }

    Ok(if has_errors { 1 } else { 0 })
}

fn empty_output(started_at: Instant, diagnostics: Vec<Diagnostic>) -> ScanCommandOutput {
    ScanCommandOutput {
        status: OutputStatus::Failed,
        command: "scan".to_string(),
        diagnostics,
        stats: OutputStats {
            files_scanned: 0,
            directories_scanned: 0,
            duration_ms: started_at.elapsed().as_millis(),
        },
        index_path: None,
        index: empty_project_index(),
    }
}
