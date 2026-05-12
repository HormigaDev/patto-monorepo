use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::diagnostic::{Diagnostic, DiagnosticCode, DiagnosticLevel};
use crate::lang::{self, Lang};
use crate::output::scan_output::{CommandIndex, CommandKind};

use super::parser_support::{
    build_key, extract_arguments, extract_class_name, extract_decorator_block,
    extract_extends_name, extract_string_array_property, extract_string_property,
    extract_value_property, has_run_method, is_command_file, is_definition_file,
    is_typescript_or_javascript,
};
use super::path_utils::relative_path;

#[derive(Debug, Clone)]
struct FileCommandInfo {
    relative_path: String,
    class_name: Option<String>,
    extends_name: Option<String>,
    has_base_command_ancestor: bool,
    has_run_method: bool,
    command: Option<CommandIndex>,
}

pub(super) fn scan_commands(
    root: &Path,
    files: &[PathBuf],
    diagnostics: &mut Vec<Diagnostic>,
    locale: Lang,
) -> Vec<CommandIndex> {
    let mut file_infos = Vec::new();
    let mut decorated_by_class = HashMap::<String, CommandIndex>::new();

    for file in files {
        if !is_typescript_or_javascript(file) {
            continue;
        }

        let relative_path = relative_path(root, file);
        let content = match fs::read_to_string(file) {
            Ok(content) => content,
            Err(_) => {
                diagnostics.push(
                    Diagnostic::new(
                        DiagnosticLevel::Warning,
                        DiagnosticCode::PATTO_SOURCE_FILE_READ_FAILED,
                        lang::message(
                            locale,
                            "patto_source_file_read_failed.message",
                            &[("file", relative_path.as_str())],
                        ),
                    )
                    .with_location(&relative_path, 1, 1)
                    .with_hint(lang::text(
                        locale,
                        &format!("{}.hint", DiagnosticCode::PATTO_SOURCE_FILE_READ_FAILED),
                    )),
                );
                continue;
            }
        };
        let class_name = extract_class_name(&content);
        let extends_name = extract_extends_name(&content);
        let command = if is_command_file(&relative_path) || is_definition_file(&relative_path) {
            parse_command_index(&content, &relative_path, class_name.clone())
        } else {
            None
        };

        if let (Some(class_name), Some(command)) = (&class_name, &command) {
            decorated_by_class.insert(class_name.clone(), command.clone());
        }

        file_infos.push(FileCommandInfo {
            relative_path,
            class_name,
            extends_name,
            has_base_command_ancestor: extract_extends_name(&content).as_deref()
                == Some("BaseCommand"),
            has_run_method: has_run_method(&content),
            command,
        });
    }

    let mut commands = Vec::new();
    for info in file_infos
        .iter()
        .filter(|info| is_command_file(&info.relative_path))
    {
        if let Some(command) = &info.command {
            commands.push(command.clone());
            continue;
        }

        if let Some(inherited) = info
            .extends_name
            .as_ref()
            .and_then(|name| decorated_by_class.get(name))
        {
            let mut command = inherited.clone();
            command.file = info.relative_path.clone();
            command.class_name = info.class_name.clone();
            command.extends_name = info.extends_name.clone();
            command.has_run_method = info.has_run_method;
            commands.push(command);
            continue;
        }

        commands.push(CommandIndex {
            kind: CommandKind::Unknown,
            key: None,
            file: info.relative_path.clone(),
            metadata_file: info.relative_path.clone(),
            class_name: info.class_name.clone(),
            extends_name: info.extends_name.clone(),
            has_base_command_ancestor: info.has_base_command_ancestor,
            has_run_method: info.has_run_method,
            name: None,
            parent: None,
            group: None,
            subcommand: None,
            description: None,
            category: None,
            aliases: Vec::new(),
            arguments: Vec::new(),
        });
    }

    commands.sort_by(|left, right| left.file.cmp(&right.file));
    commands
}

