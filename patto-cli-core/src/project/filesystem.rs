use anyhow::{Result, bail};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use crate::diagnostic::{Diagnostic, DiagnosticCode, DiagnosticLevel};
use crate::lang::{self, Lang};
use crate::output::scan_output::ProjectIndex;

use super::INDEX_RELATIVE_PATH;

pub fn write_project_index(root: &Path, index: &ProjectIndex) -> Result<()> {
    let patto_dir = root.join(".patto");
    ensure_safe_project_path(root, &patto_dir)?;
    fs::create_dir_all(&patto_dir)?;
    ensure_safe_project_path(root, &patto_dir)?;

    let index_path = root.join(INDEX_RELATIVE_PATH);
    ensure_safe_project_path(root, &index_path)?;
    let json = serde_json::to_string_pretty(index)?;
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(index_path)?;
    file.write_all(format!("{json}\n").as_bytes())?;

    Ok(())
}

pub fn index_write_failed_diagnostic(locale: Lang, error: &anyhow::Error) -> Diagnostic {
    Diagnostic::new(
        DiagnosticLevel::Error,
        DiagnosticCode::PATTO_SCAN_INDEX_WRITE_FAILED,
        lang::text(locale, DiagnosticCode::PATTO_SCAN_INDEX_WRITE_FAILED),
    )
    .with_hint(format!(
        "{} {}",
        lang::text(
            locale,
            &format!("{}.hint", DiagnosticCode::PATTO_SCAN_INDEX_WRITE_FAILED)
        ),
        error
    ))
}

fn ensure_safe_project_path(root: &Path, path: &Path) -> Result<()> {
    if !path.starts_with(root) {
        bail!("la ruta de salida queda fuera de la raíz del proyecto");
    }

    if let Ok(metadata) = fs::symlink_metadata(path) {
        if metadata.file_type().is_symlink() {
            bail!("la ruta de salida no puede ser un symlink");
        }
    }

    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::project::empty_project_index;
    use std::os::unix::fs::symlink;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root(name: &str) -> PathBuf {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("patto-{name}-{}-{id}", std::process::id()));
        fs::create_dir_all(&root).expect("temp root should be created");
        root
    }

    #[test]
    fn write_project_index_refuses_symlink_index_path() {
        let root = temp_root("symlink-index");
        fs::create_dir_all(root.join(".patto")).expect("patto dir should be created");
        let outside = root.join("outside.txt");
        fs::write(&outside, "do-not-touch").expect("outside file should be written");
        symlink(&outside, root.join(INDEX_RELATIVE_PATH)).expect("symlink should be created");

        let result = write_project_index(&root, &empty_project_index());

        assert!(result.is_err());
        assert_eq!(
            fs::read_to_string(&outside).expect("outside file should still be readable"),
            "do-not-touch"
        );

        fs::remove_dir_all(root).ok();
    }
}
