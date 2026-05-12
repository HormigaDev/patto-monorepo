use std::path::Path;

use crate::output::scan_output::{ArgumentIndex, ArgumentOptionValue, ArgumentOptionValueKind};

pub(super) fn extract_decorator_block(content: &str, decorator_name: &str) -> Option<String> {
    let marker = format!("@{decorator_name}");
    let start = find_code_marker(content, &marker, 0)?;
    let open_paren = content[start..].find('(')? + start;
    let mut depth = 0_u32;
    let mut in_string = None::<char>;
    let mut escaped = false;

    for (offset, character) in content[open_paren..].char_indices() {
        if let Some(quote) = in_string {
            if escaped {
                escaped = false;
                continue;
            }
            if character == '\\' {
                escaped = true;
                continue;
            }
            if character == quote {
                in_string = None;
            }
            continue;
        }

        match character {
            '"' | '\'' | '`' => in_string = Some(character),
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    let close_paren = open_paren + offset;
                    return Some(content[open_paren + 1..close_paren].to_string());
                }
            }
            _ => {}
        }
    }

    None
}

fn extract_decorator_blocks(content: &str, decorator_name: &str) -> Vec<String> {
    let marker = format!("@{decorator_name}");
    let mut blocks = Vec::new();
    let mut search_start = 0;

    while let Some(start) = find_code_marker(content, &marker, search_start) {
        let Some(open_paren) = content[start..].find('(').map(|offset| start + offset) else {
            break;
        };
        let mut depth = 0_u32;
        let mut in_string = None::<char>;
        let mut escaped = false;

        for (offset, character) in content[open_paren..].char_indices() {
            if let Some(quote) = in_string {
                if escaped {
                    escaped = false;
                    continue;
                }
                if character == '\\' {
                    escaped = true;
                    continue;
                }
                if character == quote {
                    in_string = None;
                }
                continue;
            }

            match character {
                '"' | '\'' | '`' => in_string = Some(character),
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        let close_paren = open_paren + offset;
                        blocks.push(content[open_paren + 1..close_paren].to_string());
                        search_start = close_paren + 1;
                        break;
                    }
                }
                _ => {}
            }
        }

        if search_start <= start {
            break;
        }
    }

    blocks
}

pub(super) fn extract_arguments(content: &str) -> Vec<ArgumentIndex> {
    extract_decorator_blocks(content, "Arg")
        .into_iter()
        .map(|block| ArgumentIndex {
            name: extract_string_property(&block, "name"),
            required: extract_bool_property(&block, "required").unwrap_or(false),
            raw_text: extract_bool_property(&block, "rawText").unwrap_or(false),
            type_hint: extract_value_property(&block, "type"),
            option_values: extract_argument_option_values(&block),
        })
        .collect()
}

fn extract_argument_option_values(block: &str) -> Vec<ArgumentOptionValue> {
    let Some(options_tail) = extract_property_tail(block, "options") else {
        return Vec::new();
    };
    let Some(array_end) = find_balanced_end(options_tail, '[', ']') else {
        return Vec::new();
    };
    let options = &options_tail[..=array_end];
    let mut values = Vec::new();
    let mut search_start = 0;

    while let Some(relative_start) = options[search_start..].find("value:") {
        let start = search_start + relative_start + "value:".len();
        let tail = options[start..].trim_start();
        let raw = if let Some(value) = read_quoted_value(tail) {
            value
        } else {
            tail.split([',', '\n', '\r', '}'])
                .next()
                .unwrap_or_default()
                .trim()
                .to_string()
        };

        if !raw.is_empty() {
            values.push(ArgumentOptionValue {
                kind: classify_option_value(&raw),
                raw,
            });
        }

        search_start = start;
    }

    values
}

fn find_balanced_end(value: &str, open: char, close: char) -> Option<usize> {
    let mut depth = 0_u32;
    let mut in_string = None::<char>;
    let mut escaped = false;

    for (offset, character) in value.char_indices() {
        if let Some(quote) = in_string {
            if escaped {
                escaped = false;
                continue;
            }
            if character == '\\' {
                escaped = true;
                continue;
            }
            if character == quote {
                in_string = None;
            }
            continue;
        }

        match character {
            '"' | '\'' | '`' => in_string = Some(character),
            character if character == open => depth += 1,
            character if character == close => {
                depth -= 1;
                if depth == 0 {
                    return Some(offset);
                }
            }
            _ => {}
        }
    }

    None
}

