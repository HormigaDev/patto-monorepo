use crate::diagnostic::{DiagnosticCode, DiagnosticLevel};
use crate::lang;
use crate::output::doctor_output::DoctorCheckStatus;
use crate::project::CONFIG_RELATIVE_PATH;

use super::super::context::DoctorContext;
use super::CheckResult;

pub fn run(context: &DoctorContext<'_>) -> CheckResult {
    let config = &context.project.index.config;
    let mut diagnostics = Vec::new();
    let mut details = Vec::new();
    let mut status = DoctorCheckStatus::Ok;
    let title = lang::text(context.locale, "doctor-project-config.check.title");

    if !config.exists {
        status = DoctorCheckStatus::Warning;
        diagnostics.push(context.diagnostic(
            DiagnosticCode::PATTO_DOCTOR_PROJECT_CONFIG,
            DiagnosticLevel::Warning,
            "doctor-project-config.missing.message",
            &[("path", CONFIG_RELATIVE_PATH)],
        ));
    } else if context.project.config_json.is_none() {
        status = DoctorCheckStatus::Warning;
        diagnostics.push(context.diagnostic(
            DiagnosticCode::PATTO_DOCTOR_PROJECT_CONFIG,
            DiagnosticLevel::Warning,
            "doctor-project-config.invalid-json.message",
            &[("path", CONFIG_RELATIVE_PATH)],
        ));
    } else {
        details.push(lang::message(
            context.locale,
            "doctor-project-config.detail.lang",
            &[("lang", config.lang.as_str())],
        ));
    }

    if !config.supported_lang {
        status = DoctorCheckStatus::Warning;
        diagnostics.push(context.diagnostic(
            DiagnosticCode::PATTO_DOCTOR_PROJECT_CONFIG,
            DiagnosticLevel::Warning,
            "doctor-project-config.unsupported-lang.message",
            &[("lang", config.lang.as_str())],
        ));
    }

    CheckResult {
        check: context.check("project-config", status, title, details),
        diagnostics,
    }
}
