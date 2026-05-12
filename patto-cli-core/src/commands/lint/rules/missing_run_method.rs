use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::output::lint_output::LintRuleSeverity;
use crate::output::scan_output::CommandKind;

use super::super::context::RuleContext;

pub fn run(context: &RuleContext<'_>, severity: LintRuleSeverity) -> Vec<Diagnostic> {
    context
        .project
        .index
        .commands
        .iter()
        .filter(|command| command.kind != CommandKind::Unknown)
        .filter(|command| command.file.starts_with("src/commands/"))
        .filter(|command| !command.has_run_method)
        .map(|command| {
            let mut diagnostic = context.diagnostic(
                DiagnosticCode::PATTO_LINT_MISSING_RUN_METHOD,
                severity,
                "missing-run-method.message",
                &[("file", command.file.as_str())],
            );

            if let Some(class_name) = &command.class_name {
                if let Some((line, column)) = context.location_for_text(&command.file, class_name) {
                    diagnostic = diagnostic.with_location(&command.file, line, column);
                }
            } else {
                diagnostic = diagnostic.with_location(&command.file, 1, 1);
            }

            diagnostic
        })
        .collect()
}