fn parse_command_index(
    content: &str,
    relative_path: &str,
    class_name: Option<String>,
) -> Option<CommandIndex> {
    let extends_name = extract_extends_name(content);
    let has_base_command_ancestor = extends_name.as_deref() == Some("BaseCommand");
    let has_run_method = has_run_method(content);
    let arguments = extract_arguments(content);

    if let Some(block) = extract_decorator_block(content, "SubcommandGroup") {
        let parent = extract_string_property(&block, "parent");
        let group = extract_string_property(&block, "name");
        let subcommand = extract_string_property(&block, "subcommand");
        return Some(CommandIndex {
            kind: CommandKind::SubcommandGroup,
            key: build_key([parent.as_deref(), group.as_deref(), subcommand.as_deref()]),
            file: relative_path.to_string(),
            metadata_file: relative_path.to_string(),
            class_name,
            extends_name,
            has_base_command_ancestor,
            has_run_method,
            name: None,
            parent,
            group,
            subcommand,
            description: extract_string_property(&block, "description"),
            category: extract_value_property(&block, "category"),
            aliases: Vec::new(),
            arguments,
        });
    }

    if let Some(block) = extract_decorator_block(content, "Subcommand") {
        let parent = extract_string_property(&block, "parent");
        let name = extract_string_property(&block, "name");
        return Some(CommandIndex {
            kind: CommandKind::Subcommand,
            key: build_key([parent.as_deref(), name.as_deref(), None]),
            file: relative_path.to_string(),
            metadata_file: relative_path.to_string(),
            class_name,
            extends_name,
            has_base_command_ancestor,
            has_run_method,
            name,
            parent,
            group: None,
            subcommand: None,
            description: extract_string_property(&block, "description"),
            category: extract_value_property(&block, "category"),
            aliases: Vec::new(),
            arguments,
        });
    }

    if let Some(block) = extract_decorator_block(content, "Command") {
        let name = extract_string_property(&block, "name");
        return Some(CommandIndex {
            kind: CommandKind::Command,
            key: build_key([name.as_deref(), None, None]),
            file: relative_path.to_string(),
            metadata_file: relative_path.to_string(),
            class_name,
            extends_name,
            has_base_command_ancestor,
            has_run_method,
            name,
            parent: None,
            group: None,
            subcommand: None,
            description: extract_string_property(&block, "description"),
            category: extract_value_property(&block, "category"),
            aliases: extract_string_array_property(&block, "aliases"),
            arguments,
        });
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::scan_output::{ArgumentOptionValueKind, CommandKind};

    #[test]
    fn parse_command_index_reads_command_metadata_and_arguments() {
        let content = r#"
@Arg({ name: "count", required: true, type: Number, options: [{ name: "One", value: 1 }, { name: "Two", value: 2 }] })
@Arg({ name: "text", required: false, rawText: true })
@Command({ name: "Ping", description: "Pong", category: CommandCategory.Utility, aliases: ["p", "pong"] })
export class PingCommand extends BaseCommand {
  async run(): Promise<void> {}
}
"#;

        let command = parse_command_index(
            content,
            "src/commands/general/ping.command.ts",
            Some("PingCommand".to_string()),
        )
        .expect("command metadata should be parsed");

        assert_eq!(command.kind, CommandKind::Command);
        assert_eq!(command.key.as_deref(), Some("ping"));
        assert_eq!(command.name.as_deref(), Some("Ping"));
        assert_eq!(command.description.as_deref(), Some("Pong"));
        assert_eq!(command.category.as_deref(), Some("CommandCategory.Utility"));
        assert_eq!(command.aliases, vec!["p", "pong"]);
        assert_eq!(command.class_name.as_deref(), Some("PingCommand"));
        assert_eq!(command.extends_name.as_deref(), Some("BaseCommand"));
        assert!(command.has_base_command_ancestor);
        assert!(command.has_run_method);
        assert_eq!(command.arguments.len(), 2);
        assert_eq!(command.arguments[0].name.as_deref(), Some("count"));
        assert!(command.arguments[0].required);
        assert_eq!(
            command.arguments[0].option_values[0].kind,
            ArgumentOptionValueKind::Number
        );
        assert_eq!(command.arguments[1].name.as_deref(), Some("text"));
        assert!(command.arguments[1].raw_text);
    }

    #[test]
    fn parse_command_index_reads_subcommand_group_key() {
        let content = r#"
@SubcommandGroup({ parent: "admin", name: "roles", subcommand: "add", description: "Add role" })
export class AddRoleCommand extends BaseCommand {
  run() {}
}
"#;

        let command = parse_command_index(
            content,
            "src/commands/admin/roles/add.command.ts",
            Some("AddRoleCommand".to_string()),
        )
        .expect("subcommand group metadata should be parsed");

        assert_eq!(command.kind, CommandKind::SubcommandGroup);
        assert_eq!(command.key.as_deref(), Some("admin-roles-add"));
        assert_eq!(command.parent.as_deref(), Some("admin"));
        assert_eq!(command.group.as_deref(), Some("roles"));
        assert_eq!(command.subcommand.as_deref(), Some("add"));
    }

    #[test]
    fn parse_command_index_ignores_commented_decorators() {
        let content = r#"
// @Command({ name: "ghost", description: "Not real" })
export class GhostCommand extends BaseCommand {
  async run(): Promise<void> {}
}
"#;

        assert!(parse_command_index(content, "src/commands/ghost.command.ts", None).is_none());
    }
}
