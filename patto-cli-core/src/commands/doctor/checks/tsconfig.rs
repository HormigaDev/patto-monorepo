use crate::diagnostic::{DiagnosticCode, DiagnosticLevel};
use crate::lang;
use crate::output::doctor_output::DoctorCheckStatus;

use super::super::context::DoctorContext;
use super::CheckResult;

pub fn run(context: &DoctorContext<'_>) -> CheckResult {
    let mut diagnostics = Vec::new();
    let mut details = Vec::new();
    let mut status = DoctorCheckStatus::Ok;
    let title = lang::text(context.locale, "doctor-tsconfig.check.title");
    let Some(tsconfig) = context.read_json("tsconfig.json") else {
        diagnostics.push(context.diagnostic(
            DiagnosticCode::PATTO_DOCTOR_TSCONFIG,
            DiagnosticLevel::Error,
            "doctor-tsconfig.missing.message",
            &[],
        ));
        return CheckResult {
            check: context.check("tsconfig", DoctorCheckStatus::Error, title.clone(), details),
            diagnostics,
        };
    };

    let compiler = tsconfig
        .get("compilerOptions")
        .unwrap_or(&serde_json::Value::Null);
    for key in ["experimentalDecorators", "emitDecoratorMetadata"] {
        if compiler.get(key).and_then(|value| value.as_bool()) != Some(true) {
            status = DoctorCheckStatus::Error;
            diagnostics.push(context.diagnostic(
                DiagnosticCode::PATTO_DOCTOR_TSCONFIG,
                DiagnosticLevel::Error,
                "doctor-tsconfig.option-required.message",
                &[("key", key)],
            ));
        }
    }

    let has_alias = compiler
        .get("paths")
        .and_then(|paths| paths.get("@/*"))
        .is_some();
    if !has_alias {
        status = DoctorCheckStatus::Error;
        diagnostics.push(context.diagnostic(
            DiagnosticCode::PATTO_DOCTOR_TSCONFIG,
            DiagnosticLevel::Error,
            "doctor-tsconfig.alias-missing.message",
            &[],
        ));
    }

    details.push(lang::text(context.locale, "doctor-tsconfig.detail.read"));
    CheckResult {
        check: context.check("tsconfig", status, title, details),
        diagnostics,
    }
}
