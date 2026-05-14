use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::output::lint_output::LintRuleSeverity;
use crate::utils;

use super::super::super::context::RuleContext;
use super::source_files::source_files;
use super::translation_calls::extract_translation_calls;

pub fn run(context: &RuleContext<'_>, severity: LintRuleSeverity) -> Vec<Diagnostic> {
    if !context.feature_enabled("i18n") {
        return Vec::new();
    }

    let mut diagnostics = Vec::new();
    for file in source_files(context) {
        let Some(content) = context.read_file(&file) else {
            continue;
        };
        let calls = extract_translation_calls(&content);

        for offset in calls.dynamic_offsets {
            let (line, column) = utils::offset_to_line_column(&content, offset);
            diagnostics.push(
                context
                    .diagnostic(
                        DiagnosticCode::PATTO_LINT_I18N_DYNAMIC_KEYS,
                        severity,
                        "i18n-dynamic-keys.message",
                        &[],
                    )
                    .with_location(file.as_str(), line, column),
            );
        }
    }

    diagnostics
}
