use std::path::{Component, Path, PathBuf};

use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::output::lint_output::LintRuleSeverity;
use crate::utils;

use super::super::context::RuleContext;

pub fn run(context: &RuleContext<'_>, severity: LintRuleSeverity) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for file in context
        .project
        .files
        .iter()
        .filter(|file| is_source_file(file))
    {
        let Some(content) = context.read_file(file) else {
            continue;
        };
        let content = utils::strip_line_comments(&content);

        for import_path in extract_alias_imports(&content) {
            if resolves_alias_import(context.root(), &import_path) {
                continue;
            }

            let mut diagnostic = context.diagnostic(
                DiagnosticCode::PATTO_LINT_BROKEN_ALIAS_IMPORTS,
                severity,
                "broken-alias-imports.message",
                &[("import", import_path.as_str())],
            );
            if let Some((line, column)) = context.location_for_text(file, &import_path) {
                diagnostic = diagnostic.with_location(file, line, column);
            } else {
                diagnostic = diagnostic.with_location(file, 1, 1);
            }
            diagnostics.push(diagnostic);
        }
    }

    diagnostics
}

fn is_source_file(file: &str) -> bool {
    file.starts_with("src/")
        && (file.ends_with(".ts")
            || file.ends_with(".tsx")
            || file.ends_with(".js")
            || file.ends_with(".jsx"))
}

fn extract_alias_imports(content: &str) -> Vec<String> {
    let mut imports = Vec::new();
    for quote in ["'", "\""] {
        let marker = format!("{quote}@/");
        let mut search_start = 0;
        while let Some(relative_start) = content[search_start..].find(&marker) {
            let start = search_start + relative_start + quote.len();
            let tail = &content[start..];
            let Some(end) = tail[1..].find(quote).map(|offset| offset + 1) else {
                break;
            };
            imports.push(tail[..end].to_string());
            search_start = start + end + quote.len();
        }
    }
    imports.sort();
    imports.dedup();
    imports
}

fn resolves_alias_import(root: &Path, import_path: &str) -> bool {
    let Some(relative) = safe_alias_relative(import_path) else {
        return false;
    };
    let base = root.join("src").join(&relative);
    if base.is_file() {
        return true;
    }

    ["ts", "tsx", "js", "jsx"].iter().any(|extension| {
        root.join("src")
            .join(format!("{}.{}", relative.display(), extension))
            .is_file()
    }) || ["ts", "tsx", "js", "jsx"]
        .iter()
        .any(|extension| base.join(format!("index.{extension}")).is_file())
}

fn safe_alias_relative(import_path: &str) -> Option<PathBuf> {
    let relative = import_path.strip_prefix("@/")?;
    if relative.trim().is_empty() {
        return None;
    }

    let path = Path::new(relative);
    let mut safe = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(value) => safe.push(value),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return None,
        }
    }

    if safe.as_os_str().is_empty() {
        None
    } else {
        Some(safe)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root() -> PathBuf {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "patto-alias-imports-test-{}-{id}",
            std::process::id()
        ));
        fs::create_dir_all(root.join("src/utils")).expect("fixture should be created");
        root
    }

    #[test]
    fn safe_alias_relative_rejects_paths_that_escape_src() {
        assert!(safe_alias_relative("@/../package.json").is_none());
        assert!(safe_alias_relative("@/../../secrets").is_none());
        assert!(safe_alias_relative("@/").is_none());
    }

    #[test]
    fn resolves_alias_import_accepts_files_inside_src_only() {
        let root = temp_root();
        fs::write(root.join("package.json"), "{}").expect("package should be written");
        fs::write(root.join("src/utils/logger.ts"), "export {}").expect("source should be written");

        assert!(resolves_alias_import(&root, "@/utils/logger"));
        assert!(!resolves_alias_import(&root, "@/../package.json"));

        fs::remove_dir_all(root).ok();
    }
}
