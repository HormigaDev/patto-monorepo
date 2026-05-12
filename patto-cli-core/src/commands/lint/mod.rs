mod config;
mod context;
mod rules;

use anyhow::Result;
use std::time::Instant;

use crate::cli::CommonArgs;
use crate::diagnostic::{Diagnostic, DiagnosticLevel};
use crate::lang;
use crate::output::lint_output::{
    LintCommandOutput, LintRuleSetting, LintRuleSeverity, LintSummary,
};
use crate::output::{OutputStats, OutputStatus, print_json};
use crate::project::{ProjectScan, scan_project};
use crate::utils::root_not_exists;

use self::config::resolve_rule_config;
use self::context::RuleContext;

pub(crate) struct LintEvaluation {
    pub diagnostics: Vec<Diagnostic>,
    pub rules: Vec<LintRuleSetting>,
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
            println!("{}", lang::text(lang, "cli.lint.root-invalid"));
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
        evaluation.rules,
    );

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
                "cli.lint.completed",
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

pub(crate) fn evaluate(project_scan: &ProjectScan, lang: crate::lang::Lang) -> LintEvaluation {
    let (rule_config, rule_config_diagnostics) =
        resolve_rule_config(project_scan.config_json.as_ref(), &project_scan.root, lang);
    let context = RuleContext {
        project: project_scan,
        locale: lang,
    };

    let mut diagnostics = rule_config_diagnostics;
    diagnostics.extend(rules::run_enabled_rules(&context, &rule_config));

    LintEvaluation {
        diagnostics,
        rules: rule_config.to_output(),
    }
}

fn build_output(
    diagnostics: Vec<Diagnostic>,
    stats: OutputStats,
    rules: Vec<LintRuleSetting>,
) -> LintCommandOutput {
    let summary = LintSummary {
        rules_run: rules
            .iter()
            .filter(|rule| rule.severity != LintRuleSeverity::Off)
            .count(),
        rules_off: rules
            .iter()
            .filter(|rule| rule.severity == LintRuleSeverity::Off)
            .count(),
        errors: diagnostics
            .iter()
            .filter(|diagnostic| matches!(diagnostic.level, DiagnosticLevel::Error))
            .count(),
        warnings: diagnostics
            .iter()
            .filter(|diagnostic| matches!(diagnostic.level, DiagnosticLevel::Warning))
            .count(),
        infos: diagnostics
            .iter()
            .filter(|diagnostic| matches!(diagnostic.level, DiagnosticLevel::Info))
            .count(),
    };

    LintCommandOutput {
        status: if summary.errors > 0 {
            OutputStatus::Failed
        } else {
            OutputStatus::Ok
        },
        command: "lint".to_string(),
        diagnostics,
        stats,
        summary,
        rules,
    }
}
