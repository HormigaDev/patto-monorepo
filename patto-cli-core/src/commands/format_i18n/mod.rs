use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::cli::CommonArgs;
use crate::diagnostic::{Diagnostic, DiagnosticCode, DiagnosticLevel};
use crate::lang;
use crate::output::format_i18n_output::{
    FormatI18nCommandOutput, FormatI18nFile, FormatI18nFileStatus, FormatI18nSummary,
};
use crate::output::{print_json, OutputStats, OutputStatus};
use crate::utils::root_not_exists;

pub fn run(args: CommonArgs, locale: crate::lang::Lang) -> Result<i32> {
    let started_at = Instant::now();
    let mut diagnostics = Vec::new();

    if root_not_exists(&mut diagnostics, &args.root, locale) {
        let output = build_output(
            diagnostics,
            OutputStats {
                files_scanned: 0,
                directories_scanned: 0,
                duration_ms: started_at.elapsed().as_millis(),
            },
            Vec::new(),
        );
        if args.json {
            print_json(&output)?;
        } else {
            println!("{}", lang::text(locale, "cli.format-i18n.root-invalid"));
        }
        return Ok(1);
    }

    let locale_dir = args.root.join("src").join("i18n").join("locale");
    let mut files = Vec::new();
    let mut files_scanned = 0;

    if !locale_dir.is_dir() {
        diagnostics.push(
            Diagnostic::new(
                DiagnosticLevel::Warning,
                DiagnosticCode::PATTO_FORMAT_I18N_LOCALE_DIR_MISSING,
                lang::text(locale, "patto_format_i18n_locale_dir_missing"),
            )
            .with_hint(lang::text(
                locale,
                "patto_format_i18n_locale_dir_missing.hint",
            )),
        );
    } else {
        for path in locale_files(&locale_dir)? {
            files_scanned += 1;
            let relative = relative_path(&args.root, &path);

            match fs::read_to_string(&path) {
                Ok(source) => match format_locale_source(
                    &source,
                    path.file_stem().and_then(|value| value.to_str()),
                ) {
                    FormatSourceResult::Formatted(next_source) => {
                        match fs::write(&path, next_source) {
                            Ok(()) => files.push(FormatI18nFile {
                                path: relative,
                                status: FormatI18nFileStatus::Formatted,
                            }),
                            Err(_) => {
                                diagnostics.push(
                                    Diagnostic::new(
                                        DiagnosticLevel::Error,
                                        DiagnosticCode::PATTO_FORMAT_I18N_FILE_WRITE_FAILED,
                                        lang::message(
                                            locale,
                                            "patto_format_i18n_file_write_failed.message",
                                            &[("file", relative.as_str())],
                                        ),
                                    )
                                    .with_hint(lang::text(
                                        locale,
                                        "patto_format_i18n_file_write_failed.hint",
                                    )),
                                );
                                files.push(FormatI18nFile {
                                    path: relative,
                                    status: FormatI18nFileStatus::Skipped,
                                });
                            }
                        }
                    }
                    FormatSourceResult::Unchanged => files.push(FormatI18nFile {
                        path: relative,
                        status: FormatI18nFileStatus::Unchanged,
                    }),
                    FormatSourceResult::Unsupported => {
                        diagnostics.push(
                            Diagnostic::new(
                                DiagnosticLevel::Warning,
                                DiagnosticCode::PATTO_FORMAT_I18N_FILE_UNSUPPORTED,
                                lang::message(
                                    locale,
                                    "patto_format_i18n_file_unsupported.message",
                                    &[("file", relative.as_str())],
                                ),
                            )
                            .with_hint(lang::text(
                                locale,
                                "patto_format_i18n_file_unsupported.hint",
                            )),
                        );
                        files.push(FormatI18nFile {
                            path: relative,
                            status: FormatI18nFileStatus::Skipped,
                        });
                    }
                },
                Err(_) => {
                    diagnostics.push(
                        Diagnostic::new(
                            DiagnosticLevel::Error,
                            DiagnosticCode::PATTO_FORMAT_I18N_FILE_READ_FAILED,
                            lang::message(
                                locale,
                                "patto_format_i18n_file_read_failed.message",
                                &[("file", relative.as_str())],
                            ),
                        )
                        .with_hint(lang::text(
                            locale,
                            "patto_format_i18n_file_read_failed.hint",
                        )),
                    );
                    files.push(FormatI18nFile {
                        path: relative,
                        status: FormatI18nFileStatus::Skipped,
                    });
                }
            }
        }
    }

    let output = build_output(
        diagnostics,
        OutputStats {
            files_scanned,
            directories_scanned: usize::from(locale_dir.is_dir()),
            duration_ms: started_at.elapsed().as_millis(),
        },
        files,
    );

    if args.json {
        print_json(&output)?;
    } else {
        let formatted = output.summary.files_formatted.to_string();
        let unchanged = output.summary.files_unchanged.to_string();
        let skipped = output.summary.files_skipped.to_string();

        println!(
            "{}",
            lang::message(
                locale,
                "cli.format-i18n.completed",
                &[
                    ("formatted", formatted.as_str()),
                    ("unchanged", unchanged.as_str()),
                    ("skipped", skipped.as_str()),
                ],
            )
        );
    }

    Ok(
        if output
            .diagnostics
            .iter()
            .any(|diagnostic| matches!(diagnostic.level, DiagnosticLevel::Error))
        {
            1
        } else {
            0
        },
    )
}

