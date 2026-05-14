use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone)]
pub(super) struct TranslationCall {
    pub(super) key: String,
    pub(super) offset: usize,
}

#[derive(Debug, Default)]
pub(super) struct TranslationCalls {
    pub(super) static_calls: Vec<TranslationCall>,
    pub(super) dynamic_offsets: Vec<usize>,
}

#[derive(Debug, Default)]
struct TranslationTypeIndex {
    functions: BTreeSet<String>,
    variables: BTreeSet<String>,
}

pub(super) fn extract_translation_calls(content: &str) -> TranslationCalls {
    let searchable = mask_comments(content);
    let type_index = extract_translation_type_index(&searchable);
    let mut calls = TranslationCalls::default();
    let mut markers = vec!["this.t(".to_string()];
    markers.extend(
        extract_translator_aliases(&searchable)
            .into_keys()
            .map(|alias| format!("{alias}(")),
    );

    for marker in markers {
        collect_calls_for_marker(content, &searchable, &type_index, &marker, &mut calls);
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
    type_index: &TranslationTypeIndex,
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
                    if !is_translation_key_expression(content, searchable, type_index, arg_start) {
                        calls.dynamic_offsets.push(arg_start);
                    }
                } else {
                    calls.static_calls.push(TranslationCall {
                        key,
                        offset: arg_start + quote.len_utf8(),
                    });
                }
            }
        } else if !is_translation_key_expression(content, searchable, type_index, arg_start) {
            calls.dynamic_offsets.push(arg_start);
        }

        search_start = call_start + marker.len();
    }
}

fn is_translation_key_expression(
    content: &str,
    searchable: &str,
    type_index: &TranslationTypeIndex,
    start: usize,
) -> bool {
    let Some((expression, _end)) = read_argument_expression(content, searchable, start) else {
        return false;
    };
    let expression = expression.trim();

    if expression.contains(" as TranslationKey")
        || expression.contains(" as const satisfies TranslationKey")
        || expression.contains(" satisfies TranslationKey")
    {
        return true;
    }

    let Some((identifier, rest)) = read_identifier(expression) else {
        return false;
    };
    let rest = rest.trim_start();

    if rest.starts_with('(') {
        return type_index.functions.contains(identifier);
    }

    rest.is_empty() && type_index.variables.contains(identifier)
}

