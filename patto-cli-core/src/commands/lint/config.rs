use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

use crate::diagnostic::{Diagnostic, DiagnosticCode, DiagnosticLevel};
use crate::lang::{self, Lang};
use crate::output::lint_output::{LintRuleSetting, LintRuleSeverity};
use crate::project::{CONFIG_RELATIVE_PATH, find_text_location};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LintRule {
    DuplicateCommands,
    DuplicateAliases,
    InvalidCommandNames,
    UnknownCommandFiles,
    DecoratedBaseCommand,
    MissingRunMethod,
    SubcommandConsistency,
    GhostParentMix,
    InvalidArguments,
    CommandFolderConvention,
    BrokenAliasImports,
    PluginSpecifiedCommands,
    ShardingRedisConfig,
    ComponentHandlerMethods,
    FeatureConfig,
    I18nMissingKeys,
    I18nDynamicKeys,
    I18nLocaleParity,
}

impl LintRule {
    pub fn id(self) -> &'static str {
        match self {
            Self::DuplicateCommands => DiagnosticCode::PATTO_LINT_DUPLICATE_COMMANDS,
            Self::DuplicateAliases => DiagnosticCode::PATTO_LINT_DUPLICATE_ALIASES,
            Self::InvalidCommandNames => DiagnosticCode::PATTO_LINT_INVALID_COMMAND_NAMES,
            Self::UnknownCommandFiles => DiagnosticCode::PATTO_LINT_UNKNOWN_COMMAND_FILES,
            Self::DecoratedBaseCommand => DiagnosticCode::PATTO_LINT_DECORATED_BASE_COMMAND,
            Self::MissingRunMethod => DiagnosticCode::PATTO_LINT_MISSING_RUN_METHOD,
            Self::SubcommandConsistency => DiagnosticCode::PATTO_LINT_SUBCOMMAND_CONSISTENCY,
            Self::GhostParentMix => DiagnosticCode::PATTO_LINT_GHOST_PARENT_MIX,
            Self::InvalidArguments => DiagnosticCode::PATTO_LINT_INVALID_ARGUMENTS,
            Self::CommandFolderConvention => DiagnosticCode::PATTO_LINT_COMMAND_FOLDER_CONVENTION,
            Self::BrokenAliasImports => DiagnosticCode::PATTO_LINT_BROKEN_ALIAS_IMPORTS,
            Self::PluginSpecifiedCommands => DiagnosticCode::PATTO_LINT_PLUGIN_SPECIFIED_COMMANDS,
            Self::ShardingRedisConfig => DiagnosticCode::PATTO_LINT_SHARDING_REDIS_CONFIG,
            Self::ComponentHandlerMethods => DiagnosticCode::PATTO_LINT_COMPONENT_HANDLER_METHODS,
            Self::FeatureConfig => DiagnosticCode::PATTO_LINT_FEATURE_CONFIG,
            Self::I18nMissingKeys => DiagnosticCode::PATTO_LINT_I18N_MISSING_KEYS,
            Self::I18nDynamicKeys => DiagnosticCode::PATTO_LINT_I18N_DYNAMIC_KEYS,
            Self::I18nLocaleParity => DiagnosticCode::PATTO_LINT_I18N_LOCALE_PARITY,
        }
    }

    fn default_severity(self) -> LintRuleSeverity {
        match self {
            Self::DuplicateCommands
            | Self::DuplicateAliases
            | Self::DecoratedBaseCommand
            | Self::MissingRunMethod
            | Self::SubcommandConsistency
            | Self::BrokenAliasImports
            | Self::ComponentHandlerMethods => LintRuleSeverity::Error,
            Self::InvalidCommandNames
            | Self::UnknownCommandFiles
            | Self::GhostParentMix
            | Self::InvalidArguments
            | Self::CommandFolderConvention
            | Self::PluginSpecifiedCommands
            | Self::ShardingRedisConfig
            | Self::FeatureConfig
            | Self::I18nMissingKeys
            | Self::I18nDynamicKeys
            | Self::I18nLocaleParity => LintRuleSeverity::Warning,
        }
    }
}

pub fn all_rules() -> [LintRule; 18] {
    [
        LintRule::DuplicateCommands,
        LintRule::DuplicateAliases,
        LintRule::InvalidCommandNames,
        LintRule::UnknownCommandFiles,
        LintRule::DecoratedBaseCommand,
        LintRule::MissingRunMethod,
        LintRule::SubcommandConsistency,
        LintRule::GhostParentMix,
        LintRule::InvalidArguments,
        LintRule::CommandFolderConvention,
        LintRule::BrokenAliasImports,
        LintRule::PluginSpecifiedCommands,
        LintRule::ShardingRedisConfig,
        LintRule::ComponentHandlerMethods,
        LintRule::FeatureConfig,
        LintRule::I18nMissingKeys,
        LintRule::I18nDynamicKeys,
        LintRule::I18nLocaleParity,
    ]
}

pub struct RuleConfig {
    severities: HashMap<LintRule, LintRuleSeverity>,
}

impl RuleConfig {
    pub fn severity(&self, rule: LintRule) -> LintRuleSeverity {
        self.severities
            .get(&rule)
            .copied()
            .unwrap_or_else(|| rule.default_severity())
    }