fn read_quoted_value(value: &str) -> Option<String> {
    let quote = value.chars().next()?;
    if quote != '"' && quote != '\'' && quote != '`' {
        return None;
    }

    let mut escaped = false;
    let mut result = String::new();
    for character in value[quote.len_utf8()..].chars() {
        if escaped {
            result.push(character);
            escaped = false;
            continue;
        }
        if character == '\\' {
            escaped = true;
            continue;
        }
        if character == quote {
            return Some(result);
        }
        result.push(character);
    }

    None
}

fn classify_option_value(value: &str) -> ArgumentOptionValueKind {
    if value == "true" || value == "false" {
        ArgumentOptionValueKind::Boolean
    } else if value.parse::<f64>().is_ok() {
        ArgumentOptionValueKind::Number
    } else if !value.is_empty() {
        ArgumentOptionValueKind::String
    } else {
        ArgumentOptionValueKind::Unknown
    }
}

pub(super) fn extract_string_property(block: &str, key: &str) -> Option<String> {
    let value = extract_property_tail(block, key)?;
    let quote = value.chars().next()?;
    if quote != '"' && quote != '\'' && quote != '`' {
        return None;
    }

    let mut escaped = false;
    let mut result = String::new();
    for character in value[quote.len_utf8()..].chars() {
        if escaped {
            result.push(character);
            escaped = false;
            continue;
        }
        if character == '\\' {
            escaped = true;
            continue;
        }
        if character == quote {
            return Some(result);
        }
        result.push(character);
    }

    None
}

