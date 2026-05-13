use crate::diagnostic::{Diagnostic, DiagnosticCode, DiagnosticLevel};
use crate::lang;
use crate::output::doctor_output::DoctorCheckStatus;
use crate::project::{CONFIG_RELATIVE_PATH, find_text_location};

use super::super::context::DoctorContext;
use super::CheckResult;

const REQUIRED_FILES: [&str; 10] = [
    "src/i18n/index.ts",
    "src/i18n/translator.ts",
    "src/i18n/types.ts",
    "src/i18n/registry.ts",
    "src/i18n/locale.resolver.ts",
    "src/i18n/store/locale.store.ts",
    "src/i18n/locale/es.ts",
    "src/i18n/locale/en.ts",
    "src/i18n/locale/pt.ts",
    "src/core/structures/BaseCommand.ts",
];

pub fn run(context: &DoctorContext<'_>) -> CheckResult {
    let title = lang::text(context.locale, "doctor-i18n.check.title");

    if !context.feature_enabled("i18n") {
        return CheckResult {
            check: context.check(
                "i18n",
                DoctorCheckStatus::Skipped,
                title,
                vec![lang::text(context.locale, "doctor-i18n.detail.disabled")],
            ),
            diagnostics: Vec::new(),
        };
    }

    let mut status = DoctorCheckStatus::Ok;
    let mut details = Vec::new();
    let mut diagnostics = Vec::new();

    for file in REQUIRED_FILES {
        if context.root().join(file).is_file() {
            details.push(lang::message(
                context.locale,
                "doctor-i18n.detail.file-present",
                &[("file", file)],
            ));
            continue;
        }

        status = DoctorCheckStatus::Warning;
        details.push(lang::message(
            context.locale,
            "doctor-i18n.detail.file-missing",
            &[("file", file)],
        ));
        diagnostics.push(with_i18n_config_location(
            context,
            context.diagnostic(
                DiagnosticCode::PATTO_DOCTOR_I18N,
                DiagnosticLevel::Warning,
                "doctor-i18n.missing-file.message",
                &[("file", file)],
            ),
        ));
    }

    if let Some(base_command) = context.read_file("src/core/structures/BaseCommand.ts") {
        if !base_command.contains("get t") || !base_command.contains("i18n.for") {
            status = DoctorCheckStatus::Warning;
            diagnostics.push(with_i18n_config_location(
                context,
                context.diagnostic(
                    DiagnosticCode::PATTO_DOCTOR_I18N,
                    DiagnosticLevel::Warning,
                    "doctor-i18n.base-command-helper.message",
                    &[],
                ),
            ));
        }
    }

    CheckResult {
        check: context.check("i18n", status, title, details),
        diagnostics,
    }
}

fn with_i18n_config_location(context: &DoctorContext<'_>, diagnostic: Diagnostic) -> Diagnostic {
    if let Some((line, column)) = find_text_location(context.root(), CONFIG_RELATIVE_PATH, "i18n") {
        diagnostic.with_location(CONFIG_RELATIVE_PATH, line, column)
    } else {
        diagnostic.with_location(CONFIG_RELATIVE_PATH, 1, 1)
    }
}
