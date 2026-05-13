use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::output::lint_output::LintRuleSeverity;
use crate::utils;

use super::super::context::RuleContext;

pub fn run(context: &RuleContext<'_>, severity: LintRuleSeverity) -> Vec<Diagnostic> {
    let file = "src/config/plugins.config.ts";
    let Some(content) = context.read_file(file) else {
        return Vec::new();
    };
    let code = utils::mask_typescript_non_code(&content);

    let mut diagnostics = Vec::new();
    let mut search_start = 0;

    while let Some(relative_start) = code[search_start..].find("PluginScope.Specified") {
        let start = search_start + relative_start;
        let block_end = code[start..]
            .find("});")
            .map(|offset| start + offset)
            .unwrap_or(code.len());
        let block = &code[start..block_end];
        let compact_block = block.split_whitespace().collect::<String>();
        let has_commands = compact_block.contains("commands:")
            && !compact_block.contains("commands:[]");

        if !has_commands {
            let mut diagnostic = context.diagnostic(
                DiagnosticCode::PATTO_LINT_PLUGIN_SPECIFIED_COMMANDS,
                severity,
                "plugin-specified-commands.message",
                &[],
            );
            let (line, column) = utils::offset_to_line_column(&content, start);
            diagnostic = diagnostic.with_location(file, line, column);
            diagnostics.push(diagnostic);
        }

        search_start = block_end.saturating_add(3);
    }

    diagnostics
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::lang::Lang;
    use crate::output::lint_output::LintRuleSeverity;
    use crate::project::{ProjectScan, empty_project_index};

    use super::*;

    #[test]
    fn ignores_commented_plugin_scope_specified_blocks() {
        let root = temp_root("plugin-specified-comments");
        let config_dir = root.join("src/config");
        fs::create_dir_all(&config_dir).expect("config dir should be created");
        fs::write(
            config_dir.join("plugins.config.ts"),
            r#"import { PluginRegistry, PluginScope } from './plugin.registry';

/**
 * Ejemplo 4: Aplicar plugin a comandos específicos
 */
// PluginRegistry.register({
//     plugin: new AuditLogPlugin(),
//     scope: PluginScope.Specified,
//     commands: [BanCommand, KickCommand],
// });

PluginRegistry.register({
    plugin: new CooldownPlugin(StoreRegistry.getCooldownStore()),
    scope: PluginScope.Specified,
    folderPath: '',
});
"#,
        )
        .expect("plugins config should be written");
        let project = ProjectScan {
            root: root.clone(),
            index: empty_project_index(),
            diagnostics: Vec::new(),
            files_scanned: 0,
            directories_scanned: 0,
            config_json: None,
            files: Vec::new(),
        };
        let context = RuleContext {
            project: &project,
            locale: Lang::Es,
        };

        let diagnostics = run(&context, LintRuleSeverity::Warning);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].line, Some(14));
        assert_eq!(diagnostics[0].column, Some(12));

        let _ = fs::remove_dir_all(root);
    }

    fn temp_root(name: &str) -> std::path::PathBuf {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("patto-{name}-{}-{id}", std::process::id()))
    }
}
