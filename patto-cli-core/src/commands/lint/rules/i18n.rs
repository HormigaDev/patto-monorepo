use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::output::lint_output::LintRuleSeverity;
use crate::utils;

use super::super::context::RuleContext;

#[derive(Debug, Clone)]
struct LocaleFile {
    locale: String,
    file: String,
    keys: BTreeSet<String>,
}

#[derive(Debug, Clone)]
struct TranslationCall {
    key: String,
    offset: usize,
}

#[derive(Debug, Default)]
struct TranslationCalls {
    static_calls: Vec<TranslationCall>,
    dynamic_offsets: Vec<usize>,
}

pub fn run_missing_keys(context: &RuleContext<'_>, severity: LintRuleSeverity) -> Vec<Diagnostic> {
    if !context.feature_enabled("i18n") {
        return Vec::new();
    }

    let locales = collect_locale_files(context);
    if locales.is_empty() {
        return Vec::new();
    }

    let mut diagnostics = Vec::new();
    let mut reported = BTreeSet::new();

    for file in source_files(context) {
        let Some(content) = context.read_file(&file) else {
            continue;
        };
        let calls = extract_translation_calls(&content);

        for call in calls.static_calls {
            for locale in &locales {
                if locale.keys.contains(&call.key) {
                    continue;
                }

                let marker = (file.clone(), call.key.clone(), locale.locale.clone());
                if !reported.insert(marker) {
                    continue;
                }

                let (line, column) = utils::offset_to_line_column(&content, call.offset);
                diagnostics.push(
                    context
                        .diagnostic(
                            DiagnosticCode::PATTO_LINT_I18N_MISSING_KEYS,
                            severity,
                            "i18n-missing-keys.message",
                            &[
                                ("key", call.key.as_str()),
                                ("locale", locale.locale.as_str()),
                                ("file", locale.file.as_str()),
                            ],
                        )
                        .with_location(file.as_str(), line, column),
                );
            }
        }
    }

    diagnostics
}

pub fn run_dynamic_keys(context: &RuleContext<'_>, severity: LintRuleSeverity) -> Vec<Diagnostic> {
    if !context.feature_enabled("i18n") {
        return Vec::new();
    }

    let mut diagnostics = Vec::new();
    for file in source_files(context) {
        let Some(content) = context.read_file(&file) else {
            continue;
        };
        let calls = extract_translation_calls(&content);

        for offset in calls.dynamic_offsets {
            let (line, column) = utils::offset_to_line_column(&content, offset);
            diagnostics.push(
                context
                    .diagnostic(
                        DiagnosticCode::PATTO_LINT_I18N_DYNAMIC_KEYS,
                        severity,
                        "i18n-dynamic-keys.message",
                        &[],
                    )
                    .with_location(file.as_str(), line, column),
            );
        }
    }

    diagnostics
}

pub fn run_locale_parity(context: &RuleContext<'_>, severity: LintRuleSeverity) -> Vec<Diagnostic> {
    if !context.feature_enabled("i18n") {
        return Vec::new();
    }

    let locales = collect_locale_files(context);
    if locales.len() < 2 {
        return Vec::new();
    }

    let mut all_keys = BTreeSet::new();
    for locale in &locales {
        all_keys.extend(locale.keys.iter().cloned());
    }

    let mut diagnostics = Vec::new();
    for locale in &locales {
        for key in all_keys.difference(&locale.keys) {
            diagnostics.push(
                context
                    .diagnostic(
                        DiagnosticCode::PATTO_LINT_I18N_LOCALE_PARITY,
                        severity,
                        "i18n-locale-parity.message",
                        &[("key", key.as_str()), ("locale", locale.locale.as_str())],
                    )
                    .with_location(locale.file.as_str(), 1, 1),
            );
        }
    }

    diagnostics
}

fn source_files(context: &RuleContext<'_>) -> Vec<String> {
    context
        .project
        .files
        .iter()
        .filter(|file| is_source_file(file))
        .cloned()
        .collect()
}

fn is_source_file(file: &str) -> bool {
    file.starts_with("src/")
        && !file.starts_with("src/i18n/")
        && (file.ends_with(".ts")
            || file.ends_with(".tsx")
            || file.ends_with(".js")
            || file.ends_with(".jsx"))
}

fn collect_locale_files(context: &RuleContext<'_>) -> Vec<LocaleFile> {
    let locale_dir = context.root().join("src/i18n/locale");
    let Ok(entries) = fs::read_dir(&locale_dir) else {
        return Vec::new();
    };

    let mut locales = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() || !is_typescript_or_javascript(&path) {
            continue;
        }
        let Some(locale) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        let Ok(content) = fs::read_to_string(&path) else {
            continue;
        };

        locales.push(LocaleFile {
            locale: locale.to_string(),
            file: relative_to_root(context.root(), &path),
            keys: extract_translation_keys(&content),
        });
    }

    locales.sort_by(|left, right| left.locale.cmp(&right.locale));
    locales
}

fn is_typescript_or_javascript(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("ts" | "tsx" | "js" | "jsx")
    )
}

