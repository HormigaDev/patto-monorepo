use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

use crate::diagnostic::{Diagnostic, DiagnosticCode, DiagnosticLevel};
use crate::lang::{self, Lang};
use crate::output::scan_output::{
    CommandIndex, CommandKind, ProjectIndex, ProjectPathsIndex, ScanSummary,
};

use super::package::{scan_package_json, scan_project_config};
use super::parser::scan_commands;
use super::path_utils::{path_to_string, relative_path, relative_path_looks_like};
use super::{INDEX_SCHEMA_VERSION, ProjectScan};

#[derive(Debug)]
struct WalkResult {
    files: Vec<PathBuf>,
    files_scanned: usize,
    directories_scanned: usize,
}

pub fn scan_project(root: &Path, locale: Lang) -> Result<ProjectScan> {
    let root = fs::canonicalize(root)?;
    let mut diagnostics = Vec::new();
    let walk = walk_project(&root)?;
    let (config, config_json) = scan_project_config(&root, &mut diagnostics, locale);
    let package = scan_package_json(&root, &mut diagnostics, locale);
    let paths = scan_project_paths(&root, &mut diagnostics, locale);
    let commands = scan_commands(&root, &walk.files, &mut diagnostics, locale);
    let summary = build_summary(&walk, &commands);

    Ok(ProjectScan {
        root: root.clone(),
        index: ProjectIndex {
            schema_version: INDEX_SCHEMA_VERSION,
            generated_by: "patto-core scan".to_string(),
            root: path_to_string(&root),
            config,
            package,
            paths,
            summary,
            commands,
        },
        diagnostics,
        files_scanned: walk.files_scanned,
        directories_scanned: walk.directories_scanned,
        config_json,
        files: walk
            .files
            .iter()
            .map(|file| relative_path(&root, file))
            .collect(),
    })
}

fn scan_project_paths(
    root: &Path,
    diagnostics: &mut Vec<Diagnostic>,
    locale: Lang,
) -> ProjectPathsIndex {
    let commands_dir_exists = root.join("src/commands").is_dir();
    if !commands_dir_exists {
        diagnostics.push(
            Diagnostic::new(
                DiagnosticLevel::Warning,
                DiagnosticCode::PATTO_COMMANDS_DIR_MISSING,
                lang::text(locale, DiagnosticCode::PATTO_COMMANDS_DIR_MISSING),
            )
            .with_hint(lang::text(
                locale,
                &format!("{}.hint", DiagnosticCode::PATTO_COMMANDS_DIR_MISSING),
            )),
        );
    }

    ProjectPathsIndex {
        commands_dir_exists,
        definitions_dir_exists: root.join("src/definitions").is_dir(),
        core_dir_exists: root.join("src/core").is_dir(),
        config_dir_exists: root.join("src/config").is_dir(),
        events_dir_exists: root.join("src/events").is_dir(),
        plugins_dir_exists: root.join("src/plugins").is_dir(),
    }
}

fn build_summary(walk: &WalkResult, commands: &[CommandIndex]) -> ScanSummary {
    ScanSummary {
        files_scanned: walk.files_scanned,
        directories_scanned: walk.directories_scanned,
        command_files: walk
            .files
            .iter()
            .filter(|path| relative_path_looks_like(path, "src/commands", ".command."))
            .count(),
        definition_files: walk
            .files
            .iter()
            .filter(|path| relative_path_looks_like(path, "src/definitions", ".definition."))
            .count(),
        event_files: walk
            .files
            .iter()
            .filter(|path| relative_path_looks_like(path, "src/events", ".event."))
            .count(),
        plugin_files: walk
            .files
            .iter()
            .filter(|path| relative_path_looks_like(path, "src/plugins", ".plugin."))
            .count(),
        commands: commands
            .iter()
            .filter(|command| command.kind == CommandKind::Command)
            .count(),
        subcommands: commands
            .iter()
            .filter(|command| command.kind == CommandKind::Subcommand)
            .count(),
        subcommand_groups: commands
            .iter()
            .filter(|command| command.kind == CommandKind::SubcommandGroup)
            .count(),
        unknown_command_files: commands
            .iter()
            .filter(|command| command.kind == CommandKind::Unknown)
            .count(),
    }
}

fn walk_project(root: &Path) -> Result<WalkResult> {
    let mut result = WalkResult {
        files: Vec::new(),
        files_scanned: 0,
        directories_scanned: 0,
    };

    walk_dir(root, &mut result)?;
    result.files.sort();
    Ok(result)
}

fn walk_dir(dir: &Path, result: &mut WalkResult) -> Result<()> {
    result.directories_scanned += 1;

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            if should_skip_dir(&path) {
                continue;
            }
            walk_dir(&path, result)?;
        } else if file_type.is_file() {
            result.files_scanned += 1;
            result.files.push(path);
        }
    }

    Ok(())
}

fn should_skip_dir(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some(".git" | ".patto" | "node_modules" | "dist" | "build" | "target" | "coverage")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::scan_output::{CommandKind, PackageManager};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root(name: &str) -> PathBuf {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("patto-{name}-{}-{id}", std::process::id()));
        fs::create_dir_all(&root).expect("temp root should be created");
        root
    }

    #[test]
    fn scan_project_inherits_metadata_from_decorated_definition() {
        let root = temp_root("scan-inherited-definition");
        fs::create_dir_all(root.join(".patto")).expect("patto dir should be created");
        fs::create_dir_all(root.join("src/definitions"))
            .expect("definitions dir should be created");
        fs::create_dir_all(root.join("src/commands/general"))
            .expect("commands dir should be created");
        fs::write(root.join(".patto/config.json"), r#"{ "lang": "es" }"#)
            .expect("config should be written");
        fs::write(
            root.join("package.json"),
            r#"{ "name": "fixture", "version": "1.0.0", "packageManager": "pnpm@9.0.0" }"#,
        )
        .expect("package should be written");
        fs::write(
            root.join("src/definitions/base.definition.ts"),
            r#"
@Command({ name: "base", description: "Base command", aliases: ["b"] })
export abstract class BaseDefinition extends BaseCommand {}
"#,
        )
        .expect("definition should be written");
        fs::write(
            root.join("src/commands/general/base.command.ts"),
            r#"
export class BaseCommandImpl extends BaseDefinition {
  async run(): Promise<void> {}
}
"#,
        )
        .expect("command should be written");

        let scan = scan_project(&root, Lang::Es).expect("project should scan");

        assert!(scan.diagnostics.is_empty());
        assert_eq!(scan.index.package.package_manager, PackageManager::Pnpm);
        assert_eq!(scan.index.commands.len(), 1);
        let command = &scan.index.commands[0];
        assert_eq!(command.kind, CommandKind::Command);
        assert_eq!(command.file, "src/commands/general/base.command.ts");
        assert_eq!(command.metadata_file, "src/definitions/base.definition.ts");
        assert_eq!(command.key.as_deref(), Some("base"));
        assert_eq!(command.aliases, vec!["b"]);
        assert_eq!(command.class_name.as_deref(), Some("BaseCommandImpl"));
        assert_eq!(command.extends_name.as_deref(), Some("BaseDefinition"));
        assert!(command.has_run_method);

        fs::remove_dir_all(root).ok();
    }
}