fn build_output(
    diagnostics: Vec<Diagnostic>,
    stats: OutputStats,
    files: Vec<FormatI18nFile>,
) -> FormatI18nCommandOutput {
    let files_formatted = files
        .iter()
        .filter(|file| matches!(file.status, FormatI18nFileStatus::Formatted))
        .count();
    let files_unchanged = files
        .iter()
        .filter(|file| matches!(file.status, FormatI18nFileStatus::Unchanged))
        .count();
    let files_skipped = files
        .iter()
        .filter(|file| matches!(file.status, FormatI18nFileStatus::Skipped))
        .count();
    let has_errors = diagnostics
        .iter()
        .any(|diagnostic| matches!(diagnostic.level, DiagnosticLevel::Error));

    FormatI18nCommandOutput {
        status: if has_errors {
            OutputStatus::Failed
        } else {
            OutputStatus::Ok
        },
        command: "format-i18n".to_string(),
        diagnostics,
        stats,
        summary: FormatI18nSummary {
            files_found: files.len(),
            files_formatted,
            files_unchanged,
            files_skipped,
        },
        files,
    }
}

fn locale_files(locale_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in fs::read_dir(locale_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|value| value.to_str()) == Some("ts") {
            files.push(path);
        }
    }

    files.sort();
    Ok(files)
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

#[derive(Debug, PartialEq, Eq)]
enum FormatSourceResult {
    Formatted(String),
    Unchanged,
    Unsupported,
}

fn format_locale_source(source: &str, expected_export: Option<&str>) -> FormatSourceResult {
    let Some((open, close)) = find_exported_object_span(source, expected_export) else {
        return FormatSourceResult::Unsupported;
    };

    let body = &source[open + 1..close];
    let entries = split_top_level_entries(body);

    if entries.iter().all(|entry| entry.trim().is_empty()) {
        return FormatSourceResult::Unchanged;
    }

    let mut keyed_entries = Vec::new();

    for entry in entries {
        if entry.trim().is_empty() {
            continue;
        }

        let Some(key) = extract_entry_key(&entry) else {
            return FormatSourceResult::Unsupported;
        };

        keyed_entries.push((key, entry));
    }

    keyed_entries.sort_by(|left, right| left.0.cmp(&right.0));

    let indent = infer_entry_indent(body).unwrap_or_else(|| "    ".to_string());
    let mut next = String::new();
    next.push_str(&source[..open + 1]);

    for (_, entry) in keyed_entries {
        next.push('\n');
        next.push_str(&render_entry(&entry, &indent));
        next.push(',');
    }

    next.push('\n');
    next.push_str(&source[close..]);

    if next == source {
        FormatSourceResult::Unchanged
    } else {
        FormatSourceResult::Formatted(next)
    }
}

