use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::output::lint_output::LintRuleSeverity;

use super::super::context::RuleContext;

pub fn run(context: &RuleContext<'_>, severity: LintRuleSeverity) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for command in context
        .project
        .index
        .commands
        .iter()
        .filter(|command| command.file.starts_with("src/commands/"))
    {
        let Some(content) = context.read_file(&command.file) else {
            continue;
        };

        for method in extract_component_methods(&content) {
            if has_static_method(&content, &method) {
                continue;
            }

            let mut diagnostic = context.diagnostic(
                DiagnosticCode::PATTO_LINT_COMPONENT_HANDLER_METHODS,
                severity,
                "component-handler-methods.message",
                &[("method", method.as_str())],
            );
            if let Some((line, column)) = context.location_for_text(&command.file, &method) {
                diagnostic = diagnostic.with_location(&command.file, line, column);
            } else {
                diagnostic = diagnostic.with_location(&command.file, 1, 1);
            }
            diagnostics.push(diagnostic);
        }
    }

    diagnostics
}

fn extract_component_methods(content: &str) -> Vec<String> {
    let mut methods = Vec::new();
    let mut search_start = 0;

    while let Some(relative_start) = content[search_start..].find("method:") {
        let start = search_start + relative_start + "method:".len();
        let tail = content[start..].trim_start();
        let Some(method) = read_quoted(tail) else {
            search_start = start;
            continue;
        };
        methods.push(method);
        search_start = start;
    }

    methods.sort();
    methods.dedup();
    methods
}

fn read_quoted(value: &str) -> Option<String> {
    let quote = value.chars().next()?;
    if quote != '\'' && quote != '"' && quote != '`' {
        return None;
    }
    let end = value[quote.len_utf8()..].find(quote)? + quote.len_utf8();
    Some(value[quote.len_utf8()..end].to_string())
}

fn has_static_method(content: &str, method: &str) -> bool {
    content.contains(&format!("static async {method}("))
        || content.contains(&format!("static {method}("))
        || content.contains(&format!("public static async {method}("))
        || content.contains(&format!("public static {method}("))
}
