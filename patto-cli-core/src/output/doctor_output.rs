use serde::Serialize;

use crate::diagnostic::Diagnostic;

use super::{OutputStats, OutputStatus};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DoctorCommandOutput {
    pub status: OutputStatus,
    pub command: String,
    pub diagnostics: Vec<Diagnostic>,
    pub stats: OutputStats,
    pub summary: DoctorSummary,
    pub checks: Vec<DoctorCheck>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DoctorSummary {
    pub checks_run: usize,
    pub ok: usize,
    pub warnings: usize,
    pub errors: usize,
    pub skipped: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DoctorCheck {
    pub id: String,
    pub status: DoctorCheckStatus,
    pub message: String,
    pub details: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DoctorCheckStatus {
    Ok,
    Warning,
    Error,
    Skipped,
}
