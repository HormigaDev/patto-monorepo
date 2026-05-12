use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::output::lint_output::LintRuleSeverity;
use crate::utils;

use super::super::context::RuleContext;

pub fn run(context: &RuleContext<'_>, severity: LintRuleSeverity) -> Vec<Diagnostic> {
    let file = "src/config/plugins.config.ts";
    let Some(content) = context.read_file(file) else {
        return Vec::new();
    };
    let content = utils::strip_line_comments(&content);

    let mut diagnostics = Vec::new();
    let mut search_start = 0;

    while let Some(relative_start) = content[search_start..].find("PluginScope.Specified") {
        let start = search_start + relative_start;
        let block_end = content[start..]
            .find("});")
            .map(|offset| start + offset)
            .unwrap_or(content.len());
        let block = &content[start..block_end];
        let has_commands = block.contains("commands:")
            && !block.contains("commands: []")
            && !block.contains("commands:[]");

        if !has_commands {
            let mut diagnostic = context.diagnostic(
                DiagnosticCode::PATTO_LINT_PLUGIN_SPECIFIED_COMMANDS,
                severity,
                "plugin-specified-commands.message",
                &[],
            );
            if let Some((line, column)) = context.location_for_text(file, "PluginScope.Specified") {
                diagnostic = diagnostic.with_location(file, line, column);
            }
            diagnostics.push(diagnostic);
        }

        search_start = block_end.saturating_add(3);
    }

    diagnostics
}