    pub fn to_output(&self) -> Vec<LintRuleSetting> {
        all_rules()
            .into_iter()
            .map(|rule| LintRuleSetting {
                rule: rule.id().to_string(),
                severity: self.severity(rule),
            })
            .collect()
    }
}

pub fn resolve_rule_config(
    config_json: Option<&Value>,
    root: &Path,
    locale: Lang,
) -> (RuleConfig, Vec<Diagnostic>) {
    let mut severities = all_rules()
        .into_iter()
        .map(|rule| (rule, rule.default_severity()))
        .collect::<HashMap<_, _>>();
    let mut diagnostics = Vec::new();

    let Some(lint_rules) = config_json
        .and_then(|value| value.get("lint-rules"))
        .and_then(Value::as_object)
    else {
        return (RuleConfig { severities }, diagnostics);
    };

    for rule in all_rules() {
        let Some(raw_severity) = lint_rules.get(rule.id()).and_then(Value::as_str) else {
            continue;
        };

        if let Some(severity) = parse_rule_severity(raw_severity) {
            severities.insert(rule, severity);
        } else {
            let mut diagnostic = Diagnostic::new(
                DiagnosticLevel::Warning,
                DiagnosticCode::PATTO_LINT_RULE_CONFIG_INVALID,
                lang::message(
                    locale,
                    "patto_lint_rule_config_invalid.message",
                    &[("rule", rule.id()), ("severity", raw_severity)],
                ),
            )
            .with_hint(lang::text(
                locale,
                &format!("{}.hint", DiagnosticCode::PATTO_LINT_RULE_CONFIG_INVALID),
            ));

            if let Some((line, column)) =
                find_text_location(root, CONFIG_RELATIVE_PATH, raw_severity)
            {
                diagnostic = diagnostic.with_location(CONFIG_RELATIVE_PATH, line, column);
            }

            diagnostics.push(diagnostic);
        }
    }

    (RuleConfig { severities }, diagnostics)
}

fn parse_rule_severity(value: &str) -> Option<LintRuleSeverity> {
    match value.to_ascii_lowercase().as_str() {
        "off" => Some(LintRuleSeverity::Off),
        "info" => Some(LintRuleSeverity::Info),
        "warning" | "warn" => Some(LintRuleSeverity::Warning),
        "error" => Some(LintRuleSeverity::Error),
        _ => None,
    }
}

impl From<LintRuleSeverity> for DiagnosticLevel {
    fn from(value: LintRuleSeverity) -> Self {
        match value {
            LintRuleSeverity::Off | LintRuleSeverity::Info => DiagnosticLevel::Info,
            LintRuleSeverity::Warning => DiagnosticLevel::Warning,
            LintRuleSeverity::Error => DiagnosticLevel::Error,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root() -> std::path::PathBuf {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "patto-lint-config-test-{}-{id}",
            std::process::id()
        ));
        fs::create_dir_all(root.join(".patto")).expect("temp root should be created");
        root
    }

    #[test]
    fn parse_rule_severity_accepts_expected_values() {
        assert_eq!(parse_rule_severity("off"), Some(LintRuleSeverity::Off));
        assert_eq!(parse_rule_severity("INFO"), Some(LintRuleSeverity::Info));
        assert_eq!(parse_rule_severity("warn"), Some(LintRuleSeverity::Warning));
        assert_eq!(
            parse_rule_severity("warning"),
            Some(LintRuleSeverity::Warning)
        );
        assert_eq!(parse_rule_severity("error"), Some(LintRuleSeverity::Error));
        assert_eq!(parse_rule_severity("fatal"), None);
    }

    #[test]
    fn resolve_rule_config_applies_configured_severity() {
        let root = temp_root();
        let config = json!({
            "lint-rules": {
                "duplicate-commands": "off",
                "missing-run-method": "info"
            }
        });

        let (rule_config, diagnostics) = resolve_rule_config(Some(&config), &root, Lang::Es);

        assert!(diagnostics.is_empty());
        assert_eq!(
            rule_config.severity(LintRule::DuplicateCommands),
            LintRuleSeverity::Off
        );
        assert_eq!(
            rule_config.severity(LintRule::MissingRunMethod),
            LintRuleSeverity::Info
        );

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn resolve_rule_config_reports_invalid_severity_with_location() {
        let root = temp_root();
        let config_path = root.join(CONFIG_RELATIVE_PATH);
        fs::write(
            &config_path,
            r#"{
  "lint-rules": {
    "duplicate-commands": "fatal"
  }
}
"#,
        )
        .expect("config fixture should be written");
        let config = json!({
            "lint-rules": {
                "duplicate-commands": "fatal"
            }
        });

        let (_, diagnostics) = resolve_rule_config(Some(&config), &root, Lang::Es);

        assert_eq!(diagnostics.len(), 1);
        let diagnostic = &diagnostics[0];
        assert_eq!(
            diagnostic.code,
            DiagnosticCode::PATTO_LINT_RULE_CONFIG_INVALID
        );
        assert!(diagnostic.message.contains("duplicate-commands"));
        assert!(diagnostic.message.contains("fatal"));
        assert_eq!(diagnostic.file.as_deref(), Some(CONFIG_RELATIVE_PATH));
        assert!(diagnostic.line.is_some());
        assert!(diagnostic.column.is_some());

        fs::remove_dir_all(root).ok();
    }
}