fn extract_bool_property(block: &str, key: &str) -> Option<bool> {
    let value = extract_property_tail(block, key)?;
    let value = value
        .split([',', '\n', '\r', '}'])
        .next()
        .unwrap_or_default()
        .trim();

    match value {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

pub(super) fn extract_value_property(block: &str, key: &str) -> Option<String> {
    let value = extract_property_tail(block, key)?;
    let value = value
        .split([',', '\n', '\r', '}'])
        .next()
        .unwrap_or_default()
        .trim();

    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

pub(super) fn extract_string_array_property(block: &str, key: &str) -> Vec<String> {
    let value = match extract_property_tail(block, key) {
        Some(value) => value.trim_start(),
        None => return Vec::new(),
    };

    if !value.starts_with('[') {
        return Vec::new();
    }

    let end = value.find(']').unwrap_or(value.len());
    let array = &value[..end];
    let mut items = Vec::new();
    let mut in_string = None::<char>;
    let mut escaped = false;
    let mut current = String::new();

    for character in array.chars() {
        if let Some(quote) = in_string {
            if escaped {
                current.push(character);
                escaped = false;
                continue;
            }
            if character == '\\' {
                escaped = true;
                continue;
            }
            if character == quote {
                items.push(current.clone());
                current.clear();
                in_string = None;
                continue;
            }
            current.push(character);
            continue;
        }

        if character == '"' || character == '\'' || character == '`' {
            in_string = Some(character);
        }
    }

    items
}

fn extract_property_tail<'a>(block: &'a str, key: &str) -> Option<&'a str> {
    let mut search_start = 0;

    while let Some(start) = find_code_marker(block, key, search_start) {
        let key_end = start + key.len();
        let valid_prefix = block[..start]
            .chars()
            .next_back()
            .map(|character| !is_identifier_character(character))
            .unwrap_or(true);
        let valid_suffix = block[key_end..]
            .chars()
            .next()
            .map(|character| !is_identifier_character(character))
            .unwrap_or(true);

        if valid_prefix && valid_suffix {
            let tail = block[key_end..].trim_start();
            if let Some(value) = tail.strip_prefix(':') {
                return Some(value.trim_start());
            }
        }

        search_start = key_end;
    }

    None
}

pub(super) fn extract_class_name(content: &str) -> Option<String> {
    let class_pos = find_code_marker(content, "class", 0)? + "class".len();
    extract_identifier(&content[class_pos..])
}

pub(super) fn extract_extends_name(content: &str) -> Option<String> {
    let class_pos = find_code_marker(content, "class", 0)?;
    let after_class = &content[class_pos..];
    let extends_pos = after_class.find(" extends ")? + " extends ".len();
    extract_identifier(&after_class[extends_pos..])
}

pub(super) fn has_run_method(content: &str) -> bool {
    let code = mask_non_code_content(content);
    let mut search_start = 0;

    while let Some(relative_start) = code[search_start..].find("run") {
        let start = search_start + relative_start;
        let end = start + "run".len();
        let valid_prefix = code[..start]
            .chars()
            .next_back()
            .map(|character| !is_identifier_character(character))
            .unwrap_or(true);
        let valid_suffix = code[end..]
            .chars()
            .next()
            .map(|character| !is_identifier_character(character))
            .unwrap_or(true);

        if valid_prefix && valid_suffix {
            let tail = code[end..].trim_start();
            if tail.starts_with('(') || tail.starts_with("():") {
                return true;
            }
        }

        search_start = end;
    }

    false
}

fn extract_identifier(value: &str) -> Option<String> {
    let identifier = value
        .chars()
        .skip_while(|character| character.is_whitespace())
        .take_while(|character| is_identifier_character(*character))
        .collect::<String>();

    if identifier.is_empty() {
        None
    } else {
        Some(identifier)
    }
}

fn is_identifier_character(character: char) -> bool {
    character.is_alphanumeric() || character == '_' || character == '$'
}

fn find_code_marker(content: &str, marker: &str, from: usize) -> Option<usize> {
    let mut index = 0;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut in_string = None::<char>;
    let mut escaped = false;

    while index < content.len() {
        if in_line_comment {
            let character = next_char(content, index)?;
            if character == '\n' {
                in_line_comment = false;
            }
            index += character.len_utf8();
            continue;
        }

        if in_block_comment {
            if content[index..].starts_with("*/") {
                in_block_comment = false;
                index += 2;
            } else {
                index += next_char(content, index)?.len_utf8();
            }
            continue;
        }

        if let Some(quote) = in_string {
            let character = next_char(content, index)?;
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
            index += 2;
            continue;
        }
        if content[index..].starts_with("/*") {
            in_block_comment = true;
            index += 2;
            continue;
        }

        let character = next_char(content, index)?;
        if matches!(character, '"' | '\'' | '`') {
            in_string = Some(character);
            index += character.len_utf8();
            continue;
        }

        if index >= from && content[index..].starts_with(marker) {
            return Some(index);
        }

        index += character.len_utf8();
    }

    None
}

fn mask_non_code_content(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut index = 0;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut in_string = None::<char>;
    let mut escaped = false;

    while index < content.len() {
        let character = match next_char(content, index) {
            Some(character) => character,
            None => break,
        };

        if in_line_comment {
            if character == '\n' {
                in_line_comment = false;
                result.push('\n');
            } else {
                result.push(' ');
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
                result.push(if character == '\n' { '\n' } else { ' ' });
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
            result.push(if character == '\n' { '\n' } else { ' ' });
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
            result.push(' ');
            index += character.len_utf8();
            continue;
        }

        result.push(character);
        index += character.len_utf8();
    }

    result
}

fn next_char(content: &str, index: usize) -> Option<char> {
    content[index..].chars().next()
}

pub(super) fn build_key(parts: [Option<&str>; 3]) -> Option<String> {
    let values = parts
        .into_iter()
        .flatten()
        .map(|part| part.to_lowercase())
        .collect::<Vec<_>>();

    if values.is_empty() {
        None
    } else {
        Some(values.join("-"))
    }
}

pub(super) fn is_typescript_or_javascript(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("ts" | "tsx" | "js" | "jsx")
    )
}

pub(super) fn is_command_file(relative_path: &str) -> bool {
    relative_path.starts_with("src/commands/")
        && (relative_path.ends_with(".command.ts") || relative_path.ends_with(".command.js"))
}

pub(super) fn is_definition_file(relative_path: &str) -> bool {
    relative_path.starts_with("src/definitions/")
        && (relative_path.ends_with(".definition.ts") || relative_path.ends_with(".definition.js"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_run_method_ignores_comments_strings_and_other_identifiers() {
        assert!(!has_run_method(
            r#"
export class NoRunCommand extends BaseCommand {
  // async run(): Promise<void> {}
  label = "run()";
  brun() {}
}
"#
        ));
        assert!(has_run_method(
            r#"
export class RunCommand extends BaseCommand {
  public async run(): Promise<void> {}
}
"#
        ));
    }

    #[test]
    fn extract_property_tail_uses_exact_property_names() {
        let block = r#"{ username: "wrong", name: "right" }"#;

        assert_eq!(
            extract_string_property(block, "name").as_deref(),
            Some("right")
        );
    }
}
