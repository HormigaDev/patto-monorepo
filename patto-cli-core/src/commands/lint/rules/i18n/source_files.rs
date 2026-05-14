use super::super::super::context::RuleContext;

pub(super) fn source_files(context: &RuleContext<'_>) -> Vec<String> {
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
