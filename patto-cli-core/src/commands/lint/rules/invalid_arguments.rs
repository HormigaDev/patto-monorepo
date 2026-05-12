use std::collections::HashSet;

use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::lang;
use crate::output::lint_output::LintRuleSeverity;
use crate::output::scan_output::{ArgumentIndex, ArgumentOptionValueKind, CommandIndex};

use super::super::context::RuleContext;

pub fn run(context: &RuleContext<'_>, severity: LintRuleSeverity) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for command in &context.project.index.commands {
        diagnostics.extend(check_duplicate_names(context, command, severity));
        diagnostics.extend(check_required_order(context, command, severity));
        diagnostics.extend(check_raw_text_position(context, command, severity));
        diagnostics.extend(check_choice_types(context, command, severity));
    }

    diagnostics
}

fn check_duplicate_names(
    context: &RuleContext<'_>,
    command: &CommandIndex,
    severity: LintRuleSeverity,
) -> Vec<Diagnostic> {
    let mut seen = HashSet::new();
    let mut diagnostics = Vec::new();

    for argument in &command.arguments {
        let Some(name) = argument.name.as_deref() else {
            continue;
        };
        if !seen.insert(name.to_string()) {
            diagnostics.push(argument_diagnostic(
                context,
                command,
                argument,
                severity,
                "invalid-arguments.duplicate.message",
                &[("name", name)],
            ));
        }
    }

    diagnostics
}

fn check_required_order(
    context: &RuleContext<'_>,
    command: &CommandIndex,
    severity: LintRuleSeverity,
) -> Vec<Diagnostic> {
    let mut seen_optional = false;
    let mut diagnostics = Vec::new();
    let unnamed = lang::text(context.locale, "common.unnamed");

    for argument in &command.arguments {
        if !argument.required {
            seen_optional = true;
            continue;
        }
        if seen_optional {
            diagnostics.push(argument_diagnostic(
                context,
                command,
                argument,
                severity,
                "invalid-arguments.required-order.message",
                &[("name", argument.name.as_deref().unwrap_or(unnamed.as_str()))],
            ));
        }
    }

    diagnostics
}

fn check_raw_text_position(
    context: &RuleContext<'_>,
    command: &CommandIndex,
    severity: LintRuleSeverity,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let unnamed = lang::text(context.locale, "common.unnamed");

    for (index, argument) in command.arguments.iter().enumerate() {
        if argument.raw_text && index + 1 < command.arguments.len() {
            diagnostics.push(argument_diagnostic(
                context,
                command,
                argument,
                severity,
                "invalid-arguments.raw-text-position.message",
                &[("name", argument.name.as_deref().unwrap_or(unnamed.as_str()))],
            ));
        }
    }

    diagnostics
}

fn check_choice_types(
    context: &RuleContext<'_>,
    command: &CommandIndex,
    severity: LintRuleSeverity,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for argument in &command.arguments {
        let Some(type_hint) = &argument.type_hint else {
            continue;
        };
        let expects_number = type_hint.contains("Number") || type_hint.contains("Integer");
        let expects_string = type_hint.contains("String");

        for option in &argument.option_values {
            let invalid = (expects_number && option.kind != ArgumentOptionValueKind::Number)
                || (expects_string && option.kind != ArgumentOptionValueKind::String);
            if invalid {
                diagnostics.push(argument_diagnostic(
                    context,
                    command,
                    argument,
                    severity,
                    "invalid-arguments.choice-type.message",
                    &[
                        ("choice", option.raw.as_str()),
                        ("type", type_hint.as_str()),
                    ],
                ));
            }
        }
    }

    diagnostics
}

fn argument_diagnostic(
    context: &RuleContext<'_>,
    command: &CommandIndex,
    argument: &ArgumentIndex,
    severity: LintRuleSeverity,
    message_key: &str,
    args: &[(&str, &str)],
) -> Diagnostic {
    let mut diagnostic = context.diagnostic(
        DiagnosticCode::PATTO_LINT_INVALID_ARGUMENTS,
        severity,
        message_key,
        args,
    );

    if let Some(name) = argument.name.as_deref() {
        if let Some((line, column)) =
            context.location_for_value(&command.metadata_file, "name", name)
        {
            diagnostic = diagnostic.with_location(&command.metadata_file, line, column);
        } else {
            diagnostic = diagnostic.with_location(&command.metadata_file, 1, 1);
        }
    } else {
        diagnostic = diagnostic.with_location(&command.metadata_file, 1, 1);
    }

    diagnostic
}
