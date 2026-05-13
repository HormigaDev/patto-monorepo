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

pub fn mask_typescript_non_code(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut index = 0;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut in_string = None::<char>;
    let mut escaped = false;

    while index < content.len() {
        let Some(character) = content[index..].chars().next() else {
            break;
        };

        if in_line_comment {
            if character == '\n' {
                in_line_comment = false;
                result.push('\n');
            } else {
                push_masked_character(&mut result, character);
            }
            index += character.len_utf8();
            continue;
        }

        if in_block_comment {
            if content[index..].starts_with("*/") {
                in_block_comment = false;
                result.push_str("  ");
                index += 2;
            } else {
                push_masked_character(&mut result, character);
                index += character.len_utf8();
            }
            continue;
        }

        if let Some(quote) = in_string {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == quote {
                in_string = None;
            }
            push_masked_character(&mut result, character);
            index += character.len_utf8();
            continue;
        }

        if content[index..].starts_with("//") {
            in_line_comment = true;
            result.push_str("  ");
            index += 2;
            continue;
        }

        if content[index..].starts_with("/*") {
            in_block_comment = true;
            result.push_str("  ");
            index += 2;
            continue;
        }

        if matches!(character, '"' | '\'' | '`') {
            in_string = Some(character);
            push_masked_character(&mut result, character);
            index += character.len_utf8();
            continue;
        }

        result.push(character);
        index += character.len_utf8();
    }

    result
}

fn push_masked_character(result: &mut String, character: char) {
    if character == '\n' {
        result.push('\n');
    } else {
        for _ in 0..character.len_utf8() {
            result.push(' ');
        }
    }
}

pub fn offset_to_line_column(content: &str, offset: usize) -> (u32, u32) {
    let mut line = 1_u32;
    let mut column = 1_u32;

    for (index, character) in content.char_indices() {
        if index >= offset {
            return (line, column);
        }
        if character == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }

    (line, column)
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

    #[test]
    fn mask_typescript_non_code_preserves_offsets_and_ignores_comments_strings() {
        let content = concat!(
            "// PluginScope.Specified\n",
            "const text = \"PluginScope.Specified\";\n",
            "/* Raíz PluginScope.Specified */\n",
            "scope: PluginScope.Specified,\n",
        );
        let masked = mask_typescript_non_code(content);
        let offset = masked
            .find("PluginScope.Specified")
            .expect("code occurrence should remain");
        let original_offset = content
            .rfind("PluginScope.Specified")
            .expect("original code occurrence should exist");

        assert_eq!(offset, original_offset);
        assert_eq!(offset_to_line_column(content, offset), (4, 8));
        assert_eq!(masked.matches("PluginScope.Specified").count(), 1);
    }
}
