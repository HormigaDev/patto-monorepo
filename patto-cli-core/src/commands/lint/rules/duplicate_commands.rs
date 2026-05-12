use std::collections::HashMap;

use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::output::lint_output::LintRuleSeverity;
use crate::output::scan_output::CommandKind;

use super::super::context::RuleContext;

pub fn run(context: &RuleContext<'_>, severity: LintRuleSeverity) -> Vec<Diagnostic> {
    let mut by_key = HashMap::new();

    for command in context
        .project
        .index
        .commands
        .iter()
        .filter(|command| command.kind != CommandKind::Unknown)
    {
        if let Some(key) = &command.key {
            by_key
                .entry(key.clone())
                .or_insert_with(Vec::new)
                .push(command);
        }
    }

    by_key
        .into_iter()
        .filter(|(_, commands)| commands.len() > 1)
        .flat_map(|(key, commands)| {
            commands
                .into_iter()
                .map(move |command| {
                    let diagnostic = context.diagnostic(
                        DiagnosticCode::PATTO_LINT_DUPLICATE_COMMANDS,
                        severity,
                        "duplicate-commands.message",
                        &[("key", key.as_str())],
                    );
                    context.attach_command_location(diagnostic, command)
                })
                .collect::<Vec<_>>()
        })
        .collect()
}
