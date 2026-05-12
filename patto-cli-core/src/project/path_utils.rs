use std::path::Path;

pub(super) fn relative_path_looks_like(path: &Path, prefix: &str, marker: &str) -> bool {
    let normalized = normalize_path(path);
    normalized.contains(prefix) && normalized.contains(marker)
}

pub(super) fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .map(normalize_path)
        .unwrap_or_else(|_| normalize_path(path))
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

pub(super) fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}
