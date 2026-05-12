mod broken_alias_imports;
mod command_folder_convention;
mod component_handler_methods;
mod decorated_base_command;
mod duplicate_aliases;
mod duplicate_commands;
mod ghost_parent_mix;
mod invalid_arguments;
mod invalid_command_names;
mod missing_run_method;
mod plugin_specified_commands;
mod sharding_redis_config;
mod subcommand_consistency;
mod unknown_command_files;

use crate::diagnostic::Diagnostic;
use crate::output::lint_output::LintRuleSeverity;

use super::config::{LintRule, RuleConfig, all_rules};
use super::context::RuleContext;

pub fn run_enabled_rules(context: &RuleContext<'_>, config: &RuleConfig) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for rule in all_rules() {
        let severity = config.severity(rule);
        if severity == LintRuleSeverity::Off {
            continue;
        }

        diagnostics.extend(match rule {
            LintRule::DuplicateCommands => duplicate_commands::run(context, severity),
            LintRule::DuplicateAliases => duplicate_aliases::run(context, severity),
            LintRule::InvalidCommandNames => invalid_command_names::run(context, severity),
            LintRule::UnknownCommandFiles => unknown_command_files::run(context, severity),
            LintRule::DecoratedBaseCommand => decorated_base_command::run(context, severity),
            LintRule::MissingRunMethod => missing_run_method::run(context, severity),
            LintRule::SubcommandConsistency => subcommand_consistency::run(context, severity),
            LintRule::GhostParentMix => ghost_parent_mix::run(context, severity),
            LintRule::InvalidArguments => invalid_arguments::run(context, severity),
            LintRule::CommandFolderConvention => command_folder_convention::run(context, severity),
            LintRule::BrokenAliasImports => broken_alias_imports::run(context, severity),
            LintRule::PluginSpecifiedCommands => plugin_specified_commands::run(context, severity),
            LintRule::ShardingRedisConfig => sharding_redis_config::run(context, severity),
            LintRule::ComponentHandlerMethods => component_handler_methods::run(context, severity),
        });
    }

    diagnostics
}
