use crate::diagnostic::{DiagnosticCode, DiagnosticLevel};
use crate::lang;
use crate::output::doctor_output::DoctorCheckStatus;

use super::super::context::DoctorContext;
use super::CheckResult;

pub fn run(context: &DoctorContext<'_>) -> CheckResult {
    let package = &context.project.index.package;
    let mut diagnostics = Vec::new();
    let mut details = Vec::new();
    let title = lang::text(context.locale, "doctor-build-output.check.title");

    let Some(main) = package.main.as_deref() else {
        return CheckResult {
            check: context.check(
                "build-output",
                DoctorCheckStatus::Skipped,
                title.clone(),
                vec![lang::text(
                    context.locale,
                    "doctor-build-output.detail.main-missing",
                )],
            ),
            diagnostics,
        };
    };

    details.push(lang::message(
        context.locale,
        "doctor-build-output.detail.main",
        &[("main", main)],
    ));
    if context.root().join(main).is_file() {
        return CheckResult {
            check: context.check(
                "build-output",
                DoctorCheckStatus::Ok,
                title.clone(),
                details,
            ),
            diagnostics,
        };
    }

    diagnostics.push(context.diagnostic(
        DiagnosticCode::PATTO_DOCTOR_BUILD_OUTPUT,
        DiagnosticLevel::Info,
        "doctor-build-output.missing.message",
        &[("main", main)],
    ));

    CheckResult {
        check: context.check("build-output", DoctorCheckStatus::Warning, title, details),
        diagnostics,
    }
}