fn read_argument_expression(
    content: &str,
    searchable: &str,
    start: usize,
) -> Option<(String, usize)> {
    let mut index = start;
    let mut paren_depth = 0_u32;
    let mut bracket_depth = 0_u32;
    let mut brace_depth = 0_u32;
    let mut in_string = None::<char>;
    let mut escaped = false;

    while index < searchable.len() {
        let character = searchable[index..].chars().next()?;

        if let Some(quote) = in_string {
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

        match character {
            '\'' | '"' | '`' => in_string = Some(character),
            '(' => paren_depth += 1,
            ')' if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 => {
                return Some((content[start..index].to_string(), index));
            }
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            '{' => brace_depth += 1,
            '}' => brace_depth = brace_depth.saturating_sub(1),
            ',' if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 => {
                return Some((content[start..index].to_string(), index));
            }
            _ => {}
        }

        index += character.len_utf8();
    }

    None
}

fn extract_translation_type_index(content: &str) -> TranslationTypeIndex {
    let mut index = TranslationTypeIndex::default();

    for line in content.lines() {
        let trimmed = line.trim_start();
        collect_translation_key_function(trimmed, &mut index);
        collect_translation_key_variable(trimmed, &mut index);
    }

    index
}

fn collect_translation_key_function(line: &str, index: &mut TranslationTypeIndex) {
    if let Some(rest) = line.strip_prefix("function ") {
        if let Some((name, rest)) = read_identifier(rest) {
            if rest.contains(": TranslationKey") {
                index.functions.insert(name.to_string());
            }
        }
        return;
    }

    let Some(rest) = line
        .strip_prefix("const ")
        .or_else(|| line.strip_prefix("let "))
    else {
        return;
    };
    let Some((name, rest)) = read_identifier(rest) else {
        return;
    };
    if rest.contains("=>") && rest.contains(": TranslationKey") {
        index.functions.insert(name.to_string());
    }
}

fn collect_translation_key_variable(line: &str, index: &mut TranslationTypeIndex) {
    let Some(rest) = line
        .strip_prefix("const ")
        .or_else(|| line.strip_prefix("let "))
    else {
        return;
    };
    let Some((name, rest)) = read_identifier(rest) else {
        return;
    };
    let rest = rest.trim_start();
    if rest.starts_with(": TranslationKey") || rest.starts_with(": readonly TranslationKey") {
        index.variables.insert(name.to_string());
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

pub(super) fn read_string_literal(
    content: &str,
    start: usize,
    quote: char,
) -> Option<(String, usize)> {
    let mut value = String::new();
    let mut escaped = false;
    let mut index = start + quote.len_utf8();

    while index < content.len() {
        let character = content[index..].chars().next()?;
        if escaped {
            value.push(character);
        } else if character == '\\' {
            escaped = true;
            index += character.len_utf8();
            continue;
        } else if character == quote {
            return Some((value, index + quote.len_utf8()));
        } else {
            value.push(character);
        }
        escaped = false;
        index += character.len_utf8();
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_translation_calls_reads_literals_from_this_t_and_aliases() {
        let calls = extract_translation_calls(
            "const t = this.t;\nthis.t('ping.title');\nt(\"ping.body\");",
        );

        assert_eq!(
            calls
                .static_calls
                .iter()
                .map(|call| call.key.as_str())
                .collect::<Vec<_>>(),
            vec!["ping.title", "ping.body"]
        );
        assert!(calls.dynamic_offsets.is_empty());
    }

    #[test]
    fn extract_translation_calls_reports_untyped_dynamic_keys() {
        let calls = extract_translation_calls(
            "const t = this.t;\nthis.t(key);\nt(messageKey);\nthis.t(`help.${field}`);",
        );

        assert_eq!(calls.static_calls.len(), 0);
        assert_eq!(calls.dynamic_offsets.len(), 3);
    }

    #[test]
    fn extract_translation_calls_ignores_comments() {
        let calls = extract_translation_calls(
            "// this.t('ignored.line')\n/** this.t('ignored.block') */\nthis.t('real.key');",
        );

        assert_eq!(calls.static_calls.len(), 1);
        assert_eq!(calls.static_calls[0].key, "real.key");
        assert!(calls.dynamic_offsets.is_empty());
    }

    #[test]
    fn extract_translation_calls_allows_translation_key_function_results() {
        let calls = extract_translation_calls(
            "function categoryKey(tag: Category, field: 'name' | 'description'): TranslationKey {\n  return `category.${tag}.${field}` as TranslationKey;\n}\nconst t = this.t;\nt(categoryKey(category.tag, 'name'));\nthis.t(categoryKey(category.tag, 'description'));",
        );

        assert!(calls.static_calls.is_empty());
        assert!(calls.dynamic_offsets.is_empty());
    }

    #[test]
    fn extract_translation_calls_allows_arrow_functions_returning_translation_key() {
        let calls = extract_translation_calls(
            "const toKey = (name: string): TranslationKey => `help.${name}` as TranslationKey;\nthis.t(toKey(name));",
        );

        assert!(calls.static_calls.is_empty());
        assert!(calls.dynamic_offsets.is_empty());
    }

    #[test]
    fn extract_translation_calls_allows_translation_key_variables_and_casts() {
        let calls = extract_translation_calls(
            "const typedKey: TranslationKey = 'help.root.title';\nthis.t(typedKey);\nthis.t(rawKey as TranslationKey);\nthis.t(`category.${tag}.name` as TranslationKey);",
        );

        assert!(calls.static_calls.is_empty());
        assert!(calls.dynamic_offsets.is_empty());
    }

    #[test]
    fn extract_translation_calls_still_reports_untyped_function_results() {
        let calls = extract_translation_calls(
            "function unsafeKey(name: string): string { return name; }\nthis.t(unsafeKey(name));",
        );

        assert_eq!(calls.dynamic_offsets.len(), 1);
    }

    #[test]
    fn extract_translation_calls_reads_first_argument_with_nested_commas() {
        let calls = extract_translation_calls(
            "function keyFor(a: string, b: string): TranslationKey { return `x.${a}.${b}` as TranslationKey; }\nthis.t(keyFor('a,b', nested(value, other)), 'arg');\nthis.t(plain(value, other));",
        );

        assert_eq!(calls.dynamic_offsets.len(), 1);
    }
}
