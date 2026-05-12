use crate::diagnostic::{DiagnosticCode, DiagnosticLevel};
use crate::lang;
use crate::output::doctor_output::DoctorCheckStatus;

use super::super::context::DoctorContext;
use super::CheckResult;

pub fn run(context: &DoctorContext<'_>) -> CheckResult {
    let env = context.read_env_file();
    let sharding_enabled = env
        .get("SHARDING_ENABLED")
        .map(|value| value.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let mut diagnostics = Vec::new();
    let mut details = Vec::new();
    let title = lang::text(context.locale, "doctor-sharding-redis.check.title");

    if !sharding_enabled {
        return CheckResult {
            check: context.check(
                "sharding-redis",
                DoctorCheckStatus::Skipped,
                title.clone(),
                vec![lang::text(
                    context.locale,
                    "doctor-sharding-redis.detail.disabled",
                )],
            ),
            diagnostics,
        };
    }

    let mut status = DoctorCheckStatus::Ok;
    details.push(lang::text(
        context.locale,
        "doctor-sharding-redis.detail.enabled",
    ));

    if env
        .get("REDIS_URL")
        .map(|value| value.trim().is_empty())
        .unwrap_or(true)
    {
        status = DoctorCheckStatus::Error;
        diagnostics.push(context.diagnostic(
            DiagnosticCode::PATTO_DOCTOR_SHARDING_REDIS,
            DiagnosticLevel::Error,
            "doctor-sharding-redis.redis-url.message",
            &[],
        ));
    }

    for file in [
        "src/core/store/redis.cooldown.store.ts",
        "src/core/store/redis.payload.store.ts",
        "src/core/store/store.registry.ts",
    ] {
        if context.root().join(file).is_file() {
            details.push(lang::message(
                context.locale,
                "doctor-sharding-redis.detail.store-present",
                &[("file", file)],
            ));
        } else {
            status = DoctorCheckStatus::Error;
            diagnostics.push(context.diagnostic(
                DiagnosticCode::PATTO_DOCTOR_SHARDING_REDIS,
                DiagnosticLevel::Error,
                "doctor-sharding-redis.missing-store.message",
                &[("file", file)],
            ));
        }
    }

    CheckResult {
        check: context.check("sharding-redis", status, title, details),
        diagnostics,
    }
}
