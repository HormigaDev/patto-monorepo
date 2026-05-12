use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::output::lint_output::LintRuleSeverity;
use crate::output::scan_output::{CommandIndex, CommandKind};

use super::super::context::RuleContext;

pub fn run(context: &RuleContext<'_>, severity: LintRuleSeverity) -> Vec<Diagnostic> {
    context
        .project
        .index
        .commands
        .iter()
        .filter(|command| command.file.starts_with("src/commands/"))
        .filter(|command| !command.file.contains("/examples/"))
        .filter(|command| !matches_convention(command))
        .map(|command| {
            let diagnostic = context.diagnostic(
                DiagnosticCode::PATTO_LINT_COMMAND_FOLDER_CONVENTION,
                severity,
                "command-folder-convention.message",
                &[("file", command.file.as_str())],
            );
            context.attach_command_location(diagnostic, command)
        })
        .collect()
}

fn matches_convention(command: &CommandIndex) -> bool {
    match command.kind {
        CommandKind::Command => command.file.matches('/').count() >= 2,
        CommandKind::Subcommand => command
            .parent
            .as_ref()
            .map(|parent| command.file.starts_with(&format!("src/commands/{parent}/")))
            .unwrap_or(false),
        CommandKind::SubcommandGroup => command
            .parent
            .as_ref()
            .zip(command.group.as_ref())
            .map(|(parent, group)| {
                command
                    .file
                    .starts_with(&format!("src/commands/{parent}/{group}/"))
            })
            .unwrap_or(false),
        CommandKind::Unknown => true,
    }
}
