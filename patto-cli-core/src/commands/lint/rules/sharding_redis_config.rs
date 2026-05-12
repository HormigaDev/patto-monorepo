use std::collections::HashMap;

use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::output::lint_output::LintRuleSeverity;
use crate::utils;

use super::super::context::RuleContext;

pub fn run(context: &RuleContext<'_>, severity: LintRuleSeverity) -> Vec<Diagnostic> {
    let env = read_env(context);
    let sharding_enabled = env
        .get("SHARDING_ENABLED")
        .map(|value| value.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if !sharding_enabled {
        return Vec::new();
    }

    let mut diagnostics = Vec::new();
    if env
        .get("REDIS_URL")
        .map(|value| value.trim().is_empty())
        .unwrap_or(true)
    {
        diagnostics.push(config_diagnostic(
            context,
            severity,
            ".env",
            "sharding-redis-config.redis-url.message",
            &[],
        ));
    }

    for required_file in [
        "src/core/store/redis.cooldown.store.ts",
        "src/core/store/redis.payload.store.ts",
        "src/core/store/store.registry.ts",
    ] {
        if !context.root().join(required_file).is_file() {
            diagnostics.push(config_diagnostic(
                context,
                severity,
                required_file,
                "sharding-redis-config.missing-store.message",
                &[("file", required_file)],
            ));
        }
    }

    diagnostics
}

fn read_env(context: &RuleContext<'_>) -> HashMap<String, String> {
    utils::read_env_file(context.root(), ".env")
}

fn config_diagnostic(
    context: &RuleContext<'_>,
    severity: LintRuleSeverity,
    file: &str,
    message_key: &str,
    args: &[(&str, &str)],
) -> Diagnostic {
    context
        .diagnostic(
            DiagnosticCode::PATTO_LINT_SHARDING_REDIS_CONFIG,
            severity,
            message_key,
            args,
        )
        .with_location(file, 1, 1)
}
