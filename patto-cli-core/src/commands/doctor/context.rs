use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::diagnostic::Diagnostic;
use crate::lang::{self, Lang};
use crate::output::doctor_output::{DoctorCheck, DoctorCheckStatus};
use crate::project::ProjectScan;
use crate::utils;

const COMMAND_TIMEOUT: Duration = Duration::from_secs(3);

pub struct DoctorContext<'a> {
    pub project: &'a ProjectScan,
    pub locale: Lang,
}

impl<'a> DoctorContext<'a> {
    pub fn root(&self) -> &Path {
        &self.project.root
    }

    pub fn read_file(&self, relative_file: &str) -> Option<String> {
        fs::read_to_string(self.root().join(relative_file)).ok()
    }

    pub fn read_json(&self, relative_file: &str) -> Option<Value> {
        self.read_file(relative_file)
            .and_then(|content| serde_json::from_str::<Value>(&content).ok())
    }

    pub fn read_env_file(&self) -> HashMap<String, String> {
        utils::read_env_file(self.root(), ".env")
    }

    pub fn feature_enabled(&self, feature: &str) -> bool {
        self.project
            .config_json
            .as_ref()
            .and_then(|value| value.get("features"))
            .and_then(Value::as_object)
            .and_then(|features| features.get(feature))
            .and_then(Value::as_bool)
            .unwrap_or(false)
    }

    pub fn command_version(&self, command: &str, arg: &str) -> Option<String> {
        let mut child = Command::new(command)
            .arg(arg)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .ok()?;
        let started_at = Instant::now();

        loop {
            if child.try_wait().ok()?.is_some() {
                let output = child.wait_with_output().ok()?;
                if !output.status.success() {
                    return None;
                }

                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return if stdout.is_empty() {
                    None
                } else {
                    Some(stdout)
                };
            }

            if started_at.elapsed() >= COMMAND_TIMEOUT {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }

            thread::sleep(Duration::from_millis(25));
        }
    }

    pub fn check(
        &self,
        id: impl Into<String>,
        status: DoctorCheckStatus,
        message: impl Into<String>,
        details: Vec<String>,
    ) -> DoctorCheck {
        DoctorCheck {
            id: id.into(),
            status,
            message: message.into(),
            details,
        }
    }

    pub fn diagnostic(
        &self,
        code: &'static str,
        level: crate::diagnostic::DiagnosticLevel,
        message_key: &str,
        args: &[(&str, &str)],
    ) -> Diagnostic {
        Diagnostic::new(level, code, lang::message(self.locale, message_key, args))
            .with_hint(lang::text(self.locale, &format!("{code}.hint")))
    }
}
