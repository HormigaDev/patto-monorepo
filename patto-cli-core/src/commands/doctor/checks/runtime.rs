use crate::diagnostic::{DiagnosticCode, DiagnosticLevel};
use crate::lang;
use crate::output::doctor_output::DoctorCheckStatus;
use crate::output::scan_output::PackageManager;

use super::super::context::DoctorContext;
use super::CheckResult;

pub fn run(context: &DoctorContext<'_>) -> CheckResult {
    let mut diagnostics = Vec::new();
    let mut details = Vec::new();
    let mut status = DoctorCheckStatus::Ok;
    let title = lang::text(context.locale, "doctor-runtime.check.title");

    match context.command_version("node", "--version") {
        Some(version) => {
            details.push(lang::message(
                context.locale,
                "doctor-runtime.detail.command-version",
                &[("command", "node"), ("version", version.as_str())],
            ));
            if parse_node_major(&version).unwrap_or(0) < 18 {
                status = DoctorCheckStatus::Error;
                diagnostics.push(context.diagnostic(
                    DiagnosticCode::PATTO_DOCTOR_RUNTIME,
                    DiagnosticLevel::Error,
                    "doctor-runtime.node-version.message",
                    &[("version", version.as_str())],
                ));
            }
        }
        None => {
            status = DoctorCheckStatus::Error;
            diagnostics.push(context.diagnostic(
                DiagnosticCode::PATTO_DOCTOR_RUNTIME,
                DiagnosticLevel::Error,
                "doctor-runtime.node-missing.message",
                &[],
            ));
        }
    }

    match context.project.index.package.package_manager {
        PackageManager::Npm => record_pm(
            context,
            "npm",
            "--version",
            &mut details,
            &mut status,
            &mut diagnostics,
        ),
        PackageManager::Pnpm => record_pm(
            context,
            "pnpm",
            "--version",
            &mut details,
            &mut status,
            &mut diagnostics,
        ),
        PackageManager::Yarn => record_pm(
            context,
            "yarn",
            "--version",
            &mut details,
            &mut status,
            &mut diagnostics,
        ),
        PackageManager::Unknown => {
            status = downgrade_to_warning(status);
            details.push(lang::text(
                context.locale,
                "doctor-runtime.detail.package-manager-unknown",
            ));
        }
    }

    CheckResult {
        check: context.check("runtime", status, title, details),
        diagnostics,
    }
}

fn record_pm(
    context: &DoctorContext<'_>,
    command: &str,
    arg: &str,
    details: &mut Vec<String>,
    status: &mut DoctorCheckStatus,
    diagnostics: &mut Vec<crate::diagnostic::Diagnostic>,
) {
    match context.command_version(command, arg) {
        Some(version) => details.push(lang::message(
            context.locale,
            "doctor-runtime.detail.command-version",
            &[("command", command), ("version", version.as_str())],
        )),
        None => {
            *status = DoctorCheckStatus::Error;
            diagnostics.push(context.diagnostic(
                DiagnosticCode::PATTO_DOCTOR_RUNTIME,
                DiagnosticLevel::Error,
                "doctor-runtime.package-manager-missing.message",
                &[("command", command)],
            ));
        }
    }
}

fn parse_node_major(version: &str) -> Option<u32> {
    version
        .trim_start_matches('v')
        .split('.')
        .next()?
        .parse()
        .ok()
}

fn downgrade_to_warning(status: DoctorCheckStatus) -> DoctorCheckStatus {
    match status {
        DoctorCheckStatus::Ok => DoctorCheckStatus::Warning,
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_node_major_accepts_node_version_formats() {
        assert_eq!(parse_node_major("v20.11.1"), Some(20));
        assert_eq!(parse_node_major("18.19.0"), Some(18));
    }

    #[test]
    fn parse_node_major_returns_none_for_invalid_versions() {
        assert_eq!(parse_node_major("not-a-version"), None);
        assert_eq!(parse_node_major(""), None);
    }

    #[test]
    fn downgrade_to_warning_keeps_errors() {
        assert_eq!(
            downgrade_to_warning(DoctorCheckStatus::Ok),
            DoctorCheckStatus::Warning
        );
        assert_eq!(
            downgrade_to_warning(DoctorCheckStatus::Error),
            DoctorCheckStatus::Error
        );
    }
}
