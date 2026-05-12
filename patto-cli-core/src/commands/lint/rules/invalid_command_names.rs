use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::output::lint_output::LintRuleSeverity;
use crate::output::scan_output::{CommandIndex, CommandKind};

use super::super::context::RuleContext;

pub fn run(context: &RuleContext<'_>, severity: LintRuleSeverity) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for command in context
        .project
        .index
        .commands
        .iter()
        .filter(|command| command.kind != CommandKind::Unknown)
    {
        for (property, value) in command_name_parts(command) {
            if is_valid_discord_name(value) {
                continue;
            }

            let mut diagnostic = context.diagnostic(
                DiagnosticCode::PATTO_LINT_INVALID_COMMAND_NAMES,
                severity,
                "invalid-command-names.message",
                &[("value", value)],
            );

            if let Some((line, column)) =
                context.location_for_value(&command.metadata_file, property, value)
            {
                diagnostic = diagnostic.with_location(&command.metadata_file, line, column);
            } else {
                diagnostic = diagnostic.with_location(&command.metadata_file, 1, 1);
            }

            diagnostics.push(diagnostic);
        }
    }

    diagnostics
}

fn command_name_parts(command: &CommandIndex) -> Vec<(&'static str, &str)> {
    let mut parts = Vec::new();

    if let Some(parent) = command.parent.as_deref() {
        parts.push(("parent", parent));
    }
    if let Some(group) = command.group.as_deref() {
        parts.push(("name", group));
    }
    if command.kind != CommandKind::SubcommandGroup {
        if let Some(name) = command.name.as_deref() {
            parts.push(("name", name));
        }
    }
    if let Some(subcommand) = command.subcommand.as_deref() {
        parts.push(("subcommand", subcommand));
    }

    parts
}

fn is_valid_discord_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 32
        && value.chars().all(|character| {
            character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || character == '-'
                || character == '_'
        })
}
