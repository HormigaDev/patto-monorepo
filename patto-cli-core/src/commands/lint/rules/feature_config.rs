use serde_json::Value;

use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::output::lint_output::LintRuleSeverity;
use crate::project::CONFIG_RELATIVE_PATH;

use super::super::context::RuleContext;

const KNOWN_FEATURES: [&str; 1] = ["i18n"];

pub fn run(context: &RuleContext<'_>, severity: LintRuleSeverity) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let Some(config) = context.project.config_json.as_ref() else {
        return diagnostics;
    };
    let Some(features) = config.get("features") else {
        return diagnostics;
    };

    let Some(features) = features.as_object() else {
        diagnostics.push(with_config_location(
            context,
            context.diagnostic(
                DiagnosticCode::PATTO_LINT_FEATURE_CONFIG,
                severity,
                "feature-config.invalid-shape.message",
                &[],
            ),
            "features",
        ));
        return diagnostics;
    };

    for (feature, enabled) in features {
        if !KNOWN_FEATURES.contains(&feature.as_str()) {
            diagnostics.push(with_config_location(
                context,
                context.diagnostic(
                    DiagnosticCode::PATTO_LINT_FEATURE_CONFIG,
                    severity,
                    "feature-config.unknown.message",
                    &[("feature", feature.as_str())],
                ),
                feature,
            ));
            continue;
        }

        if !matches!(enabled, Value::Bool(_)) {
            diagnostics.push(with_config_location(
                context,
                context.diagnostic(
                    DiagnosticCode::PATTO_LINT_FEATURE_CONFIG,
                    severity,
                    "feature-config.non-boolean.message",
                    &[("feature", feature.as_str())],
                ),
                feature,
            ));
        }
    }

    diagnostics
}

fn with_config_location(
    context: &RuleContext<'_>,
    diagnostic: Diagnostic,
    marker: &str,
) -> Diagnostic {
    if let Some((line, column)) = context.location_for_text(CONFIG_RELATIVE_PATH, marker) {
        diagnostic.with_location(CONFIG_RELATIVE_PATH, line, column)
    } else {
        diagnostic.with_location(CONFIG_RELATIVE_PATH, 1, 1)
    }
}
