mod checks;
mod context;

use anyhow::Result;
use std::time::Instant;

use crate::cli::CommonArgs;
use crate::diagnostic::{Diagnostic, DiagnosticLevel};
use crate::lang;
use crate::output::doctor_output::{
    DoctorCheck, DoctorCheckStatus, DoctorCommandOutput, DoctorSummary,
};
use crate::output::{OutputStats, OutputStatus, print_json};
use crate::project::{ProjectScan, scan_project};
use crate::utils::root_not_exists;

use self::context::DoctorContext;

pub(crate) struct DoctorEvaluation {
    pub diagnostics: Vec<Diagnostic>,
    pub checks: Vec<DoctorCheck>,
}

pub fn run(args: CommonArgs, lang: crate::lang::Lang) -> Result<i32> {
    let started_at = Instant::now();
    let mut diagnostics = Vec::new();

    if root_not_exists(&mut diagnostics, &args.root, lang) {
        let output = build_output(
            diagnostics,
            OutputStats {
                files_scanned: 0,
                directories_scanned: 0,
                duration_ms: started_at.elapsed().as_millis(),
            },
            Vec::new(),
        );
        if args.json {
            print_json(&output)?;
        } else {
            println!("{}", lang::text(lang, "cli.doctor.root-invalid"));
        }
        return Ok(1);
    }

    let project_scan = scan_project(&args.root, lang)?;
    diagnostics.extend(project_scan.diagnostics.clone());
    let evaluation = evaluate(&project_scan, lang);
    diagnostics.extend(evaluation.diagnostics);

    let output = build_output(
        diagnostics,
        OutputStats {
            files_scanned: project_scan.files_scanned,
            directories_scanned: project_scan.directories_scanned,
            duration_ms: started_at.elapsed().as_millis(),
        },
        evaluation.checks,
    );

    if args.json {
        print_json(&output)?;
    } else {
        let ok = output.summary.ok.to_string();
        let warnings = output.summary.warnings.to_string();
        let errors = output.summary.errors.to_string();
        let skipped = output.summary.skipped.to_string();
        println!(
            "{}",
            lang::message(
                lang,
                "cli.doctor.completed",
                &[
                    ("ok", ok.as_str()),
                    ("warnings", warnings.as_str()),
                    ("errors", errors.as_str()),
                    ("skipped", skipped.as_str()),
                ],
            )
        );
    }

    Ok(if output.summary.errors > 0 { 1 } else { 0 })
}

pub(crate) fn evaluate(project_scan: &ProjectScan, lang: crate::lang::Lang) -> DoctorEvaluation {
    let context = DoctorContext {
        project: project_scan,
        locale: lang,
    };

    let results = checks::run_all(&context);
    let mut diagnostics = Vec::new();
    let mut checks = Vec::new();
    for result in results {
        checks.push(result.check);
        diagnostics.extend(result.diagnostics);
    }

    DoctorEvaluation {
        diagnostics,
        checks,
    }
}

fn build_output(
    diagnostics: Vec<Diagnostic>,
    stats: OutputStats,
    checks: Vec<DoctorCheck>,
) -> DoctorCommandOutput {
    let diagnostic_errors = diagnostics
        .iter()
        .filter(|diagnostic| matches!(diagnostic.level, DiagnosticLevel::Error))
        .count();
    let diagnostic_warnings = diagnostics
        .iter()
        .filter(|diagnostic| matches!(diagnostic.level, DiagnosticLevel::Warning))
        .count();
    let check_errors = checks
        .iter()
        .filter(|check| check.status == DoctorCheckStatus::Error)
        .count();
    let check_warnings = checks
        .iter()
        .filter(|check| check.status == DoctorCheckStatus::Warning)
        .count();
    let summary = DoctorSummary {
        checks_run: checks.len(),
        ok: checks
            .iter()
            .filter(|check| check.status == DoctorCheckStatus::Ok)
            .count(),
        warnings: check_warnings.max(diagnostic_warnings),
        errors: check_errors.max(diagnostic_errors),
        skipped: checks
            .iter()
            .filter(|check| check.status == DoctorCheckStatus::Skipped)
            .count(),
    };

    DoctorCommandOutput {
        status: if summary.errors > 0 {
            OutputStatus::Failed
        } else {
            OutputStatus::Ok
        },
        command: "doctor".to_string(),
        diagnostics,
        stats,
        summary,
        checks,
    }
}
