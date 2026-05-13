use serde::Serialize;

use crate::diagnostic::Diagnostic;

use super::{OutputStats, OutputStatus};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanCommandOutput {
    pub status: OutputStatus,
    pub command: String,
    pub diagnostics: Vec<Diagnostic>,
    pub stats: OutputStats,
    pub index_path: Option<String>,
    pub index: ProjectIndex,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectIndex {
    pub schema_version: u32,
    pub generated_by: String,
    pub root: String,
    pub config: ProjectConfigIndex,
    pub package: PackageIndex,
    pub paths: ProjectPathsIndex,
    pub summary: ScanSummary,
    pub commands: Vec<CommandIndex>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectConfigIndex {
    pub path: String,
    pub exists: bool,
    pub lang: String,
    pub supported_lang: bool,
    pub features: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageIndex {
    pub path: String,
    pub exists: bool,
    pub name: Option<String>,
    pub version: Option<String>,
    pub main: Option<String>,
    pub package_manager: PackageManager,
    pub scripts: Vec<String>,
    pub dependencies: Vec<String>,
    pub dev_dependencies: Vec<String>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PackageManager {
    Npm,
    Pnpm,
    Yarn,
    Unknown,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectPathsIndex {
    pub commands_dir_exists: bool,
    pub definitions_dir_exists: bool,
    pub core_dir_exists: bool,
    pub config_dir_exists: bool,
    pub events_dir_exists: bool,
    pub plugins_dir_exists: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanSummary {
    pub files_scanned: usize,
    pub directories_scanned: usize,
    pub command_files: usize,
    pub definition_files: usize,
    pub event_files: usize,
    pub plugin_files: usize,
    pub commands: usize,
    pub subcommands: usize,
    pub subcommand_groups: usize,
    pub unknown_command_files: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandIndex {
    pub kind: CommandKind,
    pub key: Option<String>,
    pub file: String,
    pub metadata_file: String,
    pub class_name: Option<String>,
    pub extends_name: Option<String>,
    pub has_base_command_ancestor: bool,
    pub has_run_method: bool,
    pub name: Option<String>,
    pub parent: Option<String>,
    pub group: Option<String>,
    pub subcommand: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub aliases: Vec<String>,
    pub arguments: Vec<ArgumentIndex>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum CommandKind {
    Command,
    Subcommand,
    SubcommandGroup,
    Unknown,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArgumentIndex {
    pub name: Option<String>,
    pub required: bool,
    pub raw_text: bool,
    pub type_hint: Option<String>,
    pub option_values: Vec<ArgumentOptionValue>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArgumentOptionValue {
    pub raw: String,
    pub kind: ArgumentOptionValueKind,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ArgumentOptionValueKind {
    String,
    Number,
    Boolean,
    Unknown,
}
