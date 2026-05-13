use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub code: String,
    pub message: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub hint: Option<String>,
}

impl Diagnostic {
    pub fn new(
        level: DiagnosticLevel,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            level,
            code: code.into(),
            message: message.into(),
            hint: None,
            file: None,
            line: None,
            column: None,
        }
    }

    pub fn with_location(mut self, file: impl Into<String>, line: u32, column: u32) -> Self {
        self.file = Some(file.into());
        self.line = Some(line);
        self.column = Some(column);
        self
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Info,
}

pub struct DiagnosticCode;

impl DiagnosticCode {
    pub const PATTO_PROJECT_ROOT_MISSING: &'static str = "patto_project_root_missing";
    pub const PATTO_ROOT_NOT_DIRECTORY: &'static str = "patto_root_not_directory";
    pub const PATTO_CONFIG_MISSING: &'static str = "patto_config_missing";
    pub const PATTO_CONFIG_INVALID: &'static str = "patto_config_invalid";
    pub const PATTO_CONFIG_LANG_UNSUPPORTED: &'static str = "patto_config_lang_unsupported";
    pub const PATTO_PACKAGE_JSON_MISSING: &'static str = "patto_package_json_missing";
    pub const PATTO_COMMANDS_DIR_MISSING: &'static str = "patto_commands_dir_missing";
    pub const PATTO_SCAN_INDEX_WRITE_FAILED: &'static str = "patto_scan_index_write_failed";
    pub const PATTO_SOURCE_FILE_READ_FAILED: &'static str = "patto_source_file_read_failed";
    pub const PATTO_LINT_RULE_CONFIG_INVALID: &'static str = "patto_lint_rule_config_invalid";
    pub const PATTO_LINT_DUPLICATE_COMMANDS: &'static str = "duplicate-commands";
    pub const PATTO_LINT_DUPLICATE_ALIASES: &'static str = "duplicate-aliases";
    pub const PATTO_LINT_UNKNOWN_COMMAND_FILES: &'static str = "unknown-command-files";
    pub const PATTO_LINT_INVALID_COMMAND_NAMES: &'static str = "invalid-command-names";
    pub const PATTO_LINT_DECORATED_BASE_COMMAND: &'static str = "decorated-base-command";
    pub const PATTO_LINT_MISSING_RUN_METHOD: &'static str = "missing-run-method";
    pub const PATTO_LINT_SUBCOMMAND_CONSISTENCY: &'static str = "subcommand-consistency";
    pub const PATTO_LINT_GHOST_PARENT_MIX: &'static str = "ghost-parent-mix";
    pub const PATTO_LINT_INVALID_ARGUMENTS: &'static str = "invalid-arguments";
    pub const PATTO_LINT_COMMAND_FOLDER_CONVENTION: &'static str = "command-folder-convention";
    pub const PATTO_LINT_BROKEN_ALIAS_IMPORTS: &'static str = "broken-alias-imports";
    pub const PATTO_LINT_PLUGIN_SPECIFIED_COMMANDS: &'static str = "plugin-specified-commands";
    pub const PATTO_LINT_SHARDING_REDIS_CONFIG: &'static str = "sharding-redis-config";
    pub const PATTO_LINT_COMPONENT_HANDLER_METHODS: &'static str = "component-handler-methods";
    pub const PATTO_LINT_FEATURE_CONFIG: &'static str = "feature-config";
    pub const PATTO_LINT_I18N_MISSING_KEYS: &'static str = "i18n-missing-keys";
    pub const PATTO_LINT_I18N_DYNAMIC_KEYS: &'static str = "i18n-dynamic-keys";
    pub const PATTO_LINT_I18N_LOCALE_PARITY: &'static str = "i18n-locale-parity";
    pub const PATTO_DOCTOR_RUNTIME: &'static str = "doctor-runtime";
    pub const PATTO_DOCTOR_PACKAGE_JSON: &'static str = "doctor-package-json";
    pub const PATTO_DOCTOR_ENV: &'static str = "doctor-env";
    pub const PATTO_DOCTOR_TSCONFIG: &'static str = "doctor-tsconfig";
    pub const PATTO_DOCTOR_PROJECT_CONFIG: &'static str = "doctor-project-config";
    pub const PATTO_DOCTOR_SHARDING_REDIS: &'static str = "doctor-sharding-redis";
    pub const PATTO_DOCTOR_BUILD_OUTPUT: &'static str = "doctor-build-output";
    pub const PATTO_DOCTOR_I18N: &'static str = "doctor-i18n";
}
