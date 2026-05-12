use crate::diagnostic::{DiagnosticCode, DiagnosticLevel};
use crate::lang;
use crate::output::doctor_output::DoctorCheckStatus;

use super::super::context::DoctorContext;
use super::CheckResult;

pub fn run(context: &DoctorContext<'_>) -> CheckResult {
    let mut diagnostics = Vec::new();
    let mut details = Vec::new();
    let mut status = DoctorCheckStatus::Ok;
    let title = lang::text(context.locale, "doctor-env.check.title");

    let has_env = context.root().join(".env").is_file();
    let has_template = context.root().join(".env.template").is_file();

    if has_template {
        details.push(lang::text(
            context.locale,
            "doctor-env.detail.template-present",
        ));
    } else {
        status = DoctorCheckStatus::Warning;
        diagnostics.push(context.diagnostic(
            DiagnosticCode::PATTO_DOCTOR_ENV,
            DiagnosticLevel::Warning,
            "doctor-env.template-missing.message",
            &[],
        ));
    }

    if !has_env {
        status = DoctorCheckStatus::Warning;
        diagnostics.push(context.diagnostic(
            DiagnosticCode::PATTO_DOCTOR_ENV,
            DiagnosticLevel::Warning,
            "doctor-env.env-missing.message",
            &[],
        ));
        return CheckResult {
            check: context.check("env", status, title.clone(), details),
            diagnostics,
        };
    }

    details.push(lang::text(context.locale, "doctor-env.detail.env-present"));
    let env = context.read_env_file();
    for required in ["BOT_TOKEN", "CLIENT_ID"] {
        if env
            .get(required)
            .map(|value| value.trim().is_empty())
            .unwrap_or(true)
        {
            status = DoctorCheckStatus::Warning;
            diagnostics.push(context.diagnostic(
                DiagnosticCode::PATTO_DOCTOR_ENV,
                DiagnosticLevel::Warning,
                "doctor-env.required-missing.message",
                &[("name", required)],
            ));
        }
    }

    if env
        .get("USE_MESSAGE_CONTENT")
        .map(|value| value.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        details.push(lang::text(
            context.locale,
            "doctor-env.detail.message-content",
        ));
    }

    CheckResult {
        check: context.check("env", status, title, details),
        diagnostics,
    }
}
