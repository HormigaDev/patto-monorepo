use serde_json::Value;
use std::fs;
use std::path::Path;

use crate::diagnostic::{Diagnostic, DiagnosticCode, DiagnosticLevel};
use crate::lang::{self, Lang};
use crate::output::scan_output::{PackageIndex, PackageManager, ProjectConfigIndex};

use super::CONFIG_RELATIVE_PATH;

pub(super) fn scan_project_config(
    root: &Path,
    diagnostics: &mut Vec<Diagnostic>,
    locale: Lang,
) -> (ProjectConfigIndex, Option<Value>) {
    let config_path = root.join(CONFIG_RELATIVE_PATH);
    if !config_path.exists() {
        diagnostics.push(
            Diagnostic::new(
                DiagnosticLevel::Warning,
                DiagnosticCode::PATTO_CONFIG_MISSING,
                lang::text(locale, DiagnosticCode::PATTO_CONFIG_MISSING),
            )
            .with_hint(lang::text(
                locale,
                &format!("{}.hint", DiagnosticCode::PATTO_CONFIG_MISSING),
            )),
        );

        return (
            ProjectConfigIndex {
                path: CONFIG_RELATIVE_PATH.to_string(),
                exists: false,
                lang: "es".to_string(),
                supported_lang: true,
            },
            None,
        );
    }

    let raw_config = match fs::read_to_string(&config_path) {
        Ok(value) => value,
        Err(_) => {
            diagnostics.push(config_invalid_diagnostic(locale));
            return (
                ProjectConfigIndex {
                    path: CONFIG_RELATIVE_PATH.to_string(),
                    exists: true,
                    lang: "es".to_string(),
                    supported_lang: true,
                },
                None,
            );
        }
    };

    let parsed_config = serde_json::from_str::<Value>(&raw_config);
    let lang_value = parsed_config
        .as_ref()
        .ok()
        .and_then(|value| value.get("lang"))
        .and_then(Value::as_str)
        .unwrap_or("es")
        .to_string();

    if parsed_config.is_err() {
        diagnostics.push(config_invalid_diagnostic(locale));
    }

    let supported_lang = lang_value == "es";
    if !supported_lang {
        diagnostics.push(
            Diagnostic::new(
                DiagnosticLevel::Warning,
                DiagnosticCode::PATTO_CONFIG_LANG_UNSUPPORTED,
                lang::text(locale, DiagnosticCode::PATTO_CONFIG_LANG_UNSUPPORTED),
            )
            .with_hint(lang::text(
                locale,
                &format!("{}.hint", DiagnosticCode::PATTO_CONFIG_LANG_UNSUPPORTED),
            )),
        );
    }

    (
        ProjectConfigIndex {
            path: CONFIG_RELATIVE_PATH.to_string(),
            exists: true,
            lang: lang_value,
            supported_lang,
        },
        parsed_config.ok(),
    )
}

fn config_invalid_diagnostic(locale: Lang) -> Diagnostic {
    Diagnostic::new(
        DiagnosticLevel::Warning,
        DiagnosticCode::PATTO_CONFIG_INVALID,
        lang::text(locale, DiagnosticCode::PATTO_CONFIG_INVALID),
    )
    .with_hint(lang::text(
        locale,
        &format!("{}.hint", DiagnosticCode::PATTO_CONFIG_INVALID),
    ))
}

pub(super) fn scan_package_json(
    root: &Path,
    diagnostics: &mut Vec<Diagnostic>,
    locale: Lang,
) -> PackageIndex {
    let package_path = root.join("package.json");
    if !package_path.exists() {
        diagnostics.push(
            Diagnostic::new(
                DiagnosticLevel::Warning,
                DiagnosticCode::PATTO_PACKAGE_JSON_MISSING,
                lang::text(locale, DiagnosticCode::PATTO_PACKAGE_JSON_MISSING),
            )
            .with_hint(lang::text(
                locale,
                &format!("{}.hint", DiagnosticCode::PATTO_PACKAGE_JSON_MISSING),
            )),
        );
        return empty_package_index();
    }

    let package_json = fs::read_to_string(&package_path)
        .ok()
        .and_then(|content| serde_json::from_str::<Value>(&content).ok());

    PackageIndex {
        path: "package.json".to_string(),
        exists: true,
        name: package_json
            .as_ref()
            .and_then(|value| value.get("name"))
            .and_then(Value::as_str)
            .map(str::to_string),
        version: package_json
            .as_ref()
            .and_then(|value| value.get("version"))
            .and_then(Value::as_str)
            .map(str::to_string),
        main: package_json
            .as_ref()
            .and_then(|value| value.get("main"))
            .and_then(Value::as_str)
            .map(str::to_string),
        package_manager: detect_package_manager(root, package_json.as_ref()),
        scripts: sorted_object_keys(package_json.as_ref().and_then(|value| value.get("scripts"))),
        dependencies: sorted_object_keys(
            package_json
                .as_ref()
                .and_then(|value| value.get("dependencies")),
        ),
        dev_dependencies: sorted_object_keys(
            package_json
                .as_ref()
                .and_then(|value| value.get("devDependencies")),
        ),
    }
}

pub(super) fn empty_package_index() -> PackageIndex {
    PackageIndex {
        path: "package.json".to_string(),
        exists: false,
        name: None,
        version: None,
        main: None,
        package_manager: PackageManager::Unknown,
        scripts: Vec::new(),
        dependencies: Vec::new(),
        dev_dependencies: Vec::new(),
    }
}

fn detect_package_manager(root: &Path, package_json: Option<&Value>) -> PackageManager {
    if let Some(package_manager) = package_json
        .and_then(|value| value.get("packageManager"))
        .and_then(Value::as_str)
    {
        if package_manager.starts_with("pnpm@") {
            return PackageManager::Pnpm;
        }
        if package_manager.starts_with("npm@") {
            return PackageManager::Npm;
        }
        if package_manager.starts_with("yarn@") {
            return PackageManager::Yarn;
        }
    }

    if root.join("pnpm-lock.yaml").exists() {
        PackageManager::Pnpm
    } else if root.join("yarn.lock").exists() {
        PackageManager::Yarn
    } else if root.join("package-lock.json").exists() {
        PackageManager::Npm
    } else {
        PackageManager::Unknown
    }
}

fn sorted_object_keys(value: Option<&Value>) -> Vec<String> {
    let mut keys = value
        .and_then(Value::as_object)
        .map(|object| object.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    keys.sort();
    keys
}
