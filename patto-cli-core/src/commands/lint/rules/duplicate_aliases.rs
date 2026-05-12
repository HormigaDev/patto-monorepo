use std::collections::{HashMap, HashSet};

use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::output::lint_output::LintRuleSeverity;
use crate::output::scan_output::{CommandIndex, CommandKind};

use super::super::context::RuleContext;

pub fn run(context: &RuleContext<'_>, severity: LintRuleSeverity) -> Vec<Diagnostic> {
    let command_names = context
        .project
        .index
        .commands
        .iter()
        .filter(|command| command.kind == CommandKind::Command)
        .filter_map(|command| command.name.as_deref())
        .map(str::to_string)
        .collect::<HashSet<_>>();
    let mut by_alias = HashMap::<String, Vec<&CommandIndex>>::new();

    for command in context
        .project
        .index
        .commands
        .iter()
        .filter(|command| command.kind == CommandKind::Command)
    {
        for alias in &command.aliases {
            by_alias
                .entry(alias.to_ascii_lowercase())
                .or_default()
                .push(command);
        }
    }

    let mut diagnostics = Vec::new();

    for (alias, owners) in by_alias {
        if owners.len() > 1 {
            for command in owners {
                diagnostics.push(alias_diagnostic(
                    context,
                    command,
                    &alias,
                    severity,
                    "duplicate-aliases.duplicate.message",
                ));
            }
            continue;
        }

        if command_names.contains(&alias) {
            diagnostics.push(alias_diagnostic(
                context,
                owners[0],
                &alias,
                severity,
                "duplicate-aliases.name-conflict.message",
            ));
        }
    }

    diagnostics
}

fn alias_diagnostic(
    context: &RuleContext<'_>,
    command: &CommandIndex,
    alias: &str,
    severity: LintRuleSeverity,
    message_key: &str,
) -> Diagnostic {
    let mut diagnostic = context.diagnostic(
        DiagnosticCode::PATTO_LINT_DUPLICATE_ALIASES,
        severity,
        message_key,
        &[("alias", alias)],
    );

    if let Some((line, column)) = context.location_for_text(&command.metadata_file, alias) {
        diagnostic = diagnostic.with_location(&command.metadata_file, line, column);
    } else {
        diagnostic = diagnostic.with_location(&command.metadata_file, 1, 1);
    }

    diagnostic
}
