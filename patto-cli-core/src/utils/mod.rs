use std::path::PathBuf;
use std::{collections::HashMap, fs, path::Path};

use crate::diagnostic::{Diagnostic, DiagnosticCode, DiagnosticLevel};
use crate::lang::{self, Lang};

pub fn root_not_exists(diagnostics: &mut Vec<Diagnostic>, root: &PathBuf, locate: Lang) -> bool {
    if !root.exists() {
        diagnostics.push(
            Diagnostic::new(
                DiagnosticLevel::Error,
                DiagnosticCode::PATTO_PROJECT_ROOT_MISSING,
                lang::text(locate, DiagnosticCode::PATTO_PROJECT_ROOT_MISSING),
            )
            .with_hint(lang::text(
                locate,
                &format!("{}.hint", DiagnosticCode::PATTO_PROJECT_ROOT_MISSING),
            )),
        );
        return true;
    }

    if !root.is_dir() {
        diagnostics.push(
            Diagnostic::new(
                DiagnosticLevel::Error,
                DiagnosticCode::PATTO_ROOT_NOT_DIRECTORY,
                lang::text(locate, DiagnosticCode::PATTO_ROOT_NOT_DIRECTORY),
            )
            .with_hint(lang::text(
                locate,
                &format!("{}.hint", DiagnosticCode::PATTO_ROOT_NOT_DIRECTORY),
            )),
        )
    }

    diagnostics
        .iter()
        .any(|diagnostic| matches!(diagnostic.level, DiagnosticLevel::Error))
}

pub fn read_env_file(root: &Path, relative_file: &str) -> HashMap<String, String> {
    let mut values = HashMap::new();
    let Ok(content) = fs::read_to_string(root.join(relative_file)) else {
        return values;
    };

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = trimmed.split_once('=') {
            values.insert(key.trim().to_string(), value.trim().to_string());
        }
    }

    values
}

pub fn strip_line_comments(content: &str) -> String {
    content
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn root_not_exists_reports_only_missing_root_when_path_does_not_exist() {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "patto-missing-root-test-{}-{id}",
            std::process::id()
        ));
        let mut diagnostics = Vec::new();

        assert!(root_not_exists(&mut diagnostics, &root, Lang::Es));
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].code,
            DiagnosticCode::PATTO_PROJECT_ROOT_MISSING
        );
    }
}
