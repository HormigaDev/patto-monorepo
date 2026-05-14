use serde::Serialize;

use crate::diagnostic::Diagnostic;

use super::{OutputStats, OutputStatus};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FormatI18nCommandOutput {
    pub status: OutputStatus,
    pub command: String,
    pub diagnostics: Vec<Diagnostic>,
    pub stats: OutputStats,
    pub summary: FormatI18nSummary,
    pub files: Vec<FormatI18nFile>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FormatI18nSummary {
    pub files_found: usize,
    pub files_formatted: usize,
    pub files_unchanged: usize,
    pub files_skipped: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FormatI18nFile {
    pub path: String,
    pub status: FormatI18nFileStatus,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FormatI18nFileStatus {
    Formatted,
    Unchanged,
    Skipped,
}
