use std::collections::HashSet;

use crate::diagnostic::{DiagnosticCode, DiagnosticLevel};
use crate::lang;
use crate::output::doctor_output::DoctorCheckStatus;

use super::super::context::DoctorContext;
use super::CheckResult;

pub fn run(context: &DoctorContext<'_>) -> CheckResult {
    let package = &context.project.index.package;
    let mut diagnostics = Vec::new();
    let mut details = Vec::new();
    let mut status = DoctorCheckStatus::Ok;
    let title = lang::text(context.locale, "doctor-package-json.check.title");

    if !package.exists {
        diagnostics.push(context.diagnostic(
            DiagnosticCode::PATTO_DOCTOR_PACKAGE_JSON,
            DiagnosticLevel::Error,
            "doctor-package-json.missing.message",
            &[],
        ));
        return CheckResult {
            check: context.check(
                "package-json",
                DoctorCheckStatus::Error,
                title.clone(),
                details,
            ),
            diagnostics,
        };
    }

    let unknown_name = lang::text(context.locale, "common.unnamed");
    let unknown_version = lang::text(context.locale, "common.unknown-version");
    let name = package.name.as_deref().unwrap_or(unknown_name.as_str());
    let version = package
        .version
        .as_deref()
        .unwrap_or(unknown_version.as_str());
    details.push(lang::message(
        context.locale,
        "doctor-package-json.detail.package",
        &[("name", name), ("version", version)],
    ));

    let dependencies = package
        .dependencies
        .iter()
        .chain(package.dev_dependencies.iter())
        .cloned()
        .collect::<HashSet<_>>();
    for required in ["discord.js", "dotenv", "reflect-metadata", "typescript"] {
        if !dependencies.contains(required) {
            status = DoctorCheckStatus::Error;
            diagnostics.push(context.diagnostic(
                DiagnosticCode::PATTO_DOCTOR_PACKAGE_JSON,
                DiagnosticLevel::Error,
                "doctor-package-json.missing-dependency.message",
                &[("dependency", required)],
            ));
        }
    }

    for script in ["dev", "build", "start", "test", "lint"] {
        if !package.scripts.iter().any(|candidate| candidate == script) {
            status = downgrade_to_warning(status);
            diagnostics.push(context.diagnostic(
                DiagnosticCode::PATTO_DOCTOR_PACKAGE_JSON,
                DiagnosticLevel::Warning,
                "doctor-package-json.missing-script.message",
                &[("script", script)],
            ));
        }
    }

    let scripts = package.scripts.join(", ");
    details.push(lang::message(
        context.locale,
        "doctor-package-json.detail.scripts",
        &[("scripts", scripts.as_str())],
    ));

    CheckResult {
        check: context.check("package-json", status, title, details),
        diagnostics,
    }
}

fn downgrade_to_warning(status: DoctorCheckStatus) -> DoctorCheckStatus {
    match status {
        DoctorCheckStatus::Ok => DoctorCheckStatus::Warning,
        other => other,
    }
}