fn find_exported_object_span(
    source: &str,
    expected_export: Option<&str>,
) -> Option<(usize, usize)> {
    let mut search_from = 0usize;

    while let Some(relative_index) = source[search_from..].find("export const ") {
        let export_index = search_from + relative_index;
        let name_start = export_index + "export const ".len();
        let name_end = source[name_start..]
            .find(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '$'))
            .map(|offset| name_start + offset)?;
        let export_name = &source[name_start..name_end];

        if expected_export.is_none_or(|expected| expected == export_name) {
            let after_export = &source[export_index..];
            let equals_index = export_index + after_export.find('=')?;
            let open = equals_index + source[equals_index..].find('{')?;
            let close = find_matching_brace(source, open)?;

            return Some((open, close));
        }

        search_from = name_end;
    }

    None
}

fn find_matching_brace(source: &str, open: usize) -> Option<usize> {
    let mut state = ScanState::Normal;
    let mut depth = 0usize;
    let mut chars = source[open..].char_indices().peekable();

    while let Some((offset, ch)) = chars.next() {
        let index = open + offset;

        match state {
            ScanState::Normal => match ch {
                '{' => depth += 1,
                '}' => {
                    depth = depth.checked_sub(1)?;
                    if depth == 0 {
                        return Some(index);
                    }
                }
                '\'' => state = ScanState::Single,
                '"' => state = ScanState::Double,
                '`' => state = ScanState::Template,
                '/' if chars.peek().is_some_and(|(_, next)| *next == '/') => {
                    chars.next();
                    state = ScanState::LineComment;
                }
                '/' if chars.peek().is_some_and(|(_, next)| *next == '*') => {
                    chars.next();
                    state = ScanState::BlockComment;
                }
                _ => {}
            },
            ScanState::Single => match ch {
                '\\' => {
                    chars.next();
                }
                '\'' => state = ScanState::Normal,
                _ => {}
            },
            ScanState::Double => match ch {
                '\\' => {
                    chars.next();
                }
                '"' => state = ScanState::Normal,
                _ => {}
            },
            ScanState::Template => match ch {
                '\\' => {
                    chars.next();
                }
                '`' => state = ScanState::Normal,
                _ => {}
            },
            ScanState::LineComment => {
                if ch == '\n' {
                    state = ScanState::Normal;
                }
            }
            ScanState::BlockComment => {
                if ch == '*' && chars.peek().is_some_and(|(_, next)| *next == '/') {
                    chars.next();
                    state = ScanState::Normal;
                }
            }
        }
    }

    None
}

fn split_top_level_entries(body: &str) -> Vec<String> {
    let mut entries = Vec::new();
    let mut start = 0usize;
    let mut state = ScanState::Normal;
    let mut paren_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut chars = body.char_indices().peekable();

    while let Some((index, ch)) = chars.next() {
        match state {
            ScanState::Normal => match ch {
                ',' if paren_depth == 0 && brace_depth == 0 && bracket_depth == 0 => {
                    entries.push(body[start..index].to_string());
                    start = index + ch.len_utf8();
                }
                '(' => paren_depth += 1,
                ')' => paren_depth = paren_depth.saturating_sub(1),
                '{' => brace_depth += 1,
                '}' => brace_depth = brace_depth.saturating_sub(1),
                '[' => bracket_depth += 1,
                ']' => bracket_depth = bracket_depth.saturating_sub(1),
                '\'' => state = ScanState::Single,
                '"' => state = ScanState::Double,
                '`' => state = ScanState::Template,
                '/' if chars.peek().is_some_and(|(_, next)| *next == '/') => {
                    chars.next();
                    state = ScanState::LineComment;
                }
                '/' if chars.peek().is_some_and(|(_, next)| *next == '*') => {
                    chars.next();
                    state = ScanState::BlockComment;
                }
                _ => {}
            },
            ScanState::Single => match ch {
                '\\' => {
                    chars.next();
                }
                '\'' => state = ScanState::Normal,
                _ => {}
            },
            ScanState::Double => match ch {
                '\\' => {
                    chars.next();
                }
                '"' => state = ScanState::Normal,
                _ => {}
            },
            ScanState::Template => match ch {
                '\\' => {
                    chars.next();
                }
                '`' => state = ScanState::Normal,
                _ => {}
            },
            ScanState::LineComment => {
                if ch == '\n' {
                    state = ScanState::Normal;
                }
            }
            ScanState::BlockComment => {
                if ch == '*' && chars.peek().is_some_and(|(_, next)| *next == '/') {
                    chars.next();
                    state = ScanState::Normal;
                }
            }
        }
    }

    entries.push(body[start..].to_string());
    entries
}

