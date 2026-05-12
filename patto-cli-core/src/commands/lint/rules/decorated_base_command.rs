use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::lang;
use crate::output::lint_output::LintRuleSeverity;
use crate::output::scan_output::CommandKind;

use super::super::context::RuleContext;

pub fn run(context: &RuleContext<'_>, severity: LintRuleSeverity) -> Vec<Diagnostic> {
    let unknown_class = lang::text(context.locale, "common.unknown-class");
    context
        .project
        .index
        .commands
        .iter()
        .filter(|command| command.kind != CommandKind::Unknown)
        .filter(|command| !command.has_base_command_ancestor)
        .map(|command| {
            let diagnostic = context.diagnostic(
                DiagnosticCode::PATTO_LINT_DECORATED_BASE_COMMAND,
                severity,
                "decorated-base-command.message",
                &[(
                    "class",
                    command
                        .class_name
                        .as_deref()
                        .unwrap_or(unknown_class.as_str()),
                )],
            );
            context.attach_command_location(diagnostic, command)
        })
        .collect()
}
