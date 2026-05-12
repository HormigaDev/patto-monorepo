use std::fs;
use std::path::Path;

pub fn find_value_location(
    root: &Path,
    relative_file: &str,
    property: &str,
    value: &str,
) -> Option<(u32, u32)> {
    let content = fs::read_to_string(root.join(relative_file)).ok()?;
    let needle = format!("{property}:");
    let property_start = content.find(&needle)?;
    let property_tail = &content[property_start + needle.len()..];
    let value_offset = property_tail.find(value)?;
    let absolute_offset = property_start + needle.len() + value_offset;
    offset_to_line_column(&content, absolute_offset)
}

pub fn find_text_location(root: &Path, relative_file: &str, value: &str) -> Option<(u32, u32)> {
    let content = fs::read_to_string(root.join(relative_file)).ok()?;
    let offset = content.find(value)?;
    offset_to_line_column(&content, offset)
}

fn offset_to_line_column(content: &str, offset: usize) -> Option<(u32, u32)> {
    let mut line = 1_u32;
    let mut column = 1_u32;

    for (index, character) in content.char_indices() {
        if index >= offset {
            return Some((line, column));
        }
        if character == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }

    Some((line, column))
}
