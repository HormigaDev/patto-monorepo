use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::output::lint_output::LintRuleSeverity;
use crate::output::scan_output::CommandKind;

use super::super::context::RuleContext;

pub fn run(context: &RuleContext<'_>, severity: LintRuleSeverity) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for command in &context.project.index.commands {
        match command.kind {
            CommandKind::Subcommand => {
                if command.parent.as_deref().unwrap_or_default().is_empty()
                    || command.name.as_deref().unwrap_or_default().is_empty()
                {
                    diagnostics.push(inconsistent(context, command, severity));
                }
            }
            CommandKind::SubcommandGroup => {
                if command.parent.as_deref().unwrap_or_default().is_empty()
                    || command.group.as_deref().unwrap_or_default().is_empty()
                    || command.subcommand.as_deref().unwrap_or_default().is_empty()
                {
                    diagnostics.push(inconsistent(context, command, severity));
                }
            }
            CommandKind::Command | CommandKind::Unknown => {}
        }
    }

    diagnostics
}

fn inconsistent(
    context: &RuleContext<'_>,
    command: &crate::output::scan_output::CommandIndex,
    severity: LintRuleSeverity,
) -> Diagnostic {
    let diagnostic = context.diagnostic(
        DiagnosticCode::PATTO_LINT_SUBCOMMAND_CONSISTENCY,
        severity,
        "subcommand-consistency.message",
        &[("file", command.file.as_str())],
    );
    context.attach_command_location(diagnostic, command)
}
