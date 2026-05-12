use anyhow::Result;
use std::time::Instant;

use crate::cli::CommonArgs;
use crate::commands::{doctor, lint};
use crate::diagnostic::{Diagnostic, DiagnosticLevel};
use crate::lang::{self, Lang};
use crate::output::check_output::{
    CheckCommandOutput, CheckDoctorSection, CheckLintSection, CheckScanSection, CheckSummary,
};
use crate::output::doctor_output::{DoctorCheck, DoctorCheckStatus, DoctorSummary};
use crate::output::lint_output::{LintRuleSetting, LintRuleSeverity, LintSummary};
use crate::output::{OutputStats, OutputStatus, print_json};
use crate::project::{
    INDEX_RELATIVE_PATH, empty_project_index, index_write_failed_diagnostic, scan_project,
    write_project_index,
};
use crate::utils::root_not_exists;

pub fn run(args: CommonArgs, lang: Lang) -> Result<i32> {
    let started_at = Instant::now();
    let mut root_diagnostics = Vec::new();

    if root_not_exists(&mut root_diagnostics, &args.root, lang) {
        let output = empty_output(started_at, root_diagnostics);
        if args.json {
            print_json(&output)?;
        } else {
            println!("{}", lang::text(lang, "cli.check.root-invalid"));
        }
        return Ok(1);
    }

    let project_scan = scan_project(&args.root, lang)?;
    let mut scan_diagnostics = project_scan.diagnostics.clone();
    let mut index_path = None;

    if let Err(error) = write_project_index(&project_scan.root, &project_scan.index) {
        scan_diagnostics.push(index_write_failed_diagnostic(lang, &error));
    } else {
        index_path = Some(INDEX_RELATIVE_PATH.to_string());
    }

    let lint_evaluation = lint::evaluate(&project_scan, lang);
    let doctor_evaluation = doctor::evaluate(&project_scan, lang);

    let lint_summary = build_lint_summary(&lint_evaluation.diagnostics, &lint_evaluation.rules);
    let doctor_summary =
        build_doctor_summary(&doctor_evaluation.diagnostics, &doctor_evaluation.checks);

    let scan_section = CheckScanSection {
        status: status_from_diagnostics(&scan_diagnostics),
        diagnostics: scan_diagnostics.clone(),
        index_path,
        files_scanned: project_scan.files_scanned,
        directories_scanned: project_scan.directories_scanned,
        commands: project_scan.index.summary.commands,
        subcommands: project_scan.index.summary.subcommands,
        subcommand_groups: project_scan.index.summary.subcommand_groups,
    };

    let lint_section = CheckLintSection {
        status: status_from_error_count(lint_summary.errors),
        diagnostics: lint_evaluation.diagnostics.clone(),
        summary: lint_summary,
        rules: lint_evaluation.rules,
    };

    let doctor_section = CheckDoctorSection {
        status: status_from_error_count(doctor_summary.errors),
        diagnostics: doctor_evaluation.diagnostics.clone(),
        summary: doctor_summary,
        checks: doctor_evaluation.checks,
    };

    let mut diagnostics = Vec::new();
    diagnostics.extend(scan_section.diagnostics.clone());
    diagnostics.extend(lint_section.diagnostics.clone());
    diagnostics.extend(doctor_section.diagnostics.clone());

    let summary = CheckSummary {
        scan_errors: count_errors(&scan_section.diagnostics),
        lint_errors: lint_section.summary.errors,
        doctor_errors: doctor_section.summary.errors,
        errors: count_errors(&diagnostics),
        warnings: count_warnings(&diagnostics),
        infos: count_infos(&diagnostics),
    };

    let output = CheckCommandOutput {
        status: status_from_error_count(summary.errors),
        command: "check".to_string(),
        diagnostics,
        stats: OutputStats {
            files_scanned: project_scan.files_scanned,
            directories_scanned: project_scan.directories_scanned,
            duration_ms: started_at.elapsed().as_millis(),
        },
        summary,
        scan: scan_section,
        lint: lint_section,
        doctor: doctor_section,
    };

    if args.json {
        print_json(&output)?;
    } else {
        let errors = output.summary.errors.to_string();
        let warnings = output.summary.warnings.to_string();
        let infos = output.summary.infos.to_string();
        println!(
            "{}",
            lang::message(
                lang,
                "cli.check.completed",
                &[
                    ("errors", errors.as_str()),
                    ("warnings", warnings.as_str()),
                    ("infos", infos.as_str()),
                ],
            )
        );
    }

    Ok(if output.summary.errors > 0 { 1 } else { 0 })
}

