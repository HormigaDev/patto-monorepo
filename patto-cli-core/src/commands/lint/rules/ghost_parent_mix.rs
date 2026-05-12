use std::collections::HashSet;

use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::lang;
use crate::output::lint_output::LintRuleSeverity;
use crate::output::scan_output::CommandKind;

use super::super::context::RuleContext;

pub fn run(context: &RuleContext<'_>, severity: LintRuleSeverity) -> Vec<Diagnostic> {
    let unnamed = lang::text(context.locale, "common.unnamed");
    let grouped_parents = context
        .project
        .index
        .commands
        .iter()
        .filter_map(|command| match command.kind {
            CommandKind::Subcommand | CommandKind::SubcommandGroup => command.parent.clone(),
            _ => None,
        })
        .collect::<HashSet<_>>();

    context
        .project
        .index
        .commands
        .iter()
        .filter(|command| command.kind == CommandKind::Command)
        .filter(|command| {
            command
                .name
                .as_ref()
                .map(|name| grouped_parents.contains(name))
                .unwrap_or(false)
        })
        .map(|command| {
            let diagnostic = context.diagnostic(
                DiagnosticCode::PATTO_LINT_GHOST_PARENT_MIX,
                severity,
                "ghost-parent-mix.message",
                &[("name", command.name.as_deref().unwrap_or(unnamed.as_str()))],
            );
            context.attach_command_location(diagnostic, command)
        })
        .collect()
}
