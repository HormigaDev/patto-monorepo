use std::collections::BTreeSet;

use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::output::lint_output::LintRuleSeverity;

use super::super::super::context::RuleContext;
use super::locale_files::collect_locale_files;

pub fn run(context: &RuleContext<'_>, severity: LintRuleSeverity) -> Vec<Diagnostic> {
    if !context.feature_enabled("i18n") {
        return Vec::new();
    }

    let locales = collect_locale_files(context);
    if locales.len() < 2 {
        return Vec::new();
    }

    let mut all_keys = BTreeSet::new();
    for locale in &locales {
        all_keys.extend(locale.keys.iter().cloned());
    }

    let mut diagnostics = Vec::new();
    for locale in &locales {
        for key in all_keys.difference(&locale.keys) {
            diagnostics.push(
                context
                    .diagnostic(
                        DiagnosticCode::PATTO_LINT_I18N_LOCALE_PARITY,
                        severity,
                        "i18n-locale-parity.message",
                        &[("key", key.as_str()), ("locale", locale.locale.as_str())],
                    )
                    .with_location(locale.file.as_str(), 1, 1),
            );
        }
    }

    diagnostics
}