fn empty_output(started_at: Instant, diagnostics: Vec<Diagnostic>) -> CheckCommandOutput {
    let scan_index = empty_project_index();
    let summary = CheckSummary {
        errors: count_errors(&diagnostics),
        warnings: count_warnings(&diagnostics),
        infos: count_infos(&diagnostics),
        scan_errors: count_errors(&diagnostics),
        lint_errors: 0,
        doctor_errors: 0,
    };

    CheckCommandOutput {
        status: OutputStatus::Failed,
        command: "check".to_string(),
        diagnostics: diagnostics.clone(),
        stats: OutputStats {
            files_scanned: 0,
            directories_scanned: 0,
            duration_ms: started_at.elapsed().as_millis(),
        },
        summary,
        scan: CheckScanSection {
            status: OutputStatus::Failed,
            diagnostics,
            index_path: None,
            files_scanned: 0,
            directories_scanned: 0,
            commands: scan_index.summary.commands,
            subcommands: scan_index.summary.subcommands,
            subcommand_groups: scan_index.summary.subcommand_groups,
        },
        lint: CheckLintSection {
            status: OutputStatus::Ok,
            diagnostics: Vec::new(),
            summary: LintSummary {
                rules_run: 0,
                rules_off: 0,
                errors: 0,
                warnings: 0,
                infos: 0,
            },
            rules: Vec::new(),
        },
        doctor: CheckDoctorSection {
            status: OutputStatus::Ok,
            diagnostics: Vec::new(),
            summary: DoctorSummary {
                checks_run: 0,
                ok: 0,
                warnings: 0,
                errors: 0,
                skipped: 0,
            },
            checks: Vec::new(),
        },
    }
}

fn build_lint_summary(diagnostics: &[Diagnostic], rules: &[LintRuleSetting]) -> LintSummary {
    LintSummary {
        rules_run: rules
            .iter()
            .filter(|rule| rule.severity != LintRuleSeverity::Off)
            .count(),
        rules_off: rules
            .iter()
            .filter(|rule| rule.severity == LintRuleSeverity::Off)
            .count(),
        errors: count_errors(diagnostics),
        warnings: count_warnings(diagnostics),
        infos: count_infos(diagnostics),
    }
}

fn build_doctor_summary(diagnostics: &[Diagnostic], checks: &[DoctorCheck]) -> DoctorSummary {
    let diagnostic_errors = count_errors(diagnostics);
    let diagnostic_warnings = count_warnings(diagnostics);
    let check_errors = checks
        .iter()
        .filter(|check| check.status == DoctorCheckStatus::Error)
        .count();
    let check_warnings = checks
        .iter()
        .filter(|check| check.status == DoctorCheckStatus::Warning)
        .count();

    DoctorSummary {
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
    }
}

fn status_from_diagnostics(diagnostics: &[Diagnostic]) -> OutputStatus {
    status_from_error_count(count_errors(diagnostics))
}

fn status_from_error_count(errors: usize) -> OutputStatus {
    if errors > 0 {
        OutputStatus::Failed
    } else {
        OutputStatus::Ok
    }
}

fn count_errors(diagnostics: &[Diagnostic]) -> usize {
    diagnostics
        .iter()
        .filter(|diagnostic| matches!(diagnostic.level, DiagnosticLevel::Error))
        .count()
}

fn count_warnings(diagnostics: &[Diagnostic]) -> usize {
    diagnostics
        .iter()
        .filter(|diagnostic| matches!(diagnostic.level, DiagnosticLevel::Warning))
        .count()
}

fn count_infos(diagnostics: &[Diagnostic]) -> usize {
    diagnostics
        .iter()
        .filter(|diagnostic| matches!(diagnostic.level, DiagnosticLevel::Info))
        .count()
}