fn extract_entry_key(entry: &str) -> Option<String> {
    let entry = strip_leading_comments(entry.trim_start());
    let mut chars = entry.char_indices();
    let (_, quote) = chars.next()?;

    if quote != '\'' && quote != '"' {
        return None;
    }

    let mut key = String::new();
    let mut escaped = false;

    for (index, ch) in chars {
        if escaped {
            key.push(ch);
            escaped = false;
            continue;
        }

        if ch == '\\' {
            escaped = true;
            continue;
        }

        if ch == quote {
            let rest = &entry[index + ch.len_utf8()..];
            return rest.trim_start().starts_with(':').then_some(key);
        }

        key.push(ch);
    }

    None
}

fn strip_leading_comments(mut value: &str) -> &str {
    loop {
        let trimmed = value.trim_start();

        if let Some(rest) = trimmed.strip_prefix("//") {
            if let Some(newline) = rest.find('\n') {
                value = &rest[newline + 1..];
                continue;
            }
            return "";
        }

        if let Some(rest) = trimmed.strip_prefix("/*") {
            if let Some(end) = rest.find("*/") {
                value = &rest[end + 2..];
                continue;
            }
            return "";
        }

        return trimmed;
    }
}

fn infer_entry_indent(body: &str) -> Option<String> {
    body.lines()
        .find(|line| !line.trim().is_empty())
        .map(|line| line.chars().take_while(|ch| ch.is_whitespace()).collect())
}

fn render_entry(entry: &str, indent: &str) -> String {
    let mut trimmed = entry.trim();

    while trimmed.ends_with(',') {
        trimmed = trimmed[..trimmed.len() - 1].trim_end();
    }

    let mut lines = trimmed.lines();
    let Some(first_line) = lines.next() else {
        return String::new();
    };

    let mut rendered = String::new();
    rendered.push_str(indent);
    rendered.push_str(first_line.trim_end());

    for line in lines {
        rendered.push('\n');
        rendered.push_str(line.trim_end());
    }

    rendered
}

#[derive(Debug, PartialEq, Eq)]
enum ScanState {
    Normal,
    Single,
    Double,
    Template,
    LineComment,
    BlockComment,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorts_string_translation_keys() {
        let source =
            "export const es = {\n    'zeta.ache.be': 'A',\n    'be.ache.zeta': 'B',\n};\n";
        let expected =
            "export const es = {\n    'be.ache.zeta': 'B',\n    'zeta.ache.be': 'A',\n};\n";

        assert_eq!(
            format_locale_source(source, Some("es")),
            FormatSourceResult::Formatted(expected.to_string())
        );
    }

    #[test]
    fn keeps_values_attached_when_values_contain_commas_and_braces() {
        let source = "export const es = {\n    'z.key': (value: string) => `Hola, ${value}`,\n    'a.key': ({ count }: { count: number }) => count === 1 ? 'Uno' : `${count}, varios`,\n};\n";
        let expected = "export const es = {\n    'a.key': ({ count }: { count: number }) => count === 1 ? 'Uno' : `${count}, varios`,\n    'z.key': (value: string) => `Hola, ${value}`,\n};\n";

        assert_eq!(
            format_locale_source(source, Some("es")),
            FormatSourceResult::Formatted(expected.to_string())
        );
    }

    #[test]
    fn ignores_commas_inside_nested_literals() {
        let source = "export const es = {\n    'b.key': ['B, one', 'B, two'].join(', '),\n    'a.key': { label: 'A, one' }.label,\n};\n";
        let expected = "export const es = {\n    'a.key': { label: 'A, one' }.label,\n    'b.key': ['B, one', 'B, two'].join(', '),\n};\n";

        assert_eq!(
            format_locale_source(source, Some("es")),
            FormatSourceResult::Formatted(expected.to_string())
        );
    }

    #[test]
    fn returns_unchanged_when_already_sorted() {
        let source = "export const es = {\n    'a.key': 'A',\n    'b.key': 'B',\n};\n";

        assert_eq!(
            format_locale_source(source, Some("es")),
            FormatSourceResult::Unchanged
        );
    }

    #[test]
    fn skips_files_without_exported_locale_object() {
        assert_eq!(
            format_locale_source("export const LOCALE = 'es';\n", Some("es")),
            FormatSourceResult::Unsupported
        );
    }
}