fn relative_to_root(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn extract_translation_keys(content: &str) -> BTreeSet<String> {
    let mut keys = BTreeSet::new();

    for line in content.lines() {
        let trimmed = line.trim_start();
        let Some(quote) = trimmed.chars().next() else {
            continue;
        };
        if quote != '\'' && quote != '"' {
            continue;
        }
        let Some((key, end)) = read_string_literal(trimmed, 0, quote) else {
            continue;
        };
        if trimmed[end..].trim_start().starts_with(':') {
            keys.insert(key);
        }
    }

    keys
}

fn extract_translation_calls(content: &str) -> TranslationCalls {
    let searchable = mask_comments(content);
    let mut calls = TranslationCalls::default();
    let mut markers = vec!["this.t(".to_string()];
    markers.extend(
        extract_translator_aliases(&searchable)
            .into_keys()
            .map(|alias| format!("{alias}(")),
    );

    for marker in markers {
        collect_calls_for_marker(content, &searchable, &marker, &mut calls);
    }

    calls
        .static_calls
        .sort_by(|left, right| left.offset.cmp(&right.offset));
    calls.dynamic_offsets.sort();
    calls.dynamic_offsets.dedup();
    calls
}

fn collect_calls_for_marker(
    content: &str,
    searchable: &str,
    marker: &str,
    calls: &mut TranslationCalls,
) {
    let mut search_start = 0;
    while let Some(relative_start) = searchable[search_start..].find(marker) {
        let call_start = search_start + relative_start;
        if !has_call_boundary(searchable, call_start) {
            search_start = call_start + marker.len();
            continue;
        }

        let mut arg_start = call_start + marker.len();
        while let Some(character) = searchable[arg_start..].chars().next() {
            if !character.is_whitespace() {
                break;
            }
            arg_start += character.len_utf8();
        }

        let Some(quote) = content[arg_start..].chars().next() else {
            break;
        };
        if matches!(quote, '\'' | '"' | '`') {
            if let Some((key, _end)) = read_string_literal(content, arg_start, quote) {
                if quote == '`' && key.contains("${") {
                    calls.dynamic_offsets.push(arg_start);
                } else {
                    calls.static_calls.push(TranslationCall {
                        key,
                        offset: arg_start + quote.len_utf8(),
                    });
                }
            }
        } else {
            calls.dynamic_offsets.push(arg_start);
        }

        search_start = call_start + marker.len();
    }
}

fn has_call_boundary(content: &str, start: usize) -> bool {
    if start == 0 {
        return true;
    }

    let Some(previous) = content[..start].chars().next_back() else {
        return true;
    };

    !(previous.is_ascii_alphanumeric() || previous == '_' || previous == '$' || previous == '.')
}

fn mask_comments(content: &str) -> String {
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
            result.push(character);
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == quote {
                in_string = None;
            }
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

        if matches!(character, '\'' | '"' | '`') {
            in_string = Some(character);
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

fn extract_translator_aliases(content: &str) -> BTreeMap<String, ()> {
    let mut aliases = BTreeMap::new();

    for line in content.lines() {
        let trimmed = line.trim_start();
        let Some(rest) = trimmed
            .strip_prefix("const ")
            .or_else(|| trimmed.strip_prefix("let "))
        else {
            continue;
        };
        let Some((alias, rest)) = read_identifier(rest) else {
            continue;
        };
        let rest = rest.trim_start();
        let Some(rest) = rest.strip_prefix('=') else {
            continue;
        };
        let rest = rest.trim_start();
        if rest.starts_with("this.t") || rest.starts_with("i18n.for(") {
            aliases.insert(alias.to_string(), ());
        }
    }

    aliases
}

fn read_identifier(value: &str) -> Option<(&str, &str)> {
    let mut end = 0;
    for (index, character) in value.char_indices() {
        if index == 0 {
            if !(character.is_ascii_alphabetic() || character == '_' || character == '$') {
                return None;
            }
            end = character.len_utf8();
            continue;
        }

        if character.is_ascii_alphanumeric() || character == '_' || character == '$' {
            end = index + character.len_utf8();
        } else {
            break;
        }
    }

    if end == 0 {
        None
    } else {
        Some((&value[..end], &value[end..]))
    }
}

fn read_string_literal(content: &str, start: usize, quote: char) -> Option<(String, usize)> {
    let mut value = String::new();
    let mut escaped = false;
    let mut index = start + quote.len_utf8();

    while index < content.len() {
        let character = content[index..].chars().next()?;
        if escaped {
            value.push(character);
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if character == quote {
            return Some((value, index + quote.len_utf8()));
        } else {
            value.push(character);
        }
        index += character.len_utf8();
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_translation_keys_reads_quoted_object_keys() {
        let keys = extract_translation_keys("  'a.b': 'A',\n  \"c.d\": () => 'C',\n  plain: true,");

        assert!(keys.contains("a.b"));
        assert!(keys.contains("c.d"));
        assert!(!keys.contains("plain"));
    }

    #[test]
    fn extract_translation_calls_reads_this_t_and_aliases() {
        let calls = extract_translation_calls(
            "const t = this.t;\nthis.t('ping.title');\nt(\"ping.body\");\nthis.t(key);",
        );

        assert_eq!(
            calls
                .static_calls
                .iter()
                .map(|call| call.key.as_str())
                .collect::<Vec<_>>(),
            vec!["ping.title", "ping.body"]
        );
        assert_eq!(calls.dynamic_offsets.len(), 1);
    }

    #[test]
    fn extract_translation_calls_ignores_comments() {
        let calls = extract_translation_calls(
            "// this.t('ignored.line')
/** this.t('ignored.block') */
this.t('real.key');",
        );

        assert_eq!(calls.static_calls.len(), 1);
        assert_eq!(calls.static_calls[0].key, "real.key");
    }
}
