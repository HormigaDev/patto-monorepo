mod filesystem;
mod locations;
mod package;
mod parser;
mod parser_support;
mod path_utils;
mod scanner;

use serde_json::Value;
use std::path::PathBuf;

use crate::diagnostic::Diagnostic;
use crate::output::scan_output::{
    ProjectConfigIndex, ProjectIndex, ProjectPathsIndex, ScanSummary,
};

pub use filesystem::{index_write_failed_diagnostic, write_project_index};
pub use locations::{find_text_location, find_value_location};
pub use scanner::scan_project;

pub const INDEX_SCHEMA_VERSION: u32 = 1;
pub const INDEX_RELATIVE_PATH: &str = ".patto/index.json";
pub const CONFIG_RELATIVE_PATH: &str = ".patto/config.json";

#[derive(Debug)]
pub struct ProjectScan {
    pub root: PathBuf,
    pub index: ProjectIndex,
    pub diagnostics: Vec<Diagnostic>,
    pub files_scanned: usize,
    pub directories_scanned: usize,
    pub config_json: Option<Value>,
    pub files: Vec<String>,
}

pub fn empty_project_index() -> ProjectIndex {
    ProjectIndex {
        schema_version: INDEX_SCHEMA_VERSION,
        generated_by: "patto-core scan".to_string(),
        root: String::new(),
        config: ProjectConfigIndex {
            path: CONFIG_RELATIVE_PATH.to_string(),
            exists: false,
            lang: "es".to_string(),
            supported_lang: true,
        },
        package: package::empty_package_index(),
        paths: ProjectPathsIndex {
            commands_dir_exists: false,
            definitions_dir_exists: false,
            core_dir_exists: false,
            config_dir_exists: false,
            events_dir_exists: false,
            plugins_dir_exists: false,
        },
        summary: ScanSummary {
            files_scanned: 0,
            directories_scanned: 0,
            command_files: 0,
            definition_files: 0,
            event_files: 0,
            plugin_files: 0,
            commands: 0,
            subcommands: 0,
            subcommand_groups: 0,
            unknown_command_files: 0,
        },
        commands: Vec::new(),
    }
}
