use serde::Serialize;

use crate::diagnostic::Diagnostic;

use super::{OutputStats, OutputStatus};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LintCommandOutput {
    pub status: OutputStatus,
    pub command: String,
    pub diagnostics: Vec<Diagnostic>,
    pub stats: OutputStats,
    pub summary: LintSummary,
    pub rules: Vec<LintRuleSetting>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LintSummary {
    pub rules_run: usize,
    pub rules_off: usize,
    pub errors: usize,
    pub warnings: usize,
    pub infos: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LintRuleSetting {
    pub rule: String,
    pub severity: LintRuleSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LintRuleSeverity {
    Off,
    Info,
    Warning,
    Error,
}
