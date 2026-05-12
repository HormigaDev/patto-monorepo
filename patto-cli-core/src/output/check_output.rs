use serde::Serialize;

use crate::diagnostic::Diagnostic;

use super::doctor_output::{DoctorCheck, DoctorSummary};
use super::lint_output::{LintRuleSetting, LintSummary};
use super::{OutputStats, OutputStatus};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckCommandOutput {
    pub status: OutputStatus,
    pub command: String,
    pub diagnostics: Vec<Diagnostic>,
    pub stats: OutputStats,
    pub summary: CheckSummary,
    pub scan: CheckScanSection,
    pub lint: CheckLintSection,
    pub doctor: CheckDoctorSection,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckSummary {
    pub errors: usize,
    pub warnings: usize,
    pub infos: usize,
    pub scan_errors: usize,
    pub lint_errors: usize,
    pub doctor_errors: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckScanSection {
    pub status: OutputStatus,
    pub diagnostics: Vec<Diagnostic>,
    pub index_path: Option<String>,
    pub files_scanned: usize,
    pub directories_scanned: usize,
    pub commands: usize,
    pub subcommands: usize,
    pub subcommand_groups: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckLintSection {
    pub status: OutputStatus,
    pub diagnostics: Vec<Diagnostic>,
    pub summary: LintSummary,
    pub rules: Vec<LintRuleSetting>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckDoctorSection {
    pub status: OutputStatus,
    pub diagnostics: Vec<Diagnostic>,
    pub summary: DoctorSummary,
    pub checks: Vec<DoctorCheck>,
}
