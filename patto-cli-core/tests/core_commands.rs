use serde_json::Value;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

struct FixtureProject {
    root: PathBuf,
}

impl FixtureProject {
    fn new(name: &str) -> Self {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("patto-cli-{name}-{}-{id}", std::process::id()));
        fs::create_dir_all(&root).expect("fixture root should be created");
        Self { root }
    }

    fn path(&self) -> &Path {
        &self.root
    }

    fn write(&self, relative: &str, content: &str) {
        let path = self.root.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("fixture parent should be created");
        }
        fs::write(path, content).expect("fixture file should be written");
    }
}

impl Drop for FixtureProject {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).ok();
    }
}

fn patto_core<I, S>(args: I) -> Output
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Command::new(env!("CARGO_BIN_EXE_patto-core"))
        .args(args)
        .output()
        .expect("patto-core should run")
}

fn json_output(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "stdout should be valid json: {error}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

fn minimal_project(name: &str) -> FixtureProject {
    let project = FixtureProject::new(name);
    project.write(
        ".patto/config.json",
        r#"{
  "schemaVersion": 1,
  "lang": "es"
}
"#,
    );
    project.write(
        "package.json",
        r#"{
  "name": "patto-fixture",
  "version": "1.0.0",
  "main": "dist/index.js",
  "scripts": {
    "dev": "tsx src/index.ts",
    "build": "tsc",
    "start": "node dist/index.js",
    "test": "vitest",
    "lint": "eslint ."
  },
  "dependencies": {
    "discord.js": "^14.0.0",
    "dotenv": "^16.0.0",
    "reflect-metadata": "^0.2.0"
  },
  "devDependencies": {
    "typescript": "^5.0.0"
  }
}
"#,
    );
    project.write(
        "tsconfig.json",
        r#"{
  "compilerOptions": {
    "experimentalDecorators": true,
    "emitDecoratorMetadata": true,
    "baseUrl": ".",
    "paths": {
      "@/*": ["src/*"]
    }
  }
}
"#,
    );
    project.write(".env.template", "BOT_TOKEN=\nCLIENT_ID=\n");
    project.write(".env", "BOT_TOKEN=test\nCLIENT_ID=test\n");
    project.write(
        "src/commands/general/ping.command.ts",
        r#"
@Command({ name: "ping", description: "Ping command", aliases: ["p"] })
export class PingCommand extends BaseCommand {
  async run(): Promise<void> {}
}
"#,
    );
    project
}

#[test]
fn scan_outputs_index_and_writes_index_file() {
    let project = minimal_project("scan");

    let output = patto_core([
        "scan",
        "--root",
        project.path().to_str().expect("path should be utf8"),
        "--json",
    ]);
    let body = json_output(&output);

    assert!(output.status.success());
    assert_eq!(body["status"], "ok");
    assert_eq!(body["command"], "scan");
    assert_eq!(body["indexPath"], ".patto/index.json");
    assert_eq!(body["index"]["summary"]["commands"], 1);
    assert_eq!(body["index"]["commands"][0]["key"], "ping");
    assert!(project.path().join(".patto/index.json").is_file());
}

#[test]
fn lint_reports_duplicate_commands_as_structured_json() {
    let project = minimal_project("lint-duplicates");
    project.write(
        "src/commands/admin/ping.command.ts",
        r#"
@Command({ name: "ping", description: "Duplicate ping" })
export class AdminPingCommand extends BaseCommand {
  async run(): Promise<void> {}
}
"#,
    );

    let output = patto_core([
        "lint",
        "--root",
        project.path().to_str().expect("path should be utf8"),
        "--json",
    ]);
    let body = json_output(&output);

    assert!(!output.status.success());
    assert_eq!(body["status"], "failed");
    assert_eq!(body["command"], "lint");
    assert!(body["summary"]["errors"].as_u64().unwrap_or_default() >= 1);
    assert!(
        body["diagnostics"]
            .as_array()
            .expect("diagnostics should be an array")
            .iter()
            .any(|diagnostic| diagnostic["code"] == "duplicate-commands"
                && diagnostic["level"] == "error"
                && diagnostic["file"].is_string()
                && diagnostic["line"].is_number()
                && diagnostic["column"].is_number())
    );
}

#[test]
fn lint_respects_rule_severity_overrides_from_project_config() {
    let project = minimal_project("lint-config");
    project.write(
        ".patto/config.json",
        r#"{
  "schemaVersion": 1,
  "lang": "es",
  "lint-rules": {
    "missing-run-method": "off"
  }
}
"#,
    );
    project.write(
        "src/commands/general/no-run.command.ts",
        r#"
@Command({ name: "norun", description: "No run" })
export class NoRunCommand extends BaseCommand {}
"#,
    );

    let output = patto_core([
        "lint",
        "--root",
        project.path().to_str().expect("path should be utf8"),
        "--json",
    ]);
    let body = json_output(&output);

    assert!(output.status.success());
    assert!(
        body["diagnostics"]
            .as_array()
            .expect("diagnostics should be an array")
            .iter()
            .all(|diagnostic| diagnostic["code"] != "missing-run-method")
    );
    assert!(
        body["rules"]
            .as_array()
            .expect("rules should be an array")
            .iter()
            .any(|rule| rule["rule"] == "missing-run-method" && rule["severity"] == "off")
    );
}

#[test]
fn check_returns_structured_root_error_for_missing_root() {
    let missing_root =
        std::env::temp_dir().join(format!("patto-cli-missing-root-{}", std::process::id()));
    fs::remove_dir_all(&missing_root).ok();

    let output = patto_core([
        "check",
        "--root",
        missing_root.to_str().expect("path should be utf8"),
        "--json",
    ]);
    let body = json_output(&output);

    assert!(!output.status.success());
    assert_eq!(body["status"], "failed");
    assert_eq!(body["command"], "check");
    assert_eq!(body["scan"]["status"], "failed");
    assert!(
        body["diagnostics"]
            .as_array()
            .expect("diagnostics should be an array")
            .iter()
            .any(
                |diagnostic| diagnostic["code"] == "patto_project_root_missing"
                    && diagnostic["hint"]
                        .as_str()
                        .unwrap_or_default()
                        .contains("--root")
            )
    );
}
