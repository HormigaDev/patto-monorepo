use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use super::super::super::context::RuleContext;
use super::translation_calls::read_string_literal;

#[derive(Debug, Clone)]
pub(super) struct LocaleFile {
    pub(super) locale: String,
    pub(super) file: String,
    pub(super) keys: BTreeSet<String>,
}

pub(super) fn collect_locale_files(context: &RuleContext<'_>) -> Vec<LocaleFile> {
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
}
