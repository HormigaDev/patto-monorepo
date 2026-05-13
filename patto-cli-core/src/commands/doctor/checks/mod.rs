mod build_output;
mod env;
mod i18n;
mod package_json;
mod project_config;
mod runtime;
mod sharding_redis;
mod tsconfig;

use crate::diagnostic::Diagnostic;
use crate::output::doctor_output::DoctorCheck;

use super::context::DoctorContext;

pub struct CheckResult {
    pub check: DoctorCheck,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn run_all(context: &DoctorContext<'_>) -> Vec<CheckResult> {
    vec![
        runtime::run(context),
        package_json::run(context),
        env::run(context),
        tsconfig::run(context),
        project_config::run(context),
        i18n::run(context),
        sharding_redis::run(context),
        build_output::run(context),
    ]
}
