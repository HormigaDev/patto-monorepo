pub mod check_output;
pub mod doctor_output;
pub mod lint_output;
pub mod scan_output;

use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputStatus {
    Ok,
    Failed,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputStats {
    pub files_scanned: usize,
    pub directories_scanned: usize,
    pub duration_ms: u128,
}

pub fn print_json<T: Serialize>(value: &T) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(value)?;
    println!("{json}");

    Ok(())
}
