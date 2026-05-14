use std::collections::BTreeSet;

use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::output::lint_output::LintRuleSeverity;
use crate::utils;

use super::super::super::context::RuleContext;
use super::locale_files::collect_locale_files;
use super::source_files::source_files;
use super::translation_calls::extract_translation_calls;

pub fn run(context: &RuleContext<'_>, severity: LintRuleSeverity) -> Vec<Diagnostic> {
    if !context.feature_enabled("i18n") {
        return Vec::new();
    }

    let locales = collect_locale_files(context);
    if locales.is_empty() {
        return Vec::new();
    }

    let mut diagnostics = Vec::new();
    let mut reported = BTreeSet::new();

    for file in source_files(context) {
        let Some(content) = context.read_file(&file) else {
            continue;
        };
        let calls = extract_translation_calls(&content);

        for call in calls.static_calls {
            for locale in &locales {
                if locale.keys.contains(&call.key) {
                    continue;
                }

                let marker = (file.clone(), call.key.clone(), locale.locale.clone());
                if !reported.insert(marker) {
                    continue;
                }

                let (line, column) = utils::offset_to_line_column(&content, call.offset);
                diagnostics.push(
                    context
                        .diagnostic(
                            DiagnosticCode::PATTO_LINT_I18N_MISSING_KEYS,
                            severity,
                            "i18n-missing-keys.message",
                            &[
                                ("key", call.key.as_str()),
                                ("locale", locale.locale.as_str()),
                                ("file", locale.file.as_str()),
                            ],
                        )
                        .with_location(file.as_str(), line, column),
                );
            }
        }
    }

    diagnostics
}
